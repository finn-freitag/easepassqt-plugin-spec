//! Conformance plugin for the types `about` and `initializer`.
//! Shows how one plugin combines several worlds. Behaviour: see ../README.md.

wit_bindgen::generate!({
    inline: "
        package conformance:about;

        world plugin {
            include easepass:plugin/about-plugin@1.0.0;
            include easepass:plugin/initializer-plugin@1.0.0;
        }
    ",
    path: "../../wit",
    world: "conformance:about/plugin",
    generate_all,
});

use easepass::plugin::log::{log, Level};
use easepass::plugin::types::Error;
use exports::easepass::plugin::about::{Guest as About, PluginInfo};
use exports::easepass::plugin::initializer::Guest as Initializer;

struct Plugin;

impl About for Plugin {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "Conformance: about".into(),
            description: "Exports about and initializer. Needs no permission.".into(),
            author: "Ease Pass plugin spec".into(),
            author_url: Some("https://github.com/finn-freitag/easepassqt-plugin-spec".into()),
        }
    }
}

impl Initializer for Plugin {
    fn init() -> Result<(), Error> {
        log(Level::Info, "conformance-about: init");
        Ok(())
    }
}

export!(Plugin);
