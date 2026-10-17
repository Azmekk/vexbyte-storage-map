//! Tauri adapter for Vexbyte Storage Map. It only converts between Tauri commands and the
//! library crates in `crates/`; all real work happens there.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[cfg(not(windows))]
compile_error!("Vexbyte Storage Map only builds for Windows.");

mod app_state;
mod bindings_export;
mod commands;

use app_state::AppState;

/// Starts the app and blocks until the last window closes.
///
/// # Errors
///
/// Fails when Tauri cannot create the app or its main window.
pub fn run() -> Result<(), tauri::Error> {
    tracing_subscriber::fmt::init();

    let bindings_builder = bindings_export::create_bindings_builder();

    #[cfg(debug_assertions)]
    if let Err(error) = bindings_export::export_typescript_bindings(&bindings_builder) {
        tracing::error!(%error, "could not export the TypeScript bindings");
    }

    let invoke_handler = bindings_builder.invoke_handler();

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            bindings_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
}
