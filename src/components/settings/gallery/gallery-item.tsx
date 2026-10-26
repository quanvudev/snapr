import { Button } from "@/components/ui/button";
import { ContextMenu, ContextMenuContent, ContextMenuTrigger } from "@/components/ui/context-menu";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem as Item } from "@/lib/api";
import { fileName, isViewable } from "@/lib/gallery";
import { ItemMenu } from "./item-menu";
import { Thumb } from "./thumb";

export function GalleryItem({ item, actions, onOpen }: { item: Item; actions: GalleryActions; onOpen: () => void }) {
  return (
    <ContextMenu>
      <ContextMenuTrigger
        render={
          <Button
            variant="ghost"
            title={fileName(item.path)}
            className="relative aspect-square h-auto overflow-hidden bg-muted p-0"
            onClick={isViewable(item) ? onOpen : undefined}
          />
        }
      >
        <Thumb item={item} />
        {!isViewable(item) && (
          <span className="absolute inset-x-0 bottom-0 truncate bg-background/80 px-1.5 py-0.5 text-xs">
            {fileName(item.path)}
          </span>
        )}
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ItemMenu item={item} actions={actions} />
      </ContextMenuContent>
    </ContextMenu>
  );
}
