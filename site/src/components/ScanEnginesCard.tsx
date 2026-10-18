import type { CSSProperties } from "react";
import { SpotlightCard } from "./SpotlightCard";

export function ScanEnginesCard({ order }: { order: number }) {
  return (
    <SpotlightCard size="wide" order={order}>
      <span className="eyebrow">Two ways to scan</span>
      <h3 className="card-headline">Fast on any drive. Even faster as admin.</h3>
      <p>
        Standard scan works on any drive or folder, with no admin rights. Run the app as
        administrator and Fast scan reads your Windows drive's own file index directly, which is
        many times quicker on big drives.
      </p>
      <div className="engines">
        <div className="engine" style={{ "--engine-color": "var(--video)" } as CSSProperties}>
          <b>Standard scan</b>
          <span>every drive · no admin</span>
        </div>
        <div className="engine" style={{ "--engine-color": "var(--archive)" } as CSSProperties}>
          <b>Fast scan</b>
          <span>Windows drives · run as admin</span>
        </div>
      </div>
    </SpotlightCard>
  );
}
