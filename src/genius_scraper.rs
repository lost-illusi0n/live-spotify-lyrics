use crate::CurrentTrack;
use std::collections::HashMap;
use druid::ArcStr;
use serde::Deserialize;
use select::predicate::Class;
use select::node::Node;

pub struct GeniusScraper {
    cache: HashMap<CurrentTrack, ArcStr>
}

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

impl GeniusScraper {
    pub(crate) fn new() -> Self {
        GeniusScraper { cache: Default::default() }
    }

    fn get_genius_track_url_for(track: &CurrentTrack) -> Option<String> {
        let results = reqwest::blocking::get(format!("https://genius.com/api/search/multi?q={}", track))
            .unwrap()
            .json::<SearchResults>()
            .unwrap();

        let section = results.response.sections.get(1)?;
        let hit = section.hits.first()?;
        let track_url = &hit.result.url;

        Some(track_url.clone())
    }

    fn _scrape(track: &CurrentTrack) -> Option<String> {
        let track_url = GeniusScraper::get_genius_track_url_for(&track)?;

        let body = reqwest::blocking::get(track_url).ok()?.text().ok()?;

        println!("{}", body);

        let doc = select::document::Document::from(&*body);

        Some(doc.find(Class("Lyrics__Container-sc-1ynbvzw-8"))
            .map(|n| format!("{}\n", GeniusScraper::text(&n, '\n')))
            .fold("".to_string(), |acc, x| format!("{}{}", acc,  x))
        )
    }

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
        if self.cache.contains_key(&track) {
            return self.cache.get(&track).cloned().unwrap().to_string()
        }

        let lyrics = ArcStr::from(GeniusScraper::_scrape(&track).unwrap_or("No lyrics found on Genius! :-(".to_string()));

        self.cache.insert(track, lyrics.clone());

        return lyrics.to_string();
    }
}