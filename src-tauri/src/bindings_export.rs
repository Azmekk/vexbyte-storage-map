use std::path::Path;

use specta_typescript::Typescript;
use tauri::Wry;
use tauri_specta::{Builder, collect_commands};

use crate::commands::volume_commands;

/// Where the generated bindings live in the frontend. Built from the manifest directory so the
/// export works no matter which directory the app or the test runs from.
const TYPESCRIPT_BINDINGS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/lib/backend/generated.ts"
);

/// Registers every command and event that crosses the IPC boundary.
pub fn create_bindings_builder() -> Builder<Wry> {
    Builder::<Wry>::new()
        .commands(collect_commands![volume_commands::list_volumes])
        // Byte counts are u64. JavaScript numbers hold integers exactly up to 2^53 bytes (8 PiB),
        // far above any single volume, so plain numbers are used instead of BigInt.
        .dangerously_cast_bigints_to_number()
}

/// Writes the TypeScript bindings to `src/lib/backend/generated.ts`.
pub fn export_typescript_bindings(
    bindings_builder: &Builder<Wry>,
) -> Result<(), specta_typescript::Error> {
    export_typescript_bindings_to(bindings_builder, Path::new(TYPESCRIPT_BINDINGS_PATH))
}

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
