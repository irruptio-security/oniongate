# Host an onion site

Onion Host turns a service already running on `127.0.0.1` into a **v3 onion
site** reachable over Tor. No port forwarding, no public IP, no DNS record, and
no certificate authority.

There are two kinds of site, and the difference is entirely about what happens
to the key.

## Temporary or permanent?

| | Temporary | Permanent |
| --- | --- | --- |
| Address after a restart | New every time | Always the same |
| Who holds the key | Nobody — Tor discards it at creation | Tor, in its own data directory |
| Survives process exit | Desktop: no; standalone CLI: until Tor stops | Identity: yes; site is offline while Tor is stopped |
| Best for | Previews, one-off handoffs, testing | A site you want people to bookmark |

**Temporary sites** are created over Tor's control port with the `DiscardPK`
flag, which tells Tor to throw the key away the moment the address exists. That
address can never be recreated — not by you, not by anyone who later compromises
your machine. Stopping the site, stopping Tor, or quitting OnionGate destroys it.

**Permanent sites** need their key to survive, or the address could not come
back. Tor generates and keeps that key in a `HiddenServiceDir` inside its own
data directory, at owner-only permissions. OnionGate never reads, copies, logs,
or exports the key; it only reads the public `hostname` file to learn the
address.

::: warning A permanent key is a real secret
Anyone who can read that directory can impersonate your site: a local attacker
running as your user, an unencrypted disk backup, or a synced folder. Use
full-disk encryption, and don't put the OnionGate data directory in Dropbox or
iCloud. Stopping Tor takes the site offline but does **not** destroy or rotate
its identity — only deleting it does.
:::

## Before you start

**Connect Tor first.** Onion Host talks to the managed Tor control port, so the
create button stays disabled until Tor is up.

**Bind your server to loopback only.** Onion Host inspects the listener and
refuses to publish a port bound to a wildcard address (`0.0.0.0`, `*`, or `::`).
A wildcard listener is already reachable from your local network, and publishing
it as an onion would expose the same service twice.

Common ways to bind to loopback:

```bash
python3 -m http.server 3000 --bind 127.0.0.1
npm run dev -- --host 127.0.0.1 --port 3000
hugo server --bind 127.0.0.1 --port 3000
```

Check before continuing. This should show `127.0.0.1:3000` and never `*:3000`:

```bash
lsof -nP -iTCP:3000 -sTCP:LISTEN   # macOS
ss -ltn 'sport = :3000'            # Linux
```

::: tip A permanent site can be created before its server exists
If nothing is listening yet, creating a permanent site is still allowed — the
site simply won't serve anything until you start the server. The wildcard check
only rejects a listener that is *currently* running and bound too widely, so
re-run **Test & audit** after starting your server to confirm it stayed on
loopback.
:::

## Create a site

On the **Host** page, choose **Temporary** or **Permanent**, then fill in:

| Field | Meaning |
| --- | --- |
| Site name | Permanent only. A label so you can tell sites apart. |
| Localhost port | The port your server already listens on, e.g. `3000`. |
| Onion port | The port visitors use. Keep `80` so links work without a suffix. |
| Private | Requires a client credential to connect. Recommended, on by default. |

You get back a 56-character address ending in `.onion`.

For permanent sites, OnionGate derives an internal ID from the label: lowercase
ASCII letters/numbers, punctuation collapsed to dashes, and a 48-character base
limit. Duplicate IDs receive `-2`, `-3`, and so on. The ID is not the onion
address; the CLI uses it for management. Renaming is not currently exposed, so
choose a useful label.

A new permanent site can take up to a minute to publish its descriptor. Until
then the page shows that Tor is still creating the address.

A new private permanent site starts with a discarded-private-key authorization
lock, so it is not briefly public before you issue the first usable credential.
Issue at least one named client before expecting anyone to connect.

## Client authorization

A **private** site requires v3 client authorization: visitors need a credential
before they can even fetch the service descriptor. Without it, the address alone
is useless to them. A **public** site is reachable by anyone who learns the
address.

The two tiers handle credentials differently.

### Temporary sites

One credential is generated at creation and shown once, alongside a QR code.
It looks like:

```
descriptor:x25519:BASE32SECRET…
```

### Permanent sites

You issue **named** credentials, as many as you need, and revoke them
individually. Give each person their own so that removing one person's access
doesn't disturb anyone else's.

Only the *public* half of each credential is stored, in the site's
`authorized_clients/` directory. The private half is displayed once at issue
time and never written to disk by OnionGate. If someone loses theirs, revoke it
and issue a new one — it cannot be recovered.

