import { mountPointFallbackName } from "@/features/volumes/volumeDisplayName";
import type { ScanMethod } from "@/lib/backend/scans";
import { describeSkipReason, type VolumeIndexStatus } from "@/lib/backend/search";
import type { VolumeKind } from "@/lib/backend/volumes";
import { formatRelativeDate } from "@/lib/formatting/formatRelativeDate";
import { currentPlatform } from "@/lib/platform/currentPlatform";

export type VolumeIndexTone = "ready" | "busy" | "waiting" | "failed" | "skipped";

export type VolumeIndexStateDescription = {
  label: string;
  tone: VolumeIndexTone;
  /** A sentence with the reason, for failed and skipped drives. */
  detail: string | null;
};

/** What is happening to one drive's index, in a word or two plus an optional reason. */
export function describeVolumeIndexState(volume: VolumeIndexStatus): VolumeIndexStateDescription {
  const hasIndex = volume.index !== null;
  const { state } = volume;

  switch (state.kind) {
    case "ready":
      return { label: "Ready", tone: "ready", detail: null };
    case "building":
      return { label: hasIndex ? "Refreshing" : "Reading", tone: "busy", detail: null };
    case "idle":
      return { label: hasIndex ? "Refresh queued" : "Waiting", tone: "waiting", detail: null };
    case "failed":
      return {
        label: hasIndex ? "Refresh failed" : "Failed",
        tone: "failed",
        detail: hasIndex ? `${state.reason} The previous index still answers.` : state.reason,
      };
    case "skipped":
      return { label: "Not indexed", tone: "skipped", detail: describeSkipReason(state.reason) };
  }
}

/** `Fast` for the MFT read, `Standard` for the folder walk. */
export function indexMethodLabel(method: ScanMethod): string {
  return method === "masterFileTable" ? "Fast" : "Standard";
}

/** `built just now`, `built 5 minutes ago`. */
export function describeIndexAge(
  builtAtUnixMilliseconds: number,
  nowUnixMilliseconds: number,
): string {
  return `built ${formatRelativeDate(builtAtUnixMilliseconds, nowUnixMilliseconds)}`;
}

const FALLBACK_NAME_BY_VOLUME_KIND: Record<VolumeKind, string> = {
  localDisk: "Local Disk",
  removable: "Removable Disk",
  network: "Network Drive",
  optical: "Optical Drive",
  ramDisk: "RAM Disk",
  cloudVirtual: "Cloud Drive",
  unknown: "Drive",
};

function fallbackVolumeName(volume: Pick<VolumeIndexStatus, "mountPoint" | "volumeKind">): string {
  return currentPlatform().os === "windows"
    ? FALLBACK_NAME_BY_VOLUME_KIND[volume.volumeKind]
    : mountPointFallbackName(volume.mountPoint);
}

/**
 * `Windows (C:)`, or `Local Disk (C:)` for a drive without a label, as in the sidebar. On Linux
 * `USB (/media/martin/USB)`, or `File System (/)` for a root without a label.
 */
export function volumeIndexDisplayName(
  volume: Pick<VolumeIndexStatus, "label" | "mountPoint" | "volumeKind">,
): string {
  const trimmedMountPoint = volume.mountPoint.replace(/[\\/]+$/, "");
  const mountPointLabel = trimmedMountPoint.length > 0 ? trimmedMountPoint : volume.mountPoint;
  const trimmedLabel = volume.label.trim();
  const name = trimmedLabel.length > 0 ? trimmedLabel : fallbackVolumeName(volume);
  return `${name} (${mountPointLabel})`;
}

/** The drives whose first index is still being read or waiting for a build slot. */
export function volumesWarmingUp(volumes: readonly VolumeIndexStatus[]): VolumeIndexStatus[] {
  return volumes.filter(
    (volume) =>
      volume.index === null && (volume.state.kind === "building" || volume.state.kind === "idle"),
  );
}

/** The memory of every drive's index together. */
export function totalEstimatedIndexMemoryBytes(volumes: readonly VolumeIndexStatus[]): number {
  return volumes.reduce((total, volume) => total + (volume.index?.estimatedMemoryBytes ?? 0), 0);
}

/**
 * Changes whenever the set of searchable drives changes or one of their indexes is replaced, and
 * only then (not on progress ticks), so an open query can be run again exactly when it would get a
 * different answer.
 */
export function searchableVolumesKey(volumes: readonly VolumeIndexStatus[]): string {
  return volumes
    .filter((volume) => volume.index !== null)
    .map((volume) => `${volume.mountPoint}@${volume.index?.builtAtUnixMilliseconds ?? 0}`)
    .join("|");
}
