use super::file_reference::FileReference;
use super::little_endian_fields::{read_u16, read_u32, read_u64};
use super::record_fixups::{FixupError, apply_update_sequence_fixups};

const FILE_RECORD_SIGNATURE: &[u8; 4] = b"FILE";
const RECORD_IN_USE_FLAG: u16 = 0x0001;
const RECORD_IS_DIRECTORY_FLAG: u16 = 0x0002;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum FileRecordError {
    /// Unused records are often all zeros; records NTFS found corrupt are marked `BAAD`.
    #[error("the record does not start with the FILE signature")]
    MissingSignature,
    #[error("the record header is truncated or inconsistent")]
    InvalidHeader,
    #[error(transparent)]
    FixupFailed(#[from] FixupError),
}

/// The fixed header of an MFT file record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileRecordHeader {
    pub(crate) sequence_number: u16,
    pub(crate) first_attribute_offset: usize,
    flags: u16,
    /// The number of bytes of the record that are in use; attributes end before this.
    pub(crate) used_size: usize,
    /// For an extension record, the base record it belongs to; null for a base record.
    pub(crate) base_record: FileReference,
}

impl FileRecordHeader {
    pub(crate) fn is_in_use(self) -> bool {
        self.flags & RECORD_IN_USE_FLAG != 0
    }

    pub(crate) fn is_directory(self) -> bool {
        self.flags & RECORD_IS_DIRECTORY_FLAG != 0
    }

    pub(crate) fn is_base_record(self) -> bool {
        self.base_record.is_null()
    }
}

/// Checks the signature, applies the fixups in place and parses the header of one record.
pub(crate) fn prepare_file_record(record: &mut [u8]) -> Result<FileRecordHeader, FileRecordError> {
    if record.get(..4) != Some(FILE_RECORD_SIGNATURE.as_slice()) {
        return Err(FileRecordError::MissingSignature);
    }
    apply_update_sequence_fixups(record)?;

    let header = FileRecordHeader {
        sequence_number: read_u16(record, 16).ok_or(FileRecordError::InvalidHeader)?,
        first_attribute_offset: read_u16(record, 20).ok_or(FileRecordError::InvalidHeader)?
            as usize,
        flags: read_u16(record, 22).ok_or(FileRecordError::InvalidHeader)?,
        used_size: read_u32(record, 24).ok_or(FileRecordError::InvalidHeader)? as usize,
        base_record: FileReference::from_raw(
            read_u64(record, 32).ok_or(FileRecordError::InvalidHeader)?,
        ),
    };

    let is_consistent = header.used_size <= record.len()
        && header.first_attribute_offset >= 42
        && header.first_attribute_offset < header.used_size;
    if !is_consistent {
        return Err(FileRecordError::InvalidHeader);
    }

    Ok(header)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::test_fixtures::RecordBuilder;

    #[test]
    fn parses_the_header_of_a_directory_record() {
        let mut record = RecordBuilder::new(7).directory().build();

        let header = prepare_file_record(&mut record).unwrap();

        assert_eq!(header.sequence_number, 7);
        assert!(header.is_in_use());
        assert!(header.is_directory());
        assert!(header.is_base_record());
    }

    #[test]
    fn recognizes_a_record_that_is_not_in_use() {
        let mut record = RecordBuilder::new(2).not_in_use().build();

        let header = prepare_file_record(&mut record).unwrap();

        assert!(!header.is_in_use());
    }

    #[test]
    fn rejects_records_without_signature() {
        let mut empty_record = vec![0_u8; 1024];
        let mut bad_record = RecordBuilder::new(1).build();
        bad_record[..4].copy_from_slice(b"BAAD");

        assert_eq!(
            prepare_file_record(&mut empty_record),
            Err(FileRecordError::MissingSignature)
        );
        assert_eq!(
            prepare_file_record(&mut bad_record),
            Err(FileRecordError::MissingSignature)
        );
    }

    #[test]
    fn reads_the_base_record_of_an_extension_record() {
        let base = FileReference::from_parts(40, 3);
        let mut record = RecordBuilder::new(1).extension_of(base).build();

        let header = prepare_file_record(&mut record).unwrap();

        assert!(!header.is_base_record());
        assert_eq!(header.base_record, base);
    }
}
