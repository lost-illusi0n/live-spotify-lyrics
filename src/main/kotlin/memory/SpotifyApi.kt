package memory

interface SpotifyApi {
    val currentTrack: CurrentTrack?

    val isPlaying: Boolean

    val isConnected: Boolean
}

data class CurrentTrack(val title: String, val author: String) {
    override fun toString(): String = "$title $author"
}