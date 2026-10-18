import { useRef } from "react";
import { useOrbitCanvas } from "../hooks/useOrbitCanvas";

type OrbitCanvasProps = {
  className: string;
  scale?: number;
  alpha?: number;
  followPointer?: boolean;
};

export function OrbitCanvas({
  className,
  scale = 1,
  alpha = 1,
  followPointer = false,
}: OrbitCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  useOrbitCanvas(canvasRef, { scale, alpha, followPointer });
  return (
    <div className={className} aria-hidden="true">
      <canvas ref={canvasRef} className="orbit-canvas" />
    </div>
  );
}
