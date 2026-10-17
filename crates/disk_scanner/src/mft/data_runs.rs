/// A contiguous piece of a non-resident attribute: `cluster_count` clusters starting at
/// `first_cluster`, or a sparse hole without clusters when `first_cluster` is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DataRun {
    pub(crate) first_cluster: Option<u64>,
    pub(crate) cluster_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataRunError {
    #[error("a data run is cut off at byte {offset}")]
    Truncated { offset: usize },
    #[error("a data run at byte {offset} has an invalid field size")]
    InvalidFieldSize { offset: usize },
    #[error("a data run at byte {offset} points before the start of the volume")]
    NegativeCluster { offset: usize },
}

/// Decodes the mapping pairs ("data runs") of a non-resident attribute.
///
/// Each run starts with a header byte: the low nibble is the size in bytes of the run length,
/// the high nibble the size of the cluster offset. The offset is signed and relative to the
/// previous run's first cluster; an offset size of 0 marks a sparse run. A zero header byte
/// ends the list.
pub(crate) fn decode_data_runs(encoded_runs: &[u8]) -> Result<Vec<DataRun>, DataRunError> {
    let mut runs = Vec::new();
    let mut offset = 0;
    let mut previous_first_cluster: i64 = 0;

    loop {
        let header = *encoded_runs
            .get(offset)
            .ok_or(DataRunError::Truncated { offset })?;
        if header == 0 {
            return Ok(runs);
        }
        let length_field_size = usize::from(header & 0x0F);
        let offset_field_size = usize::from(header >> 4);
        if length_field_size == 0 || length_field_size > 8 || offset_field_size > 8 {
            return Err(DataRunError::InvalidFieldSize { offset });
        }

        let length_bytes = encoded_runs
            .get(offset + 1..offset + 1 + length_field_size)
            .ok_or(DataRunError::Truncated { offset })?;
        let cluster_count = read_unsigned_little_endian(length_bytes);

        let offset_start = offset + 1 + length_field_size;
        let offset_bytes = encoded_runs
            .get(offset_start..offset_start + offset_field_size)
            .ok_or(DataRunError::Truncated { offset })?;

        let first_cluster = if offset_field_size == 0 {
            None
        } else {
            let relative_offset = read_signed_little_endian(offset_bytes);
            previous_first_cluster = previous_first_cluster
                .checked_add(relative_offset)
                .filter(|&cluster| cluster >= 0)
                .ok_or(DataRunError::NegativeCluster { offset })?;
            Some(previous_first_cluster.cast_unsigned())
        };

        runs.push(DataRun {
            first_cluster,
            cluster_count,
        });
        offset = offset_start + offset_field_size;
    }
}

fn read_unsigned_little_endian(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .rev()
        .fold(0_u64, |value, &byte| (value << 8) | u64::from(byte))
}

fn read_signed_little_endian(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let unsigned_value = read_unsigned_little_endian(bytes);
    let stored_bits = 8 * u32::try_from(bytes.len()).unwrap_or(8);
    let unused_bits = 64 - stored_bits;
    // Shifting left then arithmetically right sign-extends the highest stored bit.
    (unsigned_value << unused_bits).cast_signed() >> unused_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_runs_with_relative_offsets() {
        // 0x18 clusters at 0x5634, then 0x20 clusters 0x100 clusters earlier, then 3 sparse.
        let encoded = [
            0x21, 0x18, 0x34, 0x56, 0x21, 0x20, 0x00, 0xFF, 0x01, 0x03, 0x00,
        ];

        let runs = decode_data_runs(&encoded).unwrap_or_default();

        assert_eq!(
            runs,
            [
                DataRun {
                    first_cluster: Some(0x5634),
                    cluster_count: 0x18
                },
                DataRun {
                    first_cluster: Some(0x5534),
                    cluster_count: 0x20
                },
                DataRun {
                    first_cluster: None,
                    cluster_count: 3
                },
            ]
        );
    }

    #[test]
    fn rejects_truncated_and_invalid_runs() {
        assert_eq!(
            decode_data_runs(&[0x21, 0x18, 0x34]),
            Err(DataRunError::Truncated { offset: 0 })
        );
        assert_eq!(
            decode_data_runs(&[0x09, 0x01]),
            Err(DataRunError::InvalidFieldSize { offset: 0 })
        );
        assert_eq!(
            decode_data_runs(&[0x11, 0x01, 0xF0, 0x00]),
            Err(DataRunError::NegativeCluster { offset: 0 })
        );
        assert_eq!(
            decode_data_runs(&[]),
            Err(DataRunError::Truncated { offset: 0 })
        );
    }

    #[test]
    fn sign_extends_offsets() {
        assert_eq!(read_signed_little_endian(&[0xFF]), -1);
        assert_eq!(read_signed_little_endian(&[0x00, 0x80]), -32768);
        assert_eq!(read_signed_little_endian(&[0x7F]), 127);
    }
}
