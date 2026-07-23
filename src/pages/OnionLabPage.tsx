import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { QRCodeSVG } from "qrcode.react";
import { Button } from "@/components/ui/button";
import { CopyButton } from "@/components/ui/copy-button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import type { TorApp } from "@/hooks/useTorApp";

type OnionProject = {
  service_id: string;
  hostname: string;
  local_port: number;
  virtual_port: number;
  private: boolean;
  client_credential: string | null;
};

type OnionAudit = {
  published: boolean;
  latency_ms: number | null;
  http_status: number | null;
  security_headers: string[];
  warnings: string[];
  listener: { loopback_only: boolean; detail: string };
};

export function OnionLabPage({ app }: { app: TorApp }) {
  const { busy, run, status } = app;
  const [localPort, setLocalPort] = useState("3000");
  const [virtualPort, setVirtualPort] = useState("80");
  const [isPrivate, setPrivate] = useState(true);
  const [projects, setProjects] = useState<OnionProject[]>([]);
  const [audits, setAudits] = useState<Record<string, OnionAudit>>({});
  const [listError, setListError] = useState<string | null>(null);

  const refresh = () =>
    invoke<OnionProject[]>("list_onion_services")
      .then((list) => {
        setProjects(list);
        setListError(null);
      })
      .catch((error) => {
        setProjects([]);
        setListError(typeof error === "string" ? error : String(error));
      });

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <section className="flex flex-col gap-5">
      <header>
        <h2 className="text-xl font-semibold tracking-tight">Onion Lab</h2>
        <p className="mt-1 text-sm text-muted">
          Expose a loopback-only development server as an ephemeral v3 onion service.
          Keys are discarded when the service stops.
        </p>
      </header>

      <div className="rounded-xl border border-line bg-panel p-4">
        <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
          <label>
            <div className="mb-1 text-xs font-semibold text-muted">Localhost port</div>
            <Input
              type="number"
              min={1}
              max={65535}
              value={localPort}
              disabled={busy}
              onChange={(event) => setLocalPort(event.target.value)}
            />
          </label>
          <label>
            <div className="mb-1 text-xs font-semibold text-muted">Onion port</div>
            <Input
              type="number"
              min={1}
              max={65535}
              value={virtualPort}
              disabled={busy}
              onChange={(event) => setVirtualPort(event.target.value)}
            />
          </label>
          <div className="flex h-9 items-center gap-2">
            <span id="onion-private-label" className="text-xs font-semibold">
              Private
            </span>
            <Switch
              checked={isPrivate}
              disabled={busy}
              aria-labelledby="onion-private-label"
              onCheckedChange={setPrivate}
            />
          </div>
        </div>
        <p className="mt-2 text-xs text-muted">
          Private is recommended and generates a one-time client credential. Wildcard listeners
          such as 0.0.0.0 are rejected.
        </p>
        <Button
          className="mt-3"
          disabled={busy || !status?.control_up}
          onClick={() =>
            void run(async () => {
              const project = await invoke<OnionProject>("start_onion_service", {
                localPort: Number(localPort),
                virtualPort: Number(virtualPort),
                private: isPrivate,
              });
              await refresh();
              return `Created ${project.hostname}`;
            })
          }
        >
          Create ephemeral onion
        </Button>
      </div>

      {listError ? (
        <p className="text-xs text-danger">
          Couldn't list onion services: {listError}
        </p>
      ) : null}

      {projects.length ? (
        <div className="space-y-3">
          {projects.map((project) => {
            const audit = audits[project.service_id];
            const clientSetup = project.client_credential
              ? `${project.hostname}:auth:${project.client_credential}`
              : project.hostname;
            return (
              <div
                key={project.service_id}
                className="rounded-xl border border-line bg-panel p-4"
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="truncate font-mono text-xs">{project.hostname}</div>
                    <div className="mt-1 text-[11px] text-muted">
                      127.0.0.1:{project.local_port} → onion:{project.virtual_port} ·{" "}
                      {project.private ? "client auth required" : "public"}
                    </div>
                  </div>
                  <div className="flex gap-1">
                    <CopyButton
                      value={`http://${project.hostname}`}
                      label="Copy link"
                    />
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={busy}
                      onClick={() =>
                        void run(async () => {
                          const next = await invoke<OnionAudit>("audit_onion_service", {
                            serviceId: project.service_id,
                          });
                          setAudits((current) => ({
                            ...current,
                            [project.service_id]: next,
                          }));
                          return next.published
                            ? "Onion service is published"
                            : "Descriptor is not reachable yet";
                        })
                      }
                    >
                      Test & audit
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={busy}
                      onClick={() =>
                        void run(async () => {
                          const message = await invoke<string>("stop_onion_service", {
                            serviceId: project.service_id,
                          });
                          await refresh();
                          return message;
                        })
                      }
                    >
                      Destroy
                    </Button>
                  </div>
                </div>

                {project.client_credential ? (
                  <div className="mt-3 flex flex-wrap items-center gap-3 rounded-lg border border-warn/30 bg-warn/5 p-3">
                    <QRCodeSVG value={clientSetup} size={92} level="M" />
                    <div className="min-w-0 flex-1">
                      <div className="text-xs font-semibold">One-time client credential</div>
                      <div className="mt-1 break-all font-mono text-[10px] text-muted">
                        {project.client_credential}
                      </div>
                      <CopyButton
                        className="mt-2"
                        variant="secondary"
                        value={clientSetup}
                        label="Copy client setup"
                      />
                    </div>
                  </div>
                ) : null}

                {audit ? (
                  <div className="mt-3 text-xs text-muted">
                    {audit.listener.detail} · {audit.published ? "published" : "not published"}
                    {audit.latency_ms != null ? ` · ${audit.latency_ms} ms` : ""}
                    {audit.http_status != null ? ` · HTTP ${audit.http_status}` : ""}
                    {audit.security_headers.length
                      ? ` · headers: ${audit.security_headers.join(", ")}`
                      : ""}
                    {audit.warnings.length ? ` · ${audit.warnings.join(" · ")}` : ""}
                  </div>
                ) : null}
              </div>
            );
          })}
        </div>
      ) : (
        <p className="text-sm text-muted">No active ephemeral onion projects.</p>
      )}
    </section>
  );
}
