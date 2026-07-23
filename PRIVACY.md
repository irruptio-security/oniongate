# Privacy

OnionGate has no analytics, advertising identifier, remote logging, or
diagnostic upload.

Stored locally:

- user settings and bridge lines;
- non-secret session counters;
- the crash-recovery mutation journal;
- up to 20 redacted verification reports;
- an optional persistence baseline.

Never intentionally stored:

- onion service private keys;
- v3 client authorization credentials;
- public IP values from verification;
- destination history;
- full process command lines;
- Tor control authentication cookies.

Network requests occur only for requested product functions: Tor bootstrap and
traffic, IP verification, Onionoo relay search, signed update checks, optional
artifact hash reputation opened by the user, and official dependency downloads
during development/builds.
