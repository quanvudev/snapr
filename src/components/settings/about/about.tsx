import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { APP_INFO } from "@/lib/app-info";
import icon from "../../../../src-tauri/icons/128x128.png";
import { SettingsCard } from "../settings-card";
import { HymtNotice } from "./hymt-notice";
import { UpdateRow } from "./update-row";

export function About() {
  const { t, version } = useSettingsContext();
  return (
    <>
      <div className="flex items-center gap-4">
        <img src={icon} alt="" className="size-16" />
        <div>
          <div className="text-lg font-semibold">{APP_INFO.name}</div>
          <div className="text-xs text-muted-foreground">{version}</div>
        </div>
      </div>
      <SettingsCard>
        <UpdateRow />
        <Item variant="muted">
          <ItemContent>
            <ItemTitle>{t.author}</ItemTitle>
          </ItemContent>
          <ItemActions>{APP_INFO.author}</ItemActions>
        </Item>
        {APP_INFO.links.map(({ label, text, url }) => (
          <Item key={label} variant="muted">
            <ItemContent>
              <ItemTitle>{label}</ItemTitle>
            </ItemContent>
            <ItemActions>
              <Button variant="link" onClick={() => api.openUrl(url).catch(() => {})}>
                {text}
              </Button>
            </ItemActions>
          </Item>
        ))}
        <Item variant="muted">
          <ItemContent>
            <ItemTitle>{t.config}</ItemTitle>
          </ItemContent>
          <ItemActions>
            <Button variant="outline" onClick={() => api.openConfigFile()}>
              {t.open}
            </Button>
          </ItemActions>
        </Item>
        <HymtNotice />
      </SettingsCard>
    </>
  );
}
