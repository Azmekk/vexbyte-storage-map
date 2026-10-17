use memchr::memmem;

/// One search term, compiled for matching against lowercase UTF-8 text.
///
/// A term without wildcards matches anywhere inside the text. A term with `*` or `?` is a glob
/// that has to match the whole text: `*` stands for any run of characters, `?` for exactly one.
#[derive(Debug, Clone)]
pub(crate) enum TextPattern {
    Substring {
        finder: Box<memmem::Finder<'static>>,
    },
    Glob {
        lowercase_pattern: Box<[u8]>,
    },
}

impl TextPattern {
    /// Compiles `lowercase_term`, which the caller has already lowercased.
    pub(crate) fn new(lowercase_term: &str) -> Self {
        let has_wildcards = lowercase_term.contains(['*', '?']);
        if has_wildcards {
            Self::Glob {
                lowercase_pattern: lowercase_term.as_bytes().into(),
            }
        } else {
            Self::Substring {
                finder: Box::new(memmem::Finder::new(lowercase_term.as_bytes()).into_owned()),
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn is_glob(&self) -> bool {
        matches!(self, Self::Glob { .. })
    }

    pub(crate) fn matches(&self, lowercase_text: &[u8]) -> bool {
        match self {
            Self::Substring { finder } => finder.find(lowercase_text).is_some(),
            Self::Glob { lowercase_pattern } => glob_matches(lowercase_pattern, lowercase_text),
        }
    }
}

/// Matches a whole UTF-8 text against a glob in linear time for patterns with one `*` and in
/// `O(pattern * text)` at worst, by remembering only the most recent `*` (the classic
/// backtracking wildcard algorithm). `?` consumes one UTF-8 character, not one byte.
fn glob_matches(pattern: &[u8], text: &[u8]) -> bool {
    let mut pattern_position = 0;
    let mut text_position = 0;
    let mut last_star: Option<(usize, usize)> = None;

    while text_position < text.len() {
        match pattern.get(pattern_position) {
            Some(b'*') => {
                last_star = Some((pattern_position, text_position));
                pattern_position += 1;
                continue;
            }
            Some(b'?') => {
                pattern_position += 1;
                text_position += utf8_character_length_at(text, text_position);
                continue;
            }
            Some(&literal) if literal == text[text_position] => {
                pattern_position += 1;
                text_position += 1;
                continue;
            }
            _ => {}
        }

        let Some((star_pattern_position, star_text_position)) = last_star else {
            return false;
        };
        let retry_text_position =
            star_text_position + utf8_character_length_at(text, star_text_position);
        last_star = Some((star_pattern_position, retry_text_position));
        pattern_position = star_pattern_position + 1;
        text_position = retry_text_position;
    }

    pattern[pattern_position.min(pattern.len())..]
        .iter()
        .all(|&byte| byte == b'*')
}

fn utf8_character_length_at(text: &[u8], position: usize) -> usize {
    let length = match text.get(position) {
        Some(&lead_byte) if lead_byte < 0xC0 => 1,
        Some(&lead_byte) if lead_byte < 0xE0 => 2,
        Some(&lead_byte) if lead_byte < 0xF0 => 3,
        Some(_) => 4,
        None => 1,
    };
    length.min(text.len().saturating_sub(position)).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(term: &str, text: &str) -> bool {
        TextPattern::new(term).matches(text.as_bytes())
    }

    #[test]
    fn a_plain_term_matches_anywhere() {
        assert!(matches("report", "annual report 2024.pdf"));
        assert!(matches("2024", "annual report 2024.pdf"));
        assert!(matches("", "anything"));
        assert!(!matches("reports", "annual report.pdf"));
        assert!(!TextPattern::new("report").is_glob());
    }

    #[test]
    fn a_star_matches_any_run_and_the_glob_spans_the_whole_name() {
        assert!(matches("*.jpg", "beach.jpg"));
        assert!(matches("*.jpg", ".jpg"));
        assert!(!matches("*.jpg", "beach.jpg.bak"));
        assert!(matches("img*", "img_0001.png"));
        assert!(!matches("img*", "my img.png"));
        assert!(matches("a*b*c", "aXXbYYc"));
        assert!(matches("a*b*c", "abc"));
        assert!(!matches("a*b*c", "acb"));
        assert!(matches("*", ""));
        assert!(matches("**", "x"));
        assert!(TextPattern::new("*.jpg").is_glob());
    }

    #[test]
    fn a_star_backtracks_over_repeated_prefixes() {
        assert!(matches("*aab", "aaaab"));
        assert!(matches("*ab*ab", "xxabyyabab"));
        assert!(!matches("*ab*ab", "xxabyyaba"));
    }

    #[test]
    fn a_question_mark_matches_exactly_one_character() {
        assert!(matches("file?.txt", "file1.txt"));
        assert!(!matches("file?.txt", "file.txt"));
        assert!(!matches("file?.txt", "file12.txt"));
        assert!(matches("??", "ab"));
        assert!(!matches("??", "a"));
    }

    #[test]
    fn a_question_mark_consumes_a_whole_multibyte_character() {
        assert!(matches("?ber.pdf", "über.pdf"));
        assert!(matches("caf?", "café"));
        assert!(!matches("caf??", "café"));
        assert!(matches("*é", "café"));
        assert!(matches("*\u{1F600}?", "smile\u{1F600}x"));
    }

    #[test]
    fn non_ascii_substrings_match() {
        assert!(matches("über", "überweisung.pdf"));
        assert!(!matches("über", "uberweisung.pdf"));
    }
}
