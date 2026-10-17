use super::RawAttribute;
use crate::mft::little_endian_fields::read_u32;

/// The reparse tag of a `$REPARSE_POINT` attribute: what kind of link or special file the
/// record is (junction, symlink, cloud file and so on).
pub(crate) fn parse_reparse_tag(attribute: &RawAttribute<'_>) -> Option<u32> {
    let value = attribute.resident_value()?;
    read_u32(value, 0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::attributes::attributes_of;
    use crate::mft::file_record::prepare_file_record;
    use crate::mft::test_fixtures::RecordBuilder;

    #[test]
    fn reads_the_tag() {
        let mut record = RecordBuilder::new(1).reparse_point(0xA000_0003).build();
        let header = prepare_file_record(&mut record).unwrap();
        let attribute = attributes_of(&record, &header).next().unwrap();

        assert_eq!(parse_reparse_tag(&attribute), Some(0xA000_0003));
    }
}
