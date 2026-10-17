use std::ffi::{OsStr, OsString};

/// Joins arguments into one Windows command line that `CommandLineToArgvW` (and Rust's
/// `std::env::args_os`) splits back into exactly the same arguments.
pub(super) fn join_command_line_arguments(arguments: &[OsString]) -> OsString {
    let mut command_line = OsString::new();
    for (position, argument) in arguments.iter().enumerate() {
        if position > 0 {
            command_line.push(" ");
        }
        command_line.push(quote_command_line_argument(argument));
    }
    command_line
}

/// Backslashes are literal except right before a double quote, where each pair becomes one
/// backslash. So `D:\My Photos\` must be written as `"D:\My Photos\\"`, or the closing quote
/// would be escaped.
fn quote_command_line_argument(argument: &OsStr) -> OsString {
    let text = argument.to_string_lossy();
    let needs_quotes = text.is_empty() || text.contains([' ', '\t', '\n', '\u{b}', '"']);
    if !needs_quotes {
        return argument.to_os_string();
    }

    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    let mut pending_backslashes = 0;
    for character in text.chars() {
        match character {
            '\\' => pending_backslashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', pending_backslashes * 2 + 1));
                quoted.push('"');
                pending_backslashes = 0;
            }
            other => {
                quoted.extend(std::iter::repeat_n('\\', pending_backslashes));
                quoted.push(other);
                pending_backslashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', pending_backslashes * 2));
    quoted.push('"');

    OsString::from(quoted)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn joined(arguments: &[&str]) -> String {
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        join_command_line_arguments(&arguments)
            .into_string()
            .unwrap()
    }

    #[test]
    fn leaves_simple_arguments_alone() {
        assert_eq!(joined(&["--scan-volume", r"C:\"]), r"--scan-volume C:\");
    }

    #[test]
    fn doubles_trailing_backslashes_inside_quotes() {
        assert_eq!(
            joined(&["--scan-folder", r"D:\My Photos\"]),
            r#"--scan-folder "D:\My Photos\\""#
        );
    }

    #[test]
    fn escapes_quotes_and_the_backslashes_before_them() {
        assert_eq!(joined(&[r#"a\"b c"#]), r#""a\\\"b c""#);
        assert_eq!(joined(&[""]), r#""""#);
    }
}
