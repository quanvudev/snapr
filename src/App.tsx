import { SettingsProvider } from "@/components/settings-provider";
import { Settings } from "@/components/settings";
import { Setup } from "@/components/setup";
import { useSettings } from "@/hooks/use-settings";
import { useTheme } from "@/hooks/use-theme";
import "./App.css";

function App() {
  const settings = useSettings();
  useTheme(settings?.cfg.theme);
  if (!settings) return null;
  return <SettingsProvider value={settings}>{settings.view === "setup" ? <Setup /> : <Settings />}</SettingsProvider>;
}

export default App;
