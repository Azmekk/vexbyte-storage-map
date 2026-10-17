use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub(super) fn to_null_terminated_wide_string(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}
