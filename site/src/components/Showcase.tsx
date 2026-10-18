import "./Showcase.css";
import { motion, useReducedMotion, useScroll, useTransform } from "motion/react";
import { useRef } from "react";
import type { Release } from "../lib/release/latestRelease";
import { CountUp } from "./CountUp";

const BYTES_PER_MEGABYTE = 1_048_576;

export function Showcase({ release }: { release: Release }) {
  const sectionRef = useRef<HTMLElement>(null);
  const prefersReducedMotion = useReducedMotion();
  // Scroll-linked but never pinned: the window settles flat as it rises into view.
  const { scrollYProgress } = useScroll({
    target: sectionRef,
    offset: ["start end", "start 0.25"],
  });
  const rotateX = useTransform(scrollYProgress, [0, 1], [28, 0]);
  const scale = useTransform(scrollYProgress, [0, 1], [0.86, 1]);
  const y = useTransform(scrollYProgress, [0, 1], [-30, 0]);
  const opacity = useTransform(scrollYProgress, [0, 1], [0.55, 1]);

  return (
    <section className="showcase" ref={sectionRef}>
      <div className="wrap">
        <motion.figure
          className="window"
          style={prefersReducedMotion ? undefined : { rotateX, scale, y, opacity }}
        >
          <div className="window-bar">
            <span />
            <span />
            <span />
            <span className="window-title">Vexbyte Storage Map</span>
          </div>
          <img
            src="screenshots/treemap-dark.png"
            alt="Vexbyte Storage Map showing a treemap of a folder next to a sortable folder list"
            width="1400"
            height="900"
          />
        </motion.figure>
        <p className="window-caption">Example scan of a Program Files folder.</p>
        <ul className="stats">
          {release.windowsInstaller.sizeInBytes !== null && (
            <li className="stat">
              <b>
                <CountUp
                  value={release.windowsInstaller.sizeInBytes / BYTES_PER_MEGABYTE}
                  decimals={1}
                  suffix=" MB"
                />
              </b>
              <small>Windows installer download</small>
            </li>
          )}
          <li className="stat">
            <b>
              <CountUp value={5} />
            </b>
            <small>views of every scan</small>
          </li>
          <li className="stat">
            <b>
              <CountUp value={0} suffix=" bytes" />
            </b>
            <small>written to your drives while scanning</small>
          </li>
          <li className="stat">
            <b>No admin</b>
            <small>needed to install</small>
          </li>
        </ul>
      </div>
    </section>
  );
}
