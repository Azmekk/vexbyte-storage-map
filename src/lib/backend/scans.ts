import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  commands,
  events,
  type ScanCommandError,
  type ScanFinished,
  type ScanId,
  type ScanMethod,
  type ScanOverview,
  type ScanProgressUpdate,
  type ScanTarget,
} from "./generated";
import { createProgressChannel } from "./progressChannel";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type {
  ScanCommandError,
  ScanFailure,
  ScanFinished,
  ScanId,
  ScanMethod,
  ScanOutcome,
  ScanOverview,
  ScanPhase,
  ScanProgressUpdate,
  ScanState,
  ScanStatus,
  ScanSummary,
  ScanTarget,
  TopLevelDirectoryProgress,
  TreeTotals,
} from "./generated";

function describeScanCommandError(commandError: ScanCommandError): string {
  switch (commandError.kind) {
    case "scanNotFound":
      return "This scan is no longer available. Please scan again.";
    case "fastScanRequiresElevation":
      return "The fast scan needs administrator rights. Restart as admin or use the standard scan.";
    case "cloudVirtualVolumeNotScanned":
      return `${commandError.mountPoint} is a streamed cloud drive. Cloud drives aren't scanned, so nothing gets downloaded.`;
    case "scanStartFailed":
      return `The scan could not be started. ${commandError.details}`;
  }
}

/**
 * Starts a scan in the background and returns its id right away. `onProgress` receives about ten
 * updates per second until the scan ends; listen with `subscribeToScanFinished` to learn how it
 * ended.
 */
export async function startScan(
  target: ScanTarget,
  method: ScanMethod,
  onProgress: (progress: ScanProgressUpdate) => void,
): Promise<ScanId> {
  const progressChannel = createProgressChannel(onProgress);
  const result = await commands.startScan(target, method, progressChannel);
  return unwrapCommandResult(result, describeScanCommandError);
}

/** Asks a running scan to stop; it then finishes with the `cancelled` outcome. */
export async function cancelScan(scanId: ScanId): Promise<void> {
  const result = await commands.cancelScan(scanId);
  unwrapCommandResult(result, describeScanCommandError);
}

/** Status of a scan plus, once finished, its summary, current totals and root node. */
export async function getScanSummary(scanId: ScanId): Promise<ScanOverview> {
  const result = await commands.getScanSummary(scanId);
  return unwrapCommandResult(result, describeScanCommandError);
}

/** Every scan of this session, oldest first. */
export async function listScans(): Promise<ScanOverview[]> {
  const result = await commands.listScans();
  return unwrapCommandResult(result, describeScanCommandError);
}

/** Forgets a scan and frees its memory. A running scan is cancelled first. */
export async function discardScan(scanId: ScanId): Promise<void> {
  const result = await commands.discardScan(scanId);
  unwrapCommandResult(result, describeScanCommandError);
}

/**
 * Calls `onScanFinished` whenever any scan ends. The result is stored before the event is sent,
 * so tree queries work inside the callback. Resolves to a function that stops listening.
 */
export function subscribeToScanFinished(
  onScanFinished: (scanFinished: ScanFinished) => void,
): Promise<UnlistenFn> {
  return events.scanFinished.listen((event) => onScanFinished(event.payload));
}
