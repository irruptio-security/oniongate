import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "@/components/ui/button";
import { Segmented } from "@/components/ui/segmented";
import type { TorApp } from "@/hooks/useTorApp";
import { HardenPage } from "@/pages/HardenPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { LogsPage } from "@/pages/LogsPage";
import { cn } from "@/lib/utils";

type PostureCheck = {
  id: string;
  title: string;
  status: "pass" | "warn" | "info";
  detail: string;
  source: string;
  remediation: string | null;
};

type PersistenceEntry = {
  id: string;
  kind: string;
  path: string;
  signed: boolean | null;
  team_id: string | null;
};

type PersistenceReport = {
  entries: PersistenceEntry[];
  baseline_exists: boolean;
  added: PersistenceEntry[];
  removed_ids: string[];
};

type ArtifactReport = {
  path: string;
  kind: string;
  sha256: string;
  quarantined: boolean;
  signature_valid: boolean;
  notarized: boolean;
  identifier: string | null;
  team_id: string | null;
  authorities: string[];
  entitlements: string[];
  detail: string;
  reputation_url: string;
};

type HostTool = {
  id: string;
  name: string;
  installed: boolean;
  official_url: string;
  purpose: string;
};

type LoginItemsSnapshot = {
  available: boolean;
  items: string[];
  detail: string;
};

