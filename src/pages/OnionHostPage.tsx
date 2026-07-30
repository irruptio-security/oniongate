import { useEffect, useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import { Button } from "@/components/ui/button";
import { CopyButton } from "@/components/ui/copy-button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import type { TorApp } from "@/hooks/useTorApp";
import type { OnionAudit, OnionProject, PermanentSite } from "@/lib/types";

function AuditLine({ audit }: { audit: OnionAudit }) {
  return (
    <div className="mt-3 text-xs text-muted">
      {audit.listener.detail} ·{" "}
      {audit.published ? "published" : "not published"}
      {audit.latency_ms != null ? ` · ${audit.latency_ms} ms` : ""}
      {audit.http_status != null ? ` · HTTP ${audit.http_status}` : ""}
      {audit.security_headers.length
        ? ` · headers: ${audit.security_headers.join(", ")}`
        : ""}
      {audit.warnings.length ? ` · ${audit.warnings.join(" · ")}` : ""}
    </div>
  );
}

function TemporaryCard({
  project,
  app,
}: {
  project: OnionProject;
  app: TorApp;
}) {
  const { busy, onionAudits, auditTemporarySite, stopTemporarySite } = app;
  const audit = onionAudits[project.service_id];
  const clientSetup = project.client_credential
    ? `${project.hostname}:auth:${project.client_credential}`
    : project.hostname;

  return (
    <div className="rounded-xl border border-line bg-panel p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="truncate font-mono text-xs">{project.hostname}</div>
          <div className="mt-1 text-[11px] text-muted">
            127.0.0.1:{project.local_port} → onion:{project.virtual_port} ·{" "}
            {project.private ? "client auth required" : "public"}
          </div>
        </div>
        <div className="flex gap-1">
          <CopyButton value={`http://${project.hostname}`} label="Copy link" />
          <Button
            size="sm"
            variant="secondary"
            disabled={busy}
            onClick={() => auditTemporarySite(project.service_id)}
          >
            Test &amp; audit
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            onClick={() => stopTemporarySite(project.service_id)}
          >
            Destroy
          </Button>
        </div>
      </div>

      {project.client_credential ? (
        <div className="mt-3 flex flex-wrap items-center gap-3 rounded-lg border border-warn/30 bg-warn/5 p-3">
          <QRCodeSVG value={clientSetup} size={92} level="M" />
          <div className="min-w-0 flex-1">
            <div className="text-xs font-semibold">
              One-time client credential
            </div>
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

      {audit ? <AuditLine audit={audit} /> : null}
    </div>
  );
}

function PermanentCard({ site, app }: { site: PermanentSite; app: TorApp }) {
  const {
    busy,
    onionAudits,
    auditPermanentSite,
    removePermanentSite,
    addPermanentSiteClient,
    revokePermanentSiteClient,
    setPermanentSiteAuth,
  } = app;
  const [clientName, setClientName] = useState("");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const audit = onionAudits[site.id];

  return (
    <div className="rounded-xl border border-line bg-panel p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="text-sm font-semibold">{site.nickname}</div>
          {site.hostname ? (
            <div className="mt-0.5 truncate font-mono text-xs">
              {site.hostname}
            </div>
          ) : (
            <div className="mt-0.5 text-xs text-muted">
              Tor is still creating this site's address…
            </div>
          )}
          <div className="mt-1 text-[11px] text-muted">
            127.0.0.1:{site.local_port} → onion:{site.virtual_port} ·{" "}
            {site.auth_enabled
              ? `${site.clients.length} authorized client${site.clients.length === 1 ? "" : "s"}`
              : "public"}
          </div>
        </div>
        <div className="flex gap-1">
          {site.hostname ? (
            <CopyButton value={`http://${site.hostname}`} label="Copy link" />
          ) : null}
          <Button
            size="sm"
            variant="secondary"
            disabled={busy || !site.hostname}
            onClick={() => auditPermanentSite(site.id)}
          >
            Test &amp; audit
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            onClick={() => {
              if (confirmDelete) {
                removePermanentSite(site.id);
                setConfirmDelete(false);
              } else {
                setConfirmDelete(true);
              }
            }}
          >
            {confirmDelete ? "Delete for good?" : "Delete"}
          </Button>
        </div>
      </div>

      <div className="mt-3 rounded-lg border border-line bg-panel-2 p-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div>
            <span
              id={`auth-${site.id}`}
              className="text-xs font-semibold"
            >
              Require client authorization
            </span>
            <p className="mt-0.5 text-[11px] text-muted">
              {site.auth_enabled
                ? "Only holders of a credential can reach this site."
                : "Anyone who learns the address can connect."}
            </p>
          </div>
          <Switch
            checked={site.auth_enabled}
            disabled={busy}
            aria-labelledby={`auth-${site.id}`}
            onCheckedChange={(next) => setPermanentSiteAuth(site.id, next)}
          />
        </div>

        {site.clients.length ? (
          <ul className="mt-3 space-y-1">
            {site.clients.map((name) => (
              <li
                key={name}
                className="flex items-center justify-between gap-2 text-xs"
              >
                <span className="truncate font-mono">{name}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy}
                  onClick={() => revokePermanentSiteClient(site.id, name)}
                >
                  Revoke
                </Button>
              </li>
            ))}
          </ul>
        ) : null}

        <div className="mt-3 flex flex-wrap items-end gap-2">
          <label className="min-w-[10rem] flex-1">
            <div className="mb-1 text-[11px] font-semibold text-muted">
              New credential name
            </div>
            <Input
              value={clientName}
              placeholder="alice"
              disabled={busy}
              onChange={(event) => setClientName(event.target.value)}
            />
          </label>
          <Button
            size="sm"
            variant="secondary"
            disabled={busy || !clientName.trim()}
            onClick={() => {
              addPermanentSiteClient(site.id, clientName);
              setClientName("");
            }}
          >
            Issue credential
          </Button>
        </div>
      </div>

      {audit ? <AuditLine audit={audit} /> : null}
    </div>
  );
}

export function OnionHostPage({ app }: { app: TorApp }) {
  const {
    busy,
    status,
    onionProjects,
    permanentSites,
    onionError,
    issuedCredential,
    dismissIssuedCredential,
    refreshOnionHost,
    startTemporarySite,
    addPermanentSite,
  } = app;

  const [permanent, setPermanent] = useState(false);
  const [nickname, setNickname] = useState("");
  const [localPort, setLocalPort] = useState("3000");
  const [virtualPort, setVirtualPort] = useState("80");
  const [isPrivate, setPrivate] = useState(true);

  useEffect(() => {
    void refreshOnionHost();
  }, [refreshOnionHost]);

  const torDown = !status?.control_up;

  return (
    <section className="flex flex-col gap-5">
      <header>
        <h2 className="text-xl font-semibold tracking-tight">Onion Host</h2>
        <p className="mt-1 text-sm text-muted">
          Publish a loopback-only server as a v3 onion site. Temporary sites
          vanish for good when they stop; permanent sites keep the same address
          across restarts.
        </p>
      </header>

      <div className="rounded-xl border border-line bg-panel p-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button
            size="sm"
            variant={permanent ? "ghost" : "secondary"}
            disabled={busy}
            onClick={() => setPermanent(false)}
          >
            Temporary
          </Button>
          <Button
            size="sm"
            variant={permanent ? "secondary" : "ghost"}
            disabled={busy}
            onClick={() => setPermanent(true)}
          >
            Permanent
          </Button>
        </div>

        <p className="mt-2 text-xs text-muted">
          {permanent
            ? "Tor generates and keeps this site's key in its own data directory so the address survives restarts. Deleting the site destroys the key."
            : "Tor discards the key at creation, so this address can never be recreated — by you or anyone else."}
        </p>

        <div className="mt-3 grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
          {permanent ? (
            <label className="sm:col-span-3">
              <div className="mb-1 text-xs font-semibold text-muted">
                Site name
              </div>
              <Input
                value={nickname}
                placeholder="My blog"
                disabled={busy}
                onChange={(event) => setNickname(event.target.value)}
              />
            </label>
          ) : null}
          <label>
            <div className="mb-1 text-xs font-semibold text-muted">
              Localhost port
            </div>
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
            <div className="mb-1 text-xs font-semibold text-muted">
              Onion port
            </div>
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
          {permanent
            ? "Private is recommended. Issue named credentials after the site is created, and revoke them individually. Wildcard listeners such as 0.0.0.0 are rejected."
            : "Private is recommended and generates a one-time client credential. Wildcard listeners such as 0.0.0.0 are rejected."}
        </p>

        <Button
          className="mt-3"
          disabled={busy || torDown || (permanent && !nickname.trim())}
          onClick={() => {
            if (permanent) {
              addPermanentSite(
                nickname,
                Number(localPort),
                Number(virtualPort),
                isPrivate,
              );
              setNickname("");
            } else {
              startTemporarySite(
                Number(localPort),
                Number(virtualPort),
                isPrivate,
              );
            }
          }}
        >
          {permanent ? "Create permanent site" : "Create temporary site"}
        </Button>
        {torDown ? (
          <p className="mt-2 text-xs text-muted">
            Connect Tor on the Connect page first.
          </p>
        ) : null}
      </div>

      {issuedCredential ? (
        <div className="rounded-xl border border-warn/30 bg-warn/5 p-4">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="text-xs font-semibold">
                Credential for "{issuedCredential.client_name}" — shown once
              </div>
              <p className="mt-1 text-[11px] text-muted">
                OnionGate does not store this. If it is lost, revoke the
                credential and issue a new one.
              </p>
            </div>
            <Button size="sm" variant="ghost" onClick={dismissIssuedCredential}>
              Dismiss
            </Button>
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-3">
            <QRCodeSVG
              value={
                issuedCredential.auth_private_line ?? issuedCredential.credential
              }
              size={92}
              level="M"
            />
            <div className="min-w-0 flex-1">
              <div className="break-all font-mono text-[10px] text-muted">
                {issuedCredential.auth_private_line ??
                  issuedCredential.credential}
              </div>
              <CopyButton
                className="mt-2"
                variant="secondary"
                value={
                  issuedCredential.auth_private_line ??
                  issuedCredential.credential
                }
                label="Copy client setup"
              />
            </div>
          </div>
        </div>
      ) : null}

      {onionError ? (
        <p className="text-xs text-danger">
          Couldn't list onion sites: {onionError}
        </p>
      ) : null}

      <div className="space-y-3">
        <h3 className="text-sm font-semibold">Permanent sites</h3>
        {permanentSites.length ? (
          permanentSites.map((site) => (
            <PermanentCard key={site.id} site={site} app={app} />
          ))
        ) : (
          <p className="text-sm text-muted">
            No permanent sites yet. These keep their address across restarts.
          </p>
        )}
      </div>

      <div className="space-y-3">
        <h3 className="text-sm font-semibold">Temporary sites</h3>
        {onionProjects.length ? (
          onionProjects.map((project) => (
            <TemporaryCard
              key={project.service_id}
              project={project}
              app={app}
            />
          ))
        ) : (
          <p className="text-sm text-muted">No temporary sites running.</p>
        )}
      </div>
    </section>
  );
}
