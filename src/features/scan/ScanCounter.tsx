import { NumberTicker } from "@/components/NumberTicker";

type ScanCounterProps = {
  label: string;
  value: number;
  format: (value: number) => string;
  isProminent?: boolean;
};

export function ScanCounter({ label, value, format, isProminent = false }: ScanCounterProps) {
  return (
    <div className="grid gap-1">
      <NumberTicker
        value={value}
        formatValue={format}
        className={
          isProminent
            ? "font-semibold text-5xl tabular-nums tracking-tight"
            : "font-semibold text-3xl text-foreground/85 tabular-nums tracking-tight"
        }
      />
      <span className="text-muted-foreground text-sm">{label}</span>
    </div>
  );
}
