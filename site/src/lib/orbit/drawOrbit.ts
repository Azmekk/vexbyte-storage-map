import { EMPTY_SEGMENT_COLOR, ORBIT_RINGS } from "./rings";

export type OrbitFrame = {
  width: number;
  height: number;
  /** Seconds since the orbit started; drives the grow-in and the rotation. */
  elapsedSeconds: number;
  /** Pointer position inside the canvas, 0 to 1 on each axis. */
  pointerX: number;
  pointerY: number;
  scale: number;
  alpha: number;
};

const SEGMENT_GAP_RADIANS = 0.012;
const INK = "5, 6, 10";

function easeOutExpo(value: number): number {
  return value >= 1 ? 1 : 1 - 2 ** (-10 * value);
}

export function drawOrbit(context: CanvasRenderingContext2D, frame: OrbitFrame): void {
  const { width, height, elapsedSeconds, scale, alpha } = frame;
  context.clearRect(0, 0, width, height);
  const centerX = width / 2 + (frame.pointerX - 0.5) * 40;
  const centerY = height / 2 + (frame.pointerY - 0.5) * 30;
  const outer = Math.min(Math.max(width, height) * 0.46, 760) * scale;
  const inner = outer * 0.52;
  const thickness = (outer - inner) / ORBIT_RINGS.length;

  const glow = context.createRadialGradient(centerX, centerY, 0, centerX, centerY, outer * 1.1);
  glow.addColorStop(0, `rgba(109, 139, 255, ${0.16 * alpha})`);
  glow.addColorStop(0.5, `rgba(80, 60, 200, ${0.06 * alpha})`);
  glow.addColorStop(1, `rgba(${INK}, 0)`);
  context.fillStyle = glow;
  context.fillRect(0, 0, width, height);

  context.lineWidth = thickness * 0.42;
  ORBIT_RINGS.forEach((ring, ringIndex) => {
    const radius = inner + thickness * (ringIndex + 0.5);
    const grow = easeOutExpo(Math.max(0, Math.min(1, (elapsedSeconds - ringIndex * 0.14) / 1.6)));
    let angle = -Math.PI / 2 + elapsedSeconds * ring.speed * Math.PI * 0.5 + ringIndex * 0.7;
    for (const [color, share] of ring.segments) {
      const sweep = share * Math.PI * 2 * grow;
      if (sweep > SEGMENT_GAP_RADIANS * 2) {
        context.beginPath();
        context.arc(
          centerX,
          centerY,
          radius,
          angle + SEGMENT_GAP_RADIANS,
          angle + sweep - SEGMENT_GAP_RADIANS,
        );
        context.strokeStyle = color;
        const baseAlpha = color === EMPTY_SEGMENT_COLOR ? 0.35 : 0.3;
        context.globalAlpha = baseAlpha * alpha * (1 - ringIndex * 0.16);
        context.stroke();
      }
      angle += sweep;
    }
  });
  context.globalAlpha = 1;

  // A veil darkens the middle for the headline and fades the outer rings into the page.
  const veil = context.createRadialGradient(
    centerX,
    centerY,
    inner * 0.3,
    centerX,
    centerY,
    outer * 1.05,
  );
  veil.addColorStop(0, `rgba(${INK}, 0.55)`);
  veil.addColorStop(0.55, `rgba(${INK}, 0.1)`);
  veil.addColorStop(1, `rgba(${INK}, 0.75)`);
  context.fillStyle = veil;
  context.fillRect(0, 0, width, height);

  context.beginPath();
  context.arc(centerX, centerY, inner - 8, 0, Math.PI * 2);
  context.strokeStyle = `rgba(196, 210, 255, ${0.08 * alpha})`;
  context.lineWidth = 1;
  context.stroke();
}
