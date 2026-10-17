//! Bounds-checked reads of little-endian fields from raw NTFS structures. Every read returns
//! `None` instead of panicking when a corrupt record points outside its buffer.

pub(crate) fn read_u8(bytes: &[u8], offset: usize) -> Option<u8> {
    bytes.get(offset).copied()
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let field = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes(field.try_into().ok()?))
}

pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let field = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(field.try_into().ok()?))
}

pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let field = bytes.get(offset..offset.checked_add(8)?)?;
    Some(u64::from_le_bytes(field.try_into().ok()?))
}

/// Decodes `character_count` UTF-16LE code units starting at `offset` into `output`.
/// Unpaired surrogates, which NTFS allows in names, become replacement characters.
pub(crate) fn read_utf16_into(
    bytes: &[u8],
    offset: usize,
    character_count: usize,
    output: &mut String,
) -> Option<()> {
    let text_bytes = bytes.get(offset..offset.checked_add(character_count.checked_mul(2)?)?)?;
    let (code_unit_bytes, _) = text_bytes.as_chunks::<2>();
    let code_units = code_unit_bytes.iter().map(|pair| u16::from_le_bytes(*pair));

    output.clear();
    output.extend(
        char::decode_utf16(code_units)
            .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER)),
    );
    Some(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn reads_fields_and_rejects_out_of_bounds_offsets() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

        assert_eq!(read_u16(&bytes, 0), Some(0x0201));
        assert_eq!(read_u32(&bytes, 4), Some(0x0807_0605));
        assert_eq!(read_u64(&bytes, 0), Some(0x0807_0605_0403_0201));
        assert_eq!(read_u32(&bytes, 6), None);
        assert_eq!(read_u64(&bytes, usize::MAX), None);
    }

    #[test]
    fn decodes_utf16_names() {
        let bytes: Vec<u8> = "Ä.txt".encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut name = String::from("old");

        read_utf16_into(&bytes, 0, 5, &mut name).unwrap();

        assert_eq!(name, "Ä.txt");
        assert_eq!(read_utf16_into(&bytes, 0, 6, &mut name), None);
    }
}
