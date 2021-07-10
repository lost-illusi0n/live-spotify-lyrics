use crate::CurrentTrack;
use std::collections::HashMap;
use druid::ArcStr;
use serde::Deserialize;
use select::predicate::Class;
use select::node::Node;

// api response struct

#[derive(Deserialize)]
struct SearchResults {
    response: Response
}

#[derive(Deserialize)]
struct Response {
    sections: Vec<Section>
}

#[derive(Deserialize)]
struct Section {
    hits: Vec<Hit>
}

#[derive(Deserialize)]
struct Hit {
    result: Result
}

#[derive(Deserialize)]
struct Result {
    url: String
}

// end response struct

pub struct GeniusScraper {
    cache: HashMap<CurrentTrack, ArcStr>
}

impl GeniusScraper {
    pub(crate) fn new() -> Self {
        GeniusScraper { cache: Default::default() }
    }

    fn get_genius_track_url_for(track: &CurrentTrack) -> Option<String> {
        // request to genius api and parse it as our response struct
        let results = reqwest::blocking::get(format!("https://genius.com/api/search/multi?q={}", track))
            .unwrap()
            .json::<SearchResults>()
            .unwrap();

        // get the genius track url for the track
        let section = results.response.sections.get(1)?;
        let hit = section.hits.first()?;
        let track_url = &hit.result.url;

        Some(track_url.clone())
    }

    fn _scrape(track: &CurrentTrack) -> Option<String> {
        let track_url = GeniusScraper::get_genius_track_url_for(&track)?;

        // request the body of the track genius page so we can scrape it
        let body = reqwest::blocking::get(track_url).ok()?.text().ok()?;

        // parse the body
        let doc = select::document::Document::from(&*body);

        Some(
            doc.find(Class("Lyrics__Container-sc-1ynbvzw-8"))                              // find all containers for lyrics based by class
                .map(|n| format!("{}\n", GeniusScraper::text(&n, '\n')))     // parse each node into its' text content
                .fold("".to_string(), |acc, x| format!("{}{}", acc,  x)) // fold each text content into a single string
        )
    }

    // monkey patched version of Node::text, to allow for using a custom separator.
    pub fn text(node: &Node, separator: char) -> String {
        let mut string = String::new();
        recur(node, &mut string, separator);
        return string;

        fn recur(node: &Node, string: &mut String, seperator: char) {
            if let Some(text) = node.as_text() {
                string.push_str(&*format!("{}{}", text, seperator));
            }
            for child in node.children() {
                recur(&child, string, seperator)
            }
        }
    }

    pub fn lyrics_for(&mut self, track: CurrentTrack) -> String {
        // check for cache hit
        if self.cache.contains_key(&track) {
            return self.cache.get(&track).cloned().unwrap().to_string()
        }

        // wrap lyrics into arc for cheap clones
        let lyrics = ArcStr::from(GeniusScraper::_scrape(&track).unwrap_or("No lyrics found on Genius! :-(".to_string()));

        // insert into cache
        self.cache.insert(track, lyrics.clone());

        return lyrics.to_string();
    }
}