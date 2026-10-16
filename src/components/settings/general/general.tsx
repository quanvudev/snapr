import { ItemGroup } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { ClipboardRow } from "./clipboard-row";
import { FolderRow } from "./folder-row";
import { LanguageRow } from "./language-row";
import { StartupRow } from "./startup-row";

const heading = (s: string) => <h3 className="px-1 text-xs font-medium text-muted-foreground">{s}</h3>;

export function General() {
  const { t } = useSettingsContext();
  return (
    <>
      {heading(t.output)}
      <ItemGroup className="gap-3">
        <FolderRow withOpen />
        <ClipboardRow />
      </ItemGroup>
      {heading(t.system)}
      <ItemGroup className="gap-3">
        <StartupRow />
        <LanguageRow />
      </ItemGroup>
    </>
  );
}
