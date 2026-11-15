//! Conformance plugin for the type `password-generator`. Behaviour: see ../README.md.

wit_bindgen::generate!({
    path: "../../wit",
    world: "easepass:plugin/password-generator-plugin",
    generate_all,
});

use easepass::plugin::leak_check::{check_password, LeakStatus};
use easepass::plugin::types::{Error, ErrorCode};
use exports::easepass::plugin::password_generator::{GeneratorOptions, Guest};
use wasi::random::random::get_random_u64;

struct Plugin;

const MAX_LENGTH: u32 = 4096;
const MAX_ATTEMPTS: u32 = 10;

fn error(code: ErrorCode, message: &str) -> Error {
    Error { code, message: message.into() }
}

/// Uniform index below `bound` (rejection sampling, no modulo bias).
fn random_below(bound: u64) -> u64 {
    let zone = u64::MAX - (u64::MAX % bound);
    loop {
        let value = get_random_u64();
        if value < zone {
            return value % bound;
        }
    }
}

fn generate_once(characters: &[char], length: u32) -> String {
    (0..length)
        .map(|_| characters[random_below(characters.len() as u64) as usize])
        .collect()
}

impl Guest for Plugin {
    fn generate(options: GeneratorOptions) -> Result<String, Error> {
        let characters: Vec<char> = options.characters.chars().collect();
        if characters.is_empty() {
            return Err(error(ErrorCode::InvalidArgument, "characters must not be empty"));
        }
        if options.length == 0 {
            return Err(error(ErrorCode::InvalidArgument, "length must be at least 1"));
        }
        if options.length > MAX_LENGTH {
            return Err(error(ErrorCode::TooLarge, "length must be at most 4096"));
        }

        let mut password = generate_once(&characters, options.length);
        if options.verify_not_leaked {
            for _ in 1..MAX_ATTEMPTS {
                // `unknown` (offline) is accepted: the host decides how to tell the user.
                if check_password(&password)?.status != LeakStatus::Leaked {
                    break;
                }
                password = generate_once(&characters, options.length);
            }
        }
        Ok(password)
    }
}

export!(Plugin);
