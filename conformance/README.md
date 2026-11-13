# Conformance plugins

One small plugin per plugin type. Host implementations load them in their test suites;
the behaviour below is part of the specification of these test plugins and only changes
with a note in the commit.

`tools/check.sh` builds them to `dist/conformance-<name>.wasm` (component) and
`dist/org.easepass.conformance.<name>.epplugin` (package). `expected.json` next to each
plugin holds the types and permissions a host must compute.

| Plugin | Types | Permissions | Other imports |
| --- | --- | --- | --- |
| `about` | `about`, `initializer` | - | `log` |
| `importer` | `importer` | `pick-file` | `log` |
| `database-source` | `database-source` | `storage` | `wasi:clocks/wall-clock` |
| `password-generator` | `password-generator` | `leak-check` | `wasi:random/random` |
| `store-feed` | `store-feed` | - | - |
| `network` | `importer` | `network` | - |

## about

- `info()` returns name `Conformance: about`, description
  `Exports about and initializer. Needs no permission.`, author `Ease Pass plugin spec`,
  author-url `https://github.com/finn-freitag/easepassqt-plugin-spec`.
- `init()` logs `conformance-about: init` with level `info` and returns `ok`.

## importer

- `sources()` returns `fixed` ("Fixed items") and `csv-file` ("CSV file").
- `passwords-available(id)`: `true` for both, `not-found` for anything else.
- `import-passwords("fixed")` logs `conformance-importer: importing fixed items` (`info`)
  and returns two items:
  1. display-name `Example`, user-name `alice`, email `alice@example.com`, website
     `https://example.com`, password `correct horse battery staple`, notes
     `First conformance item`, tags `conformance`, `example`, no TOTP;
  2. display-name `Ünïcödé ✓`, user-name `bob`, password `pässwörd-🔑`, TOTP secret
     `JBSWY3DPEHPK3PXP`, 6 digits, 30 s, SHA-1; all other fields empty.
- `import-passwords("csv-file")` calls `pick-file` (title `Choose a CSV file`, extensions
  `csv`). Cancelled dialog: error `cancelled`. Otherwise every non-empty line must be
  `display name;user name;password`; other input gives `invalid-argument`.
- Hidden sources for robustness tests of the host:

  | Source id | Behaviour | Host must |
  | --- | --- | --- |
  | `test-error` | returns error `unavailable` | pass the error on |
  | `test-trap` | panics (trap) | report a plugin error |
  | `test-infinite-loop` | never returns | interrupt the call |
  | `test-allocate` | allocates 1 MiB blocks forever | stop it at the memory cap |

- Any other id: `not-found`.

## database-source

One database with id `main`, stored in the plugin's private storage (file `main.epdb`,
keys `database_name`, `read_only`, `last_modified`, `logged_in`).

- `provider-name()`: `Conformance storage`.
- `config-schema()`: object with `database_name` (string, 1-80 characters) and
  `read_only` (boolean), both required.
- `get-config()` initially returns
  `{"database_name":"Conformance database","read_only":false}`.
- `set-config(json)` accepts exactly these two properties; anything else is
  `invalid-argument`.
- `list-databases()` / `get-database("main")`: id `main`, the configured name,
  description `Plugin storage (conformance)`, availability `available`, `last-modified`
  = time of the last write (none before the first).
- `read-database("main")`: the stored bytes, `not-found` before the first write.
- `write-database("main", data)`: stores the bytes and the current wall-clock time;
  `not-allowed` if `read_only` is true.
- `login` sets, `logout` deletes the key `logged_in`.
- Any other id: `not-found`.

## password-generator

- `name()`: `Conformance generator`.
- `generate(options)`: `length` characters drawn uniformly from the characters of
  `options.characters` with `wasi:random/random`. Empty `characters` or length 0:
  `invalid-argument`; length above 4096: `too-large`.
- With `verify-not-leaked` it calls `leak-check.check-password` and tries again while the
  result is `leaked`, at most 10 passwords in total. `unknown` is accepted.

## store-feed

- `feed-sources()` returns one source: name `Conformance catalog`, URL
  `https://example.com/easepass/conformance/PluginList.json`.

## network

An importer whose **source id is a command**, so a host test can request anything. Its
manifest allows `allowed.conformance.test:*`, `*.wild.conformance.test:*`,
`port.conformance.test:8080` and `192.0.2.10:*`; test hosts map these names to a local
server.

- `sources()` returns `http` and `tcp` (descriptions only).
- `import-passwords("http <METHOD> <url> [body]")` calls `http-request` with the header
  `x-conformance: 1`, the optional body and a 5 s timeout. It returns one item:
  display-name = status code, user-name = value of the response header
  `x-conformance-reply` (empty if missing), notes = response body as text.
- `import-passwords("tcp <host> <port> <plain|tls|starttls> <text>")` connects (5 s
  timeout, `tls` = TLS from the start, `starttls` = `tcp-start-tls` after connecting),
  writes `text` and a line feed, reads up to the first line feed or the end of the
  stream, closes the connection and returns one item: display-name `tcp`, notes = the
  line.
- Errors of the host functions are returned unchanged. Malformed commands give
  `invalid-argument`, anything else `not-found`.

## negative

Hand-written components (`.wat`) that a host must reject at install time:

| File | Reason |
| --- | --- |
| `forbidden-import.wat` | imports `wasi:filesystem/preopens` |
| `unknown-namespace.wat` | imports an interface of an unknown namespace |
| `future-api-version.wat` | imports `easepass:plugin/storage@2.0.0` |
