import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import type { SectionId } from "@/hooks/use-settings";

type Props = { text: string; section: SectionId; label: string };

export function ModelUsageRow({ text, section, label }: Props) {
  const { setSection } = useSettingsContext();
  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemDescription>{text}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="link" size="sm" onClick={() => setSection(section)}>
          {label}
        </Button>
      </ItemActions>
    </Item>
  );
}
