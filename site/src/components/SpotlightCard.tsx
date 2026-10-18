import { motion } from "motion/react";
import type { ReactNode } from "react";
import { usePointerPosition } from "../hooks/usePointerPosition";

export type CardSize = "small" | "half" | "wide";

type SpotlightCardProps = {
  size: CardSize;
  /** Position in the grid, used to stagger the reveal. */
  order: number;
  children: ReactNode;
};

export function SpotlightCard({ size, order, children }: SpotlightCardProps) {
  const onPointerMove = usePointerPosition();
  return (
    <motion.article
      className={`card card-${size}`}
      onPointerMove={onPointerMove}
      initial={{ opacity: 0, y: 28 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.9, delay: (order % 3) * 0.08, ease: [0.16, 1, 0.3, 1] }}
    >
      {children}
    </motion.article>
  );
}
