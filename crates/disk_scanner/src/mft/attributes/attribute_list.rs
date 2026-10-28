use super::RawAttribute;
use crate::mft::file_reference::FileReference;
use crate::mft::little_endian_fields::{read_u16, read_u32, read_u64};

const MINIMUM_ENTRY_LENGTH: usize = 26;

/// One entry of an `$ATTRIBUTE_LIST`: which record holds (one extent of) an attribute.
///
/// A file with too many attributes for one record (many hard links, a heavily fragmented
/// file) gets extension records. The base record then lists every attribute with the record
/// that holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AttributeListEntry {
    pub(crate) attribute_type: u32,
    pub(crate) starting_virtual_cluster: u64,
    pub(crate) record: FileReference,
}

/// Parses a resident `$ATTRIBUTE_LIST`. Returns `None` when the list is non-resident; while
/// sweeping the MFT the scanner then relies on the base record references of the extension
/// records alone. Locating the MFT itself reads a non-resident list from the volume instead
/// and parses it with [`parse_attribute_list_bytes`].
pub(crate) fn parse_attribute_list(
    attribute: &RawAttribute<'_>,
) -> Option<Vec<AttributeListEntry>> {
    Some(parse_attribute_list_bytes(attribute.resident_value()?))
}

/// Parses the value of an `$ATTRIBUTE_LIST`, wherever it was read from. Stops at the first
/// entry that is cut off or malformed.
pub(crate) fn parse_attribute_list_bytes(list: &[u8]) -> Vec<AttributeListEntry> {
    let mut entries = Vec::new();
    let mut offset = 0;

    while offset + MINIMUM_ENTRY_LENGTH <= list.len() {
        let Some((entry, entry_length)) = entry_at(list, offset) else {
            break;
        };
        if entry_length < MINIMUM_ENTRY_LENGTH {
            break;
        }
        entries.push(entry);
        offset += entry_length;
    }

    entries
}

fn entry_at(list: &[u8], offset: usize) -> Option<(AttributeListEntry, usize)> {
    let entry = AttributeListEntry {
        attribute_type: read_u32(list, offset)?,
        starting_virtual_cluster: read_u64(list, offset + 8)?,
        record: FileReference::from_raw(read_u64(list, offset + 16)?),
    };
    Some((entry, read_u16(list, offset + 4)? as usize))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::attributes::{DATA_TYPE, FILE_NAME_TYPE, attributes_of};
    use crate::mft::file_record::prepare_file_record;
    use crate::mft::test_fixtures::RecordBuilder;

    #[test]
    fn lists_which_record_holds_each_attribute() {
        let base = FileReference::from_parts(30, 1);
        let extension = FileReference::from_parts(31, 1);
        let mut record = RecordBuilder::new(1)
            .attribute_list(&[(FILE_NAME_TYPE, 0, base), (DATA_TYPE, 0, extension)])
            .build();
        let header = prepare_file_record(&mut record).unwrap();
        let attribute = attributes_of(&record, &header).next().unwrap();

        let entries = parse_attribute_list(&attribute).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].attribute_type, DATA_TYPE);
        assert_eq!(entries[1].record, extension);
        assert_eq!(entries[1].starting_virtual_cluster, 0);
    }
}
