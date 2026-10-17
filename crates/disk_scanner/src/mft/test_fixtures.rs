//! Builds raw MFT records and small in-memory NTFS volumes for tests, byte by byte, following
//! the on-disk format. Real MFT dumps cannot be shipped, so every fixture is synthesized.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::io;

use super::attributes::file_name::FileNameNamespace;
use super::attributes::{
    ATTRIBUTE_LIST_TYPE, DATA_TYPE, FILE_NAME_TYPE, REPARSE_POINT_TYPE, STANDARD_INFORMATION_TYPE,
};
use super::file_reference::FileReference;
use super::mft_chunk_reader::VolumeBytes;
use super::record_fixups::UPDATE_SEQUENCE_STRIDE;

pub(crate) const RECORD_SIZE: usize = 1024;
const UPDATE_SEQUENCE_ARRAY_OFFSET: usize = 48;
const FIRST_ATTRIBUTE_OFFSET: usize = 56;
const RECORD_IN_USE_FLAG: u16 = 1;
const RECORD_IS_DIRECTORY_FLAG: u16 = 2;
const ATTRIBUTE_COMPRESSED_FLAG: u16 = 0x0001;

fn align_to_eight(length: usize) -> usize {
    length.next_multiple_of(8)
}

fn utf16_bytes(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// Builds one MFT file record.
#[derive(Clone)]
pub(crate) struct RecordBuilder {
    sequence_number: u16,
    flags: u16,
    base_record: u64,
    attributes: Vec<Vec<u8>>,
}

impl RecordBuilder {
    /// An in-use file record without attributes.
    pub(crate) fn new(sequence_number: u16) -> Self {
        Self {
            sequence_number,
            flags: RECORD_IN_USE_FLAG,
            base_record: 0,
            attributes: Vec::new(),
        }
    }

    pub(crate) fn directory(mut self) -> Self {
        self.flags |= RECORD_IS_DIRECTORY_FLAG;
        self
    }

    pub(crate) fn not_in_use(mut self) -> Self {
        self.flags &= !RECORD_IN_USE_FLAG;
        self
    }

    pub(crate) fn extension_of(mut self, base_record: FileReference) -> Self {
        self.base_record = base_record.raw();
        self
    }

    pub(crate) fn standard_information(self, modified_time: u64, file_attributes: u32) -> Self {
        let mut value = vec![0_u8; 72];
        value[8..16].copy_from_slice(&modified_time.to_le_bytes());
        value[32..36].copy_from_slice(&file_attributes.to_le_bytes());
        self.with_resident_attribute(STANDARD_INFORMATION_TYPE, "", &value)
    }

    pub(crate) fn file_name(
        self,
        parent_directory: FileReference,
        namespace: FileNameNamespace,
        name: &str,
    ) -> Self {
        let name_bytes = utf16_bytes(name);
        let mut value = vec![0_u8; 66];
        value[0..8].copy_from_slice(&parent_directory.raw().to_le_bytes());
        value[64] = u8::try_from(name.encode_utf16().count()).unwrap();
        value[65] = match namespace {
            FileNameNamespace::Posix => 0,
            FileNameNamespace::Win32 => 1,
            FileNameNamespace::Dos => 2,
            FileNameNamespace::Win32AndDos => 3,
        };
        value.extend_from_slice(&name_bytes);
        self.with_resident_attribute(FILE_NAME_TYPE, "", &value)
    }

    pub(crate) fn resident_data(self, data: &[u8]) -> Self {
        self.with_resident_attribute(DATA_TYPE, "", data)
    }

    pub(crate) fn named_resident_data(self, stream_name: &str, data: &[u8]) -> Self {
        self.with_resident_attribute(DATA_TYPE, stream_name, data)
    }

    pub(crate) fn non_resident_data(
        self,
        starting_virtual_cluster: u64,
        allocated_size: u64,
        data_size: u64,
        encoded_runs: &[u8],
    ) -> Self {
        let attribute = non_resident_attribute(&NonResidentAttribute {
            type_code: DATA_TYPE,
            flags: 0,
            starting_virtual_cluster,
            allocated_size,
            data_size,
            total_allocated_size: None,
            encoded_runs,
        });
        self.with_attribute(attribute)
    }

    pub(crate) fn compressed_non_resident_data(
        self,
        allocated_size: u64,
        data_size: u64,
        total_allocated_size: u64,
    ) -> Self {
        let attribute = non_resident_attribute(&NonResidentAttribute {
            type_code: DATA_TYPE,
            flags: ATTRIBUTE_COMPRESSED_FLAG,
            starting_virtual_cluster: 0,
            allocated_size,
            data_size,
            total_allocated_size: Some(total_allocated_size),
            encoded_runs: &[0x11, 0x04, 0x20, 0x00],
        });
        self.with_attribute(attribute)
    }

    /// Entries are `(attribute type, starting virtual cluster, record holding it)`.
    pub(crate) fn attribute_list(self, entries: &[(u32, u64, FileReference)]) -> Self {
        let mut value = Vec::new();
        for (attribute_id, &(attribute_type, starting_virtual_cluster, record)) in
            entries.iter().enumerate()
        {
            let mut entry = vec![0_u8; 32];
            entry[0..4].copy_from_slice(&attribute_type.to_le_bytes());
            entry[4..6].copy_from_slice(&32_u16.to_le_bytes());
            entry[7] = 26;
            entry[8..16].copy_from_slice(&starting_virtual_cluster.to_le_bytes());
            entry[16..24].copy_from_slice(&record.raw().to_le_bytes());
            entry[24..26].copy_from_slice(&(attribute_id as u16).to_le_bytes());
            value.extend_from_slice(&entry);
        }
        self.with_resident_attribute(ATTRIBUTE_LIST_TYPE, "", &value)
    }

    pub(crate) fn reparse_point(self, reparse_tag: u32) -> Self {
        let mut value = vec![0_u8; 8];
        value[0..4].copy_from_slice(&reparse_tag.to_le_bytes());
        self.with_resident_attribute(REPARSE_POINT_TYPE, "", &value)
    }

    fn with_resident_attribute(self, type_code: u32, name: &str, value: &[u8]) -> Self {
        let name_bytes = utf16_bytes(name);
        let value_offset = align_to_eight(24 + name_bytes.len());
        let length = align_to_eight(value_offset + value.len());

        let mut attribute = vec![0_u8; length];
        attribute[0..4].copy_from_slice(&type_code.to_le_bytes());
        attribute[4..8].copy_from_slice(&(length as u32).to_le_bytes());
        attribute[9] = name.encode_utf16().count() as u8;
        attribute[10..12].copy_from_slice(&24_u16.to_le_bytes());
        attribute[14..16].copy_from_slice(&(self.attributes.len() as u16).to_le_bytes());
        attribute[16..20].copy_from_slice(&(value.len() as u32).to_le_bytes());
        attribute[20..22].copy_from_slice(&(value_offset as u16).to_le_bytes());
        attribute[24..24 + name_bytes.len()].copy_from_slice(&name_bytes);
        attribute[value_offset..value_offset + value.len()].copy_from_slice(value);
        self.with_attribute(attribute)
    }

    fn with_attribute(mut self, attribute: Vec<u8>) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// The record as NTFS writes it to disk, with the update sequence fixups applied.
    pub(crate) fn build(&self) -> Vec<u8> {
        let mut record = vec![0_u8; RECORD_SIZE];
        record[0..4].copy_from_slice(b"FILE");
        record[4..6].copy_from_slice(&(UPDATE_SEQUENCE_ARRAY_OFFSET as u16).to_le_bytes());
        let update_sequence_entries = RECORD_SIZE / UPDATE_SEQUENCE_STRIDE + 1;
        record[6..8].copy_from_slice(&(update_sequence_entries as u16).to_le_bytes());
        record[16..18].copy_from_slice(&self.sequence_number.to_le_bytes());
        record[18..20].copy_from_slice(&1_u16.to_le_bytes());
        record[20..22].copy_from_slice(&(FIRST_ATTRIBUTE_OFFSET as u16).to_le_bytes());
        record[22..24].copy_from_slice(&self.flags.to_le_bytes());
        record[28..32].copy_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
        record[32..40].copy_from_slice(&self.base_record.to_le_bytes());

        let mut offset = FIRST_ATTRIBUTE_OFFSET;
        for attribute in &self.attributes {
            record[offset..offset + attribute.len()].copy_from_slice(attribute);
            offset += attribute.len();
        }
        record[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        let used_size = offset + 8;
        assert!(used_size <= RECORD_SIZE, "the test record is too large");
        record[24..28].copy_from_slice(&(used_size as u32).to_le_bytes());

        protect_with_update_sequence(&mut record, 0x0001);
        record
    }
}

struct NonResidentAttribute<'runs> {
    type_code: u32,
    flags: u16,
    starting_virtual_cluster: u64,
    allocated_size: u64,
    data_size: u64,
    total_allocated_size: Option<u64>,
    encoded_runs: &'runs [u8],
}

fn non_resident_attribute(description: &NonResidentAttribute<'_>) -> Vec<u8> {
    let header_length = if description.total_allocated_size.is_some() {
        72
    } else {
        64
    };
    let length = align_to_eight(header_length + description.encoded_runs.len());
    let cluster_count = description.allocated_size.div_ceil(4096).max(1);
    let last_virtual_cluster = description.starting_virtual_cluster + cluster_count - 1;

    let mut attribute = vec![0_u8; length];
    attribute[0..4].copy_from_slice(&description.type_code.to_le_bytes());
    attribute[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    attribute[8] = 1;
    attribute[10..12].copy_from_slice(&(header_length as u16).to_le_bytes());
    attribute[12..14].copy_from_slice(&description.flags.to_le_bytes());
    attribute[16..24].copy_from_slice(&description.starting_virtual_cluster.to_le_bytes());
    attribute[24..32].copy_from_slice(&last_virtual_cluster.to_le_bytes());
    attribute[32..34].copy_from_slice(&(header_length as u16).to_le_bytes());
    if description.flags & ATTRIBUTE_COMPRESSED_FLAG != 0 {
        attribute[34..36].copy_from_slice(&4_u16.to_le_bytes());
    }
    attribute[40..48].copy_from_slice(&description.allocated_size.to_le_bytes());
    attribute[48..56].copy_from_slice(&description.data_size.to_le_bytes());
    attribute[56..64].copy_from_slice(&description.data_size.to_le_bytes());
    if let Some(total_allocated_size) = description.total_allocated_size {
        attribute[64..72].copy_from_slice(&total_allocated_size.to_le_bytes());
    }
    attribute[header_length..header_length + description.encoded_runs.len()]
        .copy_from_slice(description.encoded_runs);
    attribute
}

/// Does what NTFS does before writing a record: moves the last two bytes of every 512-byte
/// stride into the update sequence array and replaces them with the update sequence number.
pub(crate) fn protect_with_update_sequence(record: &mut [u8], update_sequence_number: u16) {
    let array_offset = u16::from_le_bytes([record[4], record[5]]) as usize;
    let entry_count = u16::from_le_bytes([record[6], record[7]]) as usize;
    record[array_offset..array_offset + 2].copy_from_slice(&update_sequence_number.to_le_bytes());

    for stride in 0..entry_count - 1 {
        let stride_end = (stride + 1) * UPDATE_SEQUENCE_STRIDE - 2;
        let saved_offset = array_offset + (stride + 1) * 2;
        let original = [record[stride_end], record[stride_end + 1]];
        record[saved_offset..saved_offset + 2].copy_from_slice(&original);
        record[stride_end..stride_end + 2].copy_from_slice(&update_sequence_number.to_le_bytes());
    }
}

/// A volume image held in memory.
pub(crate) struct InMemoryVolume {
    bytes: Vec<u8>,
}

impl InMemoryVolume {
    pub(crate) fn with_size(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }

    pub(crate) fn write(&mut self, offset: usize, data: &[u8]) {
        self.bytes[offset..offset + data.len()].copy_from_slice(data);
    }
}

impl VolumeBytes for InMemoryVolume {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        let start =
            usize::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let available = self.bytes.get(start..).unwrap_or(&[]);
        let length = available.len().min(buffer.len());
        buffer[..length].copy_from_slice(&available[..length]);
        Ok(length)
    }
}
