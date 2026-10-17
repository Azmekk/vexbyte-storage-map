import { ChevronDownIcon, ScanSearchIcon, ShieldIcon, ZapIcon } from "lucide-react";
import { useMemo } from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { ScanTarget } from "@/lib/backend/scans";
import { chooseScanOptions } from "./chooseScanOptions";
import { ElevationProblemDialog } from "./ElevationProblemDialog";
import { useScanStarter } from "./useScanStarter";

type ScanTargetButtonProps = {
  mountPoint: string;
  label: string;
  supportsFastScan: boolean;
  isElevated: boolean | null;
};

/**
 * The big Scan button of a drive card. On NTFS without admin rights it is a split button whose
 * menu offers the restart as admin for the fast scan.
 */
export function ScanTargetButton({
  mountPoint,
  label,
  supportsFastScan,
  isElevated,
}: ScanTargetButtonProps) {
  const target = useMemo<ScanTarget>(() => ({ volume: { mountPoint } }), [mountPoint]);
  const scanStarter = useScanStarter(target);
  const scanOptions = chooseScanOptions(supportsFastScan, isElevated);

  const problemDialog = (
    <ElevationProblemDialog
      problem={scanStarter.elevationProblem}
      onDismiss={scanStarter.dismissElevationProblem}
      onStartStandardScan={() => scanStarter.startScanWithMethod("folderWalk")}
    />
  );

  if (scanOptions.kind === "single") {
    const isFastScan = scanOptions.method === "masterFileTable";
    return (
      <>
        <Button
          size="lg"
          className="w-full"
          disabled={scanStarter.isStarting}
          onClick={() => scanStarter.startScanWithMethod(scanOptions.method)}
        >
          {isFastScan ? <ZapIcon /> : <ScanSearchIcon />}
          {isFastScan ? `Fast scan ${label}` : `Scan ${label}`}
        </Button>
        {problemDialog}
      </>
    );
  }

  return (
    <>
      <div className="flex w-full">
        <Button
          size="lg"
          className="flex-1 rounded-r-none"
          disabled={scanStarter.isStarting}
          onClick={() => scanStarter.startScanWithMethod("folderWalk")}
        >
          <ScanSearchIcon />
          Scan {label}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              size="lg"
              className="rounded-l-none border-primary-foreground/20 border-l px-3"
              aria-label={`More ways to scan ${label}`}
            >
              <ChevronDownIcon />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-72">
            <DropdownMenuItem onSelect={() => scanStarter.startScanWithMethod("folderWalk")}>
              <ScanSearchIcon />
              <div className="grid">
                <span>Standard scan</span>
                <span className="text-muted-foreground text-xs">Works without admin rights</span>
              </div>
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={scanStarter.restartAsAdminForFastScan}>
              <ShieldIcon />
              <div className="grid">
                <span>Restart as admin for fast scan</span>
                <span className="text-muted-foreground text-xs">
                  Reads the NTFS file table directly, usually in seconds
                </span>
              </div>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      {problemDialog}
    </>
  );
}
