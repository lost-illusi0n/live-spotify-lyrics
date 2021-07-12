package memory

import memory.SpotifyMemoryReader.Companion.PLAYING_BASE_ADR
import memory.SpotifyMemoryReader.Companion.PLAYING_OFFSETS
import memory.SpotifyMemoryReader.Companion.TERMINATING_SEQUENCE
import memory.SpotifyMemoryReader.Companion.TRACK_BASE_ADR
import memory.SpotifyMemoryReader.Companion.TRACK_OFFSETS
import org.jire.kotmem.Process
import org.jire.kotmem.Processes

/**
 * The addresses we are trying to read are dynamic. Instead we found the static pointers referencing them.
 * The address we are looking for to find the current track is [TRACK_BASE_ADR] with the offsets of [TRACK_OFFSETS].
 * Once we reach the value, the track is formatted in the following way: SONG_NAME, [TERMINATING_SEQUENCE], SONG_AUTHOR, 0.
 * The is playing value can be found at [PLAYING_BASE_ADR] with the offsets of [PLAYING_OFFSETS]. 0 = not playing, 1 = playing.
 */
class SpotifyMemoryReader : SpotifyApi {
    private var spotifyProc: Process? = null

    private companion object {
        // static pointers with offsets
        private const val TRACK_BASE_ADR = 0x07940354
        private val TRACK_OFFSETS = arrayOf(0x38, 0x3C, 0x4, 0x18, 0x0)
        private const val PLAYING_BASE_ADR = 0x016C9300
        private val PLAYING_OFFSETS = arrayOf(0x34, 0x0, 0x30, 0x4, 0x48)

        private val TERMINATING_SEQUENCE = listOf<Byte>(32, -62, -73)

        // modules accessed
        private const val SPOTIFY = "Spotify.exe"
        private const val LIBCEF = "libcef.dll"
    }

    private inline fun <T> accessSpotify(action: (Process) -> T): T? {
        if (!isConnected) return null

        return try {
            action(spotifyProc!!)
        } catch (e: Exception) {
            null
        }
    }

    override val isConnected: Boolean
        get() {
            return try {
                this.spotifyProc = Processes[SPOTIFY]
                true
            } catch (e: Exception) {
                false
            }
        }

    override val isPlaying: Boolean
        get() {
            return accessSpotify {
                var baseAddress: Int = it[it[SPOTIFY].address + PLAYING_BASE_ADR]

                for ((index, offset) in PLAYING_OFFSETS.withIndex()) {
                    if (index == PLAYING_OFFSETS.size - 1) {
                        baseAddress += offset
                    } else {
                        baseAddress = it[baseAddress + offset]
                    }
                }

                it.get<Byte>(baseAddress) == 1.toByte()
            } ?: false
        }

    private val currentTrackAddress: Int?
        get() {
            return accessSpotify {
                if (!isPlaying) return null

                var baseAddress: Int = it[it[LIBCEF].address + TRACK_BASE_ADR]

                for ((index, offset) in TRACK_OFFSETS.withIndex()) {
                    if (index == TRACK_OFFSETS.size - 1) {
                        baseAddress += offset
                    } else {
                        baseAddress = it[baseAddress + offset]
                    }
                }

                baseAddress
            }
        }

    private enum class CurrentTrackMemoryState {
        TITLE,

        AUTHOR
    }

    override val currentTrack: CurrentTrack?
        get() {
            return accessSpotify { process ->
                if (!isPlaying) return null

                val buffer = mutableListOf<Byte>()
                var address: Int = currentTrackAddress!!
                var state: CurrentTrackMemoryState = CurrentTrackMemoryState.TITLE

                // lets just be consistent
                @Suppress("VARIABLE_WITH_REDUNDANT_INITIALIZER")
                var author: String? = null
                var title: String? = null

                while (true) {
                    val value: Byte = process[address++]
                    buffer += value

                    when (state) {
                        CurrentTrackMemoryState.TITLE -> {
                            if (buffer.takeLast(3) == TERMINATING_SEQUENCE) {
                                title = buffer.take(buffer.size - TERMINATING_SEQUENCE.size).toByteArray()
                                    .toString(Charsets.UTF_8).trim()

                                buffer.clear()

                                // start reading author...
                                state = CurrentTrackMemoryState.AUTHOR
                            }
                        }
                        CurrentTrackMemoryState.AUTHOR -> {
                            if (value == 0x0.toByte()) {
                                author = buffer.take(buffer.size - 1).toByteArray().toString(Charsets.UTF_8).trim()

                                buffer.clear()

                                break
                            }
                        }
                    }
                }

                CurrentTrack(title = title!!, author = author!!)
            }
        }
}