# Hosting your own onion endpoint with Onion Lab

Onion Lab turns a service already running on `127.0.0.1` into a **v3 onion
endpoint** that is reachable over Tor. It is built for development, previews,
and private handoffs — not for production hosting.

Everything Onion Lab creates is **ephemeral**. Tor generates the service key,
hands OnionGate the address, and immediately discards the key (`DiscardPK`).
Nobody — including you — can recreate the same address later. When you destroy
the project or stop Tor, the endpoint is gone for good.

## Before you start

1. **Connect Tor in OnionGate.** Onion Lab talks to the managed Tor control
   port, so the **Create** button stays disabled until Tor is up.
2. **Bind your server to loopback only.** Onion Lab inspects the listener and
   refuses to publish a port that is bound to a wildcard address (`0.0.0.0`,
   `*`, or `::`). A wildcard listener is already reachable from your local
   network, and publishing it as an onion would expose the same service twice.

Common ways to bind to loopback:

```bash
python3 -m http.server 3000 --bind 127.0.0.1
npm run dev -- --host 127.0.0.1 --port 3000
hugo server --bind 127.0.0.1 --port 3000
```

Verify before continuing — this should show `127.0.0.1:3000` and never `*:3000`:

```bash
lsof -nP -iTCP:3000 -sTCP:LISTEN   # macOS
ss -ltn 'sport = :3000'            # Linux
```

## Create the endpoint

On the **Onion Lab** page:

| Field | Meaning |
| --- | --- |
| Localhost port | The port your server already listens on, e.g. `3000`. |
| Onion port | The port visitors use on the onion address. Keep `80` so links work without a port suffix. |
| Private | Requires a client credential to connect. Recommended, and on by default. |

Press **Create ephemeral onion**. You get back a 56-character address ending in
`.onion`, mapping `onion:80` to `127.0.0.1:3000`.

## Public vs private

**Private** (recommended) adds v3 client authorization. Tor generates an x25519
keypair; the public half is registered with the service and the private half is
shown once as a credential that looks like:

```
descriptor:x25519:BASE32SECRET…
```

Without that credential, a visitor cannot even fetch the service descriptor —
the address alone is useless to them. Treat the credential as a password: anyone
holding it can reach your endpoint.

**Public** skips authorization. Anyone who learns the address can connect, so
only use it for content you would be comfortable publishing.

## Give someone access to a private endpoint

Send the address and the credential over a channel you already trust. The QR
code and **Copy client setup** button bundle both.

The visitor then registers the credential with their Tor client:

1. Create a file named `<anything>.auth_private` in their Tor client's
   `onion-auth` directory. For Tor Browser that is the `onion-auth` folder
   inside its Tor data directory (`TorBrowser-Data/Tor/onion-auth` on macOS,
   `Browser/TorBrowser/Data/Tor/onion-auth` on Linux and Windows). For a
   standalone `tor` daemon, point `ClientOnionAuthDir` at a directory in
   `torrc`.
2. Put a single line in it — the address **without** the `.onion` suffix,
   followed by the credential:

```
abcdefghijklmnopqrstuvwxyz234567abcdefghijklmnopqrstuvwx:descriptor:x25519:BASE32SECRET…
```

3. Restart Tor Browser (or reload the tor daemon), then open
   `http://<address>.onion`.

Onion Lab registers the same credential with your own Tor instance when you run
**Test & audit**, so you can verify the endpoint yourself without extra setup.

## Test & audit

**Test & audit** fetches the endpoint back through Tor and reports:

- **Listener scope** — re-checks that the local port is still loopback-only.
- **Published** — whether the descriptor has reached the directory system. A new
  service can take up to a minute; retry if it is not published yet.
- **Latency and HTTP status** — the round trip over Tor.
- **Security headers** — whether your server sends `content-security-policy`,
  `x-content-type-options`, `referrer-policy`, and `permissions-policy`.

Missing headers are a warning, not a failure. They matter because an onion
endpoint is still a website: the usual browser-side protections apply.

## Destroy it

**Destroy** removes the service from Tor, drops the client authorization, and
deletes the stored credential. Because the key was never retained, the address
can never be revived. Quitting OnionGate or stopping Tor has the same effect.

## Limits worth knowing

- **Ephemeral by design.** Every run produces a new address. Onion Lab is not a
  way to run a persistent, brandable onion site.
- **Your server is exposed as-is.** Tor anonymizes the transport; it does not
  patch your application. Directory listings, verbose error pages, debug
  toolbars, and `Server:` banners are all still visible to visitors.
- **Local content only.** If your dev server proxies to other internal hosts,
  those become reachable too. Check what it actually serves first.
- **Not a substitute for the threat model.** Read [THREAT_MODEL.md](../THREAT_MODEL.md)
  before hosting anything sensitive.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| Create button is disabled | Tor is not connected. Connect on the Home page first. |
| "Nothing is accepting TCP connections on 127.0.0.1:*port*" | The server is not running, or is on a different port. |
| "Listener is exposed on a wildcard interface" | Rebind the server to `127.0.0.1` instead of `0.0.0.0`. |
| "Descriptor is not reachable yet" | The service is still publishing. Wait a few seconds and audit again. |
| Visitor sees "Onionsite Authentication Required" | Their credential is missing, malformed, or Tor was not restarted after adding it. |
| Visitor sees "Onionsite Not Found" | The project was destroyed, or Tor was restarted since it was created. |
