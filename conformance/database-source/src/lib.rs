//! Conformance plugin for the type `database-source`: one database kept in the plugin's
//! private storage. Behaviour: see ../README.md.

wit_bindgen::generate!({
    path: "../../wit",
    world: "easepass:plugin/database-source-plugin",
    generate_all,
});

use easepass::plugin::storage;
use easepass::plugin::types::{Error, ErrorCode};
use exports::easepass::plugin::database_source::{Availability, DatabaseInfo, Guest};
use wasi::clocks::wall_clock;

struct Plugin;

const DATABASE_ID: &str = "main";
const DATABASE_FILE: &str = "main.epdb";
const KEY_NAME: &str = "database_name";
const KEY_READ_ONLY: &str = "read_only";
const KEY_MODIFIED: &str = "last_modified";
const KEY_LOGGED_IN: &str = "logged_in";
const DEFAULT_NAME: &str = "Conformance database";

const CONFIG_SCHEMA: &str = r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"database_name":{"type":"string","title":"Database name","default":"Conformance database","minLength":1,"maxLength":80},"read_only":{"type":"boolean","title":"Read only","default":false}},"required":["database_name","read_only"]}"#;

fn error(code: ErrorCode, message: &str) -> Error {
    Error { code, message: message.into() }
}

fn check_id(id: &str) -> Result<(), Error> {
    if id == DATABASE_ID {
        Ok(())
    } else {
        Err(error(ErrorCode::NotFound, "unknown database"))
    }
}

fn database_name() -> Result<String, Error> {
    Ok(storage::get_string(KEY_NAME)?.unwrap_or_else(|| DEFAULT_NAME.into()))
}

fn read_only() -> Result<bool, Error> {
    Ok(storage::get_string(KEY_READ_ONLY)?.as_deref() == Some("true"))
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Minimal parser for exactly the configuration object of `CONFIG_SCHEMA`.
/// Real plugins should use a JSON library; this keeps the test plugin dependency-free.
fn parse_config(json: &str) -> Option<(String, bool)> {
    let body = json.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut name = None;
    let mut read_only = None;
    let mut rest = body.trim();
    while !rest.is_empty() {
        let (key, after_key) = parse_string(rest)?;
        let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();
        let after_value = match key.as_str() {
            "database_name" => {
                let (value, after) = parse_string(after_colon)?;
                name = Some(value);
                after
            }
            "read_only" => {
                if let Some(after) = after_colon.strip_prefix("true") {
                    read_only = Some(true);
                    after
                } else {
                    read_only = Some(false);
                    after_colon.strip_prefix("false")?
                }
            }
            _ => return None,
        };
        rest = after_value.trim_start();
        if let Some(after_comma) = rest.strip_prefix(',') {
            rest = after_comma.trim_start();
            if rest.is_empty() {
                return None;
            }
        } else if !rest.is_empty() {
            return None;
        }
    }
    Some((name?, read_only?))
}

fn parse_string(input: &str) -> Option<(String, &str)> {
    let mut chars = input.strip_prefix('"')?.char_indices();
    let mut out = String::new();
    while let Some((index, c)) = chars.next() {
        match c {
            '"' => return Some((out, &input[index + 2..])),
            '\\' => match chars.next()?.1 {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                _ => return None,
            },
            c => out.push(c),
        }
    }
    None
}

fn info() -> Result<DatabaseInfo, Error> {
    let last_modified = match storage::get_string(KEY_MODIFIED)? {
        Some(text) => text.parse::<u64>().ok(),
        None => None,
    };
    Ok(DatabaseInfo {
        id: DATABASE_ID.into(),
        name: database_name()?,
        description: "Plugin storage (conformance)".into(),
        read_only: read_only()?,
        availability: Availability::Available,
        last_modified,
    })
}

impl Guest for Plugin {
    fn config_schema() -> String {
        CONFIG_SCHEMA.into()
    }

    fn get_config() -> Result<String, Error> {
        Ok(format!(
            "{{\"database_name\":{},\"read_only\":{}}}",
            json_string(&database_name()?),
            read_only()?
        ))
    }

    fn set_config(config_json: String) -> Result<(), Error> {
        let (name, read_only) = parse_config(&config_json)
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "configuration does not match the schema"))?;
        if name.is_empty() || name.chars().count() > 80 {
            return Err(error(ErrorCode::InvalidArgument, "database_name must have 1 to 80 characters"));
        }
        storage::set_string(KEY_NAME, &name)?;
        storage::set_string(KEY_READ_ONLY, if read_only { "true" } else { "false" })
    }

    fn list_databases() -> Result<Vec<DatabaseInfo>, Error> {
        Ok(vec![info()?])
    }

    fn get_database(id: String) -> Result<DatabaseInfo, Error> {
        check_id(&id)?;
        info()
    }

    fn read_database(id: String) -> Result<Vec<u8>, Error> {
        check_id(&id)?;
        storage::read_file(DATABASE_FILE)?
            .ok_or_else(|| error(ErrorCode::NotFound, "the database has not been saved yet"))
    }

    fn write_database(id: String, data: Vec<u8>) -> Result<(), Error> {
        check_id(&id)?;
        if read_only()? {
            return Err(error(ErrorCode::NotAllowed, "the database is read-only"));
        }
        storage::write_file(DATABASE_FILE, &data)?;
        storage::set_string(KEY_MODIFIED, &wall_clock::now().seconds.to_string())
    }

    fn login(id: String) -> Result<(), Error> {
        check_id(&id)?;
        storage::set_string(KEY_LOGGED_IN, "true")
    }

    fn logout(id: String) -> Result<(), Error> {
        check_id(&id)?;
        storage::delete_string(KEY_LOGGED_IN)
    }
}

export!(Plugin);
