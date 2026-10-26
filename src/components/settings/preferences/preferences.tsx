import { useSettingsContext } from "@/components/settings-provider";
import type { T } from "@/lib/i18n";
import { About } from "../about/about";
import { Capture } from "../capture/capture";
import { General } from "../general/general";
import { Hotkeys } from "../hotkeys/hotkeys";
import { Preview } from "../preview/preview";
import { Recording } from "../recording/recording";

const GROUPS: { title: keyof (typeof T)["en"]; component: () => React.ReactNode }[] = [
  { title: "general", component: General },
  { title: "capture", component: Capture },
  { title: "preview", component: Preview },
  { title: "recording", component: Recording },
  { title: "hotkeys", component: Hotkeys },
  { title: "about", component: About },
];

export function Preferences() {
  const { t } = useSettingsContext();
  return (
    <div className="space-y-8">
      {GROUPS.map(({ title, component: Group }) => (
        <section key={title} className="space-y-4">
          <h2 className="text-sm font-semibold">{t[title]}</h2>
          <Group />
        </section>
      ))}
    </div>
  );
}