export function SystemPage({ app }: { app: TorApp }) {
  const { busy, run } = app;
  const view = app.systemView;
  const setView = app.setSystemView;
  const [posture, setPosture] = useState<PostureCheck[] | null>(null);
  const [postureError, setPostureError] = useState<string | null>(null);
  const [persistence, setPersistence] = useState<PersistenceReport | null>(null);
  const [persistenceError, setPersistenceError] = useState<string | null>(null);
  const [artifact, setArtifact] = useState<ArtifactReport | null>(null);
  const [tools, setTools] = useState<HostTool[]>([]);
  const [loginItems, setLoginItems] = useState<LoginItemsSnapshot | null>(null);

  const refreshPosture = async () => {
    try {
      const [checks, hostTools] = await Promise.all([
        invoke<PostureCheck[]>("get_workstation_posture"),
        invoke<HostTool[]>("get_host_security_tools"),
      ]);
      setPosture(checks);
      setTools(hostTools);
      setPostureError(null);
    } catch (error) {
      setPostureError(typeof error === "string" ? error : String(error));
    }
  };

  const refreshPersistence = async () => {
    try {
      setPersistence(await invoke<PersistenceReport>("get_persistence_report"));
      setPersistenceError(null);
    } catch (error) {
      setPersistenceError(typeof error === "string" ? error : String(error));
    }
  };

  useEffect(() => {
    void refreshPosture();
  }, []);

  if (view === "harden") {
    return (
      <section className="flex flex-col gap-4">
        <Segmented
          value={view}
          options={[
            { value: "audit", label: "Posture" },
            { value: "persistence", label: "Persistence" },
            { value: "artifact", label: "Artifact Inspector" },
            { value: "harden", label: "Optional Harden" },
            { value: "settings", label: "Settings" },
            { value: "logs", label: "Logs" },
          ]}
          onChange={setView}
        />
        <HardenPage app={app} />
      </section>
    );
  }

  return (
    <section className="flex flex-col gap-5">
      <header>
        <h2 className="text-xl font-semibold tracking-tight">System</h2>
        <p className="mt-1 text-sm text-muted">
          Read-only macOS posture and change detection. Unknown items are not labeled as malware.
        </p>
      </header>
      <Segmented
        value={view}
        options={[
          { value: "audit", label: "Posture" },
          { value: "persistence", label: "Persistence" },
          { value: "artifact", label: "Artifact Inspector" },
          { value: "harden", label: "Optional Harden" },
          { value: "settings", label: "Settings" },
          { value: "logs", label: "Logs" },
        ]}
        onChange={(next) => {
          setView(next);
          if (next === "persistence") void refreshPersistence();
        }}
      />

      {view === "audit" ? (
        <>
          {postureError ? (
            <div className="flex flex-col items-start gap-2 rounded-xl border border-line bg-panel px-4 py-3">
              <div className="text-sm font-semibold">Couldn't read posture</div>
              <p className="text-xs text-muted">{postureError}</p>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => void refreshPosture()}
              >
                Retry
              </Button>
            </div>
          ) : posture === null ? (
            <p className="text-sm text-muted">Reading system posture…</p>
          ) : posture.length === 0 ? (
            <p className="text-sm text-muted">
              No posture checks are available on this platform.
            </p>
          ) : null}
          <div className="space-y-1.5">
            {(posture ?? []).map((item) => (
              <div
                key={item.id}
                className="flex items-start justify-between gap-3 rounded-xl border border-line bg-panel px-3.5 py-2.5"
              >
                <div>
                  <div className="text-sm font-semibold">{item.title}</div>
                  <div className="text-xs text-muted">{item.detail}</div>
                  {item.remediation ? (
                    <div className="mt-1 text-[10px] text-muted">{item.remediation}</div>
                  ) : null}
                </div>
                <span
                  className={cn(
                    "rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase",
                    item.status === "pass" && "bg-accent/15 text-accent-strong",
                    item.status === "warn" && "bg-warn/15 text-warn-strong",
                    item.status === "info" && "bg-panel-2 text-muted",
                  )}
                  title={item.source}
                >
                  {item.status}
                </span>
              </div>
            ))}
          </div>
          <div className="rounded-xl border border-line bg-panel p-4">
            <div className="text-sm font-semibold">Objective-See integrations</div>
            <p className="mt-0.5 text-xs text-muted">
              OnionGate detects and recommends these official tools; it does not bundle or duplicate
              them.
            </p>
            <div className="mt-2 grid gap-2 sm:grid-cols-2">
              {tools.map((tool) => (
                <button
                  key={tool.id}
                  type="button"
                  className="rounded-lg border border-line p-2 text-left"
                  onClick={() => void openUrl(tool.official_url)}
                >
                  <div className="text-xs font-semibold">
                    {tool.name} · {tool.installed ? "installed" : "not detected"}
                  </div>
                  <div className="text-[10px] text-muted">{tool.purpose}</div>
                </button>
              ))}
            </div>
          </div>
        </>
      ) : null}

      {view === "persistence" ? (
        <div className="space-y-3">
          <div className="flex gap-2">
            <Button
              variant="secondary"
              disabled={busy}
              onClick={() => void run(async () => (await refreshPersistence(), "Inventory refreshed"))}
            >
              Refresh inventory
            </Button>
            <Button
              disabled={busy}
              onClick={() =>
                void run(async () => {
                  const message = await invoke<string>("save_persistence_baseline");
                  await refreshPersistence();
                  return message;
                })
              }
            >
              Save trusted baseline
            </Button>
          </div>
          {persistenceError ? (
            <p className="text-xs text-danger">{persistenceError}</p>
          ) : !persistence ? (
            <p className="text-xs text-muted">
              Refresh the inventory to list LaunchAgents, daemons, login items,
              and other persistence points.
            </p>
          ) : null}
          {persistence ? (
            <>
              <p className="text-xs text-muted">
                {persistence.entries.length} entries · {persistence.added.length} added ·{" "}
                {persistence.removed_ids.length} removed since baseline
              </p>
              <div className="max-h-[28rem] space-y-1 overflow-auto">
                {persistence.entries.map((entry) => (
                  <div key={entry.id} className="rounded-lg border border-line bg-panel px-3 py-2">
                    <div className="text-xs font-semibold">{entry.kind}</div>
                    <div className="truncate font-mono text-[10px] text-muted">{entry.path}</div>
                    {entry.signed != null ? (
                      <div className="text-[10px] text-muted">
                        {entry.signed ? "Signature valid" : "Unsigned or invalid signature"}
                        {entry.team_id ? ` · Team ${entry.team_id}` : ""}
                      </div>
                    ) : null}
                  </div>
                ))}
              </div>
            </>
          ) : null}

          <div className="rounded-xl border border-line bg-panel p-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="min-w-0">
                <div className="text-sm font-semibold">Background / Login Items</div>
                <p className="text-[11px] text-muted">
                  Scanned only when you ask. Needs Full Disk Access and is kept out of the automatic
                  baseline so it never prompts repeatedly.
                </p>
              </div>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={busy}
                  onClick={() =>
                    void run(async () => {
                      const snapshot = await invoke<LoginItemsSnapshot>("scan_login_items");
                      setLoginItems(snapshot);
                      return snapshot.detail;
                    })
                  }
                >
                  Scan
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy}
                  onClick={() =>
                    void run(async () =>
                      invoke<string>("open_full_disk_access_settings"),
                    )
                  }
                >
                  Full Disk Access
                </Button>
              </div>
            </div>
            {loginItems ? (
              loginItems.available ? (
                <ul className="mt-2 max-h-48 space-y-1 overflow-auto text-[11px] text-muted">
                  {loginItems.items.length ? (
                    loginItems.items.map((name, index) => (
                      <li key={`${name}-${index}`} className="truncate">
                        {name}
                      </li>
                    ))
                  ) : (
                    <li>No background/login items reported.</li>
                  )}
                </ul>
              ) : (
                <p className="mt-2 text-[11px] text-warn">{loginItems.detail}</p>
              )
            ) : null}
          </div>
        </div>
      ) : null}

      {view === "artifact" ? (
        <div className="space-y-3">
          <Button
            disabled={busy}
            onClick={() =>
              void run(async () => {
                const path = await open({
                  title: "Inspect an app, DMG, PKG, or Mach-O",
                  multiple: false,
                  directory: false,
                });
                if (!path) return "Inspection cancelled";
                const result = await invoke<ArtifactReport>("inspect_artifact", { path });
                setArtifact(result);
                return "Artifact inspection complete";
              })
            }
          >
            Choose artifact
          </Button>
          {artifact ? (
            <div className="rounded-xl border border-line bg-panel p-4 text-xs">
              <div className="font-semibold">{artifact.path}</div>
              <div className="mt-2 grid gap-1 text-muted">
                <div>SHA-256: <span className="break-all font-mono">{artifact.sha256}</span></div>
                <div>Quarantine: {artifact.quarantined ? "present" : "absent"}</div>
                <div>Signature: {artifact.signature_valid ? "valid" : "invalid or unsigned"}</div>
                <div>Notarization assessment: {artifact.notarized ? "accepted" : "not accepted"}</div>
                <div>Identifier: {artifact.identifier ?? "unknown"}</div>
                <div>Team ID: {artifact.team_id ?? "unknown"}</div>
                <div>Entitlements: {artifact.entitlements.join(", ") || "none reported"}</div>
              </div>
              <Button
                className="mt-3"
                size="sm"
                variant="secondary"
                onClick={() => void openUrl(artifact.reputation_url)}
              >
                Check hash reputation
              </Button>
            </div>
          ) : null}
        </div>
      ) : null}
      {view === "settings" ? <SettingsPage app={app} /> : null}
      {view === "logs" ? <LogsPage app={app} /> : null}
    </section>
  );
}
