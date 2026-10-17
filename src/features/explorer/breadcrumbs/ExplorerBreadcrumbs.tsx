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
import { buildBreadcrumbTrail } from "./buildBreadcrumbTrail";

const MAXIMUM_VISIBLE_CRUMBS = 4;

type ExplorerBreadcrumbsProps = {
  crumbs: BreadcrumbData[];
  onOpenFolder: (folder: NodeIndex) => void;
};

export function ExplorerBreadcrumbs({ crumbs, onOpenFolder }: ExplorerBreadcrumbsProps) {
  const trail = buildBreadcrumbTrail(crumbs, MAXIMUM_VISIBLE_CRUMBS);

  return (
    <Breadcrumb className="min-w-0">
      <BreadcrumbList className="flex-nowrap">
        {trail.map((entry, entryIndex) => {
          const entryKey = entry.kind === "crumb" ? `crumb-${entry.crumb.node}` : "collapsed";
          return (
            <Fragment key={entryKey}>
              {entryIndex > 0 && <BreadcrumbSeparator />}
              <BreadcrumbItem className="min-w-0">
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
                {entry.kind === "crumb" && entry.isCurrent && (
                  <BreadcrumbPage className="truncate font-medium">
                    {entry.crumb.name}
                  </BreadcrumbPage>
                )}
                {entry.kind === "crumb" && !entry.isCurrent && (
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
