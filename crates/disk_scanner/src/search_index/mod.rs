//! A whole-volume index for Everything-style global file search, separate from the scans the
//! user starts. One `VolumeSearchIndex` per local volume answers name and path queries in
//! milliseconds. An index is an immutable value: refreshing a volume builds a new index and
//! replaces the old one as a whole, which also leaves room to apply NTFS change journal updates
//! later.

mod index_build;
mod lowercase_name_table;
mod search_query;
mod search_results;
mod text_pattern;
mod volume_search_index;

pub use index_build::{SearchIndexBuildError, build_volume_search_index, search_index_scan_method};
pub use search_query::{ParsedQuery, SearchQueryError};
pub use search_results::{
    SearchHit, SearchMatches, SearchSortOrder, search_volume_index, search_volume_indexes,
};
pub use volume_search_index::VolumeSearchIndex;
