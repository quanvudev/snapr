import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from "@/components/ui/sidebar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useSettingsContext } from "@/components/settings-provider";
import { TitleBar } from "@/components/title-bar";
import { SECTIONS } from "./sections";

export function Settings() {
  const { t, failed, version, section, setSection } = useSettingsContext();
  const Active = SECTIONS.find((s) => s.id === section)!.component;

  return (
    <SidebarProvider className="h-screen min-h-0 text-sm">
      <Sidebar collapsible="none" className="h-full border-r">
        <SidebarHeader data-tauri-drag-region className="h-12 justify-center px-4 font-semibold">
          snapr
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup>
            <SidebarMenu>
              {SECTIONS.map(({ id, icon: Icon }) => (
                <SidebarMenuItem key={id}>
                  <SidebarMenuButton isActive={section === id} onClick={() => setSection(id)}>
                    <Icon />
                    <span>{t[id]}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter className="px-4 text-xs text-muted-foreground">{version}</SidebarFooter>
      </Sidebar>

      <div className="flex min-w-0 flex-1 flex-col">
        <TitleBar title={t[section]} label={t.close} />
        <ScrollArea className="min-h-0 flex-1">
          <div className="mx-auto max-w-3xl space-y-4 px-6 pb-6">
            <Active />
            {failed && <p className="text-xs text-red-500">{t.failed}</p>}
          </div>
        </ScrollArea>
      </div>
    </SidebarProvider>
  );
}
