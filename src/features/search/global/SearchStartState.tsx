import { motion } from "motion/react";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import type { SearchIndexStatus } from "@/lib/backend/search";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { SEARCH_STARTER_EXAMPLES } from "./query/searchSyntaxHelp";
import { SearchIndexStatusCard } from "./status/SearchIndexStatusCard";

type SearchStartStateProps = {
  status: SearchIndexStatus | null;
  loadErrorMessage: string | null;
  onUseExample: (example: string) => void;
};

/** What the screen shows before anything is typed: a few queries to try and the index status. */
export function SearchStartState({
  status,
  loadErrorMessage,
  onUseExample,
}: SearchStartStateProps) {
  return (
    <motion.div
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      className="grid content-start gap-6"
    >
      <section className="grid gap-3">
        <div className="grid gap-1">
          <h2 className="font-medium text-base">Find any file on your drives by name</h2>
          <p className="text-muted-foreground text-sm">
            Results appear as you type. Combine words, wildcards and filters, or try one of these.
          </p>
        </div>
        <ul className="grid gap-2 sm:grid-cols-2">
          {SEARCH_STARTER_EXAMPLES.map((example) => (
            <li key={example.example}>
              <button
                type="button"
                onClick={() => onUseExample(example.example)}
                className="flex w-full items-center justify-between gap-3 rounded-lg border bg-card px-3 py-2.5 text-left outline-none transition-colors hover:border-(--volume-used)/40 hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
              >
                <span className="truncate font-mono text-sm">{example.example}</span>
                <span className="shrink-0 text-muted-foreground text-xs">
                  {example.description}
                </span>
              </button>
            </li>
          ))}
        </ul>
        <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-muted-foreground text-xs">
          <span className="flex items-center gap-1.5">
            <KbdGroup>
              <Kbd>↑</Kbd>
              <Kbd>↓</Kbd>
            </KbdGroup>
            select
          </span>
          <span className="flex items-center gap-1.5">
            <Kbd>Enter</Kbd>
            open
          </span>
          <span className="flex items-center gap-1.5">
            <KbdGroup>
              <Kbd>Ctrl</Kbd>
              <Kbd>C</Kbd>
            </KbdGroup>
            copy path
          </span>
          <span className="flex items-center gap-1.5">
            <Kbd>Esc</Kbd>
            clear
          </span>
        </p>
      </section>
      <SearchIndexStatusCard status={status} loadErrorMessage={loadErrorMessage} />
    </motion.div>
  );
}
