use std::time::Duration;

use druid::{AppLauncher, ArcStr, Data, Env, Event, EventCtx, Lens, Selector, Target, widget::{Controller, Flex, Label, WidgetExt}, Widget, WindowDesc, FontDescriptor, FontWeight, ExtEventSink};
use druid::widget::{Padding, Scroll, LineBreaking, FlexParams, CrossAxisAlignment};

use crate::genius_scraper::GeniusScraper;
use crate::spotify_mem_reader::SpotifyMemReader;
use std::fmt::{Display, Formatter};
use core::fmt;

mod spotify_mem_reader;
mod genius_scraper;

#[derive(Debug, Clone, Data, Lens)]
struct LiveSpotifyLyrics {
    lyrics: String,
    current_track: Option<CurrentTrack>,
}

#[derive(Debug, Clone, Data, PartialEq, Eq, Hash)]
pub struct CurrentTrack {
    title: String,
    author: String,
}

impl Display for CurrentTrack {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.title, self.author)
    }
}

fn make_ui() -> impl Widget<LiveSpotifyLyrics> {
    let song_font = FontDescriptor::default()
        .with_weight(FontWeight::BOLD)
        .with_size(20.0);

    let song_label = Label::dynamic(|track: &Option<CurrentTrack>, _|
        track.as_ref()
            .map(|track| format!("{} - {}", track.title, track.author))
            .unwrap_or(format!("Play something!"))
    )
        .with_font(song_font)
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(LiveSpotifyLyrics::current_track)
        .align_left();

    let lyric_label = Label::dynamic(|lyrics: &String, _| lyrics.to_string())
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(LiveSpotifyLyrics::lyrics)
        .expand_width();

    let lyric_scroll = Scroll::new(lyric_label)
        .vertical()
        .align_left();

    let root = Padding::new(
        10.0,
        Flex::column()
            .with_child(song_label)
            .with_default_spacer()
            .with_flex_child(lyric_scroll, FlexParams::new(1.0, CrossAxisAlignment::Baseline))
            .with_default_spacer()
            .with_child(Label::new("made by lost").with_text_size(12.0).align_left()),
    );

    let event_handler = EventHandler::new();
    return root.controller(event_handler);
}

struct EventHandler {}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl<W: Widget<LiveSpotifyLyrics>> Controller<LiveSpotifyLyrics, W> for EventHandler {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut LiveSpotifyLyrics,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(UPDATE_TRACK) => {
                data.current_track = cmd.get_unchecked(UPDATE_TRACK).clone();

                if data.current_track.is_none() {
                    data.lyrics = String::from("Start playing something!")
                } else {
                    data.lyrics = String::from("processing...");
                }

                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.is(UPDATE_LYRIC) => {
                data.lyrics = cmd.get_unchecked(UPDATE_LYRIC).clone();
                ctx.request_paint()
            }
            _ => child.event(ctx, event, data, env)
        }
    }
}

const UPDATE_TRACK: Selector<Option<CurrentTrack>> = Selector::new("update-track");
const UPDATE_LYRIC: Selector<String> = Selector::new("update-lyric");

fn main() {
    let window = WindowDesc::new(make_ui)
        .title(ArcStr::from("Live Spotify Lyrics"));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    spawn_polling_thread(event_sink);

    launcher
        .use_simple_logger()
        .launch(LiveSpotifyLyrics {
            lyrics: String::from("Start playing something!"),
            current_track: None,
        }).expect("launch failed")
}

fn spawn_polling_thread(
    event_sink: ExtEventSink
) {
    std::thread::spawn(move || unsafe {
        let mut scraper: GeniusScraper = GeniusScraper::new();
        let mut last_track: Option<CurrentTrack> = None;

        loop {
            let is_playing = SpotifyMemReader::is_playing().unwrap_or(false);
            let current_track = if is_playing { SpotifyMemReader::current_track() } else { None };

            if !cmp_eq_option!(current_track, last_track) {
                last_track = current_track.clone();
                if event_sink.submit_command(UPDATE_TRACK, current_track.clone(), Target::Auto).is_err() {
                    break;
                }

                match current_track {
                    // start processing lyrics
                    Some(track) => {
                        let lyrics = scraper.lyrics_for(track);

                        if event_sink.submit_command(UPDATE_LYRIC, lyrics, Target::Auto).is_err() {
                            break;
                        }
                    }
                    None => ()
                }
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
#[macro_export]
macro_rules! cmp_eq_option {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (Some(left_val), Some(right_val)) => *left_val == *right_val,
            (None, None) => true,
            _ => false,
        }
    }};
}