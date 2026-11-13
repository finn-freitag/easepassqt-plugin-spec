# Mapping from the C# plugin interfaces

The Windows application loads .NET assemblies that implement the interfaces of
`EasePassExtensibility`. This specification replaces them; there is no binary
compatibility. The table shows where each concept went.

| C# (`EasePassExtensibility`) | This specification |
| --- | --- |
| `IAboutPlugin` (name, description, author, author URL, icon) | export `about`; the icon is `icon.png` in the package |
| `IInitializer.Init()` | export `initializer` |
| `IPasswordImporter` (`SourceName`, `SourceIcon`, `ImportPasswords`, `PasswordsAvailable`) | export `importer`; one plugin lists any number of `import-source`s instead of one class per source; no per-source icon |
| `PasswordItem` | record `password-item` (TOTP fields grouped as `option<totp>`, `tags` added) |
| `IPasswordGenerator` (`Init` + `Generate`) | export `password-generator`; the settings are passed to `generate`, leak checks go through the `leak-check` import instead of a delegate |
| `IDatabaseProvider` | export `database-source`: `provider-name`, `list-databases`, `get-config`, `set-config` |
| `IDatabaseProvider.GetSampleJsonConfig`, `ExternalConfigEditingSupport`, `OpenExternalConfigEditor` | `config-schema`: a JSON Schema the host renders as a form. Plugins have no UI and cannot start programs |
| `IDatabaseSource` (name, description, read-only, availability, last modified) | record `database-info`, addressed by `id` |
| `IDatabaseSource.OnPropertyChanged` | removed; the host polls `get-database` |
| `IDatabaseSource.GetDatabaseFileBytes` / `SaveDatabaseFileBytes` | `read-database` / `write-database` (encrypted bytes) |
| `IDatabaseSource.Login` / `Logout` | `login` / `logout` |
| `IExtensionSource.GetExtensionSources` | export `store-feed`; URLs point to catalog JSON files |
| `ILoggerInjectable` / `ILogger` | import `log` (always available). No `Init` with system information, no `LogException`, no `Flush` |
| `IStorageInjectable` (`SaveString`, `LoadString`, `SaveFile`, `LoadFile`) | import `storage`, plus delete and list functions |
| `IFilePickerInjectable` (returns a path) | import `file-picker`; returns the file **content**, plugins have no file system |
| `IWarning` (marker for the sensitive-data dialog) | removed; "sensitive" is computed from types and permissions |
| `IDatabasePaths` (obsolete) | removed |
| direct use of .NET networking | import `network` (`http-request`, `tcp-connect`) with a host allow-list |
| `Task<T>` | synchronous calls on a worker thread of the host |
| exceptions, `null`, `bool` success values | `result<T, error>` and `option<T>` |

Not possible by design: plugins that read other programs' data from the file system or
the registry (browser or Wi-Fi password importers). Such a plugin has to ask the user to
export a file and read it through `file-picker`.
