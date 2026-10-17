//! The attributes of an MFT record and parsers for the few this scanner needs.

pub(crate) mod attribute_list;
pub(crate) mod data;
pub(crate) mod file_name;
pub(crate) mod reparse_point;
pub(crate) mod standard_information;

use super::file_record::FileRecordHeader;
use super::little_endian_fields::{read_u8, read_u16, read_u32, read_u64};

pub(crate) const STANDARD_INFORMATION_TYPE: u32 = 0x10;
pub(crate) const ATTRIBUTE_LIST_TYPE: u32 = 0x20;
pub(crate) const FILE_NAME_TYPE: u32 = 0x30;
pub(crate) const DATA_TYPE: u32 = 0x80;
pub(crate) const REPARSE_POINT_TYPE: u32 = 0xC0;
const END_OF_ATTRIBUTES_MARKER: u32 = 0xFFFF_FFFF;

const ATTRIBUTE_COMPRESSED_FLAG: u16 = 0x0001;
const ATTRIBUTE_SPARSE_FLAG: u16 = 0x8000;
/// The smallest possible attribute: a resident header without value.
const MINIMUM_ATTRIBUTE_LENGTH: usize = 24;

/// One attribute of a record, still in its raw form.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RawAttribute<'record> {
    pub(crate) type_code: u32,
    bytes: &'record [u8],
}

/// The header fields of a non-resident attribute, whose value lives in clusters elsewhere on
/// the volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NonResidentHeader<'record> {
    pub(crate) starting_virtual_cluster: u64,
    pub(crate) data_runs: &'record [u8],
    pub(crate) allocated_size: u64,
    pub(crate) data_size: u64,
    /// Present for compressed and sparse attributes: the clusters actually allocated.
    pub(crate) total_allocated_size: Option<u64>,
}

impl<'record> RawAttribute<'record> {
    pub(crate) fn is_non_resident(&self) -> bool {
        read_u8(self.bytes, 8) == Some(1)
    }

    pub(crate) fn name_length(&self) -> u8 {
        read_u8(self.bytes, 9).unwrap_or(0)
    }

    pub(crate) fn is_unnamed(&self) -> bool {
        self.name_length() == 0
    }

    fn flags(&self) -> u16 {
        read_u16(self.bytes, 12).unwrap_or(0)
    }

    pub(crate) fn is_compressed_or_sparse(&self) -> bool {
        self.flags() & (ATTRIBUTE_COMPRESSED_FLAG | ATTRIBUTE_SPARSE_FLAG) != 0
    }

    /// The value of a resident attribute, stored inside the record itself.
    pub(crate) fn resident_value(&self) -> Option<&'record [u8]> {
        if self.is_non_resident() {
            return None;
        }
        let value_length = read_u32(self.bytes, 16)? as usize;
        let value_offset = read_u16(self.bytes, 20)? as usize;
        self.bytes
            .get(value_offset..value_offset.checked_add(value_length)?)
    }

    pub(crate) fn non_resident_header(&self) -> Option<NonResidentHeader<'record>> {
        if !self.is_non_resident() {
            return None;
        }
        let data_runs_offset = read_u16(self.bytes, 32)? as usize;
        let total_allocated_size = if self.is_compressed_or_sparse() {
            read_u64(self.bytes, 64)
        } else {
            None
        };

        Some(NonResidentHeader {
            starting_virtual_cluster: read_u64(self.bytes, 16)?,
            data_runs: self.bytes.get(data_runs_offset..)?,
            allocated_size: read_u64(self.bytes, 40)?,
            data_size: read_u64(self.bytes, 48)?,
            total_allocated_size,
        })
    }
}

/// Iterates the attributes of a record that has already been prepared with fixups. Stops at
/// the end marker or at the first attribute that does not fit the record.
pub(crate) fn attributes_of<'record>(
    record: &'record [u8],
    header: &FileRecordHeader,
) -> AttributeIterator<'record> {
    let used_part = record.get(..header.used_size).unwrap_or(&[]);
    AttributeIterator {
        record: used_part,
        next_offset: header.first_attribute_offset,
    }
}

pub(crate) struct AttributeIterator<'record> {
    record: &'record [u8],
    next_offset: usize,
}

impl<'record> Iterator for AttributeIterator<'record> {
    type Item = RawAttribute<'record>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.next_offset;
        let type_code = read_u32(self.record, offset)?;
        if type_code == END_OF_ATTRIBUTES_MARKER {
            return None;
        }
        let length = read_u32(self.record, offset + 4)? as usize;
        if length < MINIMUM_ATTRIBUTE_LENGTH {
            return None;
        }
        let bytes = self.record.get(offset..offset.checked_add(length)?)?;

        self.next_offset = offset + length;
        Some(RawAttribute { type_code, bytes })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::file_record::prepare_file_record;
    use crate::mft::test_fixtures::RecordBuilder;

    #[test]
    fn iterates_attributes_until_the_end_marker() {
        let mut record = RecordBuilder::new(1)
            .standard_information(0, 0)
            .resident_data(b"hello")
            .build();
        let header = prepare_file_record(&mut record).unwrap();

        let attributes: Vec<_> = attributes_of(&record, &header).collect();

        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].type_code, STANDARD_INFORMATION_TYPE);
        assert_eq!(attributes[1].type_code, DATA_TYPE);
        assert_eq!(attributes[1].resident_value(), Some(b"hello".as_slice()));
        assert!(attributes[1].is_unnamed());
    }

    #[test]
    fn reads_non_resident_headers() {
        let mut record = RecordBuilder::new(1)
            .non_resident_data(0, 8192, 5000, &[0x11, 0x02, 0x10, 0x00])
            .build();
        let header = prepare_file_record(&mut record).unwrap();

        let data = attributes_of(&record, &header).next().unwrap();
        let non_resident = data.non_resident_header().unwrap();

        assert_eq!(non_resident.allocated_size, 8192);
        assert_eq!(non_resident.data_size, 5000);
        assert_eq!(non_resident.starting_virtual_cluster, 0);
        assert_eq!(&non_resident.data_runs[..4], &[0x11, 0x02, 0x10, 0x00]);
        assert_eq!(non_resident.total_allocated_size, None);
    }
}
