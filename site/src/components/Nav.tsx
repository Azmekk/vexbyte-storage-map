import "./Nav.css";
import { REPOSITORY_URL } from "../lib/release/repository";
import { HOME_URL } from "../lib/sitePaths";

export function Nav() {
  return (
    <header className="nav">
      <div className="wrap nav-inner">
        <a className="brand" href={`${HOME_URL}#top`}>
          <img src={`${HOME_URL}icon.png`} alt="" width="26" height="26" />
          Vexbyte Storage Map
        </a>
        <nav className="nav-links" aria-label="Main">
          <a className="hide-small" href={`${HOME_URL}#views`}>
            Views
          </a>
          <a className="hide-small" href={`${HOME_URL}#features`}>
            Features
          </a>
          <a className="hide-small" href={REPOSITORY_URL}>
            GitHub
          </a>
          <a className="nav-cta" href={`${HOME_URL}#get`}>
            Download
          </a>
        </nav>
      </div>
    </header>
  );
}
