use std::io;
use std::path::PathBuf;

use crate::device_number::DeviceNumber;
use crate::mount_entry::MountEntry;

const MOUNT_INFO_PATH: &str = "/proc/self/mountinfo";
/// Separates the optional fields from the file system fields on every mountinfo line.
const OPTIONAL_FIELDS_END: &str = "-";

/// Reads the mount table of the current process, in mount order (later mounts stack on top).
///
/// # Errors
///
/// Fails when `/proc/self/mountinfo` cannot be read.
pub fn read_mount_table() -> io::Result<Vec<MountEntry>> {
    let mount_info = std::fs::read_to_string(MOUNT_INFO_PATH)?;
    Ok(parse_mount_info(&mount_info))
}

/// Parses the text of a mountinfo file. Malformed lines are skipped.
///
/// Each line looks like
/// `36 35 98:0 /root /mnt rw,noatime master:1 - ext4 /dev/sda1 rw,errors=continue`.
#[must_use]
pub fn parse_mount_info(mount_info: &str) -> Vec<MountEntry> {
    mount_info
        .lines()
        .filter_map(parse_mount_info_line)
        .collect()
}

fn parse_mount_info_line(line: &str) -> Option<MountEntry> {
    let mut fields = line.split(' ');
    let _mount_id = fields.next()?;
    let _parent_id = fields.next()?;
    let device = DeviceNumber::parse(fields.next()?)?;
    let root = unescape_octal(fields.next()?);
    let mount_point = unescape_octal(fields.next()?);
    let _mount_options = fields.next()?;

    let mut fields = fields.skip_while(|field| *field != OPTIONAL_FIELDS_END);
    let _separator = fields.next()?;
    let file_system_type = unescape_octal(fields.next()?);
    let source = unescape_octal(fields.next()?);

    Some(MountEntry {
        mount_point: PathBuf::from(mount_point),
        root: PathBuf::from(root),
        file_system_type,
        source,
        device,
    })
}

/// The kernel writes space, tab, newline and backslash in paths as `\040`, `\011`, `\012` and
/// `\134`.
fn unescape_octal(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut unescaped = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let octal_value = bytes
            .get(index + 1..index + 4)
            .filter(|_| bytes[index] == b'\\')
            .and_then(|digits| std::str::from_utf8(digits).ok())
            .and_then(|digits| u8::from_str_radix(digits, 8).ok());

        if let Some(value) = octal_value {
            unescaped.push(value);
            index += 4;
        } else {
            unescaped.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8_lossy(&unescaped).into_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    const SAMPLE_MOUNT_INFO: &str = "\
22 1 259:2 / / rw,relatime shared:1 - ext4 /dev/nvme0n1p2 rw
23 22 0:21 / /proc rw,nosuid,nodev,noexec,relatime shared:12 - proc proc rw
24 22 0:45 /@home /home rw,relatime shared:30 - btrfs /dev/sda1 rw,subvol=/@home
25 22 8:33 / /media/martin/My\\040Stick rw,nosuid,nodev,relatime shared:40 - exfat /dev/sdc1 rw
26 22 0:50 / /mnt/c rw,noatime - 9p drvfs rw,dirsync
broken line
";

    #[test]
    fn parses_every_well_formed_line() {
        let entries = parse_mount_info(SAMPLE_MOUNT_INFO);

        assert_eq!(entries.len(), 5);
        assert_eq!(
            entries[0],
            MountEntry {
                mount_point: PathBuf::from("/"),
                root: PathBuf::from("/"),
                file_system_type: "ext4".to_owned(),
                source: "/dev/nvme0n1p2".to_owned(),
                device: DeviceNumber {
                    major: 259,
                    minor: 2
                },
            }
        );
        assert_eq!(entries[2].root, PathBuf::from("/@home"));
        assert_eq!(entries[4].file_system_type, "9p");
        assert_eq!(entries[4].source, "drvfs");
    }

    #[test]
    fn unescapes_spaces_in_mount_points() {
        let entries = parse_mount_info(SAMPLE_MOUNT_INFO);

        assert_eq!(
            entries[3].mount_point,
            PathBuf::from("/media/martin/My Stick")
        );
    }

    #[test]
    fn handles_lines_without_optional_fields() {
        let entries =
            parse_mount_info("30 22 0:60 / /mnt/data rw - xfs /dev/mapper/data rw,attr2\n");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, "/dev/mapper/data");
    }

    #[test]
    fn reads_the_mount_table_of_this_machine() {
        let entries = read_mount_table().unwrap();

        assert!(
            entries
                .iter()
                .any(|entry| entry.mount_point == std::path::Path::new("/"))
        );
    }
}
