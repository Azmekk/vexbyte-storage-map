use std::io;
use std::path::Path;

use windows::Win32::UI::Shell::{ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems};
use windows::core::PCWSTR;

use super::com_apartment::ComApartment;
use super::io_error::to_io_error;
use super::wide_path::to_shell_wide_path;

/// Opens an Explorer window on the folder containing `path` with that item selected.
pub(crate) fn reveal_in_file_manager(path: &Path) -> io::Result<()> {
    let _com_apartment = ComApartment::enter_single_threaded()?;
    let wide_path = to_shell_wide_path(path)?;

    // SAFETY: `wide_path` is null-terminated and outlives the call. The returned item id list
    // is owned by us and freed below.
    let item_id_list = unsafe { ILCreateFromPathW(PCWSTR(wide_path.as_ptr())) };
    if item_id_list.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "the shell could not resolve the path",
        ));
    }

    // SAFETY: `item_id_list` is a valid absolute id list. Passing no child items asks the shell
    // to open the parent folder and select the item the list itself names.
    let result = unsafe { SHOpenFolderAndSelectItems(item_id_list, None, 0) };

    // SAFETY: `item_id_list` came from ILCreateFromPathW and is freed exactly once.
    unsafe { ILFree(Some(item_id_list)) };

    result.map_err(|error| to_io_error(&error))
}
