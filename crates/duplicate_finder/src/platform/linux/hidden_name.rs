use std::ffi::OsStr;

/// File managers and `ls` hide names that start with a dot; Linux has no hidden attribute.
pub(super) fn is_hidden_name(name: &OsStr) -> bool {
    name.as_encoded_bytes().first() == Some(&b'.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_names_are_hidden() {
        assert!(is_hidden_name(OsStr::new(".config")));
        assert!(!is_hidden_name(OsStr::new("Pictures")));
        assert!(!is_hidden_name(OsStr::new("")));
    }
}
