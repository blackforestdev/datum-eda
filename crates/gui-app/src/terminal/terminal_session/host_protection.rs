//! Read-only naming protections over current, parked and pending session origins.
use super::*;
use crate::host_filesystem::Protection;

impl TerminalSessionRegistry {
    /// Existing reservation ownership spans worker preparation, queued answers
    /// and rejected-terminal cleanup, including after the dialog receiver drops.
    pub(crate) fn project_preparation_pending(&self) -> bool {
        self.project_open_reservations
            .load(std::sync::atomic::Ordering::Acquire)
            != 0
    }

    pub(crate) fn host_path_protection(
        &self,
        current: &datum_gui_protocol::ReviewWorkspaceState,
    ) -> Protection {
        let mut protection = Protection::default();
        add_workspace(current, &mut protection);
        for context in self.origin_contexts.values() {
            if let Some(root) = &context.project_root {
                protection.anchors.push(root.clone());
            }
            protection
                .anchors
                .push(context.launch_working_directory.clone());
        }
        for (index, slot) in self.sessions.iter().enumerate() {
            let lane = if index == self.active_index {
                &current.ui.terminal
            } else {
                &slot.parked_lane
            };
            if let Some(path) = lane
                .current_working_directory
                .as_deref()
                .and_then(local_cwd)
            {
                protection.anchors.push(path);
            }
        }
        for (root, workspace) in &self.origin_workspaces {
            protection.anchors.push(root.clone());
            add_workspace(workspace, &mut protection);
        }
        protection.anchors.sort();
        protection.anchors.dedup();
        protection.managed.sort();
        protection.managed.dedup();
        protection
    }
}

fn add_workspace(
    workspace: &datum_gui_protocol::ReviewWorkspaceState,
    protection: &mut Protection,
) {
    let Some(backing) = &workspace.backing else {
        return;
    };
    let root = &backing.request.project_root;
    protection.anchors.push(root.clone());
    protection
        .managed
        .extend([root.join("project.json"), root.join(".datum")]);
    if let Some(model) = &workspace.library.basis {
        for shard in &model.source_shards {
            protection.managed.push(shard.path.clone());
            if let Some(parent) = shard.path.parent().filter(|p| *p != root) {
                protection.managed.push(parent.to_path_buf());
            }
        }
    }
}

use crate::local_path::decode_local_path as local_cwd;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_protection_decodes_local_cwd_without_claiming_remote_paths() {
        assert_eq!(
            local_cwd("file:///tmp/exact%20name"),
            Some(PathBuf::from("/tmp/exact name"))
        );
        assert_eq!(
            local_cwd("file://localhost/tmp/a%25b"),
            Some(PathBuf::from("/tmp/a%b"))
        );
        assert_eq!(local_cwd("/tmp/ λ "), Some(PathBuf::from("/tmp/ λ ")));
        assert_eq!(
            local_cwd("/tmp/100% literal"),
            Some(PathBuf::from("/tmp/100% literal"))
        );

        assert_eq!(local_cwd("file://remote/tmp/a"), None);
        assert_eq!(local_cwd("file:///tmp/%00"), None);
        assert_eq!(local_cwd("file:///tmp/%xy"), None);
    }
}
