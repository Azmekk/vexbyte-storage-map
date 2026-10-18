import "./Footer.css";
import { ISSUES_URL, REPOSITORY_URL } from "../lib/release/repository";

export function Footer() {
  return (
    <footer className="footer">
      <div className="wrap footer-inner">
        <span>© {new Date().getFullYear()} Vexbyte · MIT license</span>
        <nav aria-label="Footer">
          <a href={REPOSITORY_URL}>Source</a>
          <a href={`${REPOSITORY_URL}/releases`}>Releases</a>
          <a href={ISSUES_URL}>Report an issue</a>
        </nav>
      </div>
    </footer>
  );
}
