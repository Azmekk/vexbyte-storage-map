import type { TargetAndTransition, Transition, Variants } from "motion/react";

/** Entering elements: short and ease-out. */
export const enterTransition: Transition = { duration: 0.24, ease: "easeOut" };

/** Leaving elements: shorter and ease-in. */
export const exitTransition: Transition = { duration: 0.16, ease: "easeIn" };

/** Hover and press feedback. */
export const feedbackTransition: Transition = { duration: 0.15, ease: "easeOut" };

/** A subtle fade with a small upward slide, for cards and panels appearing. */
export const fadeSlideInVariants: Variants = {
  hidden: { opacity: 0, y: 8 },
  visible: { opacity: 1, y: 0, transition: enterTransition },
  exit: { opacity: 0, y: -4, transition: exitTransition },
};

/** Staggers children that use `fadeSlideInVariants`. Only meant for short lists. */
export const staggeredChildrenVariants: Variants = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.05 } },
};

export const cardHoverLift: TargetAndTransition = { y: -2, transition: feedbackTransition };
