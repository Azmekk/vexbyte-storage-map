import { Fragment } from "react";
import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { BreadcrumbItem as BreadcrumbData, NodeIndex } from "@/lib/backend/tree";
import { type BreadcrumbTrailEntry, buildBreadcrumbTrail } from "./buildBreadcrumbTrail";
import { ScanRootChip } from "./ScanRootChip";
import { YourDrivesButton } from "./YourDrivesButton";

const MAXIMUM_VISIBLE_CRUMBS = 4;

function breadcrumbItemClassName(isScanRoot: boolean, entry: BreadcrumbTrailEntry): string {
  if (isScanRoot || entry.kind === "collapsed") {
    return "shrink-0";
  }
  return entry.isCurrent ? "min-w-0" : "min-w-12";
}

export type ScanRootDescription = {
  /** `C:` for a drive, the folder name for a folder. */
  label: string;
  isVolume: boolean;
};

type ExplorerBreadcrumbsProps = {
  crumbs: BreadcrumbData[];
  scanRoot: ScanRootDescription;
  onOpenFolder: (folder: NodeIndex) => void;
  onGoToYourDrives: () => void;
};

/** Your drives, then the scanned drive or folder as a chip, then the folders below it. */
export function ExplorerBreadcrumbs({
  crumbs,
  scanRoot,
  onOpenFolder,
  onGoToYourDrives,
}: ExplorerBreadcrumbsProps) {
  const trail = buildBreadcrumbTrail(crumbs, MAXIMUM_VISIBLE_CRUMBS);

  return (
    <Breadcrumb className="min-w-0">
      <BreadcrumbList className="flex-nowrap gap-1">
        <BreadcrumbItem className="shrink-0">
          <YourDrivesButton onSelect={onGoToYourDrives} />
        </BreadcrumbItem>
        {trail.map((entry, entryIndex) => {
          const entryKey = entry.kind === "crumb" ? `crumb-${entry.crumb.node}` : "collapsed";
          const isScanRoot = entryIndex === 0 && entry.kind === "crumb";
          return (
            <Fragment key={entryKey}>
              <BreadcrumbSeparator />
              <BreadcrumbItem className={breadcrumbItemClassName(isScanRoot, entry)}>
                {isScanRoot && (
                  <ScanRootChip
                    label={scanRoot.label}
                    isVolume={scanRoot.isVolume}
                    isCurrent={entry.isCurrent}
                    onSelect={() => onOpenFolder(entry.crumb.node)}
                  />
                )}
                {entry.kind === "collapsed" && (
                  <DropdownMenu>
                    <DropdownMenuTrigger
                      className="flex items-center rounded outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      aria-label="Show hidden folders of the path"
                    >
                      <BreadcrumbEllipsis className="size-6" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start">
                      {entry.hiddenCrumbs.map((hiddenCrumb) => (
                        <DropdownMenuItem
                          key={hiddenCrumb.node}
                          onSelect={() => onOpenFolder(hiddenCrumb.node)}
                        >
                          {hiddenCrumb.name}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                )}
                {!isScanRoot && entry.kind === "crumb" && entry.isCurrent && (
                  <BreadcrumbPage className="truncate font-medium">
                    {entry.crumb.name}
                  </BreadcrumbPage>
                )}
                {!isScanRoot && entry.kind === "crumb" && !entry.isCurrent && (
                  <BreadcrumbLink asChild>
                    <button
                      type="button"
                      className="max-w-40 truncate"
                      onClick={() => onOpenFolder(entry.crumb.node)}
                    >
                      {entry.crumb.name}
                    </button>
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
            </Fragment>
          );
        })}
      </BreadcrumbList>
    </Breadcrumb>
  );
}
