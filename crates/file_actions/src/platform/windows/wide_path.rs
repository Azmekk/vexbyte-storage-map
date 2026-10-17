use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix};

const VERBATIM_PREFIX: &str = r"\\?\";
const VERBATIM_UNC_PREFIX: &str = r"\\?\UNC\";

pub(super) fn to_null_terminated_wide_string(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}

/// An absolute path without the `\\?\` prefix: the shell parses display paths, and rejects
/// extended-length ones.
pub(super) fn to_shell_wide_path(path: &Path) -> io::Result<Vec<u16>> {
    let absolute_path = std::path::absolute(path)?;
    let shell_path = remove_extended_length_prefix(&absolute_path);

    Ok(to_null_terminated_wide_string(shell_path.as_os_str()))
}

/// An extended-length path (`\\?\C:\...`) for file system calls, which then work past the 260
/// character `MAX_PATH` limit.
pub(super) fn to_extended_length_wide_path(path: &Path) -> io::Result<Vec<u16>> {
    let absolute_path = std::path::absolute(path)?;
    let Some(Component::Prefix(prefix)) = absolute_path.components().next() else {
        return Ok(to_null_terminated_wide_string(absolute_path.as_os_str()));
    };

    let path_text = absolute_path.as_os_str().to_string_lossy();
    let extended_length_path = match prefix.kind() {
        Prefix::Disk(_) => PathBuf::from(format!("{VERBATIM_PREFIX}{path_text}")),
        Prefix::UNC(_, _) => {
            let without_leading_separators = path_text.trim_start_matches('\\');
            PathBuf::from(format!("{VERBATIM_UNC_PREFIX}{without_leading_separators}"))
        }
        _ => absolute_path.clone(),
    };

    Ok(to_null_terminated_wide_string(
        extended_length_path.as_os_str(),
    ))
}

fn remove_extended_length_prefix(path: &Path) -> PathBuf {
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

    fn decode(wide_path: &[u16]) -> String {
        String::from_utf16_lossy(&wide_path[..wide_path.len() - 1])
    }

    #[test]
    fn shell_paths_have_no_extended_length_prefix() {
        let wide_path = to_shell_wide_path(Path::new(r"\\?\C:\Photos\a.jpg")).unwrap();

        assert_eq!(decode(&wide_path), r"C:\Photos\a.jpg");
    }

    #[test]
    fn file_system_paths_get_the_extended_length_prefix() {
        let wide_path = to_extended_length_wide_path(Path::new(r"C:\Photos\a.jpg")).unwrap();

        assert_eq!(decode(&wide_path), r"\\?\C:\Photos\a.jpg");
        assert_eq!(wide_path.last(), Some(&0));
    }
}
