use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Bytes kept as they are in a file URI path. This is the set that `g_filename_to_uri` leaves
/// unescaped, so the URI (and its MD5 in the freedesktop thumbnail cache) matches the one GNOME,
/// KDE and other thumbnailers compute. It includes the RFC 3986 unreserved characters and '/'.
const UNESCAPED_PUNCTUATION: &[u8] = b"-._~/!$&'()*:=@";

const UPPERCASE_HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// The `file://` URI of the absolute `path`.
pub(super) fn file_uri_of(path: &Path) -> io::Result<String> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "only absolute paths have a file URI",
        ));
    }

    let mut uri = String::from("file://");
    uri.push_str(&percent_encode_path_bytes(path.as_os_str().as_bytes()));
    Ok(uri)
}

/// Escapes every byte outside the unescaped set as `%XX` with uppercase hex digits.
pub(super) fn percent_encode_path_bytes(path_bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(path_bytes.len());

    for &byte in path_bytes {
        if byte.is_ascii_alphanumeric() || UNESCAPED_PUNCTUATION.contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(UPPERCASE_HEX_DIGITS[usize::from(byte >> 4)]));
            encoded.push(char::from(UPPERCASE_HEX_DIGITS[usize::from(byte & 0x0F)]));
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn keeps_unreserved_characters_and_slashes() {
        assert_eq!(
            percent_encode_path_bytes(b"/home/martin/Photos/IMG_0001.v2~.jpg"),
            "/home/martin/Photos/IMG_0001.v2~.jpg"
        );
    }

    #[test]
    fn escapes_spaces_reserved_characters_and_percent_signs() {
        assert_eq!(
            percent_encode_path_bytes(b"/tmp/a b#c?d%e+f,g;h[i]"),
            "/tmp/a%20b%23c%3Fd%25e%2Bf%2Cg%3Bh%5Bi%5D"
        );
    }

    #[test]
    fn keeps_the_punctuation_glib_leaves_unescaped() {
        assert_eq!(
            percent_encode_path_bytes(b"/tmp/photo (1)!$&'*:=@.jpg"),
            "/tmp/photo%20(1)!$&'*:=@.jpg"
        );
    }

    #[test]
    fn escapes_utf8_and_invalid_bytes_one_byte_at_a_time() {
        assert_eq!(
            percent_encode_path_bytes("/tmp/é".as_bytes()),
            "/tmp/%C3%A9"
        );
        assert_eq!(percent_encode_path_bytes(b"/tmp/\xFF\n"), "/tmp/%FF%0A");
    }

    #[test]
    fn builds_file_uris_of_absolute_paths_only() {
        assert_eq!(
            file_uri_of(Path::new("/home/martin/My Photo.png")).unwrap(),
            "file:///home/martin/My%20Photo.png"
        );
        assert_eq!(
            file_uri_of(Path::new(OsStr::from_bytes(b"/tmp/\xE9"))).unwrap(),
            "file:///tmp/%E9"
        );
        assert_eq!(
            file_uri_of(Path::new("relative.png")).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}
