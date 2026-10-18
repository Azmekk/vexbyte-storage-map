import { CheckIcon } from "./icons/CheckIcon";
import { SpotlightCard } from "./SpotlightCard";

const PROMISES = [
  "Scans open drives read-only. Nothing is written while you look around.",
  "Removals go to the Recycle Bin, after a confirmation that lists exactly what moves.",
  "OneDrive placeholders are never downloaded, and online-only files count only the bytes on disk.",
] as const;

export function SafetyCard({ order }: { order: number }) {
  return (
    <SpotlightCard size="wide" order={order}>
      <span className="eyebrow">Safe by design</span>
      <h3>Looks at everything. Touches nothing.</h3>
      <ul className="safety-list">
        {PROMISES.map((promise) => (
          <li key={promise}>
            <CheckIcon />
            {promise}
          </li>
        ))}
      </ul>
    </SpotlightCard>
  );
}
