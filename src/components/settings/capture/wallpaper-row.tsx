import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function WallpaperRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.wallpaper}</ItemTitle>
        <ItemDescription>{t.wallpaperHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.padding.wallpaper}
          disabled={!cfg.padding.enabled}
          onCheckedChange={(wallpaper) => update({ ...cfg, padding: { ...cfg.padding, wallpaper } })}
        />
      </ItemActions>
    </Item>
  );
}
