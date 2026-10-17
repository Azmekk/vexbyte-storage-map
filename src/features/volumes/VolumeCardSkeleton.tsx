import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

export function VolumeCardSkeleton() {
  return (
    <Card aria-hidden>
      <CardHeader className="flex items-center gap-3">
        <Skeleton className="size-10 rounded-lg" />
        <div className="grid flex-1 gap-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-3 w-20" />
        </div>
      </CardHeader>
      <CardContent className="flex items-center gap-5">
        <Skeleton className="size-28 rounded-full" />
        <div className="grid flex-1 gap-3">
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-8 w-2/3" />
        </div>
      </CardContent>
      <CardFooter>
        <Skeleton className="h-10 w-full" />
      </CardFooter>
    </Card>
  );
}
