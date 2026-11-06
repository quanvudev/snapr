import { useEffect, useState } from "react";
import { Slider } from "@/components/ui/slider";

export function SliderField({
  label,
  value,
  min,
  max,
  unit,
  onCommit,
  onDraft,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  unit: string;
  onCommit: (value: number) => void;
  onDraft?: (value: number) => void;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);

  const pick = (v: number | readonly number[]) => (Array.isArray(v) ? v[0] : v) as number;

  return (
    <div className="flex w-full items-center gap-4">
      <span className="w-28 shrink-0 text-muted-foreground">{label}</span>
      <Slider
        min={min}
        max={max}
        value={[draft]}
        onValueChange={(v) => {
          setDraft(pick(v));
          onDraft?.(pick(v));
        }}
        onValueCommitted={(v) => {
          const n = pick(v);
          if (n !== value) onCommit(n);
        }}
      />
      <span className="w-16 shrink-0 text-right tabular-nums">
        {draft} {unit}
      </span>
    </div>
  );
}
