use std::iter::Map;
use crate::CurrentTrack;
use std::collections::HashMap;
use druid::ArcStr;
use serde::Deserialize;

pub struct GeniusScraper {
    cache: HashMap<CurrentTrack, ArcStr>
}

const LYRIC_CLASS: &str = ".Lyrics__Container-sc-1ynbvzw-8";

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
    #[serde(rename = "type")]
    type_name: String,
    hits: Vec<Hit>
}

#[derive(Deserialize)]
struct Hit {
    #[serde(rename = "type")]
    type_name: String,
    index: String,
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

    fn _scrape(track: &CurrentTrack) -> Option<String> {
        let results = reqwest::blocking::get(format!("https://genius.com/api/search/multi?q={}", track))
            .unwrap()
            .json::<SearchResults>()
            .unwrap();

        let section = results.response.sections.get(1)?;
        let hit = section.hits.first()?;
        let track_url = &hit.result.url;

        let body = reqwest::blocking::get(track_url).unwrap().text().unwrap();

        let fragment = scraper::Html::parse_document(&body);

        let lyrics = fragment
            .select(&scraper::Selector::parse(LYRIC_CLASS).ok()?)
            .map(|x| x.text().collect::<Vec<&str>>())
            .map(|x| x.join::<&str>("\n"))
            .fold("".to_string(), |acc, x| format!("{}\n{}", acc, x))
            .trim()
            .to_string();

        Some(lyrics)
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