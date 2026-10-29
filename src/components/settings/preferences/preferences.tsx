import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/components/settings-provider";
import type { T } from "@/lib/i18n";
import { About } from "../about/about";
import { Capture } from "../capture/capture";
import { General } from "../general/general";
import { Hotkeys } from "../hotkeys/hotkeys";
import { Recording } from "../recording/recording";
import { Text } from "../text/text";

const GROUPS: { title: keyof (typeof T)["en"]; component: () => React.ReactNode }[] = [
  { title: "general", component: General },
  { title: "capture", component: Capture },
  { title: "textGroup", component: Text },
  { title: "recording", component: Recording },
  { title: "hotkeys", component: Hotkeys },
  { title: "about", component: About },
];

export function Preferences() {
  const { t, group, setGroup } = useSettingsContext();
  const jump = (title: string) =>
    document.getElementById(`group-${title}`)?.scrollIntoView({ behavior: "smooth", block: "start" });

  useEffect(() => {
    if (!group) return;
    jump(group);
    setGroup(null);
  }, [group, setGroup]);

  return (
    <div className="space-y-8">
      <nav className="sticky top-0 z-10 -mx-6 -mt-6 flex flex-wrap gap-1 border-b bg-background px-6 py-2">
        {GROUPS.map(({ title }) => (
          <Button key={title} size="sm" variant="ghost" onClick={() => jump(title)}>
            {t[title]}
          </Button>
        ))}
      </nav>
      {GROUPS.map(({ title, component: Group }) => (
        <section key={title} id={`group-${title}`} className="scroll-mt-14 space-y-4">
          <h2 className="text-sm font-semibold">{t[title]}</h2>
          <Group />
        </section>
      ))}
    </div>
  );
}
