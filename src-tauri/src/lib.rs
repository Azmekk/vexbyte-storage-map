//! Tauri adapter for Vexbyte Storage Map. It only converts between Tauri commands and the
//! library crates in `crates/`; all real work happens there.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod app_state;
mod bindings_export;
mod commands;
mod duplicate_group_store;
mod duplicate_search_registry;
mod elevation;
mod error_details;
mod events;
mod platform;
mod scan_registry;
mod search_index_builders;
mod search_index_registry;
mod startup_arguments;
mod thumbnail_asset_scope;

use app_state::AppState;
use tauri::Manager;

/// Starts the app and blocks until the last window closes.
///
/// # Errors
///
/// Fails when Tauri cannot create the app or its main window.
pub fn run() -> Result<(), tauri::Error> {
    tracing_subscriber::fmt::init();

    let startup_scan_request =
        startup_arguments::parse_startup_scan_request(std::env::args_os().skip(1));
    let bindings_builder = bindings_export::create_bindings_builder();

    #[cfg(debug_assertions)]
    if let Err(error) = bindings_export::export_typescript_bindings(&bindings_builder) {
        tracing::error!(%error, "could not export the TypeScript bindings");
    }

    let invoke_handler = bindings_builder.invoke_handler();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            bindings_builder.mount_events(app);

            let local_data_directory = app.path().local_data_dir()?;
            app.manage(AppState::new(&local_data_directory, startup_scan_request));
            search_index_builders::start_building_all_search_indexes(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
}
