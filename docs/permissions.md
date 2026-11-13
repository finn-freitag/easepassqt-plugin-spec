# Permissions and plugin types

## The rule: computed, never declared

A WebAssembly component has no ambient abilities. It can only call the functions of the
interfaces it **imports**, and the host decides which imports it satisfies. Therefore:

1. At install time the host reads the imports and exports of `plugin.wasm` **without
   instantiating it**.
2. `permissions = map(imports)` and `types = map(exports)` using the tables below
   (machine-readable: [`spec/permissions.json`](../spec/permissions.json)).
3. The install dialog shows these computed values, never text supplied by the plugin.
4. When the plugin is loaded, the host links **only** the interfaces of the permissions
   the user approved (plus the neutral ones). A component that needs anything else cannot
   be instantiated.

`manifest.json` is informational. If it contains a `permissions` list that differs from
the computed set, the install is rejected.

[`tools/check_component.py`](../tools/check_component.py) is a reference implementation
of steps 1 and 2.

## Imports

Interface names are compared without the version; the version is checked separately
(see below).

| Imported interface | Permission id | Text in the install dialog |
| --- | --- | --- |
| `easepass:plugin/types` | - (neutral) | - |
| `easepass:plugin/log` | - (neutral) | - |
| `wasi:random/random` | - (neutral) | - |
| `wasi:clocks/wall-clock` | - (neutral) | - |
| `easepass:plugin/storage` | `storage` | Keeps its own private settings and files |
| `easepass:plugin/file-picker` | `pick-file` | Asks you to choose a file (every time) |
| `easepass:plugin/network` | `network` | Connects to: *hosts from the manifest* (any protocol) |
| `easepass:plugin/leak-check` | `leak-check` | Checks passwords against a breach service (sends a hash prefix to a third party) |

**Any other import is forbidden** and the component is rejected: `wasi:filesystem`,
`wasi:sockets`, `wasi:cli`, `wasi:http`, unknown namespaces, imports that are not
interfaces, and interfaces with an unsupported version.

Version check:

- `easepass:plugin/*@X.y.z`: `X` must be the API version the host implements (1).
- `wasi:*@0.2.z`: any patch version of WASI 0.2.

The set is fixed. New permissions only come with a new API version
([versioning](versioning.md)).

### What the permissions do not include

- **Files.** There is no file system permission. `pick-file` opens a dialog of the host
  every time; the plugin receives the name and content of the one file the user chose,
  never a path.
- **Network targets.** `network` covers every protocol (HTTP through `http-request`,
  everything else through the `tcp-*` functions), but only to the hosts and ports listed in
  `network_hosts` of the manifest. The host shows that list in the dialog and enforces it
  ([host requirements](host-requirements.md)). An entry with host `*` means "any server"
  and gets its own warning line.

## Exports

| Exported interface | Type id | Asks the user | Text in the install dialog |
| --- | --- | --- | --- |
| `easepass:plugin/about` | `about` | no | - |
| `easepass:plugin/initializer` | `initializer` | no | - |
| `easepass:plugin/importer` | `importer` | yes | Adds passwords to your database |
| `easepass:plugin/database-source` | `database-source` | yes | Stores your encrypted database elsewhere (it only receives encrypted data) |
| `easepass:plugin/password-generator` | `password-generator` | yes | Generates passwords: its output becomes your passwords |
| `easepass:plugin/store-feed` | `store-feed` | yes | Provides additional plugin lists |

A component must export at least one of these interfaces. Any other export is rejected.

## Sensitive plugins and high-risk combinations

A plugin is **sensitive** (the install dialog must be confirmed explicitly) if it has at
least one type that asks the user or at least one permission.

Some combinations get an additional warning line:

| Type | Permission | Warning |
| --- | --- | --- |
| `database-source` | `network` | This plugin sends your encrypted database over the network. |
| `password-generator` | `network` | This plugin creates your passwords and can send data over the network. |
| `importer` | `network` | This plugin adds passwords to your database and can send data over the network. |
| any | `network` with a `*` host | This plugin can connect to ANY server. |

## Honest limits

Permissions restrict what a plugin can reach, not what it does with data it legitimately
receives. A password generator may produce weak passwords; an importer with network
access sees the passwords it imports. The mitigations are the hash-based
[catalog](catalog.md), review of the official plugins and the warnings above.
