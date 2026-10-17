use super::little_endian_fields::read_u16;

/// NTFS protects multi-sector structures in 512-byte strides, independent of the physical
/// sector size.
pub(crate) const UPDATE_SEQUENCE_STRIDE: usize = 512;

const UPDATE_SEQUENCE_ARRAY_OFFSET_FIELD: usize = 4;
const UPDATE_SEQUENCE_ARRAY_COUNT_FIELD: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum FixupError {
    #[error("the update sequence array lies outside the record")]
    ArrayOutOfBounds,
    #[error("the update sequence array covers more strides than the record has")]
    StrideCountMismatch,
    #[error("stride {stride} does not end with the update sequence number (torn write)")]
    TornWrite { stride: usize },
}

/// Restores the last two bytes of every 512-byte stride of a record.
///
/// Before writing a record, NTFS replaces the last two bytes of every stride with the update
/// sequence number and keeps the original bytes in the update sequence array. A stride whose
/// last bytes do not match the number was not completely written, so the record is unusable.
pub(crate) fn apply_update_sequence_fixups(record: &mut [u8]) -> Result<(), FixupError> {
    let array_offset = read_u16(record, UPDATE_SEQUENCE_ARRAY_OFFSET_FIELD)
        .ok_or(FixupError::ArrayOutOfBounds)? as usize;
    let array_entry_count = read_u16(record, UPDATE_SEQUENCE_ARRAY_COUNT_FIELD)
        .ok_or(FixupError::ArrayOutOfBounds)? as usize;

    // The first entry is the update sequence number itself; one entry follows per stride.
    let stride_count = array_entry_count
        .checked_sub(1)
        .ok_or(FixupError::StrideCountMismatch)?;
    if stride_count == 0 || stride_count * UPDATE_SEQUENCE_STRIDE > record.len() {
        return Err(FixupError::StrideCountMismatch);
    }
    if array_offset + array_entry_count * 2 > record.len() {
        return Err(FixupError::ArrayOutOfBounds);
    }

    let update_sequence_number =
        read_u16(record, array_offset).ok_or(FixupError::ArrayOutOfBounds)?;

    for stride in 0..stride_count {
        let stride_end = (stride + 1) * UPDATE_SEQUENCE_STRIDE - 2;
        let stored_bytes = read_u16(record, stride_end).ok_or(FixupError::ArrayOutOfBounds)?;
        if stored_bytes != update_sequence_number {
            return Err(FixupError::TornWrite { stride });
        }

        let original_bytes_offset = array_offset + (stride + 1) * 2;
        let original_bytes = [
            record[original_bytes_offset],
            record[original_bytes_offset + 1],
        ];
        record[stride_end..stride_end + 2].copy_from_slice(&original_bytes);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::test_fixtures::protect_with_update_sequence;

    fn record_with_pattern() -> Vec<u8> {
        let mut record: Vec<u8> = (0..1024_u32).map(|value| (value % 251) as u8).collect();
        record[4..6].copy_from_slice(&48_u16.to_le_bytes());
        record[6..8].copy_from_slice(&3_u16.to_le_bytes());
        record
    }

    #[test]
    fn restores_the_original_stride_endings() {
        let original = record_with_pattern();
        let mut protected = original.clone();
        protect_with_update_sequence(&mut protected, 0x0042);
        assert_ne!(protected[510..512], original[510..512]);

        apply_update_sequence_fixups(&mut protected).unwrap();

        assert_eq!(protected[..48], original[..48]);
        assert_eq!(protected[54..], original[54..]);
    }

    #[test]
    fn detects_a_torn_write() {
        let mut protected = record_with_pattern();
        protect_with_update_sequence(&mut protected, 0x0042);
        protected[1022] ^= 0xFF;

        assert_eq!(
            apply_update_sequence_fixups(&mut protected),
            Err(FixupError::TornWrite { stride: 1 })
        );
    }

    #[test]
    fn rejects_an_array_that_claims_too_many_strides() {
        let mut record = record_with_pattern();
        record[6..8].copy_from_slice(&9_u16.to_le_bytes());

        assert_eq!(
            apply_update_sequence_fixups(&mut record),
            Err(FixupError::StrideCountMismatch)
        );
    }
}
