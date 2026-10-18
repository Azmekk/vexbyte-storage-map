import "./DownloadBlock.css";
import { useCopyPageLink } from "../hooks/useCopyPageLink";
import type { DownloadTarget } from "../hooks/useDownloadTarget";
import { useMagneticHover } from "../hooks/useMagneticHover";
import { formatMegabytes } from "../lib/release/formatSize";
import { isComingSoonPlatform, unsupportedPlatformName } from "../lib/release/platform";
import { RELEASES_URL, REPOSITORY_URL } from "../lib/release/repository";
import type { StartedDownload } from "../lib/release/startedDownload";
import { WindowsIcon } from "./icons/WindowsIcon";
import { LinuxDownloads } from "./LinuxDownloads";

type DownloadBlockProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: (download: StartedDownload) => void;
};

type Notice = {
  badge: string;
  text: string;
};

function describeRelease({ release }: DownloadTarget): string {
  const parts = [`v${release.version}`, "x64"];
  const { sizeInBytes } = release.windowsInstaller;
  if (sizeInBytes !== null) parts.push(formatMegabytes(sizeInBytes));
  return parts.join(" · ");
}

function noticeFor({ platform, release }: DownloadTarget): Notice | null {
  if (platform === "linux") {
    if (release.linuxPackages.length > 0) return null;
    // The site can deploy before the first Linux release has finished uploading its files.
    return {
      badge: "Coming soon to Linux",
      text: "The Linux version is on its way. For now Vexbyte Storage Map runs on 64-bit Windows 10 and 11: open this page on a Windows PC to download it, or follow the releases on GitHub.",
    };
  }
  if (platform === "linux-arm64") {
    return {
      badge: "Linux on ARM: x64 only for now",
      text: "The Linux packages are built for x64 (amd64) PCs only for now. Open this page on an x64 PC to download them, or follow the releases on GitHub.",
    };
  }
  const unsupportedName = unsupportedPlatformName(platform);
  if (unsupportedName === null) return null;
  if (isComingSoonPlatform(platform)) {
    return {
      badge: `Coming soon to ${unsupportedName}`,
      text: `The ${unsupportedName} version is on its way. For now Vexbyte Storage Map runs on 64-bit Windows 10 and 11 and on x64 Linux: open this page on one of those to download it, or follow the releases on GitHub.`,
    };
  }
  return {
    badge: `Not available for ${unsupportedName}`,
    text: "Vexbyte Storage Map runs on 64-bit Windows 10 and 11 and on x64 Linux today, with macOS coming soon. Open this page on your computer to download it, or get it from GitHub.",
  };
}

export function DownloadBlock({ downloadTarget, onDownloadStarted }: DownloadBlockProps) {
  const { platform, release } = downloadTarget;
  const magneticHover = useMagneticHover();
  const { copyState, copyPageLink } = useCopyPageLink();
  const notice = noticeFor(downloadTarget);

  if (notice !== null) {
    return (
      <div className="download">
        <div className="unsupported" role="status">
          <span className="unsupported-badge">{notice.badge}</span>
          <p>{notice.text}</p>
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

  if (platform === "linux") {
    return <LinuxDownloads downloadTarget={downloadTarget} onDownloadStarted={onDownloadStarted} />;
  }

  return (
    <div className="download">
      <a
        className="cta"
        href={release.windowsInstaller.url}
        onClick={() => onDownloadStarted({ platform: "windows" })}
        {...magneticHover}
      >
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
