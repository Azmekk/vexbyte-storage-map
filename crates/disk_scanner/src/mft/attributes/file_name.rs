use super::RawAttribute;
use crate::mft::file_reference::FileReference;
use crate::mft::little_endian_fields::{read_u8, read_u64, read_utf16_into};

const NAME_LENGTH_FIELD: usize = 64;
const NAMESPACE_FIELD: usize = 65;
const NAME_TEXT_OFFSET: usize = 66;

/// Which naming rules a `$FILE_NAME` follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileNameNamespace {
    Posix,
    Win32,
    /// The generated 8.3 short name (`PROGRA~1`). It duplicates a Win32 name and is skipped.
    Dos,
    /// A name that is valid both as Win32 and as 8.3 name, stored once.
    Win32AndDos,
}

/// The header of a `$FILE_NAME` attribute; the name itself is decoded into a caller buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileNameHeader {
    pub(crate) parent_directory: FileReference,
    pub(crate) namespace: FileNameNamespace,
}

impl FileNameHeader {
    /// NTFS stores up to one DOS 8.3 name next to a long name; only the long name is a path
    /// the user knows. Every other name of a record is a separate hard link.
    pub(crate) fn is_dos_only_name(self) -> bool {
        self.namespace == FileNameNamespace::Dos
    }
}

/// Parses a `$FILE_NAME` attribute and decodes its name into `name`.
pub(crate) fn parse_file_name(
    attribute: &RawAttribute<'_>,
    name: &mut String,
) -> Option<FileNameHeader> {
    let value = attribute.resident_value()?;
    let parent_directory = FileReference::from_raw(read_u64(value, 0)?);
    let name_length = read_u8(value, NAME_LENGTH_FIELD)? as usize;
    let namespace = match read_u8(value, NAMESPACE_FIELD)? {
        0 => FileNameNamespace::Posix,
        1 => FileNameNamespace::Win32,
        2 => FileNameNamespace::Dos,
        3 => FileNameNamespace::Win32AndDos,
        _ => return None,
    };
    read_utf16_into(value, NAME_TEXT_OFFSET, name_length, name)?;

    Some(FileNameHeader {
        parent_directory,
        namespace,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::attributes::attributes_of;
    use crate::mft::file_record::prepare_file_record;
    use crate::mft::test_fixtures::RecordBuilder;

    #[test]
    fn reads_every_name_of_a_record_with_namespace_and_parent() {
        let parent = FileReference::from_parts(5, 5);
        let other_parent = FileReference::from_parts(40, 2);
        let mut record = RecordBuilder::new(1)
            .file_name(parent, FileNameNamespace::Win32, "Long Report Name.docx")
            .file_name(parent, FileNameNamespace::Dos, "LONGRE~1.DOC")
            .file_name(other_parent, FileNameNamespace::Posix, "hard link.docx")
            .build();
        let header = prepare_file_record(&mut record).unwrap();

        let mut name = String::new();
        let mut names = Vec::new();
        for attribute in attributes_of(&record, &header) {
            let file_name = parse_file_name(&attribute, &mut name).unwrap();
            names.push((file_name, name.clone()));
        }

        assert_eq!(names.len(), 3);
        assert_eq!(names[0].1, "Long Report Name.docx");
        assert_eq!(names[0].0.parent_directory, parent);
        assert!(names[1].0.is_dos_only_name());
        assert_eq!(names[2].0.namespace, FileNameNamespace::Posix);
        assert_eq!(names[2].0.parent_directory, other_parent);
        assert_eq!(names[2].1, "hard link.docx");
    }
}
