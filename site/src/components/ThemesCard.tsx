import { SpotlightCard } from "./SpotlightCard";

export function ThemesCard({ order }: { order: number }) {
  return (
    <SpotlightCard size="small" order={order}>
      <span className="eyebrow">Looks right</span>
      <h3>Dark, light, and calm</h3>
      <p>Both themes, reduced-motion support, and canvas charts with animated zoom.</p>
    </SpotlightCard>
  );
}
