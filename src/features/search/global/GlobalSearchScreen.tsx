import { FileSearchIcon, TriangleAlertIcon } from "lucide-react";
import { AnimatePresence } from "motion/react";
import { TopBar } from "@/app/TopBar";
import { TopBarTitle } from "@/app/TopBarTitle";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import { GlobalSearchInput } from "./query/GlobalSearchInput";
import { SearchSortSelect } from "./query/SearchSortSelect";
import { SearchSyntaxHelpPopover } from "./query/SearchSyntaxHelpPopover";
import { SearchResultCountLine } from "./results/SearchResultCountLine";
import { SearchResultsList } from "./results/SearchResultsList";
import { SearchStartState } from "./SearchStartState";
import { WarmingBanner } from "./status/WarmingBanner";
import { useGlobalSearchScreen } from "./useGlobalSearchScreen";

/** Everything-style search by name or path across every indexed drive. */
export function GlobalSearchScreen() {
  const screen = useGlobalSearchScreen();
  const { search, indexStatus } = screen;
  const response = search.response;
  const hasHits = screen.hits.length > 0;
  const isEmptyAnswer =
    response !== null && response.hits.length === 0 && !search.isSearching && !screen.isQueryEmpty;

  return (
    <>
      <TopBar>
        <TopBarTitle title="Search files" />
      </TopBar>
      <search
        onKeyDown={screen.handleKeyDown}
        className="mx-auto flex min-h-0 w-full max-w-6xl flex-1 flex-col gap-3 px-6 pt-6 pb-4"
      >
        <GlobalSearchInput
          query={screen.query}
          queryErrorMessage={search.queryErrorMessage}
          isSearching={search.isSearching}
          onQueryChange={screen.setQuery}
          onClear={screen.clearQuery}
        />

        <div className="flex min-h-8 flex-wrap items-center justify-between gap-x-4 gap-y-1">
          {screen.isQueryEmpty ? (
            <span />
          ) : (
            <SearchResultCountLine response={response} isSearching={search.isSearching} />
          )}
          <div className="flex items-center gap-2">
            {!screen.isQueryEmpty && (
              <SearchSortSelect
                sortOrder={screen.sortOrder}
                onChooseSortOrder={screen.setSortOrder}
              />
            )}
            <SearchSyntaxHelpPopover onUseExample={screen.setQuery} />
          </div>
        </div>

        <AnimatePresence initial={false}>
          {screen.showWarmingBanner && (
            <WarmingBanner
              key="warming-banner"
              warmingVolumes={screen.warmingVolumes}
              skippedVolumesStillWarming={screen.skippedVolumesStillWarming}
            />
          )}
        </AnimatePresence>

        {screen.isQueryEmpty && (
          <div className="min-h-0 flex-1 overflow-y-auto">
            <SearchStartState
              status={indexStatus.status}
              loadErrorMessage={indexStatus.loadErrorMessage}
              onUseExample={screen.setQuery}
            />
          </div>
        )}

        {!screen.isQueryEmpty && search.failureMessage !== null && (
          <EmptyState
            tone="warning"
            icon={TriangleAlertIcon}
            title="The search did not finish"
            description={search.failureMessage}
            actions={
              <Button variant="outline" onClick={search.retry}>
                Search again
              </Button>
            }
          />
        )}

        {isEmptyAnswer && search.failureMessage === null && search.queryErrorMessage === null && (
          <EmptyState
            icon={FileSearchIcon}
            title="Nothing matches"
            description={
              screen.skippedVolumesStillWarming.length > 0
                ? "Nothing on the drives that are ready. The others are still warming up and will be searched as soon as they are."
                : "Check the spelling, use fewer words, or add * to match part of a name, for example report*."
            }
          />
        )}

        {!screen.isQueryEmpty && hasHits && (
          <SearchResultsList
            hits={screen.hits}
            selectedIndex={screen.selectedIndex}
            sortOrder={screen.sortOrder}
            isStale={search.queryErrorMessage !== null}
            scrollElementRef={screen.scrollElementRef}
            virtualizer={screen.virtualizer}
            onChooseSortOrder={screen.setSortOrder}
            onSelect={screen.selectIndex}
            onOpen={screen.openHitAtIndex}
          />
        )}
      </search>
    </>
  );
}
