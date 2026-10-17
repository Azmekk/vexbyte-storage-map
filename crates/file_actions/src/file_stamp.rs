use std::time::SystemTime;

/// Size, modification time and kind of a path, read from file system metadata without opening it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileStamp {
    pub(crate) size: u64,
    pub(crate) modified_time: SystemTime,
    pub(crate) is_directory: bool,
    /// The content lives in the cloud; opening it for reading would download it.
    pub(crate) is_cloud_placeholder: bool,
}
