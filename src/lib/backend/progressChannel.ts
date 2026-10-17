import { Channel } from "@tauri-apps/api/core";

/** Creates the IPC channel a long-running command reports its progress on. */
export function createProgressChannel<Progress>(
  onProgress: (progress: Progress) => void,
): Channel<Progress> {
  const channel = new Channel<Progress>();
  channel.onmessage = onProgress;
  return channel;
}
