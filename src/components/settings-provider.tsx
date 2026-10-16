import { createContext, useContext } from "react";
import type { SettingsValue } from "@/hooks/use-settings";

const SettingsContext = createContext<SettingsValue | null>(null);

export function SettingsProvider({ value, children }: { value: SettingsValue; children: React.ReactNode }) {
  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}

export function useSettingsContext() {
  const value = useContext(SettingsContext);
  if (!value) throw new Error("useSettingsContext must be used within a SettingsProvider.");
  return value;
}
