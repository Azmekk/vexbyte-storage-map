import "./Closing.css";
import { motion } from "motion/react";
import type { DownloadTarget } from "../hooks/useDownloadTarget";
import type { StartedDownload } from "../lib/release/startedDownload";
import { DownloadBlock } from "./DownloadBlock";
import { LinuxInstallNote } from "./LinuxInstallNote";
import { OrbitCanvas } from "./OrbitCanvas";
import { SmartScreenNote } from "./SmartScreenNote";

type ClosingProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: (download: StartedDownload) => void;
  startedDownload: StartedDownload | null;
};

export function Closing({ downloadTarget, onDownloadStarted, startedDownload }: ClosingProps) {
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
          Windows and Linux, with macOS maybe later. No admin needed to use it, and on Windows it
          installs for your user only.
        </p>
        <DownloadBlock downloadTarget={downloadTarget} onDownloadStarted={onDownloadStarted} />
        <SmartScreenNote isVisible={startedDownload?.platform === "windows"} />
        <LinuxInstallNote
          linuxPackage={startedDownload?.platform === "linux" ? startedDownload.linuxPackage : null}
        />
      </div>
    </section>
  );
}
