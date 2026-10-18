import type { DownloadTarget } from "../hooks/useDownloadTarget";
import { useMagneticHover } from "../hooks/useMagneticHover";
import { formatMegabytes } from "../lib/release/formatSize";
import type { LinuxPackage } from "../lib/release/latestRelease";
import { LINUX_PACKAGE_DESCRIPTIONS } from "../lib/release/linuxPackageDescriptions";
import { REPOSITORY_URL } from "../lib/release/repository";
import type { StartedDownload } from "../lib/release/startedDownload";
import { DownloadIcon } from "./icons/DownloadIcon";

type LinuxDownloadsProps = {
  downloadTarget: DownloadTarget;
  onDownloadStarted: (download: StartedDownload) => void;
};

function describePackage(version: string, linuxPackage: LinuxPackage): string {
  const parts = [`v${version}`, "x64"];
  if (linuxPackage.sizeInBytes !== null) parts.push(formatMegabytes(linuxPackage.sizeInBytes));
  return parts.join(" · ");
}

export function LinuxDownloads({ downloadTarget, onDownloadStarted }: LinuxDownloadsProps) {
  const { release, preferredLinuxPackage } = downloadTarget;
  const magneticHover = useMagneticHover();
  const primaryPackage =
    release.linuxPackages.find((linuxPackage) => linuxPackage.kind === preferredLinuxPackage) ??
    release.linuxPackages[0];
  if (!primaryPackage) return null;
  const otherPackages = release.linuxPackages.filter(
    (linuxPackage) => linuxPackage !== primaryPackage,
  );
  const primaryDescription = LINUX_PACKAGE_DESCRIPTIONS[primaryPackage.kind];

  return (
    <div className="download">
      <a
        className="cta"
        href={primaryPackage.url}
        onClick={() => onDownloadStarted({ platform: "linux", linuxPackage: primaryPackage })}
        {...magneticHover}
      >
        <span className="cta-icon">
          <DownloadIcon />
        </span>
        <span className="cta-text">
          <span className="cta-label">Download {primaryDescription.label} for Linux</span>
          <span className="cta-meta">
            {primaryDescription.distributions} · {describePackage(release.version, primaryPackage)}
          </span>
        </span>
      </a>
      <div className="linux-alternatives">
        {otherPackages.map((linuxPackage) => {
          const description = LINUX_PACKAGE_DESCRIPTIONS[linuxPackage.kind];
          return (
            <a
              key={linuxPackage.kind}
              className="secondary-button linux-package"
              href={linuxPackage.url}
              onClick={() => onDownloadStarted({ platform: "linux", linuxPackage })}
            >
              <span>{description.label}</span>
              <small>{description.distributions}</small>
            </a>
          );
        })}
      </div>
      <div className="download-links">
        <a href={REPOSITORY_URL}>Source on GitHub</a>
        <span>Linux · 64-bit (x64)</span>
      </div>
    </div>
  );
}
