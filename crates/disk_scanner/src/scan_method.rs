use serde::{Deserialize, Serialize};
use specta::Type;

/// How a drive or folder is scanned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScanMethod {
    /// Lists every directory through the file system. Works on every volume without admin rights.
    FolderWalk,
    /// Reads the NTFS Master File Table directly. Needs admin rights and an NTFS volume.
    MasterFileTable,
}
