import { LoaderCircleIcon, SearchIcon, XIcon } from "lucide-react";
import { useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/classNames";
import { useGlobalSearchStore } from "../useGlobalSearchStore";

export const SEARCH_INPUT_PLACEHOLDER = "report*.pdf  ext:jpg;png  size:>100mb";
const QUERY_ERROR_ID = "global-search-query-error";

type GlobalSearchInputProps = {
  query: string;
  queryErrorMessage: string | null;
  isSearching: boolean;
  onQueryChange: (query: string) => void;
  onClear: () => void;
};

export function GlobalSearchInput({
  query,
  queryErrorMessage,
  isSearching,
  onQueryChange,
  onClear,
}: GlobalSearchInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const hasError = queryErrorMessage !== null;

  useEffect(() => {
    const focusAndSelect = () => {
      inputRef.current?.focus();
      inputRef.current?.select();
    };
    focusAndSelect();
    return useGlobalSearchStore.subscribe((state, previousState) => {
      if (state.inputFocusRequest !== previousState.inputFocusRequest) {
        focusAndSelect();
      }
    });
  }, []);

  return (
    <div className="grid gap-1.5">
      <div
        className={cn(
          "flex h-14 items-center gap-3 rounded-xl border bg-card px-4 shadow-xs transition-[border-color,box-shadow]",
          "focus-within:border-(--volume-used) focus-within:ring-4 focus-within:ring-(--volume-used)/15",
          hasError &&
            "border-destructive focus-within:border-destructive focus-within:ring-destructive/15",
        )}
      >
        {isSearching ? (
          <LoaderCircleIcon
            aria-hidden
            className="size-5 shrink-0 animate-spin text-muted-foreground"
          />
        ) : (
          <SearchIcon aria-hidden className="size-5 shrink-0 text-muted-foreground" />
        )}
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder={SEARCH_INPUT_PLACEHOLDER}
          aria-label="Search files and folders on every drive"
          aria-invalid={hasError}
          aria-describedby={hasError ? QUERY_ERROR_ID : undefined}
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          className="h-full min-w-0 flex-1 bg-transparent text-lg outline-none placeholder:whitespace-pre placeholder:text-muted-foreground/70"
        />
        {query.length > 0 && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Clear the search (Esc)"
            onClick={() => {
              onClear();
              inputRef.current?.focus();
            }}
          >
            <XIcon />
          </Button>
        )}
      </div>
      {hasError && (
        <p id={QUERY_ERROR_ID} role="alert" className="px-1 text-destructive text-sm">
          {queryErrorMessage}
        </p>
      )}
    </div>
  );
}
