# Versioning

Three things have versions:

| What | Where | Example |
| --- | --- | --- |
| The plugin API | WIT package `easepass:plugin@MAJOR.MINOR.PATCH`, `api_version` = MAJOR | `1.0.0`, `api_version: 1` |
| A plugin | `version` in the manifest and catalog | `1.2.0` |
| The catalog file format | `schema` in the catalog | `1` |

## Plugin API

- **PATCH**: documentation and clarifications only. No change to any WIT signature.
- **MINOR**: additions that keep every existing plugin working: a new host interface that
  maps to **no** permission, or a new function in an existing interface. A plugin that
  uses an addition must state the application version it needs in `min_app_version`;
  older hosts cannot instantiate it and report "needs a newer version of Ease Pass".
- **MAJOR** (new `api_version`): everything else, in particular
  - a new permission or a changed meaning of an existing one,
  - a new plugin type,
  - changed or removed functions, records, enums or variants.

  A host may support several major versions side by side. It rejects plugins with an
  `api_version` it does not implement.

Records, enums and variants are never extended within a major version, because that would
change the binary interface of existing plugins.

The permission set of a major version is closed: within `api_version` 1 the text and the
abilities behind a permission id do not grow.

### Draft status

`1.0.0` is a draft until a host has implemented it and the first official plugins run on
it. The proof of concept settled the format (WebAssembly components, see
[host requirements](host-requirements.md)). Until then signatures may still change without a new major version. The repository
gets the tag `v1.0.0` when the draft is frozen.

## Plugins

Semantic versioning. A host offers an update when the catalog has a higher `version` for
an installed `id`. An update is installed like a new plugin: if its computed permissions,
types or `network_hosts` differ from what the user approved, the dialog is shown again.

## Catalog format

`schema` only changes for incompatible changes. New optional properties do not change it;
readers ignore properties they do not know.
