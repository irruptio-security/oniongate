import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Segmented } from "@/components/ui/segmented";
import type { TorApp } from "@/hooks/useTorApp";
import { HardenPage } from "@/pages/HardenPage";
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

type LoginItemsSnapshot = {
  available: boolean;
  items: string[];
  detail: string;
};

/**
 * Posture checks that a Harden toggle can actually resolve, so a failing check
 * can hand the user straight to the control instead of describing it.
 */
const HARDEN_FOR_CHECK: Record<string, string> = {
  filevault: "filevault",
  firewall: "app_firewall",
  remote_login: "remote_login",
};

export function SystemPage({ app }: { app: TorApp }) {
  const { busy, run } = app;
  const view = app.systemView;
  const setView = app.setSystemView;
  const [posture, setPosture] = useState<PostureCheck[] | null>(null);
  const [postureError, setPostureError] = useState<string | null>(null);
  const [persistence, setPersistence] = useState<PersistenceReport | null>(null);
  const [persistenceError, setPersistenceError] = useState<string | null>(null);
  const [loginItems, setLoginItems] = useState<LoginItemsSnapshot | null>(null);

  const refreshPosture = async () => {
    try {
      setPosture(await invoke<PostureCheck[]>("get_workstation_posture"));
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

  const openHarden = (hardenId: string) => {
    app.setHardenFocusId(hardenId);
    setView("harden");
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-5">
      <Segmented
        value={view}
        options={[
          { value: "checkup", label: "Checkup" },
          { value: "harden", label: "Harden" },
          { value: "startup", label: "Startup Items" },
        ]}
        onChange={(next) => {
          setView(next);
          if (next === "startup") void refreshPersistence();
        }}
      />

      {view === "checkup" ? (
        <div className="flex flex-col gap-3">
          <header className="flex flex-wrap items-start justify-between gap-2">
            <div className="min-w-0">
              <h2 className="text-xl font-semibold tracking-tight">Checkup</h2>
              <p className="mt-1 text-sm text-muted">
                Read-only look at this machine's security state. Unknown items
                are not labeled as malware.
              </p>
            </div>
            <Button
              size="sm"
              variant="ghost"
              disabled={busy}
              onClick={() =>
                void run(async () => {
                  await refreshPosture();
                  return "Checkup refreshed";
                })
              }
            >
              Re-run
            </Button>
          </header>

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
            {(posture ?? []).map((item) => {
              const hardenId = HARDEN_FOR_CHECK[item.id];
              const fixable = item.status !== "pass" && !!hardenId;
              return (
                <div
                  key={item.id}
                  className="flex items-start justify-between gap-3 rounded-xl border border-line bg-panel px-3.5 py-2.5"
                >
                  <div className="min-w-0">
                    <div className="text-sm font-semibold">{item.title}</div>
                    <div className="text-xs text-muted">{item.detail}</div>
                    {item.remediation ? (
                      <div className="mt-1 text-[10px] text-muted">
                        {item.remediation}
                      </div>
                    ) : null}
                  </div>
                  <div className="flex shrink-0 items-center gap-2">
                    {fixable ? (
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => openHarden(hardenId)}
                      >
                        Fix in Harden
                      </Button>
                    ) : null}
                    <span
                      className={cn(
                        "rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase",
                        item.status === "pass" &&
                          "bg-accent/15 text-accent-strong",
                        item.status === "warn" && "bg-warn/15 text-warn-strong",
                        item.status === "info" && "bg-panel-2 text-muted",
                      )}
                      title={item.source}
                    >
                      {item.status}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ) : null}

      {view === "harden" ? <HardenPage app={app} /> : null}

      {view === "startup" ? (
        <div className="space-y-3">
          <header>
            <h2 className="text-xl font-semibold tracking-tight">
              Startup Items
            </h2>
            <p className="mt-1 text-sm text-muted">
              What this machine runs on its own — LaunchAgents, daemons, and
              login items — and what changed since your trusted baseline.
            </p>
          </header>

          <div className="flex gap-2">
            <Button
              variant="secondary"
              disabled={busy}
              onClick={() =>
                void run(async () => {
                  await refreshPersistence();
                  return "Inventory refreshed";
                })
              }
            >
              Refresh inventory
            </Button>
            <Button
              disabled={busy}
              onClick={() =>
                void run(async () => {
                  const message = await invoke<string>(
                    "save_persistence_baseline",
                  );
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
                {persistence.entries.length} entries · {persistence.added.length}{" "}
                added · {persistence.removed_ids.length} removed since baseline
              </p>
              <div className="max-h-[28rem] space-y-1 overflow-auto">
                {persistence.entries.map((entry) => (
                  <div
                    key={entry.id}
                    className="rounded-lg border border-line bg-panel px-3 py-2"
                  >
                    <div className="text-xs font-semibold">{entry.kind}</div>
                    <div className="truncate font-mono text-[10px] text-muted">
                      {entry.path}
                    </div>
                    {entry.signed != null ? (
                      <div className="text-[10px] text-muted">
                        {entry.signed
                          ? "Signature valid"
                          : "Unsigned or invalid signature"}
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
                <div className="text-sm font-semibold">
                  Background / Login Items
                </div>
                <p className="text-[11px] text-muted">
                  Scanned only when you ask. Needs Full Disk Access and is kept
                  out of the automatic baseline so it never prompts repeatedly.
                </p>
              </div>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={busy}
                  onClick={() =>
                    void run(async () => {
                      const snapshot =
                        await invoke<LoginItemsSnapshot>("scan_login_items");
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
                <p className="mt-2 text-[11px] text-warn">
                  {loginItems.detail}
                </p>
              )
            ) : null}
          </div>
        </div>
      ) : null}
    </section>
  );
}
