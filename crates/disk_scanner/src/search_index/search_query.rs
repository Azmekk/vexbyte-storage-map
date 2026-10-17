//! The search syntax, a subset of the one Everything by voidtools uses. See `docs/SEARCH.md`.
//!
//! - Terms are separated by whitespace and all of them must match (AND).
//! - A term matches case-insensitively anywhere in the file name, or anywhere in the full path
//!   when it contains `\` or `/`.
//! - A term with `*` or `?` is a wildcard pattern that must match the whole name (or path).
//! - `"two words"` is one term containing a space.
//! - `ext:jpg;png` keeps files with one of the extensions.
//! - `size:>100mb`, `size:<1kb`, `size:>=1gb`, `size:<=10kb`, `size:1mb..10mb`, `size:4096`.
//! - `type:file` and `type:folder` (also `file:` and `folder:`) keep one kind of item.

use std::path::MAIN_SEPARATOR_STR;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::text_pattern::TextPattern;
use crate::file_tree::NodeKind;

/// Filters of Everything that this search does not understand yet. They are refused instead of
/// being searched for as text, which would silently find nothing.
const UNSUPPORTED_FILTER_NAMES: [&str; 22] = [
    "attrib",
    "attributes",
    "case",
    "child",
    "content",
    "da",
    "dateaccessed",
    "datecreated",
    "datemodified",
    "dc",
    "depth",
    "dm",
    "dr",
    "dupe",
    "len",
    "nopath",
    "parent",
    "path",
    "rc",
    "regex",
    "wholeword",
    "ww",
];

const BYTES_PER_KIBIBYTE: f64 = 1024.0;

/// Why a query could not be understood.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SearchQueryError {
    #[error("\"{value}\" is not a size; use for example size:>100mb, size:<1kb or size:1mb..10mb")]
    InvalidSizeFilter { value: String },
    #[error("\"{value}\" is not a type; use type:file or type:folder")]
    InvalidTypeFilter { value: String },
    #[error("the {filter}: filter is not supported")]
    UnsupportedFilter { filter: String },
}

/// Which text a term is compared with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TermScope {
    Name,
    FullPath,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchTerm {
    pub(crate) pattern: TextPattern,
    pub(crate) scope: TermScope,
}

/// An inclusive range of logical sizes in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SizeRange {
    pub(crate) minimum: u64,
    pub(crate) maximum: u64,
}

impl SizeRange {
    pub(crate) fn contains(self, size: u64) -> bool {
        (self.minimum..=self.maximum).contains(&size)
    }
}

/// One `ext:` filter: the file must have one of these lowercase extensions (without the dot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtensionAlternatives {
    pub(crate) lowercase_extensions: Vec<String>,
}

/// A query parsed once and then run against every volume.
#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    pub(crate) terms: Vec<SearchTerm>,
    pub(crate) extension_filters: Vec<ExtensionAlternatives>,
    pub(crate) size_ranges: Vec<SizeRange>,
    pub(crate) kind_filters: Vec<NodeKind>,
}

impl ParsedQuery {
    /// Parses the search box text.
    ///
    /// # Errors
    ///
    /// Fails for a malformed `size:` or `type:` value and for Everything filters that are not
    /// supported, such as `dm:`.
    pub fn parse(query_text: &str) -> Result<Self, SearchQueryError> {
        let mut query = Self::default();
        for token in split_into_tokens(query_text) {
            query.add_token(&token)?;
        }
        Ok(query)
    }

    /// True when the query restricts nothing, for example for an empty search box. Such a query
    /// is answered with no results rather than with every file.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
            && self.extension_filters.is_empty()
            && self.size_ranges.is_empty()
            && self.kind_filters.is_empty()
    }

    fn add_token(&mut self, token: &QueryToken) -> Result<(), SearchQueryError> {
        let Some((filter_name, value)) = token.filter() else {
            self.add_text_term(&token.text);
            return Ok(());
        };

        // A filter without a value while the user is still typing restricts nothing yet.
        let is_value_missing = value.trim().is_empty();
        match filter_name.as_str() {
            "ext" => self.add_extension_filter(value),
            "size" | "type" if is_value_missing => {}
            "size" => self.size_ranges.push(parse_size_filter(value)?),
            "type" => self.kind_filters.push(parse_type_filter(value)?),
            "file" => {
                self.kind_filters.push(NodeKind::File);
                self.add_text_term(value);
            }
            "folder" => {
                self.kind_filters.push(NodeKind::Directory);
                self.add_text_term(value);
            }
            unsupported if UNSUPPORTED_FILTER_NAMES.contains(&unsupported) => {
                return Err(SearchQueryError::UnsupportedFilter {
                    filter: unsupported.to_owned(),
                });
            }
            _ => self.add_text_term(&token.text),
        }
        Ok(())
    }

    fn add_text_term(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let has_path_separator = text.contains(['\\', '/']);
        let lowercase_text = text.to_lowercase();
        let (scope, term_text) = if has_path_separator {
            let with_native_separators = lowercase_text.replace(['\\', '/'], MAIN_SEPARATOR_STR);
            (TermScope::FullPath, with_native_separators)
        } else {
            (TermScope::Name, lowercase_text)
        };

        self.terms.push(SearchTerm {
            pattern: TextPattern::new(&term_text),
            scope,
        });
    }

    fn add_extension_filter(&mut self, value: &str) {
        let lowercase_extensions: Vec<String> = value
            .split([';', ','])
            .map(|extension| extension.trim().trim_start_matches('.').to_lowercase())
            .filter(|extension| !extension.is_empty())
            .collect();

        if !lowercase_extensions.is_empty() {
            self.extension_filters.push(ExtensionAlternatives {
                lowercase_extensions,
            });
        }
    }
}

