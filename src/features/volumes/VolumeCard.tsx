import { CloudIcon, ZapIcon } from "lucide-react";
import { motion } from "motion/react";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScanTargetButton } from "@/features/scan/ScanTargetButton";
import { cardHoverLift, fadeSlideInVariants } from "@/lib/motion/transitions";
import { DriveTypeIcon } from "./DriveTypeIcon";
import type { VolumeDescription } from "./describeVolume";
import { VolumeUsageDonut } from "./VolumeUsageDonut";

type VolumeCardProps = {
  volume: VolumeDescription;
  isElevated: boolean | null;
};

export function VolumeCard({ volume, isElevated }: VolumeCardProps) {
  return (
    <motion.div variants={fadeSlideInVariants} whileHover={cardHoverLift}>
      <Card className="h-full">
        <CardHeader>
          <div className="flex items-start gap-3">
            <div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-muted">
              {volume.isCloudDrive ? (
                <CloudIcon className="size-5 text-muted-foreground" aria-hidden />
              ) : (
                <DriveTypeIcon
                  driveType={volume.driveType}
                  className="size-5 text-muted-foreground"
                />
              )}
            </div>
            <div className="min-w-0 flex-1">
              <CardTitle className="truncate">{volume.displayName}</CardTitle>
              <CardDescription>
                {volume.driveLabel} · {volume.fileSystemName}
              </CardDescription>
            </div>
          </div>
          {volume.isCloudDrive && (
            <Badge variant="secondary" className="mt-3 w-fit">
              <CloudIcon />
              Cloud drive
            </Badge>
          )}
          {volume.supportsFastScan && (
            <Badge variant="secondary" className="mt-3 w-fit">
              <ZapIcon />
              {isElevated === true ? "Fast scan ready" : "Fast scan available as admin"}
            </Badge>
          )}
        </CardHeader>
        <CardContent className="flex items-center gap-5">
          <VolumeUsageDonut
            segments={volume.usageChartSegments}
            usedPercentageLabel={volume.usedPercentageLabel}
          />
          <dl className="grid gap-2 text-sm">
            <div>
              <dt className="text-muted-foreground text-xs">Used</dt>
              <dd className="font-medium tabular-nums">
                {volume.usedSizeText} of {volume.totalSizeText}
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground text-xs">Free</dt>
              <dd className="font-medium tabular-nums">{volume.freeSizeText}</dd>
            </div>
          </dl>
        </CardContent>
        <CardFooter className="mt-auto">
          {volume.isCloudDrive ? (
            <p className="text-muted-foreground text-sm">
              Streamed cloud drives aren't scanned, so nothing gets downloaded.
            </p>
          ) : (
            <ScanTargetButton
              mountPoint={volume.mountPoint}
              label={volume.driveLabel}
              supportsFastScan={volume.supportsFastScan}
              isElevated={isElevated}
            />
          )}
        </CardFooter>
      </Card>
    </motion.div>
  );
}
