import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function RadiusRow() {
  const { cfg, t, update } = useSettingsContext();
  const [draft, setDraft] = useState(String(cfg.padding.radius));
  useEffect(() => setDraft(String(cfg.padding.radius)), [cfg.padding.radius]);

  const commit = () => {
    const radius = Math.min(128, Math.max(1, Math.trunc(+draft) || 1));
    setDraft(String(radius));
    if (radius !== cfg.padding.radius) update({ ...cfg, padding: { ...cfg.padding, radius } });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.roundCorners}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Input
          type="number"
          min={1}
          max={128}
          className="w-20"
          disabled={!cfg.padding.enabled || !cfg.padding.round}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.currentTarget.blur();
          }}
        />
        <Switch
          checked={cfg.padding.round}
          disabled={!cfg.padding.enabled}
          onCheckedChange={(round) => update({ ...cfg, padding: { ...cfg.padding, round } })}
        />
      </ItemActions>
    </Item>
  );
}
