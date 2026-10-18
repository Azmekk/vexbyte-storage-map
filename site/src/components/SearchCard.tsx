import { AnimatePresence, motion } from "motion/react";
import { useRef } from "react";
import { useSearchDemo } from "../hooks/useSearchDemo";
import { SearchIcon } from "./icons/SearchIcon";
import { SpotlightCard } from "./SpotlightCard";

export function SearchCard({ order }: { order: number }) {
  const demoRef = useRef<HTMLDivElement>(null);
  const { typedQuery, demo } = useSearchDemo(demoRef);

  return (
    <SpotlightCard size="half" order={order}>
      <span className="eyebrow">Search every drive</span>
      <h3>Find any file in milliseconds</h3>
      <p>
        An in-memory index of every local drive, with wildcards and filters in the spirit of
        Everything.
      </p>
      <div className="search-box" ref={demoRef}>
        <SearchIcon />
        <span>{typedQuery}</span>
        <span className="search-caret" />
      </div>
      <div className="search-results">
        <AnimatePresence mode="popLayout" initial={false}>
          {demo.results.map(([path, size], index) => (
            <motion.div
              key={`${demo.query}-${path}`}
              className="search-result"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.4, delay: index * 0.07 }}
            >
              <span>{path}</span>
              <span>{size}</span>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </SpotlightCard>
  );
}
