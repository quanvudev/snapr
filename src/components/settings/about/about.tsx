import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemGroup, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import icon from "../../../../src-tauri/icons/128x128.png";

export function About() {
  const { cfg, t, version } = useSettingsContext();
  return (
    <>
      <div className="flex items-center gap-4">
        <img src={icon} alt="" className="size-16" />
        <div>
          <div className="text-lg font-semibold">snapr</div>
          <div className="text-xs text-muted-foreground">{version}</div>
        </div>
      </div>
      <ItemGroup className="gap-3">
        <Item variant="muted">
          <ItemContent>
            <ItemTitle>{t.config}</ItemTitle>
          </ItemContent>
          <ItemActions>
            <Button variant="outline" onClick={() => api.openConfigFile()}>
              {t.open}
            </Button>
          </ItemActions>
        </Item>
        <Item variant="muted">
          <ItemContent>
            <ItemTitle>{t.openFolder}</ItemTitle>
          </ItemContent>
          <ItemActions>
            <Button variant="outline" onClick={() => api.openFolder(cfg.output_dir)}>
              {t.open}
            </Button>
          </ItemActions>
        </Item>
      </ItemGroup>
    </>
  );
}
