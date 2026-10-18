import type { ChangelogEntry } from "../lib/changelog/parseChangelog";
import { RELEASES_URL } from "../lib/release/repository";
import { InlineCodeText } from "./InlineCodeText";

export function ChangelogEntrySection({ entry }: { entry: ChangelogEntry }) {
  return (
    <section className="changelog-entry" id={entry.version} aria-labelledby={`v${entry.version}`}>
      <header className="changelog-entry-header">
        <h2 id={`v${entry.version}`}>
          <a className="changelog-version-link" href={`#${entry.version}`}>
            {entry.version}
          </a>
        </h2>
        {entry.date !== null && <time dateTime={entry.date}>{entry.date}</time>}
        <a className="changelog-release-link" href={`${RELEASES_URL}/tag/v${entry.version}`}>
          Release on GitHub
        </a>
      </header>
      {entry.whatsNew.length > 0 && (
        <>
          <h3>What's new</h3>
          <ul className="changelog-whats-new">
            {entry.whatsNew.map((bullet) => (
              <li key={bullet}>
                <InlineCodeText text={bullet} />
              </li>
            ))}
          </ul>
        </>
      )}
      {entry.developerInformation.length > 0 && (
        <details className="changelog-developer">
          <summary>Developer information</summary>
          <ul>
            {entry.developerInformation.map((bullet) => (
              <li key={bullet}>
                <InlineCodeText text={bullet} />
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
