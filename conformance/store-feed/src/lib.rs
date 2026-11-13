//! Conformance plugin for the type `store-feed`. Behaviour: see ../README.md.

wit_bindgen::generate!({
    path: "../../wit",
    world: "easepass:plugin/store-feed-plugin",
    generate_all,
});

use easepass::plugin::types::Error;
use exports::easepass::plugin::store_feed::{FeedSource, Guest};

struct Plugin;

impl Guest for Plugin {
    fn feed_sources() -> Result<Vec<FeedSource>, Error> {
        Ok(vec![FeedSource {
            name: "Conformance catalog".into(),
            url: "https://example.com/easepass/conformance/PluginList.json".into(),
        }])
    }
}

export!(Plugin);
