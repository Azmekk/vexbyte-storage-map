use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use super::file_uri::file_uri_of;
use super::spawned_process::spawn_detached;

/// File managers answer `ShowItems` right away; the limit only guards against a hung bus.
const SHOW_ITEMS_REPLY_TIMEOUT_MILLISECONDS: u32 = 3000;

/// Opens the file manager on the folder containing `path` with the item selected, through the
/// freedesktop `FileManager1` D-Bus interface. Without one (no `dbus-send`, no session bus, no
/// file manager that implements it) the containing folder is opened with `xdg-open` instead.
pub(crate) fn reveal_in_file_manager(path: &Path) -> io::Result<()> {
    match show_item_through_file_manager_interface(path) {
        Ok(()) => Ok(()),
        Err(error) => {
            tracing::debug!(
                %error,
                path = %path.display(),
                "FileManager1.ShowItems failed; opening the containing folder instead"
            );
            open_containing_folder(path)
        }
    }
}

fn show_item_through_file_manager_interface(path: &Path) -> io::Result<()> {
    let item_uri = file_uri_of(path)?;
    let arguments = show_items_arguments(&item_uri);

    let status = Command::new("dbus-send")
        .args(&arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "dbus-send FileManager1.ShowItems exited with {status}"
        )))
    }
}

fn show_items_arguments(item_uri: &str) -> Vec<String> {
    vec![
        "--session".to_owned(),
        "--print-reply".to_owned(),
        format!("--reply-timeout={SHOW_ITEMS_REPLY_TIMEOUT_MILLISECONDS}"),
        "--dest=org.freedesktop.FileManager1".to_owned(),
        "--type=method_call".to_owned(),
        "/org/freedesktop/FileManager1".to_owned(),
        "org.freedesktop.FileManager1.ShowItems".to_owned(),
        format!("array:string:{item_uri}"),
        "string:".to_owned(),
    ]
}

fn open_containing_folder(path: &Path) -> io::Result<()> {
    let folder = folder_to_open_for(path).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "the path has no containing folder",
        )
    })?;
    spawn_detached("xdg-open", &[folder.as_os_str()])
}

/// The folder shown when the item itself cannot be selected.
fn folder_to_open_for(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_items_passes_one_uri_and_an_empty_startup_id() {
        let arguments = show_items_arguments("file:///home/martin/My%20Photo.png");

        assert_eq!(
            arguments.last().map(String::as_str),
            Some("string:"),
            "the second ShowItems argument is the startup notification id"
        );
        assert!(arguments.contains(&"array:string:file:///home/martin/My%20Photo.png".to_owned()));
        assert!(arguments.contains(&"org.freedesktop.FileManager1.ShowItems".to_owned()));
    }

    #[test]
    fn falls_back_to_the_parent_folder() {
        assert_eq!(
            folder_to_open_for(Path::new("/home/martin/photo.png")),
            Some(Path::new("/home/martin"))
        );
        assert_eq!(folder_to_open_for(Path::new("/")), None);
        assert_eq!(folder_to_open_for(Path::new("photo.png")), None);
    }
}
