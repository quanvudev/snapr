import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function RingSwitchRow() {
  const { cfg, t, update, view, setSection } = useSettingsContext();
  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{t.ring}</ItemTitle>
        <ItemDescription>{t.ringSwitchHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        {cfg.ring.enabled && view === "settings" && (
          <Button variant="outline" size="sm" onClick={() => setSection("ring")}>
            {t.ringConfigure}
          </Button>
        )}
        <Switch
          checked={cfg.ring.enabled}
          onCheckedChange={(enabled) => update({ ...cfg, ring: { ...cfg.ring, enabled } })}
        />
      </ItemActions>
    </Item>
  );
}
