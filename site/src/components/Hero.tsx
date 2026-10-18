import "./Hero.css";
import { motion, useScroll, useTransform } from "motion/react";
import { useRef } from "react";
import type { DownloadTarget } from "../hooks/useDownloadTarget";
import type { StartedDownload } from "../lib/release/startedDownload";
import { DownloadBlock } from "./DownloadBlock";
import { OrbitCanvas } from "./OrbitCanvas";

type HeroProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: (download: StartedDownload) => void;
};

const EASE_OUT = [0.16, 1, 0.3, 1] as const;

function fadeUp(delay: number) {
  return {
    initial: { opacity: 0, y: 16 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 1, delay, ease: EASE_OUT },
  };
}

export function Hero({ downloadTarget, onDownloadStarted }: HeroProps) {
  const heroRef = useRef<HTMLElement>(null);
  const { scrollYProgress } = useScroll({ target: heroRef, offset: ["start start", "end start"] });
  const contentY = useTransform(scrollYProgress, [0, 1], [0, -80]);
  const contentOpacity = useTransform(scrollYProgress, [0, 1], [1, 0.2]);

  return (
    <section className="hero" ref={heroRef}>
      <OrbitCanvas className="hero-orbit" followPointer />
      <motion.div className="wrap hero-inner" style={{ y: contentY, opacity: contentOpacity }}>
        <motion.span className="badge" {...fadeUp(0)}>
          <span className="badge-dot" />v{downloadTarget.release.version} is out
        </motion.span>
        <h1>
          <span className="line">
            <motion.span
              initial={{ y: "105%" }}
              animate={{ y: 0 }}
              transition={{ duration: 1.1, ease: EASE_OUT }}
            >
              Your disk,
            </motion.span>
          </span>
          <span className="line">
            <motion.span
              initial={{ y: "105%" }}
              animate={{ y: 0 }}
              transition={{ duration: 1.1, delay: 0.09, ease: EASE_OUT }}
            >
              <em>mapped</em> in seconds.
            </motion.span>
          </span>
        </h1>
        <motion.p className="lede" {...fadeUp(0.25)}>
          A fast disk space explorer. Scan, search and find duplicates.
        </motion.p>
        <motion.div {...fadeUp(0.4)}>
          <DownloadBlock downloadTarget={downloadTarget} onDownloadStarted={onDownloadStarted} />
        </motion.div>
      </motion.div>
    </section>
  );
}
