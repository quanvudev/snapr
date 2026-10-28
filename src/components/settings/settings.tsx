import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useSettingsContext } from "@/components/settings-provider";
import { TitleBar } from "@/components/title-bar";
import { APP_INFO } from "@/lib/app-info";
import icon from "../../../src-tauri/icons/128x128.png";
import { SECTIONS } from "./sections";

export function Settings() {
  const { cfg, t, failed, version, section, setSection } = useSettingsContext();
  const sections = SECTIONS.filter((s) => s.id !== "ring" || cfg.ring.enabled);
  const current = sections.find((s) => s.id === section) ?? sections.find((s) => s.id === "settings")!;
  const { component: Active, wide } = current;

  return (
    <SidebarProvider className="h-screen min-h-0 text-sm has-data-[variant=inset]:bg-transparent">
      <Sidebar variant="inset" collapsible="icon" className="*:data-[sidebar=sidebar]:bg-transparent">
        <SidebarHeader data-tauri-drag-region className="h-12 flex-row items-center">
          <img src={icon} alt="" className="pointer-events-none size-8 shrink-0" />
          <div className="pointer-events-none grid leading-tight group-data-[collapsible=icon]:hidden">
            <span className="font-semibold">{APP_INFO.name}</span>
            <span className="text-xs text-muted-foreground">{version}</span>
          </div>
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup>
            <SidebarMenu>
              {sections.map(({ id, icon: Icon }) => (
                <SidebarMenuItem key={id}>
                  <SidebarMenuButton isActive={current.id === id} tooltip={t[id]} onClick={() => setSection(id)}>
                    <Icon />
                    <span>{t[id]}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>

      <SidebarInset className="min-h-0 min-w-0 overflow-hidden">
        <TitleBar title={t[current.id]} label={t.close} className="gap-1 border-b pl-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mx-2 data-vertical:h-4 data-vertical:self-center" />
        </TitleBar>
        <ScrollArea className="min-h-0 flex-1">
          <div className={`mx-auto space-y-4 p-6 ${wide ? "" : "max-w-3xl"}`}>
            <Active />
            {failed && <p className="text-xs text-red-500">{t.failed}</p>}
          </div>
        </ScrollArea>
      </SidebarInset>
    </SidebarProvider>
  );
}
