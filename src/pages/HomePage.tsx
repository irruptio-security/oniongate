import { useEffect, useState } from "react";
import { Loader2, RefreshCw, UserRound } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { OnionIcon } from "@/OnionIcon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { InfoTip } from "@/components/ui/tooltip";
import type { TorApp } from "@/hooks/useTorApp";
import type { AppSettings } from "@/lib/types";
import { cn } from "@/lib/utils";
import { effectiveLocale, translate } from "@/lib/i18n";

type VpnStatus = {
  active: boolean;
  detail: string;
  warning: string;
};

type RecoveryStatus = {
  needed: boolean;
  phase: "disconnected" | "connecting" | "protected" | "degraded" | "recovering";
  detail: string;
};

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatRate(bps: number): string {
  if (bps < 1024) return `${Math.round(bps)} B/s`;
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
}

function formatUptime(secs: number): string {
  if (secs <= 0) return "Idle";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function HomePage({
  app,
  onOpenBridges,
}: {
  app: TorApp;
  onOpenBridges?: () => void;
}) {
  const {
    status,
    ips,
    settings,
    session,
    exitCountries,
    busy,
    torOn,
    proxyOn,
    protectionLabel,
    toggleTor,
    toggleProxy,
    newIdentity,
    refreshIps,
    saveSettings,
    run,
    refreshSettings,
  } = app;

  const [vpn, setVpn] = useState<VpnStatus | null>(null);
  const [recovery, setRecovery] = useState<RecoveryStatus | null>(null);

  useEffect(() => {
    void invoke<VpnStatus>("detect_vpn")
      .then(setVpn)
      .catch(() => setVpn(null));
  }, [torOn, busy]);

  useEffect(() => {
    void invoke<RecoveryStatus>("get_recovery_status")
      .then(setRecovery)
      .catch(() => setRecovery(null));
  }, [torOn, busy]);

  const badgeVariant = protectionLabel.startsWith("Protected")
    ? "success"
    : protectionLabel === "Disconnected"
      ? "default"
      : "warn";
  const locale = effectiveLocale(settings?.locale);
  const t = (key: Parameters<typeof translate>[1]) => translate(locale, key);

  const mode = settings?.connection_mode === "tun" ? "tun" : "proxy";
  const bridgesOn = settings?.bridges_enabled ?? false;
  const bridgeCount = settings?.bridge_lines?.length ?? 0;
  const bridgeSummary = bridgesOn
    ? `Bridges · ${bridgeCount} line${bridgeCount === 1 ? "" : "s"}`
    : bridgeCount > 0
      ? `Off · ${bridgeCount} saved`
      : "Off";

  return (
    <section className="flex flex-col gap-3">
      {recovery?.needed ? (
        <div className="flex items-center justify-between gap-3 rounded-xl border border-warn/50 bg-warn/10 px-4 py-3">
          <div>
            <div className="text-sm font-semibold">{t("interrupted")}</div>
            <div className="text-xs text-muted">{recovery.detail}</div>
          </div>
          <Button
            variant="secondary"
            disabled={busy}
            onClick={() =>
              void run(async () => {
                const message = await invoke<string>("emergency_restore");
                setRecovery(await invoke<RecoveryStatus>("get_recovery_status"));
                return message;
              })
            }
          >
            {t("restore")}
          </Button>
        </div>
      ) : null}

      <div className="grid gap-4 sm:grid-cols-[1fr_1.1fr] sm:items-center">
        <div className="flex flex-col items-center text-center">
          <div className="inline-flex items-center gap-1.5">
            <Badge variant={badgeVariant as "default" | "success" | "warn"}>
              {protectionLabel}
            </Badge>
            {vpn?.active ? (
              <InfoTip
                className="text-warn hover:text-warn"
                title="VPN looks active"
                status={{ label: "Detected", tone: "warn" }}
                detail={vpn.detail}
                risk={vpn.warning}
              />
            ) : null}
          </div>
          <div className="relative mt-3 flex h-44 w-44 items-center justify-center">
            <div
              className={cn(
                "pointer-events-none absolute -inset-3 rounded-full blur-3xl transition-all duration-500",
                torOn ? "bg-accent/30" : "bg-onion/15",
              )}
            />
            <button
              type="button"
              disabled={busy || !status?.tor_installed}
              onClick={toggleTor}
              aria-label={torOn ? t("disconnectAction") : t("connectAction")}
              className={cn(
                "relative z-10 flex h-40 w-40 items-center justify-center rounded-full border-2 transition-all",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-onion/50",
                torOn
                  ? "border-accent bg-gradient-to-b from-accent/20 to-panel shadow-[0_0_50px_-4px_rgba(16,185,129,0.45)]"
                  : "border-line bg-panel hover:border-onion/60 hover:shadow-[0_0_44px_-8px_rgba(124,58,237,0.4)]",
                status?.tor_installed &&
                  !busy &&
                  "cursor-pointer hover:scale-[1.02] active:scale-[0.98]",
                busy && "animate-pulse",
                (!status?.tor_installed || busy) && "opacity-60",
              )}
            >
              {busy ? (
                <div className="flex flex-col items-center gap-2">
                  <Loader2 className="h-9 w-9 animate-spin text-onion" />
                  <span className="text-[11px] font-semibold uppercase tracking-wide text-muted">
                    {t("working")}
                  </span>
                </div>
              ) : (
                <div className="flex flex-col items-center gap-1.5">
                  <OnionIcon
                    className={cn(
                      "h-12 w-12 transition-colors",
                      torOn ? "text-accent" : "text-onion",
                    )}
                  />
                  <span
                    className={cn(
                      "text-xs font-semibold transition-colors",
                      torOn ? "text-accent" : "text-onion",
                    )}
                  >
                    {torOn ? t("disconnectAction") : t("connectAction")}
                  </span>
                </div>
              )}
            </button>
          </div>
          <p
            className={cn(
              "mt-4 text-[13px] font-medium",
              torOn ? "text-accent-strong" : "text-muted",
            )}
          >
            {!status?.tor_installed
              ? (status?.install_hint ?? "Tor runtime missing")
              : busy
                ? t("working")
                : torOn
                  ? t("disconnectHint")
                  : t("connectHint")}
          </p>
          {status?.bootstrap_progress != null && torOn ? (
            <p className="mt-1 text-xs text-muted">
              Bootstrap {status.bootstrap_progress}%
            </p>
          ) : null}
        </div>

        <div className="flex flex-col gap-3">
          <div className="rounded-xl border border-line bg-panel px-4 py-3">
            <div className="text-[11px] font-semibold uppercase tracking-wide text-muted">
              Current IP
            </div>
            <div className="mt-1 truncate font-mono text-sm">
              {ips?.tor_ip ?? ips?.direct_ip ?? "…"}
            </div>
            <div className="mt-0.5 truncate text-[11px] text-muted">
              {ips?.tor_location?.label ??
                ips?.direct_location?.label ??
                (torOn ? "Via Tor" : "Direct")}
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button
              variant="secondary"
              disabled={busy}
              onClick={() =>
                run(async () => {
                  await refreshIps();
                  return "IPs refreshed";
                })
              }
            >
              <RefreshCw className="h-4 w-4" />
              Refresh IP
            </Button>
            <Button
              variant="secondary"
              disabled={busy || !torOn}
              onClick={newIdentity}
            >
              <UserRound className="h-4 w-4" />
              New identity
            </Button>
            <Button
              variant="ghost"
              disabled={
                busy ||
                !torOn ||
                !status?.proxy.supported ||
                status?.connection_mode === "tun"
              }
              onClick={toggleProxy}
            >
              System proxy: {proxyOn ? "ON" : "OFF"}
            </Button>
          </div>

          <div className="rounded-xl border border-line bg-panel p-3">
            <div className="flex items-center justify-between gap-2">
              <div>
                <div className="text-[10px] font-semibold uppercase tracking-wide text-muted">
                  Download
                </div>
                <div className="text-base font-semibold tabular-nums">
                  {formatRate(session?.rate_down_bps ?? 0)}
                </div>
              </div>
              <div className="text-right">
                <div className="text-[10px] font-semibold uppercase tracking-wide text-muted">
                  Upload
                </div>
                <div className="text-base font-semibold tabular-nums">
                  {formatRate(session?.rate_up_bps ?? 0)}
                </div>
              </div>
            </div>
            <div className="mt-2 h-16 overflow-hidden rounded-lg border border-line bg-canvas">
              <div
                className={cn(
                  "h-full w-full bg-gradient-to-r from-accent/10 via-accent/30 to-transparent transition-opacity",
                  torOn ? "opacity-100" : "opacity-30",
                )}
                style={{
                  clipPath: `polygon(0 70%, 15% ${70 - Math.min(40, (session?.rate_down_bps ?? 0) / 500)}%, 35% 55%, 55% ${60 - Math.min(35, (session?.rate_up_bps ?? 0) / 500)}%, 75% 50%, 100% 65%, 100% 100%, 0 100%)`,
                }}
              />
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-line bg-panel p-3 space-y-2.5">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-1.5">
            <div id="home-smart-connect-label" className="text-sm font-semibold">
              Smart Connect
            </div>
            <InfoTip
              title="Smart Connect"
              status={{
                label: settings?.smart_connect ? "On" : "Off",
                tone: settings?.smart_connect ? "ok" : "default",
              }}
              description="Tries direct Tor, your BridgeDB lines, then the bundled Snowflake transport."
              detail={settings?.last_connect_reason || "Each attempt and selection reason is recorded locally."}
            />
          </div>
          <Switch
            checked={settings?.smart_connect ?? true}
            disabled={busy || !settings}
            aria-labelledby="home-smart-connect-label"
            onCheckedChange={(v) => saveSettings({ smart_connect: v })}
          />
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
          <label>
            <div className="mb-1 text-xs font-semibold text-muted">Mode</div>
            <Select
              value={mode}
              disabled={busy}
              onChange={(e) => {
                const next = e.target.value;
                void run(async () => {
                  if (next === "proxy") {
                    if (status?.tun.running) await invoke<string>("stop_tun");
                    else
                      await invoke<AppSettings>("set_connection_mode", {
                        mode: "proxy",
                      });
                    await refreshSettings();
                    return "Proxy mode";
                  }
                  if (!status?.socks_up) {
                    throw "Connect Tor first, then enable TUN.";
                  }
                  const msg = await invoke<string>("start_tun");
                  await refreshSettings();
                  return msg;
                });
              }}
            >
              <option value="proxy">Proxy mode</option>
              <option value="tun">TUN mode</option>
            </Select>
          </label>
          <label>
            <div className="mb-1 text-xs font-semibold text-muted">
              Exit location
            </div>
            <Select
              value={settings?.exit_country ?? ""}
              disabled={busy}
              onChange={(e) => app.applyExitCountry(e.target.value)}
            >
              {(exitCountries.length
                ? exitCountries
                : [{ code: "", label: "Automatic" }]
              ).map((opt) => (
                <option key={opt.code || "any"} value={opt.code}>
                  {opt.label === "Any exit" ? "Automatic" : opt.label}
                </option>
              ))}
            </Select>
          </label>
        </div>
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-line bg-canvas px-3 py-2.5">
          <div className="min-w-0">
            <div className="flex items-center gap-1 text-xs font-semibold text-muted">
              Bridges
              <InfoTip
                className={bridgesOn ? "text-warn hover:text-warn" : undefined}
                title="Bridges"
                status={{
                  label: bridgesOn ? "On" : "Off",
                  tone: bridgesOn ? "warn" : "default",
                }}
                description="Read-only here. Configure Use bridges, catalogs, and lines on the Bridges tab."
                risk="Bridges can reduce anonymity — the bridge operator sees you as a client. Use only if Tor is blocked."
              />
            </div>
            <div
              className={cn(
                "mt-0.5 text-sm font-medium",
                bridgesOn ? "text-warn" : "text-ink",
              )}
            >
              {bridgeSummary}
            </div>
          </div>
          <button
            type="button"
            className="shrink-0 text-xs font-semibold text-accent underline-offset-2 hover:underline"
            onClick={onOpenBridges}
          >
            Manage on Bridges →
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <CompactStat
          label="Data"
          value={formatBytes(session?.bytes_total ?? 0)}
        />
        <CompactStat label="Circuits" value={String(session?.circuits ?? 0)} />
        <CompactStat
          label="Identity"
          value={String(session?.identity_changes ?? 0)}
        />
        <CompactStat
          label="Uptime"
          value={formatUptime(session?.uptime_secs ?? 0)}
        />
      </div>
    </section>
  );
}

function CompactStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-line bg-panel px-2.5 py-2 text-center">
      <div className="text-[10px] font-semibold uppercase tracking-wide text-muted">
        {label}
      </div>
      <div className="mt-0.5 text-sm font-semibold tabular-nums tracking-tight">
        {value}
      </div>
    </div>
  );
}
