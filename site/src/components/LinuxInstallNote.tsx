import { AnimatePresence, motion } from "motion/react";
import type { LinuxPackage } from "../lib/release/latestRelease";

function installCommands({ kind, fileName }: LinuxPackage): string[] {
  switch (kind) {
    case "deb":
      return [`sudo apt install ./${fileName}`];
    case "rpm":
      return [`sudo dnf install ./${fileName}`];
    case "appimage":
      return [`chmod +x ${fileName}`, `./${fileName}`];
  }
}

export function LinuxInstallNote({ linuxPackage }: { linuxPackage: LinuxPackage | null }) {
  return (
    <AnimatePresence>
      {linuxPackage !== null && (
        <motion.div
          className="smartscreen"
          role="status"
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
        >
          <strong>
            {linuxPackage.kind === "appimage"
              ? "Make the AppImage executable, then run it"
              : "Install it from the folder you downloaded it to"}
          </strong>
          <pre className="install-commands">
            <code>{installCommands(linuxPackage).join("\n")}</code>
          </pre>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
