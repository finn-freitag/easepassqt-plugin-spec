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
| Linear memory per instance | 64 MiB |
| Time per call (excluding time the user spends in a host dialog) | 30 s, `read-database`/`write-database` 120 s |
| Byte list (`list<u8>`) passed in one call, in either direction | 8 MiB |
| String passed in one call | 8 MiB |
| Storage per plugin | 64 MiB, 1024 keys, 1024 files |
| Open TCP connections per plugin | 8 |
| Log message | 4096 bytes, rate-limited |

The byte-list cap covers the file returned by `pick-file`, storage files, HTTP bodies,
`tcp-read`/`tcp-write` and database files. It is deliberately small: the dynamic value API
of Wasmtime (C and .NET) represents `list<u8>` as one tagged value of 32 bytes per byte,
so an 8 MiB list costs the host about 256 MiB for a moment. Larger data has to be moved
in several calls.

The memory limit is small for the same reason. A host can refuse a large byte list it is
about to pass INTO a plugin, but a list a plugin RETURNS is converted by the runtime before
the host sees its size. The plugin's memory limit is therefore the only bound: with 64 MiB
a hostile plugin can make the host allocate roughly 2 GiB for a moment. Hosts that cannot
accept that should lower the limit further.

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

## Reference hosts

The Wasmtime proof of concept of Ease Pass Qt (C API, Wasmtime 48) and a test host on
wasmtime-dotnet 50 both load the conformance importer component unchanged, inspect its
imports, refuse to instantiate it without the approved interface, and survive its trap,
infinite-loop and memory tests. Components are therefore the plugin format; there is no
core-module fallback.

Notes for host authors:

- Define an (empty) instance for `easepass:plugin/types` in the linker; components import
  it even though it only contains types.
- wasmtime-dotnet has no wrapper for import/export inspection yet. Call
  `wasmtime_component_type_import_nth`/`_export_nth` of the bundled native library, or read
  the component's import section yourself.
- After a trap or an interrupted call, discard the instance and its store.
- Components built for `wasm32-wasip2` are not accepted (their extra WASI imports are not
  on the allow-list).
