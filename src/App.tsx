import { Home, FlaskConical, ShieldCheck, Shield, Sparkles } from "lucide-react";
import { useTorApp } from "@/hooks/useTorApp";
import { ConnectPage } from "@/pages/ConnectPage";
import { AppsPage } from "@/pages/AppsPage";
import { VerifyPage } from "@/pages/VerifyPage";
import { OnionLabPage } from "@/pages/OnionLabPage";
import { SystemPage } from "@/pages/SystemPage";
import { Flash } from "@/components/Flash";
import { SetupWizard } from "@/components/SetupWizard";
import { TooltipProvider } from "@/components/ui/tooltip";
import { OnionIcon } from "@/OnionIcon";
import type { Tab } from "@/lib/types";
import { cn } from "@/lib/utils";
import { effectiveLocale } from "@/lib/i18n";
import { startWindowDrag } from "@/lib/drag";

const NAV: { id: Tab; label: string; icon: typeof Home }[] = [
  { id: "home", label: "Connect", icon: Home },
  { id: "apps", label: "Apps", icon: Sparkles },
  { id: "onion-lab", label: "Onion Lab", icon: FlaskConical },
  { id: "verify", label: "Verify", icon: ShieldCheck },
  { id: "system", label: "System", icon: Shield },
];

export default function App() {
  const app = useTorApp();
  const os = app.detect?.os;
  const locale = effectiveLocale(app.settings?.locale);
  const showWizard = !!app.settings && !app.settings.setup_complete;

  return (
    <TooltipProvider>
      <div
        dir={locale === "fa" ? "rtl" : "ltr"}
        className="flex h-screen min-h-0 w-full overflow-hidden bg-transparent"
      >
        {showWizard ? <SetupWizard app={app} /> : null}
        <aside className="flex w-[200px] shrink-0 flex-col border-r border-line/70 bg-canvas/55 backdrop-blur-[2px]">
          <div
            data-tauri-drag-region
            onMouseDown={startWindowDrag}
            className="flex items-center gap-2.5 px-4 pb-3 pt-14"
          >
            <OnionIcon className="h-7 w-7 text-onion" />
            <div className="min-w-0">
              <div className="truncate text-sm font-semibold tracking-tight">
                OnionGate
              </div>
              <div className="truncate text-[10px] text-muted">
                Tor gateway
              </div>
            </div>
          </div>
          <nav
            className="flex flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-3"
            aria-label="Primary"
          >
            {NAV.map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                type="button"
                onClick={() => app.setTab(id)}
                className={cn(
                  "flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                  app.tab === id
                    ? "bg-panel-2 text-ink"
                    : "text-muted hover:bg-panel-2/60 hover:text-ink",
                )}
              >
                <Icon className="h-4 w-4 shrink-0" strokeWidth={1.8} />
                <span className="flex min-w-0 items-center gap-1.5">
                  <span className="truncate">{label}</span>
                  {id === "system" && (app.status?.persistence_changes ?? 0) > 0 ? (
                    <span className="rounded bg-warn/15 px-1 text-[9px] font-semibold text-warn">
                      {app.status?.persistence_changes}
                    </span>
                  ) : null}
                  {id === "system" && os === "macos" ? (
                    <span
                      className="shrink-0 text-[11px] text-muted"
                      title="macOS"
                      aria-label="macOS"
                    >
                      {"\uF8FF"}
                    </span>
                  ) : null}
                  {id === "system" && os === "linux" ? (
                    <span
                      className="shrink-0 rounded border border-line px-1 text-[9px] font-semibold uppercase tracking-wide text-muted"
                      title="Linux"
                      aria-label="Linux"
                    >
                      Linux
                    </span>
                  ) : null}
                </span>
              </button>
            ))}
          </nav>
          <div className="border-t border-line px-4 py-3">
            <p className="truncate text-[10px] text-muted">
              {app.protectionLabel}
            </p>
          </div>
        </aside>

        <main className="relative min-h-0 min-w-0 flex-1 overflow-y-auto">
          <div
            data-tauri-drag-region
            onMouseDown={startWindowDrag}
            className="absolute inset-x-0 top-0 z-10 h-11"
            aria-hidden
          />
          <Flash
            message={app.message}
            error={app.error}
            onDismiss={app.clearFlash}
          />
          <div className="mx-auto min-h-full w-full max-w-4xl px-6 pb-5 pt-11 animate-[fade-in_280ms_ease-out]">
            {app.tab === "home" ? <ConnectPage app={app} /> : null}
            {app.tab === "apps" ? <AppsPage app={app} /> : null}
            {app.tab === "onion-lab" ? <OnionLabPage app={app} /> : null}
            {app.tab === "verify" ? <VerifyPage app={app} /> : null}
            {app.tab === "system" ? <SystemPage app={app} /> : null}
          </div>
        </main>
      </div>
    </TooltipProvider>
  );
}
