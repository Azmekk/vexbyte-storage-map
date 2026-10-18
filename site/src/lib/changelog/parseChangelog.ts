export type ChangelogEntry = {
  version: string;
  /** The date written after the version, as in "## 0.2.0 - 2026-10-20", or null when missing. */
  date: string | null;
  whatsNew: string[];
  developerInformation: string[];
};

type SectionName = "whatsNew" | "developerInformation";

const VERSION_HEADING = /^##\s+(\S+)(?:\s+-\s+(.+?))?\s*$/;
const SECTION_HEADING = /^###\s+(.+?)\s*$/;
const BULLET = /^[-*]\s+(.*)$/;

function sectionNameFor(heading: string): SectionName | null {
  switch (heading.toLowerCase().replace("’", "'")) {
    case "what's new":
      return "whatsNew";
    case "developer information":
      return "developerInformation";
    default:
      return null;
  }
}

/**
 * Reads the hand-written CHANGELOG.md: one "## <version> - <date>" heading per release, each with
 * "### What's new" and "### Developer information" bullet lists. Unknown sections are skipped.
 */
export function parseChangelog(markdown: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  let currentEntry: ChangelogEntry | null = null;
  let currentSection: SectionName | null = null;

  for (const rawLine of markdown.split(/\r?\n/)) {
    const line = rawLine.trimEnd();
    const versionMatch = VERSION_HEADING.exec(line);
    if (versionMatch) {
      currentEntry = {
        version: versionMatch[1] ?? "",
        date: versionMatch[2] ?? null,
        whatsNew: [],
        developerInformation: [],
      };
      currentSection = null;
      entries.push(currentEntry);
      continue;
    }
    const sectionMatch = SECTION_HEADING.exec(line);
    if (sectionMatch) {
      currentSection = sectionNameFor(sectionMatch[1] ?? "");
      continue;
    }
    if (currentEntry === null || currentSection === null) continue;
    const bullets = currentEntry[currentSection];
    const bulletMatch = BULLET.exec(line.trimStart());
    if (bulletMatch && !/^\s{2,}/.test(line)) {
      bullets.push(bulletMatch[1] ?? "");
      continue;
    }
    // An indented line continues the bullet above it.
    const continuation = line.trim();
    const lastIndex = bullets.length - 1;
    if (continuation !== "" && lastIndex >= 0) {
      bullets[lastIndex] = `${bullets[lastIndex]} ${continuation}`;
    }
  }

  return entries;
}
