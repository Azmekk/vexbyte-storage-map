use std::io;

use super::data_runs::DataRun;

/// Reads raw bytes of a volume. Implemented by the platform's read-only volume handle and by
/// an in-memory volume image in tests.
pub(crate) trait VolumeBytes {
    /// Reads up to `buffer.len()` bytes at byte `offset`. Offsets and lengths are multiples
    /// of the sector size.
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize>;
}

/// A contiguous byte range of the MFT on the volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MftExtent {
    pub(crate) volume_offset: u64,
    pub(crate) length: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum MftExtentError {
    #[error("the MFT contains a sparse run, which NTFS never creates")]
    SparseRun,
    #[error("the MFT data runs describe less data than the MFT's valid length")]
    RunsShorterThanValidLength,
}

/// Turns the data runs of `$MFT` into byte ranges, cut off at the MFT's valid data length.
pub(crate) fn mft_extents_from_runs(
    runs: &[DataRun],
    bytes_per_cluster: u32,
    valid_data_length: u64,
) -> Result<Vec<MftExtent>, MftExtentError> {
    let mut extents = Vec::with_capacity(runs.len());
    let mut remaining_length = valid_data_length;

    for run in runs {
        if remaining_length == 0 {
            break;
        }
        let first_cluster = run.first_cluster.ok_or(MftExtentError::SparseRun)?;
        let run_length = run
            .cluster_count
            .saturating_mul(u64::from(bytes_per_cluster));
        let length = run_length.min(remaining_length);

        extents.push(MftExtent {
            volume_offset: first_cluster.saturating_mul(u64::from(bytes_per_cluster)),
            length,
        });
        remaining_length -= length;
    }

    if remaining_length > 0 {
        return Err(MftExtentError::RunsShorterThanValidLength);
    }
    Ok(extents)
}

#[derive(Debug, thiserror::Error)]
pub enum MftChunkReadError {
    #[error("reading the MFT at volume offset {offset} failed")]
    ReadFailed { offset: u64, source: io::Error },
    #[error("the volume ended at offset {offset} before the MFT did")]
    UnexpectedEndOfVolume { offset: u64 },
}

/// Reads the MFT front to back in large sequential chunks that always hold whole records.
///
/// Large sequential reads are what makes the MFT scan fast: a few hundred 8 MiB reads instead
/// of a million small ones. A record that straddles two extents or two reads is carried over
/// to the next chunk.
pub(crate) struct MftChunkReader<'volume, Volume: VolumeBytes> {
    volume: &'volume Volume,
    extents: Vec<MftExtent>,
    current_extent: usize,
    offset_in_extent: u64,
    chunk_size: usize,
    record_size: usize,
    carried_over_bytes: Vec<u8>,
    next_record_number: u64,
}

impl<'volume, Volume: VolumeBytes> MftChunkReader<'volume, Volume> {
    /// `chunk_size` is rounded to a whole number of records.
    pub(crate) fn new(
        volume: &'volume Volume,
        extents: Vec<MftExtent>,
        chunk_size: usize,
        record_size: usize,
    ) -> Self {
        let chunk_size = chunk_size.max(record_size).next_multiple_of(record_size);

        Self {
            volume,
            extents,
            current_extent: 0,
            offset_in_extent: 0,
            chunk_size,
            record_size,
            carried_over_bytes: Vec::new(),
            next_record_number: 0,
        }
    }

    /// Fills `chunk` with the next whole records and returns the number of the first one, or
    /// `None` once the whole MFT has been read.
    pub(crate) fn read_next_chunk(
        &mut self,
        chunk: &mut Vec<u8>,
    ) -> Result<Option<u64>, MftChunkReadError> {
        chunk.clear();
        chunk.append(&mut self.carried_over_bytes);

        while chunk.len() < self.chunk_size && self.current_extent < self.extents.len() {
            let extent = self.extents[self.current_extent];
            let remaining_in_extent = extent.length - self.offset_in_extent;
            if remaining_in_extent == 0 {
                self.current_extent += 1;
                self.offset_in_extent = 0;
                continue;
            }

            let free_space = (self.chunk_size - chunk.len()) as u64;
            let read_length = usize::try_from(remaining_in_extent.min(free_space)).unwrap_or(0);
            let read_offset = extent.volume_offset + self.offset_in_extent;

            let filled_length = chunk.len();
            chunk.resize(filled_length + read_length, 0);
            let bytes_read = self
                .volume
                .read_at(read_offset, &mut chunk[filled_length..])
                .map_err(|source| MftChunkReadError::ReadFailed {
                    offset: read_offset,
                    source,
                })?;
            chunk.truncate(filled_length + bytes_read);
            if bytes_read == 0 {
                return Err(MftChunkReadError::UnexpectedEndOfVolume {
                    offset: read_offset,
                });
            }
            self.offset_in_extent += bytes_read as u64;
        }

        let whole_records_length = chunk.len() / self.record_size * self.record_size;
        self.carried_over_bytes
            .extend_from_slice(&chunk[whole_records_length..]);
        chunk.truncate(whole_records_length);

        if chunk.is_empty() {
            return Ok(None);
        }
        let first_record_number = self.next_record_number;
        self.next_record_number += (whole_records_length / self.record_size) as u64;
        Ok(Some(first_record_number))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::test_fixtures::InMemoryVolume;

    #[test]
    fn cuts_extents_at_the_valid_data_length() {
        let runs = [
            DataRun {
                first_cluster: Some(10),
                cluster_count: 2,
            },
            DataRun {
                first_cluster: Some(50),
                cluster_count: 4,
            },
        ];

        let extents = mft_extents_from_runs(&runs, 4096, 3 * 4096).unwrap();

        assert_eq!(
            extents,
            [
                MftExtent {
                    volume_offset: 40_960,
                    length: 8192
                },
                MftExtent {
                    volume_offset: 204_800,
                    length: 4096
                },
            ]
        );
        assert!(mft_extents_from_runs(&runs, 4096, 100 * 4096).is_err());
    }

    #[test]
    fn reads_records_across_fragmented_extents_in_order() {
        let record_size = 1024;
        let mut volume = InMemoryVolume::with_size(64 * 1024);
        // Records 0-2 at offset 4096, records 3-4 at offset 32768.
        let extents = vec![
            MftExtent {
                volume_offset: 4096,
                length: 3 * 1024,
            },
            MftExtent {
                volume_offset: 32_768,
                length: 2 * 1024,
            },
        ];
        let record_offsets = [4096, 5120, 6144, 32_768, 33_792];
        for (record_number, &offset) in record_offsets.iter().enumerate() {
            volume.write(
                offset,
                &vec![u8::try_from(record_number).unwrap(); record_size],
            );
        }

        // 1.5 records round up to two, so one chunk spans the end of the first extent and the
        // start of the second.
        let mut reader = MftChunkReader::new(&volume, extents, 1536, record_size);
        let mut chunk = Vec::new();
        let mut records_seen = Vec::new();
        while let Some(first_record_number) = reader.read_next_chunk(&mut chunk).unwrap() {
            for (position, record) in chunk.chunks_exact(record_size).enumerate() {
                assert!(record.iter().all(|&byte| byte == record[0]));
                records_seen.push((first_record_number + position as u64, record[0]));
            }
        }

        assert_eq!(records_seen, [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    }
}
