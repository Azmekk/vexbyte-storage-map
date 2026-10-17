/// An NTFS file reference: the MFT record number in the low 48 bits and the record's sequence
/// number in the high 16 bits. NTFS increments the sequence number whenever a record is
/// reused, so a reference whose sequence number differs from the record's points to a file
/// that no longer exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FileReference(u64);

const RECORD_NUMBER_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Record 5 is always the root directory of an NTFS volume.
pub(crate) const ROOT_DIRECTORY_RECORD_NUMBER: u64 = 5;
/// Records 0 to 15 are reserved for NTFS metafiles such as `$MFT`, `$LogFile` and `$Bitmap`.
pub(crate) const FIRST_ORDINARY_RECORD_NUMBER: u64 = 16;

impl FileReference {
    pub(crate) const fn from_raw(raw_reference: u64) -> Self {
        Self(raw_reference)
    }

    pub(crate) const fn from_parts(record_number: u64, sequence_number: u16) -> Self {
        Self((record_number & RECORD_NUMBER_MASK) | ((sequence_number as u64) << 48))
    }

    pub(crate) const fn record_number(self) -> u64 {
        self.0 & RECORD_NUMBER_MASK
    }

    pub(crate) const fn sequence_number(self) -> u16 {
        (self.0 >> 48) as u16
    }

    pub(crate) const fn is_null(self) -> bool {
        self.0 == 0
    }

    #[cfg(test)]
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_record_and_sequence_numbers() {
        let reference = FileReference::from_raw(0x0003_0000_0000_0010);

        assert_eq!(reference.record_number(), 16);
        assert_eq!(reference.sequence_number(), 3);
        assert_eq!(FileReference::from_parts(16, 3), reference);
        assert!(FileReference::from_raw(0).is_null());
    }
}
