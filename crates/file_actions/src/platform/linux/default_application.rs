use std::io;
use std::path::Path;

use super::spawned_process::spawn_detached;

/// Opens `path` with the application the desktop associates with its type, through `xdg-open`.
pub(crate) fn open_with_default_application(path: &Path) -> io::Result<()> {
    spawn_detached("xdg-open", &[path.as_os_str()])
}