/// One whitespace-separated piece of the query with its quotes removed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryToken {
    text: String,
    /// Where the first quote was in `text` (after removing quotes), if any. A `name:` prefix
    /// only counts as a filter when it is not quoted.
    first_quote_position: Option<usize>,
}

impl QueryToken {
    /// `("size", ">1mb")` for `size:>1mb`. Filter names are case-insensitive.
    fn filter(&self) -> Option<(String, &str)> {
        let colon_position = self.text.find(':')?;
        let is_prefix_quoted = self
            .first_quote_position
            .is_some_and(|quote_position| quote_position <= colon_position);
        let name = &self.text[..colon_position];
        let is_filter_name = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic());

        (is_filter_name && !is_prefix_quoted)
            .then(|| (name.to_ascii_lowercase(), &self.text[colon_position + 1..]))
    }
}

fn split_into_tokens(query_text: &str) -> Vec<QueryToken> {
    let mut tokens = Vec::new();
    let mut current = QueryToken {
        text: String::new(),
        first_quote_position: None,
    };
    let mut has_current_token = false;
    let mut is_inside_quotes = false;

    for character in query_text.chars() {
        if character == '"' {
            is_inside_quotes = !is_inside_quotes;
            current
                .first_quote_position
                .get_or_insert(current.text.len());
            has_current_token = true;
        } else if character.is_whitespace() && !is_inside_quotes {
            if has_current_token {
                tokens.push(std::mem::replace(
                    &mut current,
                    QueryToken {
                        text: String::new(),
                        first_quote_position: None,
                    },
                ));
                has_current_token = false;
            }
        } else {
            current.text.push(character);
            has_current_token = true;
        }
    }

    if has_current_token {
        tokens.push(current);
    }
    tokens
}

fn parse_type_filter(value: &str) -> Result<NodeKind, SearchQueryError> {
    match value.to_ascii_lowercase().as_str() {
        "file" | "files" => Ok(NodeKind::File),
        "folder" | "folders" | "directory" | "directories" | "dir" => Ok(NodeKind::Directory),
        _ => Err(SearchQueryError::InvalidTypeFilter {
            value: value.to_owned(),
        }),
    }
}

type RangeForSize = fn(u64) -> SizeRange;

fn parse_size_filter(value: &str) -> Result<SizeRange, SearchQueryError> {
    let invalid = || SearchQueryError::InvalidSizeFilter {
        value: value.to_owned(),
    };
    let trimmed_value = value.trim();

    if let Some((lower_text, upper_text)) = trimmed_value.split_once("..") {
        let minimum = parse_size_in_bytes(lower_text).ok_or_else(invalid)?;
        let maximum = parse_size_in_bytes(upper_text).ok_or_else(invalid)?;
        return Ok(SizeRange { minimum, maximum });
    }

    let comparisons: [(&str, RangeForSize); 5] = [
        (">=", |size| SizeRange {
            minimum: size,
            maximum: u64::MAX,
        }),
        ("<=", |size| SizeRange {
            minimum: 0,
            maximum: size,
        }),
        (">", |size| SizeRange {
            minimum: size.saturating_add(1),
            maximum: u64::MAX,
        }),
        ("<", |size| match size.checked_sub(1) {
            Some(maximum) => SizeRange {
                minimum: 0,
                maximum,
            },
            None => SizeRange {
                minimum: 1,
                maximum: 0,
            },
        }),
        ("=", |size| SizeRange {
            minimum: size,
            maximum: size,
        }),
    ];

    for (operator, range_for_size) in comparisons {
        if let Some(size_text) = trimmed_value.strip_prefix(operator) {
            let size = parse_size_in_bytes(size_text).ok_or_else(invalid)?;
            return Ok(range_for_size(size));
        }
    }

    let size = parse_size_in_bytes(trimmed_value).ok_or_else(invalid)?;
    Ok(SizeRange {
        minimum: size,
        maximum: size,
    })
}

