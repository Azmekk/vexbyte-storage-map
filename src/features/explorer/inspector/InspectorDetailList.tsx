import type { ReactNode } from "react";

export type InspectorDetail = { label: string; value: ReactNode };

type InspectorDetailListProps = {
  details: InspectorDetail[];
};

export function InspectorDetailList({ details }: InspectorDetailListProps) {
  return (
    <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
      {details.map((detail) => (
        <div key={detail.label} className="contents">
          <dt className="text-muted-foreground">{detail.label}</dt>
          <dd className="text-right tabular-nums">{detail.value}</dd>
        </div>
      ))}
    </dl>
  );
}
