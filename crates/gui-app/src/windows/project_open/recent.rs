//! Application-local MRU shortcuts. Opening still uses the ordinary Project worker.
use super::*;
use datum_gui_protocol::gui_menu_model::{RecentProject, RecentProjects};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const RECENT_LIMIT: usize = 10;

fn history_path() -> Result<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/state")))
        .context("No user state directory available")?;
    Ok(state.join("datum-eda/recent-projects.json"))
}

fn read_history(path: &Path) -> Result<Vec<RecentProject>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut entries: Vec<RecentProject> = serde_json::from_slice(&bytes)?;
            entries.retain(|entry| entry.path.is_absolute());
            let mut seen = std::collections::HashSet::new();
            entries.retain(|entry| seen.insert(entry.path.clone()));
            entries.truncate(RECENT_LIMIT);
            Ok(entries)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn load_into(state: &mut datum_gui_protocol::ReviewWorkspaceState) {
    match history_path()
        .and_then(|path| read_history(&path))
        .and_then(RecentProjects::from_entries)
    {
        Ok(recent) => state.ui.recent_projects = recent,
        Err(error) => append_gui_diagnostic_line(format!("Recent projects unavailable: {error:#}")),
    }
}

fn remember(path: &Path, entry: RecentProject) -> Result<Vec<RecentProject>> {
    let parent = path.parent().context("Recent projects directory missing")?;
    std::fs::create_dir_all(parent)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(parent.join("recent-projects.lock"))?;
    lock.try_lock()?;
    // Merge under the lock: another Datum window may have opened a Project.
    let mut entries = read_history(path)?;
    entries.retain(|old| old.path != entry.path);
    entries.insert(0, entry);
    entries.truncate(RECENT_LIMIT);
    let temporary = parent.join(format!(".recent-projects-{}.tmp", std::process::id()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(&entries)?)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result?;
    Ok(entries)
}

impl Runtime {
    pub(crate) fn remember_opened_project(&mut self) {
        if self.workspace().project_unopened
            || !datum_gui_protocol::gui_menu_model::native_project_edit_available(self.workspace())
        {
            return;
        }
        let Some(backing) = &self.workspace().backing else {
            return;
        };
        let project_root = backing.request.project_root.clone();
        let result = (|| -> Result<()> {
            let path = std::fs::canonicalize(&project_root)?;
            let entry = RecentProject {
                name: self.workspace().scene.project_name.clone(),
                path,
            };
            let entries = remember(&history_path()?, entry)?;
            self.session
                .workspace_mut()
                .ui
                .recent_projects
                .update(entries)?;
            Ok(())
        })();
        if let Err(error) = result {
            self.log_console_echo(
                ConsoleFeedbackSource::Viewport,
                format!("Project opened; recent-project history could not be saved: {error:#}"),
            );
        }
    }

    pub(crate) fn open_recent_project(&mut self, index: usize) -> bool {
        let Some(entry) = self
            .workspace()
            .ui
            .recent_projects
            .entries()
            .get(index)
            .cloned()
        else {
            return false;
        };
        self.open_project();
        let Some(path) = entry.path.to_str() else {
            self.session.workspace_mut().ui.project_open.refusal =
                Some("Recent Project path cannot be displayed; use Open Project.".into());
            return true;
        };
        self.init_project_navigation(datum_gui_protocol::FolderBrowserMode::Open, path);
        self.session.workspace_mut().ui.project_open.directory = path.into();
        self.begin_project_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recent_projects_persist_deduplicate_and_preserve_missing_paths() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../.datum/scratch/recent-project-tests/{}",
            uuid::Uuid::new_v4()
        ));
        let file = root.join("state/recent-projects.json");
        let entry = |n| RecentProject {
            name: format!("Project {n}"),
            path: root.join(format!("project-{n}")),
        };
        assert!(read_history(&file).unwrap().is_empty());
        for n in 0..12 {
            remember(&file, entry(n)).unwrap();
        }
        let entries = remember(&file, entry(3)).unwrap();
        assert_eq!(entries.len(), RECENT_LIMIT);
        assert_eq!(entries[0].path, entry(3).path);
        assert_eq!(entries[1].path, entry(11).path);
        assert_eq!(read_history(&file).unwrap(), entries);
        assert!(
            !entries[0].path.exists(),
            "Missing shortcuts remain available for explicit Open refusal"
        );
        std::fs::write(&file, "invalid history").unwrap();
        assert!(remember(&file, entry(1)).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "invalid history");
        std::fs::remove_dir_all(root).unwrap();
    }
}
