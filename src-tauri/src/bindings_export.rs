#[cfg(any(test, debug_assertions))]
use std::path::Path;

#[cfg(any(test, debug_assertions))]
use specta_typescript::Typescript;
use tauri::Wry;
use tauri_specta::{Builder, collect_commands, collect_events};

use crate::commands::{
    duplicate_commands, elevation_commands, file_action_commands, platform_commands, scan_commands,
    search_commands, tree_commands, update_commands, volume_commands,
};
use crate::events::{DuplicateSearchFinished, ScanFinished, SearchIndexStatusChanged};

/// Where the generated bindings live in the frontend. Built from the manifest directory so the
/// export works no matter which directory the app or the test runs from. Release builds never
/// export bindings, so the export code is left out of them.
#[cfg(any(test, debug_assertions))]
const TYPESCRIPT_BINDINGS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/lib/backend/generated.ts"
);

/// Registers every command and event that crosses the IPC boundary.
pub fn create_bindings_builder() -> Builder<Wry> {
    Builder::<Wry>::new()
        .commands(collect_commands![
            volume_commands::list_volumes,
            scan_commands::start_scan,
            scan_commands::cancel_scan,
            scan_commands::get_scan_summary,
            scan_commands::list_scans,
            scan_commands::discard_scan,
            tree_commands::get_children,
            tree_commands::get_node_details,
            tree_commands::get_path_to_root,
            tree_commands::get_visualization_subtree,
            tree_commands::get_largest_files,
            tree_commands::get_file_type_breakdown,
            tree_commands::search_nodes,
            file_action_commands::reveal_in_explorer,
            file_action_commands::open_with_default_app,
            file_action_commands::reveal_duplicate_in_explorer,
            file_action_commands::open_duplicate_with_default_app,
            file_action_commands::move_to_recycle_bin,
            file_action_commands::get_thumbnail,
            duplicate_commands::get_default_duplicate_folders,
            duplicate_commands::check_duplicate_search_folder,
            duplicate_commands::start_duplicate_search,
            duplicate_commands::cancel_duplicate_search,
            duplicate_commands::get_duplicate_search_summary,
            duplicate_commands::discard_duplicate_search,
            duplicate_commands::get_duplicate_groups,
            duplicate_commands::set_keep_rule,
            duplicate_commands::set_file_keep,
            duplicate_commands::recycle_duplicates,
            elevation_commands::get_elevation_status,
            elevation_commands::relaunch_elevated,
            elevation_commands::relaunch_as_administrator,
            elevation_commands::get_startup_scan_request,
            search_commands::get_search_index_status,
            search_commands::refresh_search_index,
            search_commands::search_files,
            search_commands::reveal_search_hit,
            search_commands::open_search_hit,
            platform_commands::get_platform_info,
            update_commands::check_for_update,
        ])
        .events(collect_events![
            ScanFinished,
            DuplicateSearchFinished,
            SearchIndexStatusChanged
        ])
        // Byte counts are u64. JavaScript numbers hold integers exactly up to 2^53 bytes (8 PiB),
        // far above any single volume, so plain numbers are used instead of BigInt.
        .dangerously_cast_bigints_to_number()
}

/// Writes the TypeScript bindings to `src/lib/backend/generated.ts`.
#[cfg(any(test, debug_assertions))]
pub fn export_typescript_bindings(
    bindings_builder: &Builder<Wry>,
) -> Result<(), specta_typescript::Error> {
    export_typescript_bindings_to(bindings_builder, Path::new(TYPESCRIPT_BINDINGS_PATH))
}

#[cfg(any(test, debug_assertions))]
fn export_typescript_bindings_to(
    bindings_builder: &Builder<Wry>,
    destination: &Path,
) -> Result<(), specta_typescript::Error> {
    bindings_builder.export(Typescript::default(), destination)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::fs;

    use super::*;

    fn read_with_unix_line_endings(path: &Path) -> String {
        fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
            .replace("\r\n", "\n")
    }

    #[test]
    fn committed_typescript_bindings_are_up_to_date() {
        let temporary_directory =
            std::env::temp_dir().join(format!("vsm-bindings-check-{}", std::process::id()));
        fs::create_dir_all(&temporary_directory).unwrap();
        let freshly_exported_path = temporary_directory.join("generated.ts");

        export_typescript_bindings_to(&create_bindings_builder(), &freshly_exported_path).unwrap();
        let freshly_exported = read_with_unix_line_endings(&freshly_exported_path);
        let committed = read_with_unix_line_endings(Path::new(TYPESCRIPT_BINDINGS_PATH));
        fs::remove_dir_all(&temporary_directory).unwrap();

        assert!(
            freshly_exported == committed,
            "src/lib/backend/generated.ts is out of date. Run `pnpm bindings` and commit the result."
        );
    }

    #[test]
    #[ignore = "writes src/lib/backend/generated.ts; run through `pnpm bindings`"]
    fn export_typescript_bindings_to_frontend() {
        export_typescript_bindings(&create_bindings_builder()).unwrap();
    }
}