/// `1.5mb` is 1,572,864 bytes. Units are binary (1 kb = 1024 bytes) as in Windows Explorer and
/// Everything; a number without a unit is bytes.
fn parse_size_in_bytes(text: &str) -> Option<u64> {
    let trimmed_text = text.trim().to_ascii_lowercase();
    let unit_start = trimmed_text
        .find(|character: char| !(character.is_ascii_digit() || character == '.'))
        .unwrap_or(trimmed_text.len());
    let (number_text, unit_text) = trimmed_text.split_at(unit_start);

    let power_of_1024 = match unit_text.trim() {
        "" | "b" | "byte" | "bytes" => 0,
        "k" | "kb" | "kib" => 1,
        "m" | "mb" | "mib" => 2,
        "g" | "gb" | "gib" => 3,
        "t" | "tb" | "tib" => 4,
        _ => return None,
    };

    if number_text.is_empty() {
        return None;
    }
    let number: f64 = number_text.parse().ok()?;
    let bytes = number * BYTES_PER_KIBIBYTE.powi(power_of_1024);
    if !bytes.is_finite() || bytes < 0.0 || bytes >= 18_446_744_073_709_551_615.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole_bytes = bytes.round() as u64;
    Some(whole_bytes)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn term_texts(query: &ParsedQuery) -> Vec<(TermScope, bool)> {
        query
            .terms
            .iter()
            .map(|term| (term.scope, term.pattern.is_glob()))
            .collect()
    }

    fn matches_name(query: &ParsedQuery, name: &str) -> bool {
        let lowercase_name = name.to_lowercase();
        query
            .terms
            .iter()
            .all(|term| term.pattern.matches(lowercase_name.as_bytes()))
    }

    #[test]
    fn splits_terms_on_whitespace() {
        let query = ParsedQuery::parse("  annual\treport  2024 ").unwrap();

        assert_eq!(query.terms.len(), 3);
        assert!(matches_name(&query, "Annual Report 2024.pdf"));
        assert!(!matches_name(&query, "Annual Report 2023.pdf"));
    }

    #[test]
    fn an_empty_or_blank_query_is_empty() {
        assert!(ParsedQuery::parse("").unwrap().is_empty());
        assert!(ParsedQuery::parse("   \t ").unwrap().is_empty());
        assert!(ParsedQuery::parse("\"\"").unwrap().is_empty());
        assert!(ParsedQuery::parse("ext: size: type:").unwrap().is_empty());
        assert!(!ParsedQuery::parse("ext:iso").unwrap().is_empty());
    }

    #[test]
    fn quotes_keep_spaces_inside_one_term() {
        let query = ParsedQuery::parse("\"annual report\" final").unwrap();

        assert_eq!(query.terms.len(), 2);
        assert!(matches_name(&query, "annual report final.pdf"));
        assert!(!matches_name(&query, "annual-report final.pdf"));
    }

    #[test]
    fn an_unclosed_quote_runs_to_the_end() {
        let query = ParsedQuery::parse("\"my holiday").unwrap();

        assert_eq!(query.terms.len(), 1);
        assert!(matches_name(&query, "my holiday.jpg"));
    }

    #[test]
    fn terms_with_separators_match_the_full_path() {
        let query = ParsedQuery::parse(r"users\me photos/2024 beach").unwrap();

        assert_eq!(
            term_texts(&query),
            [
                (TermScope::FullPath, false),
                (TermScope::FullPath, false),
                (TermScope::Name, false)
            ]
        );
        let separator = MAIN_SEPARATOR_STR;
        assert!(
            query.terms[1]
                .pattern
                .matches(format!("c:{separator}photos{separator}2024{separator}x").as_bytes())
        );
    }

    #[test]
    fn wildcards_make_a_glob() {
        let query = ParsedQuery::parse("*.JPG img_????").unwrap();

        assert_eq!(
            term_texts(&query),
            [(TermScope::Name, true), (TermScope::Name, true)]
        );
        assert!(query.terms[0].pattern.matches(b"beach.jpg"));
        assert!(!query.terms[0].pattern.matches(b"beach.jpeg"));
        assert!(query.terms[1].pattern.matches(b"img_0001"));
    }

    #[test]
    fn terms_are_case_insensitive_also_beyond_ascii() {
        let query = ParsedQuery::parse("ÜBER").unwrap();

        assert!(matches_name(&query, "Überweisung.pdf"));
    }

    #[test]
    fn parses_extension_lists() {
        let query = ParsedQuery::parse("ext:JPG;.png,heic ext:raw").unwrap();

        assert_eq!(
            query.extension_filters,
            [
                ExtensionAlternatives {
                    lowercase_extensions: vec!["jpg".into(), "png".into(), "heic".into()]
                },
                ExtensionAlternatives {
                    lowercase_extensions: vec!["raw".into()]
                }
            ]
        );
        assert_eq!(query.terms.len(), 0);
    }

    #[test]
    fn filter_names_are_case_insensitive_and_quoted_prefixes_are_text() {
        let filter = ParsedQuery::parse("EXT:pdf").unwrap();
        let quoted = ParsedQuery::parse("\"ext:pdf\"").unwrap();

        assert_eq!(filter.extension_filters.len(), 1);
        assert_eq!(quoted.extension_filters, []);
        assert_eq!(quoted.terms.len(), 1);
    }

    #[test]
    fn a_quoted_filter_value_keeps_its_spaces() {
        let query = ParsedQuery::parse("folder:\"my photos\"").unwrap();

        assert_eq!(query.kind_filters, [NodeKind::Directory]);
        assert!(matches_name(&query, "My Photos"));
        assert!(!matches_name(&query, "My"));
    }

    #[test]
    fn a_drive_path_is_a_term_not_a_filter() {
        let query = ParsedQuery::parse(r"C:\Users").unwrap();

        assert_eq!(term_texts(&query), [(TermScope::FullPath, false)]);
    }

    #[test]
    fn unknown_filter_names_are_text() {
        let query = ParsedQuery::parse("note:2024").unwrap();

        assert_eq!(query.terms.len(), 1);
        assert!(matches_name(&query, "note:2024"));
    }

    #[test]
    fn refuses_unsupported_everything_filters() {
        assert_eq!(
            ParsedQuery::parse("dm:today").unwrap_err(),
            SearchQueryError::UnsupportedFilter {
                filter: "dm".into()
            }
        );
        assert!(ParsedQuery::parse("regex:^a").is_err());
    }

    #[test]
    fn parses_type_filters() {
        assert_eq!(
            ParsedQuery::parse("type:folder").unwrap().kind_filters,
            [NodeKind::Directory]
        );
        assert_eq!(
            ParsedQuery::parse("type:FILE").unwrap().kind_filters,
            [NodeKind::File]
        );
        assert_eq!(
            ParsedQuery::parse("file: folder:").unwrap().kind_filters,
            [NodeKind::File, NodeKind::Directory]
        );
        assert_eq!(
            ParsedQuery::parse("type:pdf").unwrap_err(),
            SearchQueryError::InvalidTypeFilter {
                value: "pdf".into()
            }
        );
    }

    #[test]
    fn parses_size_comparisons_with_binary_units() {
        let range = |text: &str| ParsedQuery::parse(text).unwrap().size_ranges[0];

        assert_eq!(
            range("size:>100mb"),
            SizeRange {
                minimum: 100 * 1024 * 1024 + 1,
                maximum: u64::MAX
            }
        );
        assert_eq!(
            range("size:<1kb"),
            SizeRange {
                minimum: 0,
                maximum: 1023
            }
        );
        assert_eq!(
            range("size:>=1GB"),
            SizeRange {
                minimum: 1 << 30,
                maximum: u64::MAX
            }
        );
        assert_eq!(
            range("size:<=10k"),
            SizeRange {
                minimum: 0,
                maximum: 10 * 1024
            }
        );
        assert_eq!(
            range("size:1mb..10mb"),
            SizeRange {
                minimum: 1 << 20,
                maximum: 10 << 20
            }
        );
        assert_eq!(
            range("size:4096"),
            SizeRange {
                minimum: 4096,
                maximum: 4096
            }
        );
        assert_eq!(
            range("size:=1.5kb"),
            SizeRange {
                minimum: 1536,
                maximum: 1536
            }
        );
    }

    #[test]
    fn size_below_zero_matches_nothing() {
        let range = ParsedQuery::parse("size:<0").unwrap().size_ranges[0];

        assert!(!range.contains(0));
        assert!(!range.contains(1));
    }

    #[test]
    fn refuses_malformed_sizes() {
        for text in [
            "size:>",
            "size:big",
            "size:10xb",
            "size:.",
            "size:1..",
            "size:..1",
            "size:-5",
            "size:99999999999tb",
        ] {
            assert!(
                matches!(
                    ParsedQuery::parse(text),
                    Err(SearchQueryError::InvalidSizeFilter { .. })
                ),
                "{text} should be refused"
            );
        }
    }

    #[test]
    fn combines_filters_and_terms() {
        let query = ParsedQuery::parse("holiday ext:jpg size:>1mb type:file").unwrap();

        assert_eq!(query.terms.len(), 1);
        assert_eq!(query.extension_filters.len(), 1);
        assert_eq!(query.size_ranges.len(), 1);
        assert_eq!(query.kind_filters, [NodeKind::File]);
    }
}
