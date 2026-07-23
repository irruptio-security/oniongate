import { useState } from "react";
import { Segmented } from "@/components/ui/segmented";
import { InfoTip } from "@/components/ui/tooltip";
import type { TorApp } from "@/hooks/useTorApp";
import { PRESETS, detectPreset } from "@/lib/presets";
import { HomePage } from "@/pages/HomePage";
import { NetworkPage } from "@/pages/NetworkPage";
import { ScannerPage } from "@/pages/ScannerPage";

export function ConnectPage({ app }: { app: TorApp }) {
  const [view, setView] = useState<"connect" | "network" | "bridges">("connect");
  const settings = app.settings;
  const activePreset = settings ? detectPreset(settings) : null;
  const activeMeta = PRESETS.find((item) => item.id === activePreset);
  const presetLabel = activeMeta?.label ?? "Custom";
  const presetDescription =
    activeMeta?.description ??
    "Your routing settings don't match a named preset. Adjust them under Routing and Bridges.";
  const presetDetail = settings
    ? `${settings.connection_mode === "tun" ? "TUN" : "System proxy"} · DNS ${
        settings.remote_dns ? "via Tor" : "system"
      } · Kill switch ${settings.kill_switch ? "on" : "off"} · Session Guard ${
        settings.session_guard ? "on" : "off"
      }${settings.bridge_source && settings.bridge_source !== "none" ? ` · Bridges ${settings.bridge_source}` : ""}`
    : undefined;

  return (
    <section className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-2">
        <Segmented
          value={view}
          options={[
            { value: "connect", label: "Connect" },
            { value: "network", label: "Routing" },
            { value: "bridges", label: "Bridges" },
          ]}
          onChange={setView}
        />
        <div className="flex shrink-0 items-center gap-1 text-[11px] text-muted">
          <button
            type="button"
            onClick={() => {
              app.setSystemView("settings");
              app.setTab("system");
            }}
            aria-label={`Preset: ${presetLabel}. Change in Settings.`}
            className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 transition-colors hover:bg-panel-2 hover:text-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40"
          >
            <span className="uppercase tracking-wide">Preset</span>
            <span className="font-semibold text-ink">{presetLabel}</span>
          </button>
          <InfoTip
            title={`${presetLabel} preset`}
            description={presetDescription}
            detail={presetDetail}
            action={{
              label: "Change preset in Settings",
              onClick: () => {
                app.setSystemView("settings");
                app.setTab("system");
              },
            }}
          />
        </div>
      </div>
      {view === "connect" ? (
        <HomePage app={app} onOpenBridges={() => setView("bridges")} />
      ) : null}
      {view === "network" ? (
        <NetworkPage app={app} onOpenBridges={() => setView("bridges")} />
      ) : null}
      {view === "bridges" ? <ScannerPage app={app} /> : null}
    </section>
  );
}
