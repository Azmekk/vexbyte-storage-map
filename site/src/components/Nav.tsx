import "./Nav.css";
import { REPOSITORY_URL } from "../lib/release/repository";

export function Nav() {
  return (
    <header className="nav">
      <div className="wrap nav-inner">
        <a className="brand" href="#top">
          <img src="icon.png" alt="" width="26" height="26" />
          Vexbyte Storage Map
        </a>
        <nav className="nav-links" aria-label="Main">
          <a className="hide-small" href="#views">
            Views
          </a>
          <a className="hide-small" href="#features">
            Features
          </a>
          <a className="hide-small" href={REPOSITORY_URL}>
            GitHub
          </a>
          <a className="nav-cta" href="#get">
            Download
          </a>
        </nav>
      </div>
    </header>
  );
}
