import { Segmented } from "@/components/ui/segmented";
import type { TorApp } from "@/hooks/useTorApp";
import { SettingsPage } from "@/pages/SettingsPage";
import { LogsPage } from "@/pages/LogsPage";

/**
 * The Settings tab covers OnionGate itself. Anything that changes the machine
 * lives under System instead.
 */
export function AppSettingsPage({ app }: { app: TorApp }) {
  const view = app.settingsView;

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-5">
      <Segmented
        value={view}
        options={[
          { value: "preferences", label: "Preferences" },
          { value: "logs", label: "Logs" },
        ]}
        onChange={app.setSettingsView}
      />
      {view === "preferences" ? <SettingsPage app={app} /> : null}
      {view === "logs" ? <LogsPage app={app} /> : null}
    </section>
  );
}
