//! Safe wrappers around every Win32 call this crate makes. No other module uses `unsafe`.

mod critical_error_dialog_suppression;
mod disk_space;
mod drive_type;
mod error_codes;
mod logical_drives;
mod volume_information;
mod wide_string;

pub(crate) use critical_error_dialog_suppression::CriticalErrorDialogSuppression;
pub(crate) use disk_space::{DiskSpace, query_disk_space};
pub(crate) use drive_type::query_drive_type;
pub(crate) use error_codes::is_drive_not_ready_error;
pub(crate) use logical_drives::query_logical_drive_letters;
pub(crate) use volume_information::{VolumeInformation, query_volume_information};
