import { ChevronRight } from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem as Item } from "@/lib/api";
import type { Group } from "@/lib/gallery";
import { GalleryItem } from "./gallery-item";

type Props = {
  group: Group;
  label: string;
  collapsed: boolean;
  actions: GalleryActions;
  onToggle: () => void;
  onOpen: (item: Item) => void;
};

export function GalleryGroup({ group, label, collapsed, actions, onToggle, onOpen }: Props) {
  return (
    <Collapsible open={!collapsed} onOpenChange={onToggle} className="space-y-2">
      <CollapsibleTrigger className="group flex w-full items-center gap-1 text-left font-medium">
        <ChevronRight className="size-4 transition-transform group-data-[panel-open]:rotate-90" />
        {label}
        <span className="ml-1 text-muted-foreground">{group.items.length}</span>
      </CollapsibleTrigger>
      <CollapsibleContent className="grid grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] gap-2">
        {group.items.map((item) => (
          <GalleryItem key={item.path} item={item} actions={actions} onOpen={() => onOpen(item)} />
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}
