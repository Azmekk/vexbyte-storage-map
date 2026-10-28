use super::attributes::attribute_list::{AttributeListEntry, parse_attribute_list_bytes};
use super::attributes::data::unnamed_data_runs;
use super::attributes::{ATTRIBUTE_LIST_TYPE, DATA_TYPE, RawAttribute, attributes_of};
use super::data_runs::{DataRun, decode_data_runs};
use super::file_record::prepare_file_record;
use super::mft_chunk_reader::{MftExtent, VolumeBytes, mft_extents_from_runs};
use super::mft_scan_error::MftScanError;
use super::volume_layout::NtfsVolumeLayout;

const MFT_RECORD_NUMBER: u64 = 0;
/// An `$ATTRIBUTE_LIST` of `$MFT` holds a few dozen bytes per extension record. A list claiming
/// far more than that is corrupt, and this cap keeps a corrupt one from allocating gigabytes.
const MAXIMUM_ATTRIBUTE_LIST_SIZE: u64 = 1024 * 1024;

/// Finds where the MFT lies on the volume by reading record 0, `$MFT` itself, and decoding
/// the data runs of its `$DATA` attribute. The MFT can be fragmented; the runs give its real
/// location.
///
/// On a very fragmented MFT the `$DATA` attribute of `$MFT` is split over extension records
/// listed in its `$ATTRIBUTE_LIST`; those records are read through the runs found so far.
///
/// Windows often stores that list non-resident even when it is tiny, as soon as `$MFT` needs an
/// extension record at all (a fragmented `$BITMAP` of `$MFT` is enough). The list's own data
/// runs address clusters directly, so it can be read from the volume before the MFT is known.
pub(crate) fn locate_mft_extents<Volume: VolumeBytes>(
    volume: &Volume,
    layout: &NtfsVolumeLayout,
) -> Result<Vec<MftExtent>, MftScanError> {
    let record_size = layout.record_size();
    let mut record_zero = vec![0_u8; record_size];
    read_exactly(volume, layout.mft_start_offset(), &mut record_zero)?;

    let header = prepare_file_record(&mut record_zero).map_err(|error| {
        MftScanError::MftRecordUnreadable {
            record_number: MFT_RECORD_NUMBER,
            reason: error.to_string(),
        }
    })?;

    let mut data_run_pieces: Vec<(u64, Vec<DataRun>)> = Vec::new();
    let mut extension_records_with_data = Vec::new();
    for attribute in attributes_of(&record_zero, &header) {
        if let Some((starting_virtual_cluster, encoded_runs)) = unnamed_data_runs(&attribute) {
            data_run_pieces.push((starting_virtual_cluster, decode_data_runs(encoded_runs)?));
        } else if attribute.type_code == ATTRIBUTE_LIST_TYPE {
            let entries = read_attribute_list(volume, layout, &attribute)?;
            extension_records_with_data.extend(
                entries
                    .iter()
                    .filter(|entry| {
                        entry.attribute_type == DATA_TYPE
                            && entry.record.record_number() != MFT_RECORD_NUMBER
                    })
                    .map(|entry| entry.record.record_number()),
            );
        }
    }
    if data_run_pieces.is_empty() {
        return Err(MftScanError::MftDataMissing);
    }

    for extension_record_number in extension_records_with_data {
        let runs_so_far = runs_in_virtual_cluster_order(&data_run_pieces);
        let extents_so_far = extents_without_length_limit(&runs_so_far, layout.bytes_per_cluster);
        let offset = volume_offset_of_record(&extents_so_far, extension_record_number, record_size)
            .ok_or(MftScanError::MftTooFragmented)?;

        let mut extension_record = vec![0_u8; record_size];
        read_exactly(volume, offset, &mut extension_record)?;
        let extension_header = prepare_file_record(&mut extension_record).map_err(|error| {
            MftScanError::MftRecordUnreadable {
                record_number: extension_record_number,
                reason: error.to_string(),
            }
        })?;
        for attribute in attributes_of(&extension_record, &extension_header) {
            if let Some((starting_virtual_cluster, encoded_runs)) = unnamed_data_runs(&attribute) {
                data_run_pieces.push((starting_virtual_cluster, decode_data_runs(encoded_runs)?));
            }
        }
    }

    let all_runs = runs_in_virtual_cluster_order(&data_run_pieces);
    Ok(mft_extents_from_runs(
        &all_runs,
        layout.bytes_per_cluster,
        layout.mft_valid_data_length,
    )?)
}

