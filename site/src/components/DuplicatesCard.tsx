import { Fragment, useRef } from "react";
import { useStepCycle } from "../hooks/useStepCycle";
import { SpotlightCard } from "./SpotlightCard";

const STAGES = ["same size", "partial hash", "BLAKE3 full hash"] as const;

export function DuplicatesCard({ order }: { order: number }) {
  const pipelineRef = useRef<HTMLDivElement>(null);
  const litCount = useStepCycle(pipelineRef, STAGES.length);

  return (
    <SpotlightCard size="half" order={order}>
      <span className="eyebrow">Duplicate finder</span>
      <h3>Exact copies, proven byte for byte</h3>
      <p>
        Pictures, videos, music, documents and downloads. Keep the oldest, newest, shortest path or
        a folder you prefer, and review with thumbnails.
      </p>
      <div className="pipeline" ref={pipelineRef}>
        {STAGES.map((stage, index) => (
          <Fragment key={stage}>
            {index > 0 && <span className="pipe-arrow">→</span>}
            <span className={index < litCount ? "pipe-stage is-lit" : "pipe-stage"}>{stage}</span>
          </Fragment>
        ))}
      </div>
    </SpotlightCard>
  );
}
