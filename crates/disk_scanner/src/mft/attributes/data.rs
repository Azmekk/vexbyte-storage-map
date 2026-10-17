use super::{DATA_TYPE, RawAttribute};

/// The sizes of a file's unnamed `$DATA` stream (its normal content).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct UnnamedDataSizes {
    pub(crate) logical_size: u64,
    pub(crate) allocated_size: u64,
}

/// The sizes of an unnamed `$DATA` attribute, or `None` for named streams (alternate data
/// streams, ignored in this version) and for later extents of a fragmented attribute.
///
/// A small file's data is resident: stored inside the MFT record, occupying no clusters.
/// A non-resident attribute split over several records only carries the sizes in the
/// extent that starts at virtual cluster 0.
pub(crate) fn unnamed_data_sizes(attribute: &RawAttribute<'_>) -> Option<UnnamedDataSizes> {
    if attribute.type_code != DATA_TYPE || !attribute.is_unnamed() {
        return None;
    }

    if let Some(value) = attribute.resident_value() {
        return Some(UnnamedDataSizes {
            logical_size: value.len() as u64,
            allocated_size: 0,
        });
    }

    let non_resident = attribute.non_resident_header()?;
    if non_resident.starting_virtual_cluster != 0 {
        return None;
    }
    Some(UnnamedDataSizes {
        logical_size: non_resident.data_size,
        allocated_size: non_resident
            .total_allocated_size
            .unwrap_or(non_resident.allocated_size),
    })
}

/// The data runs of one extent of a non-resident unnamed `$DATA` attribute, with the virtual
/// cluster the extent starts at. Used to locate the MFT itself.
pub(crate) fn unnamed_data_runs<'record>(
    attribute: &RawAttribute<'record>,
) -> Option<(u64, &'record [u8])> {
    if attribute.type_code != DATA_TYPE || !attribute.is_unnamed() {
        return None;
    }
    let non_resident = attribute.non_resident_header()?;
    Some((
        non_resident.starting_virtual_cluster,
        non_resident.data_runs,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::mft::attributes::attributes_of;
    use crate::mft::file_record::prepare_file_record;
    use crate::mft::test_fixtures::RecordBuilder;

    fn data_sizes_of(builder: &RecordBuilder) -> Vec<Option<UnnamedDataSizes>> {
        let mut record = builder.build();
        let header = prepare_file_record(&mut record).unwrap();
        attributes_of(&record, &header)
            .map(|attribute| unnamed_data_sizes(&attribute))
            .collect()
    }

    #[test]
    fn resident_data_occupies_no_clusters() {
        let sizes = data_sizes_of(&RecordBuilder::new(1).resident_data(&[7; 300]));

        assert_eq!(
            sizes,
            [Some(UnnamedDataSizes {
                logical_size: 300,
                allocated_size: 0
            })]
        );
    }

    #[test]
    fn non_resident_data_reports_both_sizes() {
        let sizes = data_sizes_of(&RecordBuilder::new(1).non_resident_data(
            0,
            1_052_672,
            1_050_000,
            &[0x21, 0x01, 0x00, 0x10, 0x00],
        ));

        assert_eq!(
            sizes,
            [Some(UnnamedDataSizes {
                logical_size: 1_050_000,
                allocated_size: 1_052_672
            })]
        );
    }

    #[test]
    fn compressed_data_reports_the_clusters_actually_used() {
        let sizes = data_sizes_of(
            &RecordBuilder::new(1).compressed_non_resident_data(65_536, 60_000, 16_384),
        );

        assert_eq!(
            sizes,
            [Some(UnnamedDataSizes {
                logical_size: 60_000,
                allocated_size: 16_384
            })]
        );
    }

    #[test]
    fn ignores_named_streams_and_later_extents() {
        let sizes = data_sizes_of(
            &RecordBuilder::new(1)
                .named_resident_data("Zone.Identifier", &[1; 26])
                .non_resident_data(16, 0, 0, &[0x11, 0x01, 0x01, 0x00]),
        );

        assert_eq!(sizes, [None, None]);
    }
}
