import "./FeatureGrid.css";
import { DuplicatesCard } from "./DuplicatesCard";
import { SafetyCard } from "./SafetyCard";
import { ScanEnginesCard } from "./ScanEnginesCard";
import { SearchCard } from "./SearchCard";
import { ShortcutsCard } from "./ShortcutsCard";
import { ThemesCard } from "./ThemesCard";

export function FeatureGrid() {
  return (
    <section className="section" id="features">
      <div className="wrap">
        <p className="eyebrow">Under the hood</p>
        <h2 className="section-title">Built for massive amounts of data.</h2>
        <div className="bento">
          <ScanEnginesCard order={0} />
          <ShortcutsCard order={1} />
          <SearchCard order={2} />
          <DuplicatesCard order={3} />
          <SafetyCard order={4} />
          <ThemesCard order={5} />
        </div>
      </div>
    </section>
  );
}
