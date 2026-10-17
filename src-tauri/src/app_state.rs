/// State shared by every command, managed by Tauri.
///
/// Later milestones add the running and finished scans and duplicate searches here, keyed by
/// their ids. The file trees themselves stay in Rust; the frontend only queries small views.
#[derive(Debug, Default)]
pub struct AppState {}
