import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemFooter, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";

type Mode = "transparent" | "wallpaper" | "image";
const MODES: Mode[] = ["transparent", "wallpaper", "image"];

export function BackgroundRow() {
  const { cfg, t, update } = useSettingsContext();
  const { padding } = cfg;
  const mode: Mode = padding.image ? "image" : padding.wallpaper ? "wallpaper" : "transparent";
  const labels: Record<Mode, string> = { transparent: t.bgTransparent, wallpaper: t.wallpaper, image: t.bgImage };

  const browse = async () => {
    const picked = await open({ filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }] });
    if (typeof picked === "string") update({ ...cfg, padding: { ...padding, image: picked } });
  };

  const change = (value: string[]) => {
    const next = value[0] as Mode | undefined;
    if (!next || next === mode) return;
    if (next === "image") browse();
    else update({ ...cfg, padding: { ...padding, image: "", wallpaper: next === "wallpaper" } });
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.background}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[mode]} onValueChange={change}>
          {MODES.map((m) => (
            <ToggleGroupItem
              key={m}
              value={m}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {labels[m]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
      {mode === "image" && (
        <ItemFooter>
          <span className="min-w-0 truncate text-muted-foreground">{padding.image}</span>
          <Button size="sm" variant="outline" onClick={browse}>
            {t.browse}
          </Button>
        </ItemFooter>
      )}
    </Item>
  );
}
