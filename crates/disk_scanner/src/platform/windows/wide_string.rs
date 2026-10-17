/// Encodes `text` as a null-terminated UTF-16 string for Win32 `W` functions.
pub(super) fn to_null_terminated_wide_string(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Decodes a UTF-16 buffer that Win32 filled up to its first null character.
pub(super) fn from_null_terminated_wide_buffer(buffer: &[u16]) -> String {
    let length_without_null = buffer
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(buffer.len());

    String::from_utf16_lossy(&buffer[..length_without_null])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_text_through_a_null_terminated_buffer() {
        let wide_string = to_null_terminated_wide_string("C:\\");

        assert_eq!(wide_string.last(), Some(&0));
        assert_eq!(from_null_terminated_wide_buffer(&wide_string), "C:\\");
    }

    #[test]
    fn decodes_a_buffer_without_null_terminator() {
        let buffer: Vec<u16> = "NTFS".encode_utf16().collect();

        assert_eq!(from_null_terminated_wide_buffer(&buffer), "NTFS");
    }
}
