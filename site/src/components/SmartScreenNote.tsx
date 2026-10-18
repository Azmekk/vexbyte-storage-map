import { AnimatePresence, motion } from "motion/react";

export function SmartScreenNote({ isVisible }: { isVisible: boolean }) {
  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          className="smartscreen"
          role="status"
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
        >
          <strong>If Windows shows "Windows protected your PC"</strong>
          <ol>
            <li>
              Click <strong>More info</strong>.
            </li>
            <li>
              Click <strong>Run anyway</strong>. The installer isn't code-signed yet.
            </li>
          </ol>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
