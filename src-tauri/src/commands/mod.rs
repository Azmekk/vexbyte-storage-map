//! Tauri commands, one file per area. Every command is async so it runs off the main thread;
//! anything that blocks for more than a few milliseconds moves to a blocking thread.

pub mod duplicate_commands;
pub mod elevation_commands;
pub mod file_action_commands;
pub mod platform_commands;
pub mod scan_commands;
pub mod search_commands;
pub mod tree_commands;
pub mod update_commands;
pub mod volume_commands;
