import {
  commands,
  type ElevationCommandError,
  type ElevationStatus,
  type ScanTarget,
  type StartupScanRequest,
} from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type { ElevationCommandError, ElevationStatus, StartupScanRequest } from "./generated";

function describeElevationCommandError(commandError: ElevationCommandError): string {
  switch (commandError.kind) {
    case "alreadyElevated":
      return "The app already runs as administrator.";
    case "elevationDeclined":
      return "Administrator rights were not granted, so the app keeps running as before.";
    case "relaunchFailed":
      return `The app could not restart as administrator. ${commandError.details}`;
    case "backgroundTaskFailed":
      return "The restart stopped unexpectedly. Please try again.";
  }
}

/** Whether the app runs as administrator, which the fast scan needs. */
export async function getElevationStatus(): Promise<ElevationStatus> {
  const result = await commands.getElevationStatus();
  return unwrapCommandResult(result, describeElevationCommandError);
}

/**
 * Restarts the app as administrator with a fast scan of `target`. When the user accepts the
 * Windows prompt this window closes; when they decline it rejects with `elevationDeclined`.
 */
export async function relaunchElevated(target: ScanTarget): Promise<void> {
  const result = await commands.relaunchElevated(target);
  unwrapCommandResult(result, describeElevationCommandError);
}

/** The scan this instance was started for by `relaunchElevated`, if any. Start it once on load. */
export async function getStartupScanRequest(): Promise<StartupScanRequest | null> {
  const result = await commands.getStartupScanRequest();
  return unwrapCommandResult(result, describeElevationCommandError);
}
