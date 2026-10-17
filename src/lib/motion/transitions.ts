import type { TargetAndTransition, Transition, Variants } from "motion/react";

/** Entering elements: short and ease-out. */
export const enterTransition: Transition = { duration: 0.24, ease: "easeOut" };

/** Leaving elements: shorter and ease-in. */
export const exitTransition: Transition = { duration: 0.16, ease: "easeIn" };

/** Hover and press feedback. */
export const feedbackTransition: Transition = { duration: 0.15, ease: "easeOut" };

/** Layout reordering, for short live lists such as the biggest folders during a scan. */
export const layoutReorderTransition: Transition = {
  layout: { type: "spring", stiffness: 420, damping: 38, mass: 0.8 },
  duration: 0.24,
  ease: "easeOut",
};

/** Zoom transitions of the treemap and sunburst, in milliseconds (drawn on canvas). */
export const VISUALIZATION_ZOOM_DURATION_MILLISECONDS = 300;

/** A subtle fade with a small upward slide, for cards and panels appearing. */
export const fadeSlideInVariants: Variants = {
  hidden: { opacity: 0, y: 8 },
  visible: { opacity: 1, y: 0, transition: enterTransition },
  exit: { opacity: 0, y: -4, transition: exitTransition },
};

/** Switching between whole screens and tabs: a quick fade only. */
export const viewFadeVariants: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.18, ease: "easeOut" } },
  exit: { opacity: 0, transition: { duration: 0.12, ease: "easeIn" } },
};

/** Rows revealed by expanding a folder slide down a little from under their parent. */
export const revealedRowVariants: Variants = {
  hidden: { opacity: 0, y: -6 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.18, ease: "easeOut" } },
};

/** A side panel sliding in from the right edge. */
export const sidePanelVariants: Variants = {
  hidden: { opacity: 0, x: 16 },
  visible: { opacity: 1, x: 0, transition: enterTransition },
  exit: { opacity: 0, x: 16, transition: exitTransition },
};

/** Staggers children that use `fadeSlideInVariants`. Only meant for short lists. */
export const staggeredChildrenVariants: Variants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.05 } },
};

export const cardHoverLift: TargetAndTransition = { y: -2, transition: feedbackTransition };
