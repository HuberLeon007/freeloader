# Security model

Freeloader treats every URL, filename, referrer, browser message and filesystem path as untrusted input. The desktop core validates HTTP(S) schemes, rejects credentials and forbidden schemes, sanitizes Windows filenames, uses `.part` files, limits redirects, streams without loading whole files into memory, and atomically renames only after a successful flush.

Native Messaging is a privileged browser-to-desktop boundary and has been abused by malware. The host uses strict length-prefixed JSON framing, a 64 KiB payload limit, versioned schemas, exact extension allowlists, no wildcard origins, no shell execution, no command-string interpolation, and no cookie or Authorization forwarding. Native-host registration is per-user and uninstallable.

No shipped binary starts an HTTP server, localhost listener or WebSocket bridge. Logs remain local and redact query strings. No telemetry, analytics or remote crash reporting exists. Report vulnerabilities privately through the repository security contact before public disclosure.
