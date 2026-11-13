# Package format and manifest

## `.epplugin`

A plugin is distributed as one file `<id>.epplugin`: a ZIP archive (stored or deflate,
no encryption, no ZIP64) with these entries in its root:

| Entry | Required | Content |
| --- | --- | --- |
| `manifest.json` | yes | UTF-8 JSON, see below |
| `plugin.wasm` | yes | The WebAssembly component |
| `icon.png` | no | Square PNG, at most 256 x 256 pixels |

Hosts must:

- compute the SHA-256 of the package bytes **before** parsing anything
  ([catalog](catalog.md));
- ignore every other entry and never extract entries by their stored path (read the three
  known names only);
- reject packages where a known entry occurs twice, and enforce size caps
  ([host requirements](host-requirements.md)).

[`tools/package.py`](../tools/package.py) creates a deterministic package.

## `manifest.json`

Schema: [`schemas/manifest.schema.json`](../schemas/manifest.schema.json), examples in
[`samples/manifest`](../samples/manifest).

```json
{
  "id": "com.example.file-importer",
  "name": "File Importer",
  "version": "1.2.0",
  "api_version": 1,
  "description": "Imports passwords from CSV files.",
  "author": "Jane Doe",
  "homepage": "https://example.com/file-importer",
  "license": "MIT",
  "os": ["windows", "linux", "macos"],
  "network_hosts": ["api.example.com:443"]
}
```

| Property | Required | Meaning |
| --- | --- | --- |
| `id` | yes | Reverse-DNS id, lower case (`a-z`, `0-9`, `-`, `.`). Identifies the plugin for updates and names its private storage. |
| `name` | yes | Display name |
| `version` | yes | Semantic version of the plugin |
| `api_version` | yes | Major version of `easepass:plugin` the plugin targets |
| `description` | yes | Short description (may be empty) |
| `author`, `homepage`, `license` | no | Shown in the plugin manager |
| `os` | no | `windows`, `linux`, `macos`. Missing or empty = all |
| `min_app_version` | no | Oldest application version that can load the plugin |
| `network_hosts` | no | Allow-list for the `network` permission, see below |
| `permissions` | no | Informational; must equal the computed set if present |

Unknown properties are ignored.

The manifest is **not trusted** for anything security-relevant: types and permissions come
from the component ([permissions](permissions.md)). The only manifest value that affects
what a plugin can do is `network_hosts`, and it is shown to the user and can only narrow
the `network` permission the component already asked for.

### `network_hosts`

Each entry is `host:port`.

| Form | Matches |
| --- | --- |
| `api.example.com:443` | exactly this name and port |
| `*.example.com:443` | any subdomain of `example.com` (not `example.com` itself) |
| `192.0.2.10:8443`, `[2001:db8::1]:443` | exactly this address |
| `example.com:*` | any port |
| `*:21` | **any** host on this port |
| `*:*` | any host, any port |

Entries with host `*` exist for plugins whose server is configured by the user (for
example a database source for the user's own FTP server). They are shown with the warning
"This plugin can connect to ANY server."

Names are matched case-insensitively against the name the plugin passes to
`http-request`/`tcp-connect`, before DNS resolution. Independently of the list, the host
blocks loopback, link-local and private addresses after resolution unless the user allowed
them for this plugin.
