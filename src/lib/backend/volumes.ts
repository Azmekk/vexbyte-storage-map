import { commands, type VolumeCommandError, type VolumeSummary } from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type { DriveType, VolumeCommandError, VolumeKind, VolumeSummary } from "./generated";

function describeVolumeCommandError(commandError: VolumeCommandError): string {
  switch (commandError.kind) {
    case "volumeListFailed":
      return `Windows could not list the drives on this computer. ${commandError.details}`;
    case "backgroundTaskFailed":
      return "Listing the drives stopped unexpectedly. Please try again.";
  }
}

/** Lists the mounted volumes that are ready to be read. */
export async function listVolumes(): Promise<VolumeSummary[]> {
  const result = await commands.listVolumes();
  return unwrapCommandResult(result, describeVolumeCommandError);
}
