use super::RawAttribute;
use crate::mft::little_endian_fields::{read_u32, read_u64};

/// The part of `$STANDARD_INFORMATION` the scanner uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StandardInformation {
    /// When the file's data last changed, as an NTFS timestamp (100 ns ticks since 1601).
    pub(crate) modified_time: u64,
    /// The same bits Win32 reports as `FILE_ATTRIBUTE_*`.
    pub(crate) file_attributes: u32,
}

pub(crate) fn parse_standard_information(
    attribute: &RawAttribute<'_>,
) -> Option<StandardInformation> {
    let value = attribute.resident_value()?;

    Some(StandardInformation {
        modified_time: read_u64(value, 8)?,
        file_attributes: read_u32(value, 32)?,
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
    fn reads_modification_time_and_attributes() {
        let mut record = RecordBuilder::new(1)
            .standard_information(133_000_000_000_000_000, 0x26)
            .build();
        let header = prepare_file_record(&mut record).unwrap();
        let attribute = attributes_of(&record, &header).next().unwrap();

        let information = parse_standard_information(&attribute).unwrap();

        assert_eq!(information.modified_time, 133_000_000_000_000_000);
        assert_eq!(information.file_attributes, 0x26);
    }
}
