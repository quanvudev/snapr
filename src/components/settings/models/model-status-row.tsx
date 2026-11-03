import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { PERCENT } from "@/lib/queue";
import { labelOf, type ModelStore } from "./stores";

export function ModelStatusRow({ store }: { store: ModelStore }) {
  const { t, setSection } = useSettingsContext();
  const { selected, progress } = store;
  const downloading = progress?.id === selected ? progress : null;
  const percent = downloading && downloading.total > 0 ? Math.round((downloading.done / downloading.total) * PERCENT) : 0;
  const state = downloading
    ? t.modelDownloadingPercent.replace("{percent}", String(percent))
    : store.ready.includes(selected)
      ? t.paddleDownloaded
      : t.paddleNotDownloaded;

  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{`${store.name} · ${labelOf(store, selected)} · ${state}`}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Button variant="link" size="sm" onClick={() => setSection("models")}>
          {t.modelsLink}
        </Button>
      </ItemActions>
    </Item>
  );
}
