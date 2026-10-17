//! Case-insensitive name handling without a stored lowercase copy of every name. Names that are
//! already lowercase (most of them) are used as they are; ASCII names are lowercased byte by
//! byte into a reusable buffer; only the few other names go through `str::to_lowercase`, so
//! the results are exactly those of comparing `to_lowercase()` copies.

use std::borrow::Cow;
use std::cmp::Ordering;

use super::search_index_node::SearchNodeFlags;

/// The flags `VolumeSearchIndex` stores for a name so that queries can skip lowercasing it.
pub(crate) fn name_case_flags(name: &str) -> SearchNodeFlags {
    let mut flags = SearchNodeFlags::empty();
    if name.is_ascii() {
        flags |= SearchNodeFlags::NAME_IS_ASCII;
        if !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
            flags |= SearchNodeFlags::NAME_IS_LOWERCASE;
        }
    } else if name.to_lowercase() == name {
        flags |= SearchNodeFlags::NAME_IS_LOWERCASE;
    }
    flags
}

/// The lowercase UTF-8 bytes of `name`, written to `buffer` when the name is not lowercase
/// already.
pub(crate) fn lowercase_name<'text>(
    name: &'text str,
    flags: SearchNodeFlags,
    buffer: &'text mut Vec<u8>,
) -> &'text [u8] {
    if flags.contains(SearchNodeFlags::NAME_IS_LOWERCASE) {
        return name.as_bytes();
    }

    buffer.clear();
    if flags.contains(SearchNodeFlags::NAME_IS_ASCII) {
        buffer.extend(name.bytes().map(|byte| byte.to_ascii_lowercase()));
    } else {
        buffer.extend_from_slice(name.to_lowercase().as_bytes());
    }
    buffer
}

/// Compares two names as their lowercase forms compare, allocating only for names that are
/// neither ASCII nor lowercase already.
pub(crate) fn compare_names_ignoring_case(
    (first, first_flags): (&str, SearchNodeFlags),
    (second, second_flags): (&str, SearchNodeFlags),
) -> Ordering {
    let both_are_ascii = first_flags.contains(SearchNodeFlags::NAME_IS_ASCII)
        && second_flags.contains(SearchNodeFlags::NAME_IS_ASCII);
    if both_are_ascii {
        let first_bytes = first.bytes().map(|byte| byte.to_ascii_lowercase());
        let second_bytes = second.bytes().map(|byte| byte.to_ascii_lowercase());
        return first_bytes.cmp(second_bytes);
    }
    lowercase_copy(first, first_flags).cmp(&lowercase_copy(second, second_flags))
}

fn lowercase_copy(name: &str, flags: SearchNodeFlags) -> Cow<'_, str> {
    if flags.contains(SearchNodeFlags::NAME_IS_LOWERCASE) {
        Cow::Borrowed(name)
    } else if flags.contains(SearchNodeFlags::NAME_IS_ASCII) {
        Cow::Owned(name.to_ascii_lowercase())
    } else {
        Cow::Owned(name.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lowercase(name: &str) -> String {
        let mut buffer = Vec::new();
        String::from_utf8(lowercase_name(name, name_case_flags(name), &mut buffer).to_vec())
            .unwrap_or_default()
    }

    fn compare(first: &str, second: &str) -> Ordering {
        compare_names_ignoring_case(
            (first, name_case_flags(first)),
            (second, name_case_flags(second)),
        )
    }

    const NAMES: [&str; 16] = [
        "",
        "a",
        "A",
        "abc",
        "ABD",
        "Ärger.TXT",
        "ärger.txt",
        "straße",
        "STRASSE",
        "ΟΔΟΣ",
        "İstanbul",
        "\u{212A}elvin",
        "kelvin",
        "z",
        "Zebra 2.Jpg ",
        "日本語.txt",
    ];

    #[test]
    fn lowercases_exactly_like_to_lowercase() {
        for name in NAMES {
            assert_eq!(lowercase(name), name.to_lowercase(), "{name:?}");
        }
    }

    #[test]
    fn compares_exactly_like_lowercase_copies() {
        for first in NAMES {
            for second in NAMES {
                assert_eq!(
                    compare(first, second),
                    first.to_lowercase().cmp(&second.to_lowercase()),
                    "{first:?} and {second:?}"
                );
            }
        }
    }

    #[test]
    fn marks_names_that_need_no_lowercasing() {
        assert!(name_case_flags("photos.jpg").contains(SearchNodeFlags::NAME_IS_LOWERCASE));
        assert!(!name_case_flags("Photos.jpg").contains(SearchNodeFlags::NAME_IS_LOWERCASE));
        assert!(name_case_flags("ärger").contains(SearchNodeFlags::NAME_IS_LOWERCASE));
        assert!(!name_case_flags("ärger").contains(SearchNodeFlags::NAME_IS_ASCII));
        assert!(!name_case_flags("Ärger").contains(SearchNodeFlags::NAME_IS_LOWERCASE));
    }
}
