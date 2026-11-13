# Ease Pass plugin specification

The plugin interface of [Ease Pass Qt](https://github.com/finn-freitag/EasePassQt), the
cross-platform port of the password manager
[Ease Pass](https://github.com/FrozenAssassine/EasePass).

Plugins are **WebAssembly components**. A plugin cannot do anything on its own: it can
only call the host functions it imports, and Ease Pass decides which of them it provides.
That is why Ease Pass can show the user exactly what a plugin is able to do **before** it
is installed, without trusting anything the plugin says about itself.

| Path | Content |
| --- | --- |
| [`wit/`](wit) | The interface, package `easepass:plugin@1.0.0` ([WIT](https://component-model.bytecodealliance.org/design/wit.html)) |
| [`spec/permissions.json`](spec/permissions.json) | Permission and type table, machine-readable |
| [`schemas/`](schemas) | JSON Schemas of `manifest.json` and the plugin catalog |
| [`samples/`](samples) | Valid and invalid example files for both schemas |
| [`conformance/`](conformance) | One test plugin per plugin type (Rust) plus components a host must reject |
| [`tools/`](tools) | `check.sh` (all checks), `check_component.py` (reference permission computation), `package.py` |
| [`docs/`](docs) | [Permissions](docs/permissions.md), [package and manifest](docs/package-format.md), [catalog](docs/catalog.md), [host requirements](docs/host-requirements.md), [versioning](docs/versioning.md), [mapping from the C# interfaces](docs/csharp-mapping.md) |

Status: version 1.0.0 is a **draft** until the first host implementation is finished
(see [versioning](docs/versioning.md)).

## What a plugin can be

The **type** of a plugin is computed from the interfaces it exports
([`wit/plugin.wit`](wit/plugin.wit)). One plugin may have several types.

| Type | Exported interface | What it does |
| --- | --- | --- |
| `importer` | `easepass:plugin/importer` | Adds passwords from another service or file |
| `database-source` | `easepass:plugin/database-source` | Stores the **encrypted** database elsewhere |
| `password-generator` | `easepass:plugin/password-generator` | Generates passwords |
| `store-feed` | `easepass:plugin/store-feed` | Provides additional plugin catalogs |
| `about` | `easepass:plugin/about` | Information for the plugin manager |
| `initializer` | `easepass:plugin/initializer` | Runs code when the plugin is loaded |

## What a plugin may use

The **permissions** of a plugin are computed from the interfaces it imports
([`wit/host.wit`](wit/host.wit), details in [docs/permissions.md](docs/permissions.md)).

| Imported interface | Permission | Shown to the user as |
| --- | --- | --- |
| `easepass:plugin/log` | none | - |
| `easepass:plugin/storage` | `storage` | Keeps its own private settings and files |
| `easepass:plugin/file-picker` | `pick-file` | Asks you to choose a file (every time) |
| `easepass:plugin/network` | `network` | Connects to: *declared hosts* (any protocol) |
| `easepass:plugin/leak-check` | `leak-check` | Checks passwords against a breach service |
| `wasi:random/random`, `wasi:clocks/wall-clock` | none | - |

Everything else (file system, sockets, environment, processes, ...) does not exist for a
plugin. A component that imports anything outside this table is rejected.

## Writing a plugin (Rust)

You need Rust with the `wasm32-unknown-unknown` target and
[`wasm-tools`](https://github.com/bytecodealliance/wasm-tools). With Nix, `nix develop`
in this repository provides both.

1. Create a library crate and add this repository's `wit/` directory to it (copy it or
   use a git submodule).

   ```toml
   [lib]
   crate-type = ["cdylib"]

   [dependencies]
   wit-bindgen = "0.63"
   ```

2. Generate the bindings for the world of your plugin type and implement the exported
   interface:

   ```rust
   wit_bindgen::generate!({
       path: "wit",
       world: "easepass:plugin/importer-plugin",
       generate_all,
   });

   use easepass::plugin::types::{Error, PasswordItem};
   use exports::easepass::plugin::importer::{Guest, ImportSource};

   struct Plugin;

   impl Guest for Plugin {
       fn sources() -> Vec<ImportSource> { /* ... */ }
       fn passwords_available(source_id: String) -> Result<bool, Error> { Ok(true) }
       fn import_passwords(source_id: String) -> Result<Vec<PasswordItem>, Error> { /* ... */ }
   }

   export!(Plugin);
   ```

   The complete examples are in [`conformance/`](conformance);
   [`conformance/about`](conformance/about/src/lib.rs) shows a plugin with two types.

3. Build the module, turn it into a component and look at what it imports:

   ```bash
   cargo build --release --target wasm32-unknown-unknown
   wasm-tools component new target/wasm32-unknown-unknown/release/my_plugin.wasm -o plugin.wasm
   python3 tools/check_component.py plugin.wasm
   ```

   `check_component.py` prints the types and permissions a host will compute. The worlds
   offer every host interface, but only the ones your code really calls end up as imports
   of the component, so only those become permissions.

4. Write a [`manifest.json`](docs/package-format.md) and create the package:

   ```bash
   python3 tools/package.py --manifest manifest.json --wasm plugin.wasm --out-dir dist
   ```

Use the target `wasm32-unknown-unknown`, **not** `wasm32-wasip1`/`wasm32-wasip2`: the
standard library of the WASI targets imports WASI interfaces such as the file system and
environment even if your code never uses them, and hosts reject such components. Without
WASI there is no clock, randomness or output in `std`: use `wasi:clocks/wall-clock`,
`wasi:random/random` and `easepass:plugin/log` from the generated bindings instead.

Other languages work as long as the result is a component that only imports interfaces
from the table above.

## Rules for plugins

- No UI. Configuration is described as a JSON Schema that the host renders.
- Every fallible call returns `result<T, error>`. A trap (panic) or a timeout is reported
  to the user as a plugin error; it never takes the application down.
- Calls are synchronous for the plugin. The host runs each plugin on its own worker
  thread and stops calls that take too long or use too much memory
  ([host requirements](docs/host-requirements.md)).
- Never put secrets into log or error messages.

## Checking this repository

```bash
nix develop -c tools/check.sh
```

validates the WIT package and the schemas, builds the conformance plugins, verifies their
computed types and permissions and writes the components and packages to `dist/`. CI runs
the same script.

## License

[MIT](LICENSE). The files in `wit/deps` are subsets of the WASI 0.2 interface definitions
(Apache-2.0 WITH LLVM-exception).
