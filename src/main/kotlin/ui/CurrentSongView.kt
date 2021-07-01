package ui

import javafx.beans.property.SimpleBooleanProperty
import javafx.beans.property.SimpleObjectProperty
import javafx.scene.Parent
import kotlinx.coroutines.*
import kotlinx.coroutines.javafx.JavaFx
import memory.CurrentTrack
import memory.SpotifyApi
import memory.SpotifyMemoryReader
import tornadofx.Controller
import tornadofx.View
import tornadofx.label
import tornadofx.stringBinding
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

class CurrentSongView : View() {
    private val controller: CurrentSongController by inject()

    override val root: Parent = label(controller.currentTrackLabelProperty)
}

class CurrentSongController : Controller(), CoroutineScope {
    override val coroutineContext: CoroutineContext = EmptyCoroutineContext + SupervisorJob()

    private val spotifyApi: SpotifyApi = SpotifyMemoryReader()

    private val _isPlaying: SimpleBooleanProperty = SimpleBooleanProperty(false)
    val currentTrackProperty = SimpleObjectProperty<CurrentTrack>()

    val currentTrackLabelProperty = currentTrackProperty.stringBinding {
        if (it == null) "Not playing!"
        else """${it.title} - ${it.author}"""
    }

    init {
        launch(CoroutineName("playing-watchdog")) {
            while (isActive) {
                val isPlaying = spotifyApi.isPlaying

                // we can only update properties in the javafx thread.
                if (_isPlaying.get() != isPlaying) {
                    launch(Dispatchers.JavaFx) {
                        _isPlaying.set(isPlaying)
                    }
                }

                println(_isPlaying)
                delay(POLLING_RATE)
            }
        }

        launch(CoroutineName("current-track-watchdog")) {
            while (isActive) {
                val currentTrack = spotifyApi.currentTrack

                // we can only update properties in the javafx thread.
                if (currentTrackProperty.value != currentTrack) {
                    launch(Dispatchers.JavaFx) {
                        currentTrackProperty.set(currentTrack)
                    }
                }

                println(currentTrack)
                delay(POLLING_RATE)
            }
        }
    }

    private companion object {
        private const val POLLING_RATE = 500L
    }
}