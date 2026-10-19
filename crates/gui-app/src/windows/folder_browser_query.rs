//! Worker-only local folder matching; never resolves or publishes a Project.
use super::*;

pub(super) fn discover(
    directory: &Path,
    input: Option<&str>,
    assisted: bool,
    current: impl Fn() -> bool,
) -> Result<Listing, String> {
    let text = input.or_else(|| directory.to_str()).unwrap_or("");
    let fragment = text.rsplit('/').next().unwrap_or("");
    if !assisted
        || text.is_empty()
        || text.ends_with('/')
        || text == "~"
        || matches!(fragment, "." | "..")
    {
        return scan(directory, current);
    }
    let parent = directory.parent().unwrap_or(Path::new("/"));
    let mut listing = match scan(parent, &current) {
        Ok(listing) => listing,
        Err(_)
            if std::fs::metadata(parent).err().is_some_and(|e| {
                matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                )
            }) =>
        {
            Listing {
                entries: vec![],
                parent_identity: None,
                occupied: Default::default(),
                notice: None,
                suggestions: None,
            }
        }
        Err(error) => return Err(error),
    };
    let folded = fragment.to_lowercase();
    listing
        .entries
        .retain(|e| e.navigable() && e.name.to_lowercase().contains(&folded));
    listing.entries.sort_by_cached_key(|e| {
        (
            !e.name.to_lowercase().starts_with(&folded),
            e.name.to_lowercase(),
            e.name.clone(),
        )
    });
    listing.suggestions = Some(datum_gui_protocol::FolderSuggestions {
        input: String::new(),
        parent: parent.display().to_string(),
        fragment: fragment.into(),
    });
    Ok(listing)
}
