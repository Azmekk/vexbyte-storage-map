use std::collections::HashSet;

use super::attributes::attribute_list::parse_attribute_list;
use super::attributes::data::{UnnamedDataSizes, unnamed_data_sizes};
use super::attributes::file_name::parse_file_name;
use super::attributes::reparse_point::parse_reparse_tag;
use super::attributes::standard_information::parse_standard_information;
use super::attributes::{
    ATTRIBUTE_LIST_TYPE, FILE_NAME_TYPE, REPARSE_POINT_TYPE, RawAttribute,
    STANDARD_INFORMATION_TYPE, attributes_of,
};
use super::file_record::{FileRecordHeader, prepare_file_record};
use super::file_reference::FileReference;

const IN_USE: u8 = 1 << 0;
const IS_DIRECTORY: u8 = 1 << 1;
const HAS_REPARSE_TAG: u8 = 1 << 2;

/// What the scanner keeps of one base record after parsing it. Records are collected first
/// and linked into a tree only after the whole MFT has been read, because children usually
/// appear before their parents.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RecordSummary {
    pub(crate) sequence_number: u16,
    state: u8,
    /// Valid non-DOS names after merging extension records; more than one means hard links.
    pub(crate) name_count: u16,
    pub(crate) file_attributes: u32,
    pub(crate) reparse_tag: u32,
    pub(crate) modified_time: u64,
    pub(crate) data_sizes: UnnamedDataSizes,
}

impl RecordSummary {
    pub(crate) fn is_in_use(&self) -> bool {
        self.state & IN_USE != 0
    }

    pub(crate) fn is_directory(&self) -> bool {
        self.state & IS_DIRECTORY != 0
    }

    pub(crate) fn reparse_tag(&self) -> Option<u32> {
        (self.state & HAS_REPARSE_TAG != 0).then_some(self.reparse_tag)
    }
}

/// One `$FILE_NAME` of a record, with its text in the table's shared buffer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CollectedName {
    /// The base record the name belongs to, with the sequence number the name's own record
    /// expects that base record to have.
    pub(crate) owner: FileReference,
    pub(crate) parent_directory: FileReference,
    /// The record the name was read from: the owner itself or one of its extension records.
    source_record: u32,
    text_start: u32,
    text_length: u32,
}

/// Unnamed data sizes found in an extension record, applied to the base record once every
/// record has been read.
#[derive(Debug, Clone, Copy)]
struct ExtensionDataSizes {
    owner: FileReference,
    source_record: u32,
    sizes: UnnamedDataSizes,
}

/// Running totals while records are read, for progress reporting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RecordTotals {
    pub(crate) files: u64,
    pub(crate) directories: u64,
    pub(crate) logical_bytes: u64,
}

/// Every parsed MFT record, indexed by record number.
pub(crate) struct MftRecordTable {
    records: Vec<RecordSummary>,
    names: Vec<CollectedName>,
    name_text: String,
    extension_data_sizes: Vec<ExtensionDataSizes>,
    /// `(base, extension)` pairs from resident `$ATTRIBUTE_LIST`s. An extension record whose
    /// base lists its extensions but not this one is a stale leftover and is ignored.
    listed_extension_records: HashSet<(u64, u64)>,
    bases_with_resident_attribute_list: HashSet<u64>,
    skipped_record_count: u64,
    totals: RecordTotals,
    scratch_name: String,
}

impl MftRecordTable {
    pub(crate) fn with_record_count(record_count: u64) -> Self {
        let record_count = usize::try_from(record_count).unwrap_or(0);
        Self {
            records: vec![RecordSummary::default(); record_count],
            names: Vec::with_capacity(record_count),
            name_text: String::with_capacity(record_count * 16),
            extension_data_sizes: Vec::new(),
            listed_extension_records: HashSet::new(),
            bases_with_resident_attribute_list: HashSet::new(),
            skipped_record_count: 0,
            totals: RecordTotals::default(),
            scratch_name: String::new(),
        }
    }

    pub(crate) fn records(&self) -> &[RecordSummary] {
        &self.records
    }

    pub(crate) fn record(&self, record_number: u64) -> Option<&RecordSummary> {
        self.records.get(usize::try_from(record_number).ok()?)
    }

    pub(crate) fn names(&self) -> &[CollectedName] {
        &self.names
    }

    pub(crate) fn name_text(&self, name: &CollectedName) -> &str {
        let start = name.text_start as usize;
        let end = start + name.text_length as usize;
        self.name_text.get(start..end).unwrap_or("")
    }

    pub(crate) fn totals(&self) -> RecordTotals {
        self.totals
    }

    /// Records that had no valid FILE signature or failed their fixups.
    pub(crate) fn skipped_record_count(&self) -> u64 {
        self.skipped_record_count
    }

    /// Parses one raw record (fixups are applied in place) and adds what it contains.
    pub(crate) fn add_record(&mut self, record_number: u64, record: &mut [u8]) {
        let Ok(slot) = usize::try_from(record_number) else {
            return;
        };
        if slot >= self.records.len() {
            self.records.resize(slot + 1, RecordSummary::default());
        }
        let Ok(header) = prepare_file_record(record) else {
            self.skipped_record_count += 1;
            return;
        };

        self.records[slot].sequence_number = header.sequence_number;
        if !header.is_in_use() {
            return;
        }

        if header.is_base_record() {
            let owner = FileReference::from_parts(record_number, header.sequence_number);
            self.add_base_record(slot, owner, record, &header);
        } else {
            self.add_extension_record(record_number, record, &header);
        }
    }

