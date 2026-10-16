import { Button } from "@/components/ui/button";
import { ItemGroup, ItemSeparator } from "@/components/ui/item";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ClipboardRow, FolderRow, HotkeyRows, LanguageRow, StartupRow } from "@/components/settings";
import { useSettingsContext } from "@/components/settings-provider";
import { TitleBar } from "@/components/title-bar";

const heading = (s: string) => <h3 className="px-1 text-xs font-medium text-muted-foreground">{s}</h3>;

export function Setup() {
  const { t, rows, failed, start } = useSettingsContext();
  return (
    <div className="flex h-screen flex-col text-sm">
      <TitleBar label={t.close} />
      <ScrollArea className="min-h-0 flex-1">
        <div className="mx-auto max-w-6xl space-y-4 px-6 pb-4">
          <div>
            <h1 className="text-xl font-semibold">{t.welcome}</h1>
            <p className="text-muted-foreground">{t.welcomeSub}</p>
          </div>
          <div className="grid items-start gap-6 lg:grid-cols-2">
            <div className="space-y-3">
              {heading(t.general)}
              <ItemGroup className="gap-3">
                <FolderRow />
                <div className="rounded-md bg-muted/50 [&_[data-slot=item]]:bg-transparent">
                  <ClipboardRow />
                  <ItemSeparator className="my-0" />
                  <StartupRow />
                  <ItemSeparator className="my-0" />
                  <LanguageRow />
                </div>
              </ItemGroup>
            </div>
            <div className="space-y-3">
              {heading(t.hotkeys)}
              <ItemGroup className="gap-3">
                <HotkeyRows />
              </ItemGroup>
            </div>
          </div>
        </div>
      </ScrollArea>
      <div className="border-t px-6 py-3">
        <div className="mx-auto flex max-w-6xl items-center justify-end gap-3">
          {failed && <p className="text-xs text-red-500">{t.failed}</p>}
          <Button disabled={rows.some((r) => r.error)} onClick={start}>
            {t.start}
          </Button>
        </div>
      </div>
    </div>
  );
}
