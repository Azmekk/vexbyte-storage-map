use std::path::Path;

/// Linux file managers and the Trash take names byte for byte; no name is rewritten.
pub(crate) fn has_name_the_shell_would_rewrite(_path: &Path) -> bool {
    false
}