    fn add_base_record(
        &mut self,
        slot: usize,
        owner: FileReference,
        record: &[u8],
        header: &FileRecordHeader,
    ) {
        let mut summary = RecordSummary {
            sequence_number: header.sequence_number,
            state: IN_USE,
            ..RecordSummary::default()
        };
        if header.is_directory() {
            summary.state |= IS_DIRECTORY;
        }

        for attribute in attributes_of(record, header) {
            match attribute.type_code {
                STANDARD_INFORMATION_TYPE => {
                    if let Some(information) = parse_standard_information(&attribute) {
                        summary.modified_time = information.modified_time;
                        summary.file_attributes = information.file_attributes;
                    }
                }
                FILE_NAME_TYPE => self.collect_file_name(&attribute, owner, owner.record_number()),
                REPARSE_POINT_TYPE => {
                    if let Some(tag) = parse_reparse_tag(&attribute) {
                        summary.reparse_tag = tag;
                        summary.state |= HAS_REPARSE_TAG;
                    }
                }
                ATTRIBUTE_LIST_TYPE => {
                    if let Some(entries) = parse_attribute_list(&attribute) {
                        let base_record_number = owner.record_number();
                        self.bases_with_resident_attribute_list
                            .insert(base_record_number);
                        self.listed_extension_records.extend(
                            entries
                                .iter()
                                .map(|entry| entry.record.record_number())
                                .filter(|&listed| listed != base_record_number)
                                .map(|listed| (base_record_number, listed)),
                        );
                    }
                }
                _ => {
                    if let Some(sizes) = unnamed_data_sizes(&attribute) {
                        summary.data_sizes = sizes;
                    }
                }
            }
        }

        if summary.is_directory() {
            self.totals.directories += 1;
        } else {
            self.totals.files += 1;
            self.totals.logical_bytes = self
                .totals
                .logical_bytes
                .saturating_add(summary.data_sizes.logical_size);
        }
        self.records[slot] = summary;
    }

    /// Extension records hold attributes that did not fit into the base record: more names
    /// (hard links) or the `$DATA` extents of a heavily fragmented file.
    fn add_extension_record(
        &mut self,
        record_number: u64,
        record: &[u8],
        header: &FileRecordHeader,
    ) {
        let owner = header.base_record;

        for attribute in attributes_of(record, header) {
            if attribute.type_code == FILE_NAME_TYPE {
                self.collect_file_name(&attribute, owner, record_number);
            } else if let Some(sizes) = unnamed_data_sizes(&attribute) {
                self.totals.logical_bytes =
                    self.totals.logical_bytes.saturating_add(sizes.logical_size);
                self.extension_data_sizes.push(ExtensionDataSizes {
                    owner,
                    source_record: u32::try_from(record_number).unwrap_or(u32::MAX),
                    sizes,
                });
            }
        }
    }

    fn collect_file_name(
        &mut self,
        attribute: &RawAttribute<'_>,
        owner: FileReference,
        source_record: u64,
    ) {
        let Some(file_name) = parse_file_name(attribute, &mut self.scratch_name) else {
            return;
        };
        if file_name.is_dos_only_name() {
            return;
        }

        let text_start = u32::try_from(self.name_text.len()).unwrap_or(u32::MAX);
        self.name_text.push_str(&self.scratch_name);
        self.names.push(CollectedName {
            owner,
            parent_directory: file_name.parent_directory,
            source_record: u32::try_from(source_record).unwrap_or(u32::MAX),
            text_start,
            text_length: u32::try_from(self.scratch_name.len()).unwrap_or(0),
        });
    }

    fn is_listed_by_owner(&self, owner: FileReference, source_record: u32) -> bool {
        let owner_record = owner.record_number();
        let source_record = u64::from(source_record);

        source_record == owner_record
            || !self
                .bases_with_resident_attribute_list
                .contains(&owner_record)
            || self
                .listed_extension_records
                .contains(&(owner_record, source_record))
    }

    /// True when `reference` points to a record that is in use with the same sequence
    /// number, that is, to a file that still exists.
    pub(crate) fn is_live_reference(&self, reference: FileReference) -> bool {
        self.record(reference.record_number())
            .is_some_and(|record| {
                record.is_in_use() && record.sequence_number == reference.sequence_number()
            })
    }

    /// Applies the contents of extension records to their base records and counts the names
    /// of every record. Must run after all records were added.
    pub(crate) fn merge_extension_records(&mut self) {
        for extension in std::mem::take(&mut self.extension_data_sizes) {
            let belongs_to_owner = self.is_live_reference(extension.owner)
                && self.is_listed_by_owner(extension.owner, extension.source_record);
            if belongs_to_owner && let Ok(slot) = usize::try_from(extension.owner.record_number()) {
                self.records[slot].data_sizes = extension.sizes;
            }
        }

        let mut names = std::mem::take(&mut self.names);
        names.retain(|name| {
            self.is_live_reference(name.owner)
                && self.is_listed_by_owner(name.owner, name.source_record)
        });
        for name in &names {
            if let Ok(slot) = usize::try_from(name.owner.record_number()) {
                self.records[slot].name_count = self.records[slot].name_count.saturating_add(1);
            }
        }
        self.names = names;
    }
}
