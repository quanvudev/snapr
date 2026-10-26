import { Images, Settings as SettingsIcon } from "lucide-react";
import type { SectionId } from "@/hooks/use-settings";
import { Gallery } from "./gallery/gallery";
import { Preferences } from "./preferences/preferences";

export const SECTIONS: { id: SectionId; icon: typeof Images; component: () => React.ReactNode; wide?: boolean }[] = [
  { id: "gallery", icon: Images, component: Gallery, wide: true },
  { id: "settings", icon: SettingsIcon, component: Preferences },
];
