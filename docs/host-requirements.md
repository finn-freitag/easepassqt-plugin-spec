# Host requirements

What an application must do to be a conforming host of `easepass:plugin` version 1.
"Must" statements are requirements; the numbers are recommended defaults that a host may
tighten.

## Install

1. Hash the package bytes, then apply the [catalog rules](catalog.md).
2. Read `manifest.json` and validate it against the schema. Reject an unsupported
   `api_version`.
3. Inspect `plugin.wasm` without instantiating it; compute types and permissions and
   reject forbidden imports/exports ([permissions](permissions.md)).
4. Reject the package if `manifest.permissions` is present and differs from the computed
   set.
5. Show the computed types, permissions, `network_hosts` and warnings. Install only after
   the user confirmed.
6. Store the hash, the computed permissions and what the user approved.

## Load

- Re-check the stored hash and repeat the inspection every time.
- Build the linker explicitly: define the neutral interfaces and the interfaces of the
  approved permissions, nothing else. Never use "link all WASI".
- If the plugin exports `initializer`, call `init` once before any other export.
- A plugin instance is used by one thread at a time (one worker thread per plugin). The
  UI thread never waits for a plugin call.

## Limits

| Limit | Recommended default |
| --- | --- |
| Package size | 16 MiB |
| `plugin.wasm` size | 16 MiB |
| Linear memory per instance | 256 MiB |
| Time per call (excluding time the user spends in a host dialog) | 30 s, `read-database`/`write-database` 120 s |
| String or list passed in one call | 64 MiB |
| File returned by `pick-file` | 64 MiB |
| Storage per plugin | 64 MiB, 1024 keys, 1024 files |
| HTTP response body | 64 MiB |
| Open TCP connections per plugin | 8 |
| Log message | 4096 bytes, rate-limited |

Exceeding a limit inside a host function returns the error `too-large` or `timeout`.
Exceeding the memory or time limit of the plugin itself stops the call (fuel or epoch
interruption); the host reports a plugin error and discards the instance. A trap is
handled the same way. Neither may crash or block the application.

## Host functions

- **log**: prefix the plugin id, strip control characters, never show plugin text as host
  UI.
- **storage**: one private directory and key/value store per plugin id. File names are
  plain names (reject path separators, `.`, `..`, empty and over-long names). Removed with
  the plugin if the user chose that.
- **file-picker**: always show the host's own dialog, with the plugin name visible. Hand
  over the file name and bytes only.
- **network**:
  - check host and port against `network_hosts` before resolving the name;
  - after resolving, refuse loopback, link-local, private and unique-local addresses
    unless the user allowed them for this plugin; connect to the address that was checked;
  - `http-request`: only `http` and `https` URLs, no credentials taken from the host's
    own cookie or credential stores, follow a redirect only if its target passes the
    same checks, cap the response size and time;
  - `tcp-connect`: verify TLS certificates against the host name; no way to disable
    verification.
- **leak-check**: use a k-anonymity service; the full password or hash never leaves the
  device. Return `unknown` when the user disabled leak checks.
- **wasi:random/random**: cryptographically secure randomness of the operating system.

## Data given to plugins

- `database-source` receives and returns the encrypted database file only. The master
  password, derived keys and decrypted items are never passed to it.
- `importer` output is shown to the user before anything is added to the database.
- `password-generator` receives the generator settings only.

## Open points for the first host implementation

These are decided by the Wasmtime proof of concept and may change the draft:

- Whether both hosts (C/C++ and .NET Wasmtime bindings) can load components. If not, the
  same interface is carried by core modules with a small message ABI and the WIT files
  remain the normative description.
- Whether hosts should additionally accept components built for `wasm32-wasip2` by
  satisfying the extra WASI imports with implementations that deny everything.
