import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";

export function BackgroundRow() {
  const { cfg, t, update } = useSettingsContext();
  const set = (image: string) => update({ ...cfg, padding: { ...cfg.padding, image } });

  const browse = async () => {
    const picked = await open({ filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }] });
    if (typeof picked === "string") set(picked);
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.background}</ItemTitle>
        <ItemDescription className="truncate">{cfg.padding.image || (cfg.padding.wallpaper ? t.wallpaperInUse : t.transparent)}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="outline" disabled={!cfg.padding.enabled} onClick={browse}>
          {t.browse}
        </Button>
        <Button variant="outline" disabled={!cfg.padding.enabled || !cfg.padding.image} onClick={() => set("")}>
          {t.clear}
        </Button>
      </ItemActions>
    </Item>
  );
}
