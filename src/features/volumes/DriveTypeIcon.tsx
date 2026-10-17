import { DiscIcon, HardDriveIcon, MemoryStickIcon, NetworkIcon, UsbIcon } from "lucide-react";
import type { DriveType } from "@/lib/backend/volumes";

const ICON_BY_DRIVE_TYPE: Record<DriveType, typeof HardDriveIcon> = {
  fixed: HardDriveIcon,
  removable: UsbIcon,
  network: NetworkIcon,
  optical: DiscIcon,
  ram: MemoryStickIcon,
  unknown: HardDriveIcon,
};

type DriveTypeIconProps = {
  driveType: DriveType;
  className?: string;
};

export function DriveTypeIcon({ driveType, className }: DriveTypeIconProps) {
  const Icon = ICON_BY_DRIVE_TYPE[driveType];
  return <Icon className={className} aria-hidden />;
}
