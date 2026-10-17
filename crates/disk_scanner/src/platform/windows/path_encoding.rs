use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

const EXTENDED_LENGTH_PREFIX: &str = r"\\?\";
const DEVICE_NAMESPACE_PREFIX: &str = r"\\.\";
const UNC_PREFIX: &str = r"\\";
const EXTENDED_LENGTH_UNC_PREFIX: &str = r"\\?\UNC\";

/// Encodes an absolute path as a null-terminated UTF-16 extended-length path (`\\?\C:\...` or
/// `\\?\UNC\server\share\...`), so paths longer than 260 characters can be opened.
///
/// Extended-length paths skip all Win32 normalization, so `path` must already be absolute and
/// use backslashes without `.` or `..` components, as `std::path::absolute` produces.
pub(super) fn to_extended_length_wide_path(path: &Path) -> Vec<u16> {
    let path_text = path.as_os_str();
    let prefixed_path = extended_length_form(path_text);

    prefixed_path
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn extended_length_form(path_text: &OsStr) -> std::ffi::OsString {
    let lossy_text = path_text.to_string_lossy();

    if lossy_text.starts_with(EXTENDED_LENGTH_PREFIX)
        || lossy_text.starts_with(DEVICE_NAMESPACE_PREFIX)
    {
        return path_text.to_os_string();
    }

    let mut prefixed = std::ffi::OsString::new();
    if let Some(server_and_share) = lossy_text.strip_prefix(UNC_PREFIX) {
        prefixed.push(EXTENDED_LENGTH_UNC_PREFIX);
        prefixed.push(server_and_share);
    } else {
        prefixed.push(EXTENDED_LENGTH_PREFIX);
        prefixed.push(path_text);
    }
    prefixed
}

/// Encodes a path as a null-terminated UTF-16 string without adding any prefix.
pub(super) fn to_null_terminated_wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(wide_path: &[u16]) -> String {
        String::from_utf16_lossy(&wide_path[..wide_path.len() - 1])
    }

    #[test]
    fn prefixes_a_drive_path() {
        let wide_path = to_extended_length_wide_path(Path::new(r"C:\Users\someone"));

        assert_eq!(decode(&wide_path), r"\\?\C:\Users\someone");
        assert_eq!(wide_path.last(), Some(&0));
    }

    #[test]
    fn rewrites_a_unc_path() {
        let wide_path = to_extended_length_wide_path(Path::new(r"\\server\share\folder"));

        assert_eq!(decode(&wide_path), r"\\?\UNC\server\share\folder");
    }

    #[test]
    fn keeps_a_path_that_is_already_extended() {
        let wide_path = to_extended_length_wide_path(Path::new(r"\\?\D:\data"));

        assert_eq!(decode(&wide_path), r"\\?\D:\data");
    }
}
