import "./Changelog.css";
import { useScrollToHash } from "../hooks/useScrollToHash";
import type { ChangelogEntry } from "../lib/changelog/parseChangelog";
import { RELEASES_URL } from "../lib/release/repository";
import { HOME_URL } from "../lib/sitePaths";
import { ChangelogEntrySection } from "./ChangelogEntrySection";
import { Footer } from "./Footer";
import { Nav } from "./Nav";

export function ChangelogPage({ entries }: { entries: ChangelogEntry[] }) {
  useScrollToHash();

  return (
    <>
      <Nav />
      <main className="wrap changelog" id="top">
        <p className="eyebrow">Vexbyte Storage Map</p>
        <h1>Changelog</h1>
        <p className="lede">
          What changed in each version. <a href={`${HOME_URL}#get`}>Download the latest</a> or see
          every build on <a href={RELEASES_URL}>GitHub</a>.
        </p>
        {entries.map((entry) => (
          <ChangelogEntrySection key={entry.version} entry={entry} />
        ))}
      </main>
      <Footer />
    </>
  );
}
