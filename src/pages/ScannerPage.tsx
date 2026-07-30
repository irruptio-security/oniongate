import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { SettingRow } from "@/components/ui/setting-row";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { InfoTip } from "@/components/ui/tooltip";
import { LoadState } from "@/components/ui/load-state";
import type { TorApp } from "@/hooks/useTorApp";
import type { AppSettings } from "@/lib/types";
import { cn } from "@/lib/utils";

function bridgeLabel(line: string, index: number): string {
  const body = line.replace(/^Bridge\s+/i, "").trim();
  const parts = body.split(/\s+/);
  const transport = parts[0] ?? "bridge";
  const endpoint = parts[1] ?? "";
  const short = endpoint.length > 28 ? `${endpoint.slice(0, 28)}…` : endpoint;
  return short ? `${index + 1}. ${transport} · ${short}` : `${index + 1}. ${transport}`;
}

export function ScannerPage({ app }: { app: TorApp }) {
  const {
    status,
    settings,
    catalogBridges,
    scanResults,
    bridgeText,
    setBridgeText,
    scanTransport,
    setScanTransport,
    busy,
    run,
    refreshSettings,
    fetchBridges,
    saveBridges,
    scanBridges,
    applyReachableBridges,
  } = app;

  const torOn = status?.socks_up ?? false;

  const [showManual, setShowManual] = useState(false);

  const catalog = catalogBridges.length
    ? catalogBridges
    : (settings?.bridge_lines ?? []);

  const selectedSet = useMemo(
    () => new Set(settings?.bridge_lines ?? []),
    [settings?.bridge_lines],
  );

  if (!settings) {
    return (
      <LoadState
        label="Loading bridges…"
        error={app.bootstrapError}
        onRetry={app.retryBootstrap}
      />
    );
  }

  const toggleBridgeLine = (line: string, on: boolean) => {
    void run(async () => {
      const next = on
        ? [...new Set([...settings.bridge_lines, line])]
        : settings.bridge_lines.filter((l) => l !== line);
      await invoke<AppSettings>("set_bridge_lines", {
        text: next.join("\n"),
      });
      await refreshSettings();
      return on ? "Bridge added" : "Bridge removed";
    });
  };

  return (
    <section className="flex flex-col gap-5">
      <header>
        <h2 className="text-xl font-semibold tracking-tight">Bridges</h2>
        <p className="mt-1 text-sm text-muted">
          Use bundled censorship transports or paste lines obtained from the Tor
          Project's BridgeDB. OnionGate never downloads third-party GitHub bridge lists.
        </p>
      </header>

      <div className="space-y-2">
        <SettingRow
          title={
            <span className="inline-flex items-center gap-1.5">
              Use bridges
              <InfoTip
                title="Use bridges"
                status={{
                  label: settings.bridges_enabled ? "On" : "Off",
                  tone: settings.bridges_enabled ? "warn" : "default",
                }}
                description="Loads your selected bridge lines into Tor on Connect. Home only shows this status — it does not control bridges."
                risk="Bridges can reduce anonymity — the bridge operator sees you as a client. Use only if Tor is blocked."
              />
            </span>
          }
          description={
            settings.bridge_lines.length
              ? `${settings.bridge_lines.length} bridge line(s) selected`
              : "Select or paste bridge lines below first"
          }
        >
          <Switch
            checked={settings.bridges_enabled}
            disabled={busy || settings.bridge_lines.length === 0}
            onCheckedChange={(enabled) =>
              void run(async () => {
                const saved = await invoke<AppSettings>("set_bridges_enabled", {
                  enabled,
                });
                await refreshSettings();
                if (!enabled) {
                  if (torOn) {
                    await invoke<string>("apply_tor_config");
                    return "Bridges off · Tor restarted";
                  }
                  return "Bridges off";
                }
                if (!saved.bridges_enabled) {
                  throw "Could not enable bridges — add at least one bridge line first.";
                }
                if (torOn) {
                  await invoke<string>("apply_tor_config");
                  return "Bridges on · Tor restarted";
                }
                return "Bridges on — connect Tor to use them";
              })
            }
          />
        </SettingRow>

        <SettingRow
          title={
            <span className="inline-flex items-center gap-1.5">
              Bridge source
              <InfoTip
                title="Bridge source"
                status={{
                  label: settings.bridge_source || "none",
                  tone:
                    (settings.bridge_source || "none") === "none"
                      ? "default"
                      : "accent",
                }}
                description="Built-in Snowflake, meek, and conjure lines ship with OnionGate. Obtain obfs4/webtunnel lines directly from the Tor Project's BridgeDB."
                detail="Smart Connect can record builtin:snowflake here after selecting its fallback."
              />
            </span>
          }
          description="Controls catalogs and Use bridges; Smart Connect may record its fallback"
        >
          <Select
            className="h-9 w-52"
            value={settings.bridge_source || "none"}
            disabled={busy}
            onChange={(e) => {
              const bridge_source = e.target.value;
              void run(async () => {
                const next = { ...settings, bridge_source };
                if (bridge_source === "none") {
                  next.bridges_enabled = false;
                } else if (
                  bridge_source === "custom" &&
                  settings.bridge_lines.length > 0
                ) {
                  // leave bridges_enabled as-is; user uses Use bridges toggle
                }
                const saved = await invoke<AppSettings>("update_settings", {
                  next,
                });
                await refreshSettings();
                if (
                  bridge_source.startsWith("transport:") ||
                  bridge_source === "auto"
                ) {
                  const t =
                    bridge_source === "auto"
                      ? "all"
                      : bridge_source.replace("transport:", "");
                  setScanTransport(t === "meek" ? "meek" : t);
                }
                return `Bridge source: ${saved.bridge_source}`;
              });
            }}
          >
            <option value="none">None (recommended)</option>
            <option value="auto">Built-in censorship fallback</option>
            <option value="builtin:snowflake">Smart Connect Snowflake fallback</option>
            <option value="transport:snowflake">Built-in Snowflake</option>
            <option value="transport:meek">Built-in meek</option>
            <option value="transport:conjure">Built-in conjure</option>
            <option value="custom">Custom list</option>
          </Select>
        </SettingRow>
      </div>

      <div className="flex flex-wrap items-end gap-2 rounded-xl border border-line bg-panel p-4">
        <label className="min-w-[10rem] flex-1">
          <div className="mb-1 flex items-center gap-1 text-xs font-semibold text-muted">
            Transport
            <InfoTip
              title="Transport"
              description="Pooled transports use TCP probes. Fronted ones (snowflake / meek / conjure) probe the broker or CDN front on :443."
            />
          </div>
          <Select
            value={scanTransport}
            disabled={busy}
            onChange={(e) => setScanTransport(e.target.value)}
          >
            <option value="all">All (combined)</option>
            <option value="obfs4">obfs4</option>
            <option value="webtunnel">webtunnel</option>
            <option value="snowflake">snowflake</option>
            <option value="meek">meek</option>
            <option value="conjure">conjure</option>
            <option value="vanilla">vanilla</option>
          </Select>
        </label>
        <Button
          size="sm"
          variant="secondary"
          disabled={busy}
          onClick={() => fetchBridges(scanTransport)}
        >
          Load trusted built-ins
        </Button>
        <Button
          size="sm"
          variant="secondary"
          disabled={busy || !catalog.length}
          onClick={() => scanBridges(catalog)}
        >
          Scan catalog
        </Button>
        <Button
          size="sm"
          variant="secondary"
          disabled={busy || !scanResults?.some((r) => r.ok)}
          onClick={applyReachableBridges}
        >
          Apply reachable
        </Button>
      </div>

      <div className="rounded-xl border border-line bg-panel p-4 space-y-2">
        <div className="flex items-center justify-between gap-2">
          <div className="text-sm font-semibold">
            Catalog ({catalog.length})
          </div>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            onClick={() =>
              void run(async () => invoke<string>("apply_tor_config"))
            }
          >
            Restart Tor
          </Button>
        </div>
        {catalog.length ? (
          <div className="max-h-48 space-y-1 overflow-auto">
            {catalog.slice(0, 80).map((line, i) => (
              <label
                key={`${line}-${i}`}
                className="flex items-center gap-2 rounded-md border border-line bg-canvas px-2 py-1.5 text-xs"
              >
                <input
                  type="checkbox"
                  className="accent-[var(--accent)]"
                  checked={selectedSet.has(line)}
                  disabled={busy}
                  onChange={(e) => toggleBridgeLine(line, e.target.checked)}
                />
                <span className="min-w-0 truncate font-mono text-[11px]">
                  {bridgeLabel(line, i)}
                </span>
              </label>
            ))}
          </div>
        ) : (
          <p className="text-xs text-muted">
            Load a trusted built-in transport or paste BridgeDB lines below.
          </p>
        )}
      </div>

      {scanResults ? (
        <div className="rounded-xl border border-line bg-panel p-4 space-y-2">
          <div className="text-sm font-semibold">Scan results</div>
          <div className="max-h-48 space-y-1 overflow-auto">
            {scanResults.map((r) => (
              <div
                key={r.raw}
                className={cn(
                  "grid grid-cols-[0.7fr_1.2fr_0.7fr] gap-2 rounded-md border px-2 py-1.5 font-mono text-[10px]",
                  r.ok
                    ? "border-accent/30 bg-canvas"
                    : "border-danger/25 text-muted",
                )}
              >
                <span>{r.transport}</span>
                <span className="truncate">{r.endpoint ?? "—"}</span>
                <span className="text-right">
                  {r.ok ? `${r.latency_ms ?? "?"}ms` : (r.error ?? "fail")}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : null}

      <div className="rounded-xl border border-line bg-panel p-4">
        <div className="text-sm font-semibold">Active transports</div>
        <div className="mt-2 grid grid-cols-2 gap-1.5 sm:grid-cols-3">
          {(status?.pt ?? []).map((p) => (
            <div
              key={p.transport}
              className={cn(
                "rounded-md border px-2 py-1.5 text-[11px]",
                p.available
                  ? "border-accent/30 text-ink"
                  : "border-line text-muted",
              )}
            >
              <div className="font-semibold">{p.transport}</div>
              <div className="truncate font-mono text-[10px]">
                {p.available ? "ready" : "missing"}
              </div>
            </div>
          ))}
        </div>
      </div>

      <button
        type="button"
        className="text-xs text-muted underline-offset-2 hover:underline self-start"
        onClick={() => setShowManual((v) => !v)}
      >
        {showManual ? "Hide manual paste" : "Paste bridges manually…"}
      </button>
      {showManual ? (
        <div className="space-y-2">
          <Textarea
            value={bridgeText}
            disabled={busy}
            placeholder="Bridge obfs4 …"
            onChange={(e) => setBridgeText(e.target.value)}
          />
          <Button size="sm" variant="secondary" disabled={busy} onClick={saveBridges}>
            Save pasted lines
          </Button>
        </div>
      ) : null}

    </section>
  );
}
