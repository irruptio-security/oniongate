# Use bridges

Bridges help reach Tor when direct connections are blocked or singled out.
Configure them under **Connect → Bridges**.

::: warning Bridges are not an anonymity upgrade
A bridge hides the use of a public Tor relay from the local network, but the
bridge or transport infrastructure can observe your connection. Use direct Tor
when it works unless your threat model calls for a bridge.
:::

## Trusted sources

OnionGate accepts only:

- transport defaults bundled with the application; and
- bridge lines you obtained directly from the Tor Project's
  [BridgeDB](https://bridges.torproject.org/) and pasted yourself.

It does not download bridge feeds from GitHub, public aggregators, or other
third-party collectors. Public bridge lists are easy to poison and easy for
censors to enumerate.

Treat private BridgeDB lines as sensitive. They are stored locally in settings
when selected; do not paste them into bug reports, logs, screenshots, or public
issues.

## Transport choices

- **Snowflake** uses volunteer proxies and broker/front infrastructure. It is
  the final Smart Connect fallback.
- **meek** uses domain fronting and usually has higher latency and overhead.
- **Conjure** uses registration infrastructure from the Refraction Networking
  project. It is available only when the verified Tor runtime for that platform
  includes `conjure-client`; OnionGate packages it as a runtime resource rather
  than a Tauri `externalBin`.
- **obfs4** makes Tor traffic look random. OnionGate does not bundle private
  obfs4 bridge addresses; obtain lines from BridgeDB.
- **WebTunnel** makes the transport resemble ordinary HTTPS. Obtain lines from
  BridgeDB.
- **Vanilla** is an un-obfuscated bridge and is easier to identify.

Availability also depends on the matching bundled pluggable-transport binary.
The **Active transports** panel reports whether OnionGate found each one.

## Bridge source and “Use bridges”

**Bridge source** controls which catalog OnionGate loads:

- **None** disables bridge use and is recommended when direct Tor works.
- **Built-in censorship fallback** combines available trusted built-ins.
- **Smart Connect Snowflake fallback** (`builtin:snowflake` internally) is the
  strategy selected by Smart Connect or the Censored Network preset.
- a specific built-in transport loads only that transport's defaults;
- **Custom list** uses your saved pasted or selected lines.

The separate **Use bridges** switch controls whether the currently selected
lines are written into managed Tor's configuration. Turning it on or off while
Tor is running restarts Tor so the change is real.

The status on the Connect dashboard is read-only. Manage lines and bridge
activation here.

## Load and select lines

1. Choose a transport.
2. Select **Load trusted built-ins**, or paste BridgeDB lines under
   **Paste bridges manually**.
3. Select the lines you want in the catalog.
4. Turn on **Use bridges**.
5. Start or restart Tor.

Pasted lines may include or omit the leading `Bridge` keyword. OnionGate
normalizes it, ignores comments and blank lines, and removes exact duplicates.
It does not validate that a line came from BridgeDB.

## Scan a catalog

**Scan catalog** performs a short TCP reachability probe:

- ordinary bridge transports probe the listed host and port;
- fronted transports probe the configured broker or front on port 443;
- catalogs larger than 40 entries are evenly sampled.

A successful scan means only that the probe target accepted a TCP connection.
It does **not** authenticate a bridge, complete a Tor bootstrap, or prove that
the censor will allow the full transport. **Apply reachable** selects successful
results and saves those lines in the local bridge library in `session.db`;
connect Tor afterward to test the real path.

The catalog renders at most 80 lines at once, and the scanner samples at most 40
from a larger input.

## Smart Connect interaction

Smart Connect tries direct Tor first, then saved bridge lines, then bundled
Snowflake. As it tries strategies it may temporarily change whether bridges are
enabled. After a successful connection, the chosen strategy and reason remain
visible in the UI and stored locally.

If every strategy fails, OnionGate restores the settings from before the
attempt. It never falls back to sending application traffic directly.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| **Use bridges** is disabled | Select or paste at least one bridge line first. |
| Built-in obfs4 or WebTunnel catalog is empty | OnionGate does not ship private addresses for these transports. Use BridgeDB. |
| Scan passes but Tor does not bootstrap | The scan is only a TCP probe. Try another line or transport. |
| Tor keeps trying direct first | Smart Connect intentionally starts with direct Tor. Turn it off to use the saved bridge configuration immediately. |
| A bridge worked on one network but not another | Censorship differs by network; rescan or obtain fresh BridgeDB lines. |
| Logs expose a bridge | Clear the log view and redact the line before sharing anything. Never file it publicly. |

## Related guides

- [Quick start](/guide/quick-start)
- [Connect and route traffic](/guide/connection)
- [Local data and network activity](/reference/data-and-network)
