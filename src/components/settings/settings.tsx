import { ListTodo } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
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
import { useQueue } from "@/hooks/use-queue";
import { APP_INFO } from "@/lib/app-info";
import icon from "../../../src-tauri/icons/128x128.png";
import { QueueSheet } from "./queue-sheet";
import { SECTIONS, type SectionPlace } from "./sections";

function SectionMenu({ place }: { place: SectionPlace }) {
  const { t, section, setSection } = useSettingsContext();
  return (
    <SidebarMenu>
      {SECTIONS.filter((s) => s.place === place).map(({ id, icon: Icon }) => (
        <SidebarMenuItem key={id}>
          <SidebarMenuButton isActive={section === id} tooltip={t[id]} onClick={() => setSection(id)}>
            <Icon />
            <span>{t[id]}</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      ))}
    </SidebarMenu>
  );
}

export function Settings() {
  const { t, failed, version, section } = useSettingsContext();
  const queue = useQueue();
  const current = SECTIONS.find((s) => s.id === section) ?? SECTIONS.find((s) => s.id === "general")!;
  const { component: Active, wide, fill } = current;

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
            <SectionMenu place="top" />
          </SidebarGroup>
          <SidebarGroup>
            <SidebarGroupLabel>{t.settings}</SidebarGroupLabel>
            <SectionMenu place="settings" />
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter>
          <SectionMenu place="footer" />
        </SidebarFooter>
      </Sidebar>

      <SidebarInset className="min-h-0 min-w-0 overflow-hidden">
        <TitleBar
          title={t[current.id]}
          label={t.close}
          className="gap-1 border-b pl-4"
          actions={
            (queue.count > 0 || queue.history.length > 0) && (
              <Button variant="ghost" size="sm" title={t.queueShow} aria-label={t.queueShow} onClick={() => queue.setOpen(true)}>
                <ListTodo />
                {queue.count > 0 && <Badge variant="secondary">{queue.count}</Badge>}
              </Button>
            )
          }
        >
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mx-2 data-vertical:h-4 data-vertical:self-center" />
        </TitleBar>
        {fill ? (
          <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
            <Active />
            {failed && <p className="px-6 pb-2 text-xs text-red-500">{t.failed}</p>}
          </div>
        ) : (
          <ScrollArea className="min-h-0 flex-1">
            <div className={`mx-auto space-y-4 p-6 ${wide ? "" : "max-w-3xl"}`}>
              <Active />
              {failed && <p className="text-xs text-red-500">{t.failed}</p>}
            </div>
          </ScrollArea>
        )}
      </SidebarInset>
      <QueueSheet
        t={t}
        open={queue.open}
        onOpenChange={queue.setOpen}
        items={queue.items}
        history={queue.history}
        onCancel={queue.cancel}
        onCancelAll={queue.cancelAll}
        onRetry={queue.retry}
        onReveal={queue.reveal}
        onClear={queue.clear}
      />
    </SidebarProvider>
  );
}
