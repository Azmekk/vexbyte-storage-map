use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix};

const VERBATIM_PREFIX: &str = r"\\?\";
const VERBATIM_UNC_PREFIX: &str = r"\\?\UNC\";

/// Encodes `path` as a null-terminated UTF-16 extended-length path (`\\?\C:\...`), so Win32 calls
/// work past the 260 character `MAX_PATH` limit. Relative paths are made absolute first, because
/// the `\\?\` form disables all path normalization.
pub(super) fn to_extended_length_wide_path(path: &Path) -> io::Result<Vec<u16>> {
    let absolute_path = std::path::absolute(path)?;
    let extended_length_path = add_extended_length_prefix(&absolute_path);

    Ok(to_null_terminated_wide_string(
        extended_length_path.as_os_str(),
    ))
}

pub(super) fn to_null_terminated_wide_string(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}

fn add_extended_length_prefix(absolute_path: &Path) -> PathBuf {
    let Some(Component::Prefix(prefix)) = absolute_path.components().next() else {
        return absolute_path.to_path_buf();
    };

    let path_text = absolute_path.as_os_str().to_string_lossy();
    match prefix.kind() {
        Prefix::Disk(_) => PathBuf::from(format!("{VERBATIM_PREFIX}{path_text}")),
        Prefix::UNC(_, _) => {
            let without_leading_separators = path_text.trim_start_matches('\\');
            PathBuf::from(format!("{VERBATIM_UNC_PREFIX}{without_leading_separators}"))
        }
        Prefix::Verbatim(_)
        | Prefix::VerbatimUNC(_, _)
        | Prefix::VerbatimDisk(_)
        | Prefix::DeviceNS(_) => absolute_path.to_path_buf(),
    }
}

/// Turns `\\?\C:\Folder` into `C:\Folder` and `\\?\UNC\server\share` into `\\server\share`, the
/// forms users recognize. Other paths are returned unchanged.
pub(super) fn remove_extended_length_prefix(path: &Path) -> PathBuf {
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return path.to_path_buf();
    };

    let path_text = path.as_os_str().to_string_lossy();
    match prefix.kind() {
        Prefix::VerbatimDisk(_) => path_text
            .strip_prefix(VERBATIM_PREFIX)
            .map_or_else(|| path.to_path_buf(), PathBuf::from),
        Prefix::VerbatimUNC(_, _) => path_text.strip_prefix(VERBATIM_UNC_PREFIX).map_or_else(
            || path.to_path_buf(),
            |rest| PathBuf::from(format!(r"\\{rest}")),
        ),
        _ => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn prefixes_drive_paths() {
        assert_eq!(
            add_extended_length_prefix(Path::new(r"C:\Users\Photos")),
            PathBuf::from(r"\\?\C:\Users\Photos")
        );
    }

    #[test]
    fn prefixes_network_paths() {
        assert_eq!(
            add_extended_length_prefix(Path::new(r"\\server\share\folder")),
            PathBuf::from(r"\\?\UNC\server\share\folder")
        );
    }

    #[test]
    fn leaves_already_prefixed_paths_alone() {
        assert_eq!(
            add_extended_length_prefix(Path::new(r"\\?\C:\Users")),
            PathBuf::from(r"\\?\C:\Users")
        );
    }

    #[test]
    fn removes_prefixes_again() {
        assert_eq!(
            remove_extended_length_prefix(Path::new(r"\\?\C:\Users")),
            PathBuf::from(r"C:\Users")
        );
        assert_eq!(
            remove_extended_length_prefix(Path::new(r"\\?\UNC\server\share\folder")),
            PathBuf::from(r"\\server\share\folder")
        );
        assert_eq!(
            remove_extended_length_prefix(Path::new(r"D:\Music")),
            PathBuf::from(r"D:\Music")
        );
    }

    #[test]
    fn wide_path_is_null_terminated_and_absolute() {
        let wide_path = to_extended_length_wide_path(Path::new("relative")).unwrap();

        assert_eq!(wide_path.last(), Some(&0));
        let text = String::from_utf16_lossy(&wide_path[..wide_path.len() - 1]);
        assert!(text.starts_with(VERBATIM_PREFIX), "{text}");
        assert!(text.ends_with(r"\relative"), "{text}");
    }
}
