use std::error::Error;
use std::path::Path;

const WINDOWS_APP_MANIFEST_FILE: &str = "windows-app-manifest.xml";

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // tauri-build embeds its manifest into the app binary only. Test binaries linked against
    // Tauri then fail to start with STATUS_ENTRYPOINT_NOT_FOUND because they load the old
    // Common Controls v5. Embedding the manifest through the linker covers every binary.
    let windows_attributes = tauri_build::WindowsAttributes::new_without_app_manifest();
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows_attributes))?;

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(WINDOWS_APP_MANIFEST_FILE);
    println!("cargo:rerun-if-changed={WINDOWS_APP_MANIFEST_FILE}");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest_path.display()
    );

    Ok(())
}
