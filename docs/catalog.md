# Plugin catalog

A catalog is a JSON file served over HTTPS (for example `plugins/PluginList.json` in a
GitHub repository). Schema: [`schemas/catalog.schema.json`](../schemas/catalog.schema.json),
examples in [`samples/catalog`](../samples/catalog).

```json
{
  "schema": 1,
  "updated": "2026-10-14T00:00:00Z",
  "plugins": [{
    "id": "de.frozenassassine.fileimporter",
    "name": "File Password Importer",
    "description": "...",
    "author": "...",
    "version": "1.2.0",
    "api_version": 1,
    "os": ["windows", "linux", "macos"],
    "download_url": "https://github.com/.../de.frozenassassine.fileimporter.epplugin",
    "icon_url": "https://.../icon.png",
    "sha256": "<lower-case hex SHA-256 of the exact .epplugin bytes>",
    "sensitive": true,
    "min_app_version": "2.0.0",
    "revoked": false
  }]
}
```

`os` missing or empty means all operating systems. `sensitive` is a display hint for the
list only; the install dialog always uses the values the host computes from the package.
Unknown properties are ignored. A file with another `schema` value is not read.

## The hash decides "verified"

- The SHA-256 is computed over the downloaded or chosen file **before** it is parsed or
  extracted.
- **Install from the store:** fetch the catalog, download `download_url`, compare the
  hash with the entry. Any difference aborts the install.
- **Install from a local file:** look the hash up in the catalog.

  | Result | Meaning for the user |
  | --- | --- |
  | found, not revoked | verified |
  | found, `revoked: true` | blocked, cannot be installed |
  | not found | extra confirmation "not officially verified" |
  | catalog unreachable | distinct message "could not verify (offline)" |

- The hash is stored when a plugin is installed. The host re-checks it every time the
  plugin is loaded and refuses to load a changed file. A plugin whose hash becomes
  `revoked` in a newer catalog is disabled.

## Operating systems

The store list hides plugins whose `os` does not contain the current system by default;
a toggle shows them disabled (for example "Windows only"). Installing a local file whose
catalog entry names another system shows a warning. The catalog is authoritative for what
the store displays, the installed manifest for loading.

## Additional catalogs

Plugins of type `store-feed` return further catalog URLs. Such catalogs are read with the
same schema and rules, but only the official catalog of the application makes a package
"verified".

Signing the catalog is not part of version 1.
