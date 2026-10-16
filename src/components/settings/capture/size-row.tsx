import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";

export function SizeRow() {
  const { cfg, t, update } = useSettingsContext();
  const [draft, setDraft] = useState(String(cfg.padding.size));
  useEffect(() => setDraft(String(cfg.padding.size)), [cfg.padding.size]);

  const commit = () => {
    const size = Math.min(512, Math.max(1, Math.trunc(+draft) || 1));
    setDraft(String(size));
    if (size !== cfg.padding.size) update({ ...cfg, padding: { ...cfg.padding, size } });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.paddingSize}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Input
          type="number"
          min={1}
          max={512}
          className="w-20"
          disabled={!cfg.padding.enabled}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
          }}
        />
      </ItemActions>
    </Item>
  );
}
