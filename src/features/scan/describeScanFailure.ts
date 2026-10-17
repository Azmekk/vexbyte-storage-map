import type { ScanFailure } from "@/lib/backend/scans";

export type ScanFailureDescription = {
  title: string;
  explanation: string;
  /** Whether a standard scan is a sensible next step. */
  suggestsStandardScan: boolean;
};

export function describeScanFailure(failure: ScanFailure): ScanFailureDescription {
  switch (failure.kind) {
    case "fastScanRequiresElevation":
      return {
        title: "The fast scan needs administrator rights",
        explanation: "Restart the app as administrator, or run a standard scan instead.",
        suggestsStandardScan: true,
      };
    case "fastScanUnsupported":
      return {
        title: "The fast scan does not work on this drive",
        explanation: `${failure.path} is not on an NTFS drive. A standard scan works everywhere.`,
        suggestsStandardScan: true,
      };
    case "volumeLookupFailed":
      return {
        title: "The drive could not be read",
        explanation: `Windows could not tell which drive ${failure.path} is on. ${failure.details}`,
        suggestsStandardScan: true,
      };
    case "folderNotFoundOnVolume":
      return {
        title: "The folder was not found",
        explanation: `${failure.path} no longer exists, or it moved while the scan ran.`,
        suggestsStandardScan: false,
      };
    case "folderWalkFailed":
      return {
        title: "The scan stopped early",
        explanation: failure.details,
        suggestsStandardScan: false,
      };
    case "masterFileTableScanFailed":
      return {
        title: "The fast scan stopped early",
        explanation: `${failure.details} A standard scan reads the drive another way.`,
        suggestsStandardScan: true,
      };
    case "scanThreadFailed":
      return {
        title: "The scan stopped unexpectedly",
        explanation: failure.details,
        suggestsStandardScan: false,
      };
  }
}
