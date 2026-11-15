//! Conformance plugin for the type `importer`. Behaviour: see ../README.md.

wit_bindgen::generate!({
    path: "../../wit",
    world: "easepass:plugin/importer-plugin",
    generate_all,
});

use easepass::plugin::file_picker::{pick_file, PickOptions};
use easepass::plugin::log::{log, Level};
use easepass::plugin::types::{Error, ErrorCode, PasswordItem, Totp, TotpAlgorithm};
use exports::easepass::plugin::importer::{Guest, ImportSource};

struct Plugin;

fn error(code: ErrorCode, message: &str) -> Error {
    Error { code, message: message.into() }
}

fn item(display_name: &str, user_name: &str, password: &str) -> PasswordItem {
    PasswordItem {
        display_name: display_name.into(),
        user_name: user_name.into(),
        email: String::new(),
        website: String::new(),
        password: password.into(),
        notes: String::new(),
        tags: Vec::new(),
        totp: None,
    }
}

fn fixed_items() -> Vec<PasswordItem> {
    vec![
        PasswordItem {
            display_name: "Example".into(),
            user_name: "alice".into(),
            email: "alice@example.com".into(),
            website: "https://example.com".into(),
            password: "correct horse battery staple".into(),
            notes: "First conformance item".into(),
            tags: vec!["conformance".into(), "example".into()],
            totp: None,
        },
        PasswordItem {
            display_name: "Ünïcödé ✓".into(),
            user_name: "bob".into(),
            email: String::new(),
            website: String::new(),
            password: "pässwörd-🔑".into(),
            notes: String::new(),
            tags: Vec::new(),
            totp: Some(Totp {
                secret: "JBSWY3DPEHPK3PXP".into(),
                digits: 6,
                interval: 30,
                algorithm: TotpAlgorithm::Sha1,
            }),
        },
    ]
}

/// Lines of `display name;user name;password`. Empty lines are skipped.
fn parse_csv(data: &[u8]) -> Result<Vec<PasswordItem>, Error> {
    let text = core::str::from_utf8(data)
        .map_err(|_| error(ErrorCode::InvalidArgument, "file is not valid UTF-8"))?;
    let mut items = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() != 3 {
            return Err(error(
                ErrorCode::InvalidArgument,
                &format!("line {}: expected 3 fields, found {}", index + 1, fields.len()),
            ));
        }
        items.push(item(fields[0], fields[1], fields[2]));
    }
    Ok(items)
}

impl Guest for Plugin {
    fn sources() -> Vec<ImportSource> {
        vec![
            ImportSource {
                id: "fixed".into(),
                name: "Fixed items".into(),
                description: "Always returns the same two items.".into(),
            },
            ImportSource {
                id: "csv-file".into(),
                name: "CSV file".into(),
                description: "Lines of 'display name;user name;password'.".into(),
            },
        ]
    }

    fn import_passwords(source_id: String) -> Result<Vec<PasswordItem>, Error> {
        match source_id.as_str() {
            "fixed" => {
                log(Level::Info, "conformance-importer: importing fixed items");
                Ok(fixed_items())
            }
            "csv-file" => {
                let picked = pick_file(&PickOptions {
                    title: Some("Choose a CSV file".into()),
                    extensions: vec!["csv".into()],
                })?;
                match picked {
                    Some(file) => parse_csv(&file.data),
                    None => Err(error(ErrorCode::Cancelled, "no file chosen")),
                }
            }
            // Hidden sources for host robustness tests (not listed by `sources`).
            "test-error" => Err(error(ErrorCode::Unavailable, "requested test error")),
            "test-trap" => panic!("requested test trap"),
            "test-infinite-loop" => {
                let mut counter: u64 = 0;
                loop {
                    counter = core::hint::black_box(counter.wrapping_add(1));
                }
            }
            "test-allocate" => {
                // Grows memory without bound; a host memory cap must stop this.
                let mut blocks: Vec<Vec<u8>> = Vec::new();
                loop {
                    blocks.push(core::hint::black_box(vec![0xA5u8; 1 << 20]));
                }
            }
            _ => Err(error(ErrorCode::NotFound, "unknown source")),
        }
    }
}

export!(Plugin);
