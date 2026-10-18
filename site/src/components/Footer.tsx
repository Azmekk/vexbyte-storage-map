import "./Footer.css";
import { ISSUES_URL, RELEASES_URL, REPOSITORY_URL } from "../lib/release/repository";
import { CHANGELOG_URL } from "../lib/sitePaths";

export function Footer() {
  return (
    <footer className="footer">
      <div className="wrap footer-inner">
        <span>© {new Date().getFullYear()} Vexbyte · MIT license · Windows and Linux</span>
        <nav aria-label="Footer">
          <a href={CHANGELOG_URL}>Changelog</a>
          <a href={REPOSITORY_URL}>Source</a>
          <a href={RELEASES_URL}>Releases</a>
          <a href={ISSUES_URL}>Report an issue</a>
        </nav>
      </div>
    </footer>
  );
}
