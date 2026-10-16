import { SettingsProvider } from "@/components/settings-provider";
import { Settings } from "@/components/settings";
import { Setup } from "@/components/setup";
import { Splash } from "@/components/splash";
import { useSettings } from "@/hooks/use-settings";
import "./App.css";

function App() {
  const settings = useSettings();
  if (!settings) return null;
  if (settings.view === "splash") return <Splash version={settings.version} />;

  return <SettingsProvider value={settings}>{settings.view === "setup" ? <Setup /> : <Settings />}</SettingsProvider>;
}

export default App;
