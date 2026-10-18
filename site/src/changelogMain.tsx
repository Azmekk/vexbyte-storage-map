import "@fontsource-variable/bricolage-grotesque";
import "@fontsource-variable/onest";
import "@fontsource-variable/jetbrains-mono";
import "./styles/global.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
// The hand-written changelog at the repository root, bundled at build time.
import changelogMarkdown from "../../CHANGELOG.md?raw";
import { ChangelogPage } from "./components/ChangelogPage";
import { parseChangelog } from "./lib/changelog/parseChangelog";

const entries = parseChangelog(changelogMarkdown);
const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <ChangelogPage entries={entries} />
    </StrictMode>,
  );
}
