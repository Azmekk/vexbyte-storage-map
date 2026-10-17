use super::record_fixups::UPDATE_SEQUENCE_STRIDE;

/// NTFS geometry and the location of the Master File Table, as reported by
/// `FSCTL_GET_NTFS_VOLUME_DATA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NtfsVolumeLayout {
    pub(crate) volume_serial_number: u64,
    pub(crate) bytes_per_sector: u32,
    pub(crate) bytes_per_cluster: u32,
    pub(crate) bytes_per_file_record: u32,
    pub(crate) total_clusters: u64,
    pub(crate) mft_start_cluster: u64,
    /// The initialized part of the MFT in bytes; records beyond it have never been used.
    pub(crate) mft_valid_data_length: u64,
}

const LARGEST_SUPPORTED_RECORD_SIZE: u32 = 64 * 1024;

impl NtfsVolumeLayout {
    /// Checks that the sizes are consistent before any of them is used for arithmetic.
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        let stride = u32::try_from(UPDATE_SEQUENCE_STRIDE).unwrap_or(512);

        if !self.bytes_per_sector.is_power_of_two() || self.bytes_per_sector < stride {
            return Err("the sector size is not a power of two of at least 512 bytes");
        }
        if !self.bytes_per_cluster.is_power_of_two()
            || self.bytes_per_cluster < self.bytes_per_sector
        {
            return Err("the cluster size is not a power of two of at least one sector");
        }
        let record_size_is_valid = self.bytes_per_file_record.is_power_of_two()
            && self.bytes_per_file_record >= stride
            && self.bytes_per_file_record <= LARGEST_SUPPORTED_RECORD_SIZE
            && self
                .bytes_per_file_record
                .is_multiple_of(self.bytes_per_sector);
        if !record_size_is_valid {
            return Err("the file record size is not a supported multiple of the sector size");
        }
        if self.mft_start_cluster >= self.total_clusters {
            return Err("the MFT starts outside the volume");
        }
        Ok(())
    }

    pub(crate) fn record_size(&self) -> usize {
        self.bytes_per_file_record as usize
    }

    pub(crate) fn record_count(&self) -> u64 {
        self.mft_valid_data_length / u64::from(self.bytes_per_file_record)
    }

    pub(crate) fn mft_start_offset(&self) -> u64 {
        self.mft_start_cluster * u64::from(self.bytes_per_cluster)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn typical_layout() -> NtfsVolumeLayout {
        NtfsVolumeLayout {
            volume_serial_number: 0x1234,
            bytes_per_sector: 512,
            bytes_per_cluster: 4096,
            bytes_per_file_record: 1024,
            total_clusters: 1_000_000,
            mft_start_cluster: 786_432,
            mft_valid_data_length: 1024 * 300_000,
        }
    }

    #[test]
    fn accepts_a_typical_layout() {
        let layout = typical_layout();

        assert_eq!(layout.validate(), Ok(()));
        assert_eq!(layout.record_count(), 300_000);
        assert_eq!(layout.mft_start_offset(), 786_432 * 4096);
    }

    #[test]
    fn rejects_inconsistent_sizes() {
        let odd_record_size = NtfsVolumeLayout {
            bytes_per_file_record: 1000,
            ..typical_layout()
        };
        let tiny_cluster = NtfsVolumeLayout {
            bytes_per_cluster: 256,
            ..typical_layout()
        };

        assert!(odd_record_size.validate().is_err());
        assert!(tiny_cluster.validate().is_err());
    }
}