/// The entries of `$MFT`'s `$ATTRIBUTE_LIST`, whether it is stored inside record 0 or in its
/// own clusters on the volume.
fn read_attribute_list<Volume: VolumeBytes>(
    volume: &Volume,
    layout: &NtfsVolumeLayout,
    attribute: &RawAttribute<'_>,
) -> Result<Vec<AttributeListEntry>, MftScanError> {
    if let Some(resident_list) = attribute.resident_value() {
        return Ok(parse_attribute_list_bytes(resident_list));
    }
    let non_resident = attribute
        .non_resident_header()
        .ok_or_else(|| attribute_list_unreadable("its header is malformed"))?;
    if non_resident.data_size > MAXIMUM_ATTRIBUTE_LIST_SIZE {
        return Err(attribute_list_unreadable("it is implausibly large"));
    }
    let list_size = usize::try_from(non_resident.data_size)
        .map_err(|_| attribute_list_unreadable("it is implausibly large"))?;
    let bytes_per_cluster = layout.bytes_per_cluster as usize;

    // Volume reads are sector-sized: whole clusters are read, then the list is cut to its size.
    let mut list = Vec::with_capacity(list_size);
    for run in decode_data_runs(non_resident.data_runs)? {
        if list.len() >= list_size {
            break;
        }
        let first_cluster = run
            .first_cluster
            .ok_or_else(|| attribute_list_unreadable("it contains a sparse run"))?;
        let clusters_still_needed = (list_size - list.len()).div_ceil(bytes_per_cluster) as u64;
        let clusters_to_read = run.cluster_count.min(clusters_still_needed);
        let run_length = usize::try_from(clusters_to_read)
            .ok()
            .and_then(|clusters| clusters.checked_mul(bytes_per_cluster))
            .ok_or_else(|| attribute_list_unreadable("it is implausibly large"))?;
        let run_start = list.len();
        list.resize(run_start + run_length, 0);
        read_exactly(
            volume,
            first_cluster.saturating_mul(u64::from(layout.bytes_per_cluster)),
            &mut list[run_start..],
        )?;
    }
    if list.len() < list_size {
        return Err(attribute_list_unreadable(
            "its data runs are shorter than its size",
        ));
    }
    list.truncate(list_size);
    Ok(parse_attribute_list_bytes(&list))
}

fn attribute_list_unreadable(reason: &str) -> MftScanError {
    MftScanError::MftRecordUnreadable {
        record_number: MFT_RECORD_NUMBER,
        reason: format!("the $ATTRIBUTE_LIST of $MFT could not be read: {reason}"),
    }
}

/// Each extent of a fragmented attribute decodes its runs independently; ordered by their
/// starting virtual cluster they form the complete run list.
fn runs_in_virtual_cluster_order(pieces: &[(u64, Vec<DataRun>)]) -> Vec<DataRun> {
    let mut ordered_pieces: Vec<&(u64, Vec<DataRun>)> = pieces.iter().collect();
    ordered_pieces.sort_by_key(|(starting_virtual_cluster, _)| *starting_virtual_cluster);
    ordered_pieces
        .into_iter()
        .flat_map(|(_, runs)| runs.iter().copied())
        .collect()
}

fn extents_without_length_limit(runs: &[DataRun], bytes_per_cluster: u32) -> Vec<MftExtent> {
    runs.iter()
        .filter_map(|run| {
            Some(MftExtent {
                volume_offset: run
                    .first_cluster?
                    .saturating_mul(u64::from(bytes_per_cluster)),
                length: run
                    .cluster_count
                    .saturating_mul(u64::from(bytes_per_cluster)),
            })
        })
        .collect()
}

fn volume_offset_of_record(
    extents: &[MftExtent],
    record_number: u64,
    record_size: usize,
) -> Option<u64> {
    let mut offset_in_mft = record_number.checked_mul(record_size as u64)?;
    for extent in extents {
        if offset_in_mft < extent.length {
            return extent.volume_offset.checked_add(offset_in_mft);
        }
        offset_in_mft -= extent.length;
    }
    None
}

fn read_exactly<Volume: VolumeBytes>(
    volume: &Volume,
    offset: u64,
    buffer: &mut [u8],
) -> Result<(), MftScanError> {
    let mut filled = 0;
    while filled < buffer.len() {
        let read_offset = offset.saturating_add(filled as u64);
        let bytes_read = volume
            .read_at(read_offset, &mut buffer[filled..])
            .map_err(|source| MftScanError::VolumeReadFailed {
                offset: read_offset,
                source,
            })?;
        if bytes_read == 0 {
            return Err(MftScanError::VolumeReadFailed {
                offset: read_offset,
                source: std::io::Error::from(std::io::ErrorKind::UnexpectedEof),
            });
        }
        filled += bytes_read;
    }
    Ok(())
}
