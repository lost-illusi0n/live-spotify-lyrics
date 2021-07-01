# Live Spotify Lyrics (LSL)
A run-and-done application to help display lyrics for the current Spotify song playing, no configuration needed!

## How?
LSL tries to be as efficient as possible, to do so, it does not make requests to any API requiring authenticated requests. Instead, it will
1. read the memory of the Spotify process to locate the current track playing.
2. scrape the Genius website, and make requests to its web api which does not require authentication, to acquire lyrics for the current track. (2 requests per non-cached track)

## Supported Platforms
- Spotify Desktop Application on Windows
- Possibly More to Come

## Building
To compile the source code locally, run

`./gradlew createAllExecutables`

Once the task finishes, the application along with its dependencies will be found in `/build/app/`