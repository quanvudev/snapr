import type { CSSProperties } from "react";
import { ChevronRight } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem as Item } from "@/lib/api";
import type { Group } from "@/lib/gallery";
import { GalleryItem } from "./gallery-item";

type Props = {
  group: Group;
  label: string;
  tile: number;
  collapsed: boolean;
  actions: GalleryActions;
  selectMode: boolean;
  selected: Set<string>;
  onToggle: () => void;
  onOpen: (item: Item, from: DOMRect) => void;
  onSelect: (item: Item) => void;
  onSelectGroup: (on: boolean) => void;
  onRange: (item: Item) => void;
  onLongPress: (item: Item) => void;
};

export function GalleryGroup({
  group,
  label,
  tile,
  collapsed,
  actions,
  selectMode,
  selected,
  onToggle,
  onOpen,
  onSelect,
  onSelectGroup,
  onRange,
  onLongPress,
}: Props) {
  const count = group.items.filter((item) => selected.has(item.path)).length;
  return (
    <Collapsible open={!collapsed} onOpenChange={onToggle} className="space-y-2">
      <div className="sticky top-0 z-20 flex items-center gap-2 bg-background/95 pt-4 pb-1 backdrop-blur">
        {selectMode && (
          <Checkbox
            checked={count > 0 && count === group.items.length}
            indeterminate={count > 0 && count < group.items.length}
            onCheckedChange={(on) => onSelectGroup(on)}
          />
        )}
        <CollapsibleTrigger className="group flex flex-1 items-center gap-1 text-left font-medium">
          <ChevronRight className="size-4 transition-transform group-data-[panel-open]:rotate-90" />
          {label}
          <span className="ml-1 text-muted-foreground">{group.items.length}</span>
        </CollapsibleTrigger>
      </div>
      <CollapsibleContent
        className="grid grid-cols-[repeat(auto-fill,minmax(var(--tile),1fr))] gap-2"
        style={{ "--tile": `${tile}rem` } as CSSProperties}
      >
        {group.items.map((item) => (
          <GalleryItem
            key={item.path}
            item={item}
            actions={actions}
            selectMode={selectMode}
            selected={selected.has(item.path)}
            onOpen={(from) => onOpen(item, from)}
            onSelect={() => onSelect(item)}
            onRange={() => onRange(item)}
            onLongPress={() => onLongPress(item)}
          />
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}
