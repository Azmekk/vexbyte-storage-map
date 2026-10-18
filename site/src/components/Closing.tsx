import "./Closing.css";
import { motion } from "motion/react";
import type { DownloadTarget } from "../hooks/useDownloadTarget";
import { DownloadBlock } from "./DownloadBlock";
import { OrbitCanvas } from "./OrbitCanvas";
import { SmartScreenNote } from "./SmartScreenNote";

type ClosingProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: () => void;
  showSmartScreenNote: boolean;
};

export function Closing({ downloadTarget, onDownloadStarted, showSmartScreenNote }: ClosingProps) {
  return (
    <section className="closing" id="get">
      <OrbitCanvas className="closing-orbit" scale={0.95} alpha={0.8} />
      <div className="wrap closing-inner">
        <p className="eyebrow">Free and open source · MIT</p>
        <motion.h2
          initial={{ opacity: 0.2, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 1, ease: [0.16, 1, 0.3, 1] }}
        >
          Find out where it all went.
        </motion.h2>
        <p className="lede">
          Windows 10 and 11 today, macOS and Linux coming soon. Installs for your user only, no
          admin needed.
        </p>
        <DownloadBlock downloadTarget={downloadTarget} onDownloadStarted={onDownloadStarted} />
        <SmartScreenNote isVisible={showSmartScreenNote} />
      </div>
    </section>
  );
}
