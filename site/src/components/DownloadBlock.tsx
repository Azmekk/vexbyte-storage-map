import "./DownloadBlock.css";
import { useCopyPageLink } from "../hooks/useCopyPageLink";
import type { DownloadTarget } from "../hooks/useDownloadTarget";
import { useMagneticHover } from "../hooks/useMagneticHover";
import { formatMegabytes } from "../lib/release/formatSize";
import { unsupportedPlatformName } from "../lib/release/platform";
import { RELEASES_URL, REPOSITORY_URL } from "../lib/release/repository";
import { WindowsIcon } from "./icons/WindowsIcon";

type DownloadBlockProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: () => void;
};

function describeRelease({ release }: DownloadTarget): string {
  const parts = [`v${release.version}`, "x64"];
  if (release.sizeInBytes !== null) parts.push(formatMegabytes(release.sizeInBytes));
  return parts.join(" · ");
}

export function DownloadBlock({ downloadTarget, onDownloadStarted }: DownloadBlockProps) {
  const { platform, release } = downloadTarget;
  const magneticHover = useMagneticHover();
  const { copyState, copyPageLink } = useCopyPageLink();
  const unsupportedName = unsupportedPlatformName(platform);

  if (unsupportedName !== null) {
    return (
      <div className="download">
        <div className="unsupported" role="status">
          <span className="unsupported-badge">Not available for {unsupportedName}</span>
          <p>
            Vexbyte Storage Map runs on 64-bit Windows 10 and 11. Open this page on your Windows PC
            to download it, or get the installer from GitHub.
          </p>
          <div className="unsupported-actions">
            <a className="secondary-button" href={RELEASES_URL}>
              View releases on GitHub
            </a>
            <button className="secondary-button" type="button" onClick={copyPageLink}>
              {copyState === "copied" ? "Link copied" : "Copy link to this page"}
            </button>
          </div>
          {copyState === "failed" && (
            <p className="copy-fallback">{window.location.href.split("#")[0]}</p>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="download">
      <a className="cta" href={release.installerUrl} onClick={onDownloadStarted} {...magneticHover}>
        <span className="cta-icon">
          <WindowsIcon />
        </span>
        <span className="cta-text">
          <span className="cta-label">Download for Windows</span>
          <span className="cta-meta">{describeRelease(downloadTarget)}</span>
        </span>
      </a>
      {platform === "windows-arm64" && (
        <p className="platform-note">
          <strong>Windows on ARM:</strong> this is the x64 build. It runs through Windows' built-in
          x64 emulation.
        </p>
      )}
      <div className="download-links">
        <a href={REPOSITORY_URL}>Source on GitHub</a>
        <span>Windows 10 and 11 · 64-bit</span>
      </div>
    </div>
  );
}
