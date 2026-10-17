import { LoaderCircleIcon } from "lucide-react";
import { useState } from "react";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { FileTypeIcon } from "@/features/explorer/FileTypeIcon";
import { revealNodeInExplorer } from "@/features/explorer/revealNodeInExplorer";
import type { ScanId } from "@/lib/backend/scans";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { SEARCH_RESULT_LIMIT, useNodeSearch } from "./useNodeSearch";
import { useSearchPaletteStore } from "./useSearchPaletteStore";

type SearchPaletteProps = {
  scanId: ScanId;
};

export function SearchPalette({ scanId }: SearchPaletteProps) {
  const isOpen = useSearchPaletteStore((state) => state.isOpen);
  const setOpen = useSearchPaletteStore((state) => state.setOpen);
  const [query, setQuery] = useState("");
  const searchState = useNodeSearch(scanId, isOpen ? query : "");

  const results =
    searchState.status === "done"
      ? searchState.results
      : searchState.status === "searching"
        ? searchState.previousResults
        : [];

  return (
    <CommandDialog
      open={isOpen}
      onOpenChange={setOpen}
      title="Search this scan"
      description="Find files and folders by name"
      className="sm:max-w-2xl"
    >
      <Command shouldFilter={false} loop>
        <CommandInput
          placeholder="Search file and folder names…"
          value={query}
          onValueChange={setQuery}
        />
        <CommandList className="max-h-[min(28rem,60vh)]">
          {searchState.status === "idle" && (
            <p className="px-3 py-6 text-center text-muted-foreground text-sm">
              Type part of a name. Results are sorted by size, largest first.
            </p>
          )}
          {searchState.status === "searching" && results.length === 0 && (
            <p className="flex items-center justify-center gap-2 px-3 py-6 text-muted-foreground text-sm">
              <LoaderCircleIcon className="size-4 animate-spin motion-reduce:animate-none" />
              Searching…
            </p>
          )}
          {searchState.status === "error" && (
            <p className="px-3 py-6 text-center text-destructive text-sm">
              {searchState.errorMessage}
            </p>
          )}
          {searchState.status === "done" && (
            <CommandEmpty>
              Nothing named like “{searchState.query}”. Try a shorter part of the name.
            </CommandEmpty>
          )}
          {results.length > 0 && (
            <CommandGroup
              heading={
                results.length >= SEARCH_RESULT_LIMIT
                  ? `Largest ${SEARCH_RESULT_LIMIT} matches`
                  : `${results.length} matches`
              }
            >
              {results.map((result) => (
                <CommandItem
                  key={result.node}
                  value={String(result.node)}
                  onSelect={() => {
                    setOpen(false);
                    void revealNodeInExplorer(scanId, result.node);
                  }}
                >
                  <FileTypeIcon kind={result.kind} category={null} />
                  <div className="grid min-w-0 flex-1">
                    <span className="truncate">{result.name}</span>
                    <span className="truncate font-mono text-muted-foreground text-xs">
                      {result.fullPath ?? ""}
                    </span>
                  </div>
                  <span className="shrink-0 text-muted-foreground text-xs tabular-nums">
                    {formatBytesAsReadableSize(result.logicalSize)}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          )}
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
