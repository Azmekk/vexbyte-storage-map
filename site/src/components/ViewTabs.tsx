import "./ViewTabs.css";
import { AnimatePresence, motion } from "motion/react";
import { type KeyboardEvent, useRef } from "react";
import { AUTO_ADVANCE_MILLISECONDS, useAutoAdvance } from "../hooks/useAutoAdvance";
import { APP_VIEWS } from "../lib/views";

const EASE_OUT = [0.16, 1, 0.3, 1] as const;

export function ViewTabs() {
  const stageRef = useRef<HTMLDivElement>(null);
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const { activeIndex, select, isCycling, cycleKey } = useAutoAdvance(stageRef, APP_VIEWS.length);
  const activeView = APP_VIEWS[activeIndex] ?? APP_VIEWS[0];

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
    const step = event.key === "ArrowRight" ? 1 : APP_VIEWS.length - 1;
    const nextIndex = (activeIndex + step) % APP_VIEWS.length;
    select(nextIndex);
    tabRefs.current[nextIndex]?.focus();
  };

  return (
    <section className="section" id="views">
      <div className="wrap">
        <div className="views-head">
          <div>
            <p className="eyebrow">Pick a view</p>
            <h2 className="section-title">Every folder, sized and coloured by what fills it.</h2>
          </div>
          <div className="tabs" role="tablist" aria-label="Views" onKeyDown={onKeyDown}>
            {APP_VIEWS.map((view, index) => (
              <button
                key={view.id}
                ref={(element) => {
                  tabRefs.current[index] = element;
                }}
                className="tab"
                id={`tab-${view.id}`}
                type="button"
                role="tab"
                aria-selected={index === activeIndex}
                aria-controls="view-stage"
                tabIndex={index === activeIndex ? 0 : -1}
                onClick={() => select(index)}
              >
                {index === activeIndex && (
                  <motion.span
                    className="tab-pill"
                    layoutId="view-tab-pill"
                    transition={{ type: "spring", stiffness: 380, damping: 34 }}
                  />
                )}
                <span className="tab-swatch" style={{ background: view.swatch }} />
                <span className="tab-label">{view.name}</span>
              </button>
            ))}
          </div>
        </div>

        <div
          className="stage"
          id="view-stage"
          role="tabpanel"
          aria-labelledby={`tab-${activeView?.id}`}
          ref={stageRef}
        >
          <div className="stage-frame">
            {APP_VIEWS.map((view, index) => (
              <img
                key={view.id}
                className={index === activeIndex ? "is-active" : undefined}
                src={view.screenshot}
                alt={index === activeIndex ? view.alt : ""}
                aria-hidden={index !== activeIndex}
                loading={index === 0 ? "eager" : "lazy"}
                width="1400"
                height="900"
              />
            ))}
          </div>
          <div className="stage-caption">
            <AnimatePresence mode="wait" initial={false}>
              <motion.p
                key={activeView?.id}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -6 }}
                transition={{ duration: 0.35, ease: EASE_OUT }}
              >
                {activeView?.description}
              </motion.p>
            </AnimatePresence>
            <div className="stage-progress" aria-hidden="true">
              {isCycling && (
                <motion.span
                  key={cycleKey}
                  initial={{ scaleX: 0 }}
                  animate={{ scaleX: 1 }}
                  transition={{ duration: AUTO_ADVANCE_MILLISECONDS / 1000, ease: "linear" }}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
