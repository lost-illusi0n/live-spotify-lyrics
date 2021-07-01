package lyrics

import memory.CurrentTrack

interface LyricsApi {
    suspend fun getLyricsFor(meta: CurrentTrack): String
}