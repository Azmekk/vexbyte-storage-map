import { open } from "@tauri-apps/plugin-dialog";

/** Shows the system folder picker. Resolves to the chosen folder, or `null` when cancelled. */
export async function pickFolder(title: string): Promise<string | null> {
  const selection = await open({ directory: true, multiple: false, title });
  return typeof selection === "string" ? selection : null;
}
