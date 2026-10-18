import { SpotlightCard } from "./SpotlightCard";

export function ShortcutsCard({ order }: { order: number }) {
  return (
    <SpotlightCard size="small" order={order}>
      <span className="eyebrow">Keyboard</span>
      <h3>Shortcuts for everything</h3>
      <p>Ctrl+K to jump anywhere, F3 to search every drive, and ? for the full list.</p>
      <div className="keys">
        <kbd>Ctrl</kbd>
        <kbd>K</kbd>
        <kbd>F3</kbd>
        <kbd>Ctrl</kbd>
        <kbd>Shift</kbd>
        <kbd>F</kbd>
        <kbd>?</kbd>
      </div>
    </SpotlightCard>
  );
}
