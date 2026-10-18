export type OrbitRing = {
  /** Colour and share of the ring, like one level of a disk's sunburst. */
  segments: ReadonlyArray<readonly [string, number]>;
  /** Turns per minute-ish; the sign sets the direction. */
  speed: number;
};

export const EMPTY_SEGMENT_COLOR = "#2a3040";

export const ORBIT_RINGS: readonly OrbitRing[] = [
  {
    segments: [
      ["#10b981", 0.34],
      ["#3b82f6", 0.22],
      ["#f97316", 0.14],
      ["#8b5cf6", 0.12],
      ["#eab308", 0.08],
      [EMPTY_SEGMENT_COLOR, 0.1],
    ],
    speed: 0.012,
  },
  {
    segments: [
      ["#10b981", 0.12],
      ["#0e8f66", 0.14],
      ["#3b82f6", 0.1],
      ["#2f6ad0", 0.12],
      ["#f97316", 0.08],
      ["#ec4899", 0.07],
      ["#8b5cf6", 0.12],
      ["#eab308", 0.07],
      [EMPTY_SEGMENT_COLOR, 0.18],
    ],
    speed: -0.018,
  },
  {
    segments: [
      ["#10b981", 0.06],
      [EMPTY_SEGMENT_COLOR, 0.05],
      ["#0e8f66", 0.07],
      ["#3b82f6", 0.05],
      [EMPTY_SEGMENT_COLOR, 0.06],
      ["#2f6ad0", 0.08],
      ["#f97316", 0.04],
      ["#ef4444", 0.03],
      ["#ec4899", 0.05],
      [EMPTY_SEGMENT_COLOR, 0.07],
      ["#8b5cf6", 0.07],
      ["#6d4fd6", 0.05],
      ["#eab308", 0.05],
      [EMPTY_SEGMENT_COLOR, 0.27],
    ],
    speed: 0.026,
  },
  {
    segments: [
      ["#10b981", 0.03],
      [EMPTY_SEGMENT_COLOR, 0.06],
      ["#3b82f6", 0.04],
      [EMPTY_SEGMENT_COLOR, 0.09],
      ["#f97316", 0.02],
      [EMPTY_SEGMENT_COLOR, 0.08],
      ["#8b5cf6", 0.04],
      [EMPTY_SEGMENT_COLOR, 0.1],
      ["#eab308", 0.02],
      [EMPTY_SEGMENT_COLOR, 0.52],
    ],
    speed: -0.034,
  },
];