OnionGate refuses to revoke the final active credential because an empty
`authorized_clients/` directory would make the site public. Add a replacement
first, or use the explicit authorization-off action if making the site public is
what you intend.

Turning authorization **off** makes the site public to anyone who knows the
address. OnionGate parks the existing client files rather than deleting them, so
turning it back on restores access for everyone who already has a credential.
Credentials issued while authorization is off are also parked; issuing one does
not implicitly make the site private. Use the explicit toggle when ready.

## Give someone access

Send the address and the credential over a channel you already trust. The QR
code and **Copy client setup** button bundle both.

The visitor registers the credential with their Tor client:

1. Create a file named `<anything>.auth_private` in their Tor client's
   `onion-auth` directory. For Tor Browser that is the `onion-auth` folder
   inside its Tor data directory (`TorBrowser-Data/Tor/onion-auth` on macOS,
   `Browser/TorBrowser/Data/Tor/onion-auth` on Linux and Windows). For a
   standalone `tor` daemon, point `ClientOnionAuthDir` at a directory in `torrc`.

2. Put a single line in it — the address **without** the `.onion` suffix,
   followed by the credential:

```
abcdefghijklmnopqrstuvwxyz234567abcdefghijklmnopqrstuvwx:descriptor:x25519:BASE32SECRET…
```

3. Restart Tor Browser (or reload the tor daemon), then open
   `http://<address>.onion`.

## Test and audit

**Test & audit** fetches the site back through Tor and reports:

- **Listener scope** — re-checks that the local port is still loopback-only.
- **Published** — whether the descriptor reached the directory system. A new
  site can take a minute; retry if it isn't published yet.
- **Latency and HTTP status** — the round trip over Tor.
- **Security headers** — whether your server sends `content-security-policy`,
  `x-content-type-options`, `referrer-policy`, and `permissions-policy`.

Missing headers are a warning, not a failure. They matter because an onion site
is still a website, so the usual browser-side protections still apply.

::: info Permanent private sites can't be self-tested
OnionGate does not keep client credentials, so it cannot fetch the descriptor of
a permanent site that requires authorization. The audit still checks the
listener and says so. Test from a client that holds a credential instead.
:::

## Delete a site

Deleting a permanent site removes its key directory, which destroys the key and
makes the address unrecoverable. Destroying a temporary site does the same,
except the key was already gone.

The desktop UI requires a second **Delete for good?** click before removing a
permanent site. This confirms intent; it is not a backup or recovery mechanism.

If the key directory is deleted but running Tor refuses the reload, Tor may keep
serving the already-loaded identity from memory until it restarts. OnionGate
reports that condition explicitly. Restart Tor before treating deletion as
complete.

Disconnecting or choosing **Quit OnionGate** from the tray destroys every
desktop-owned temporary site. Closing the window only hides it; sites keep
running. A temporary site created by the standalone CLI remains loaded in Tor
until `oniongate stop` or another Tor shutdown, because CLI invocations do not
share a persistent temporary-site registry. Permanent identities survive
teardown and come back the next time Tor starts.

## Limits worth knowing

- **Your server is exposed as-is.** Tor anonymizes the transport; it does not
  patch your application. Directory listings, verbose error pages, debug
  toolbars, and `Server:` banners are all still visible to visitors.
- **Local content only.** If your dev server proxies to other internal hosts,
  those become reachable too. Check what it actually serves first.
- **Availability depends on this machine.** The site is up only while OnionGate
  and Tor are running. This is a workstation tool, not a hosting provider.
- **Not a substitute for the threat model.** Read the
  [threat model](/reference/threat-model) before hosting anything sensitive.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| Create button is disabled | Tor is not connected. Connect on the Connect page first. |
| "Nothing is accepting TCP connections on 127.0.0.1:*port*" | The server is not running, or is on a different port. |
| "Listener is exposed on a wildcard interface" | Rebind the server to `127.0.0.1` instead of `0.0.0.0`. |
| "Descriptor is not reachable yet" | The site is still publishing. Wait a few seconds and audit again. |
| A permanent site shows no address | Tor has not written `hostname` yet. Give it a minute, then refresh. |
| Visitor sees "Onionsite Authentication Required" | Their credential is missing, malformed, or Tor was not restarted after adding it. |
| Visitor sees "Onionsite Not Found" | The site was deleted, or a temporary site's Tor was restarted. |
| "Issue at least one client credential before turning authorization on" | Authorization needs at least one client file. Issue a credential first. |
