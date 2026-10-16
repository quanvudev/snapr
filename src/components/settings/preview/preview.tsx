import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";

export function Preview() {
  const { cfg, t, update } = useSettingsContext();
  const [draft, setDraft] = useState(String(cfg.preview_seconds));
  useEffect(() => setDraft(String(cfg.preview_seconds)), [cfg.preview_seconds]);

  const commit = () => {
    const n = Math.min(30, Math.max(0, Math.trunc(+draft) || 0));
    setDraft(String(n));
    if (n !== cfg.preview_seconds) update({ ...cfg, preview_seconds: n });
  };

  return (
    <ItemGroup className="gap-3">
      <Item variant="muted">
        <ItemContent>
          <ItemTitle>{t.seconds}</ItemTitle>
          <ItemDescription>{t.off}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Input
            type="number"
            min={0}
            max={30}
            className="w-20"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
            }}
          />
        </ItemActions>
      </Item>
    </ItemGroup>
  );
}
