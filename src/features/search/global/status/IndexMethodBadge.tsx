import { Badge } from "@/components/ui/badge";
import type { ScanMethod } from "@/lib/backend/scans";
import { indexMethodLabel } from "./describeVolumeIndexState";

type IndexMethodBadgeProps = {
  method: ScanMethod;
};

export function IndexMethodBadge({ method }: IndexMethodBadgeProps) {
  return (
    <Badge
      variant={method === "masterFileTable" ? "secondary" : "outline"}
      title={
        method === "masterFileTable"
          ? "Reads the NTFS Master File Table directly"
          : "Lists every folder through the file system"
      }
    >
      {indexMethodLabel(method)}
    </Badge>
  );
}
