import { useEffect, useState, type ComponentProps } from "react";
import { Input } from "@/components/ui/input";

type Props = { value: string; onCommit: (value: string) => void } & Omit<ComponentProps<typeof Input>, "value" | "onChange">;

export function CommitInput({ value, onCommit, ...props }: Props) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);

  return (
    <Input
      {...props}
      value={draft}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => draft !== value && onCommit(draft)}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
      }}
    />
  );
}
