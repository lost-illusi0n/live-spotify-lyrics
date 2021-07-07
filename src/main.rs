use iced::{
    executor, time, Align, Application, Clipboard, Column, Command, Element, Settings,
    Subscription, Text,
};

use crate::spotify_mem_reader::SpotifyMemReader;

mod spotify_mem_reader;

#[derive(Default)]
struct LiveSpotifyLyrics {
    lyrics: String,
    is_playing: bool,
}

#[derive(Debug, Default)]
pub struct CurrentTrack {
    title: String,
    author: String,
}

#[derive(Debug)]
enum Message {
    Tick(),
    // CurrentTrackUpdated(CurrentTrack)
}

impl Application for LiveSpotifyLyrics {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            LiveSpotifyLyrics {
                lyrics: "Nothing playing!".to_string(),
                is_playing: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Live Spotify Lyrics")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Tick() => {
                self.is_playing = unsafe { SpotifyMemReader::is_playing() }.unwrap_or(false)
            }
            // Message::CurrentTrackUpdated(current) => {
          //     current.to_string();
          //     ()
          // }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_micros(100000)).map(|_| Message::Tick())
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(Text::new(self.is_playing.to_string()))
            .into()
    }
}

fn main() -> iced::Result {
    LiveSpotifyLyrics::run(Settings::default())
}
