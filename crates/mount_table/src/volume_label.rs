use std::path::Path;

/// udev keeps one link per labelled file system here, named after the label.
const LABEL_LINKS_DIRECTORY: &str = "/dev/disk/by-label";

/// The file system label of the device node `source` (for example `/dev/sdb1`), as udev
/// publishes it. `None` when the file system has no label or the source is not a device.
#[must_use]
pub fn volume_label_of_source(source: &str) -> Option<String> {
    if !is_device_node_path(source) {
        return None;
    }
    let source_device = std::fs::canonicalize(source).ok()?;
    let label_links = std::fs::read_dir(LABEL_LINKS_DIRECTORY).ok()?;

    label_links
        .filter_map(Result::ok)
        .find(|link| std::fs::canonicalize(link.path()).is_ok_and(|target| target == source_device))
        .map(|link| unescape_udev_label(&link.file_name().to_string_lossy()))
}

/// udev writes characters that are not safe in file names as `\xNN`, for example a space as
/// `\x20`.
fn unescape_udev_label(link_name: &str) -> String {
    let bytes = link_name.as_bytes();
    let mut unescaped = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let escaped_byte = bytes
            .get(index..index + 4)
            .filter(|candidate| candidate.starts_with(b"\\x"))
            .and_then(|candidate| std::str::from_utf8(&candidate[2..]).ok())
            .and_then(|hex_digits| u8::from_str_radix(hex_digits, 16).ok());

        if let Some(value) = escaped_byte {
            unescaped.push(value);
            index += 4;
        } else {
            unescaped.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8_lossy(&unescaped).into_owned()
}

/// True when `path` names something under `/dev`, which is the only kind of source that can
/// have a label.
fn is_device_node_path(path: &str) -> bool {
    Path::new(path).starts_with("/dev")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescapes_spaces_and_keeps_other_text() {
        assert_eq!(unescape_udev_label(r"My\x20Stick"), "My Stick");
        assert_eq!(unescape_udev_label("DATA"), "DATA");
        assert_eq!(unescape_udev_label(r"broken\x2"), r"broken\x2");
    }

    #[test]
    fn recognizes_device_node_paths() {
        assert!(is_device_node_path("/dev/sda1"));
        assert!(!is_device_node_path("server:/export"));
        assert!(!is_device_node_path("/device"));
    }
}
