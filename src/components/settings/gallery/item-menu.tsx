import { ContextMenuItem, ContextMenuSeparator } from "@/components/ui/context-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem } from "@/lib/api";

export function ItemMenu({ item, actions }: { item: GalleryItem; actions: GalleryActions }) {
  const { t } = useSettingsContext();
  return (
    <>
      <ContextMenuItem onClick={() => actions.open(item)}>{t.openDefault}</ContextMenuItem>
      <ContextMenuItem onClick={() => actions.reveal(item)}>{t.showInExplorer}</ContextMenuItem>
      {item.kind === "photo" && <ContextMenuItem onClick={() => actions.copy(item)}>{t.copyImage}</ContextMenuItem>}
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" onClick={() => actions.remove(item)}>
        {t.deleteItem}
      </ContextMenuItem>
    </>
  );
}
