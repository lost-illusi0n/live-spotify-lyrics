package ui

import javafx.beans.property.SimpleStringProperty
import javafx.beans.property.StringProperty
import javafx.scene.Parent
import javafx.scene.control.ScrollPane
import kotlinx.coroutines.*
import kotlinx.coroutines.javafx.JavaFx
import lyrics.GeniusLyricsApi
import lyrics.LyricsApi
import tornadofx.*
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

class LyricsView : View() {
    private val lyricsController: LyricsController by inject()

    override val root: Parent = scrollpane {
        isFitToWidth = true

        label(lyricsController.lyricsProperty) {
            isWrapText = true
        }
    }
}

class LyricsController : Controller(), CoroutineScope {
    private val currentSongController: CurrentSongController by inject()
    private val lyricsView: LyricsView by inject()
    private val lyricsApi: LyricsApi = GeniusLyricsApi()

    val lyricsProperty: StringProperty = SimpleStringProperty("Start playing a song!")

    init {
        currentSongController.currentTrackProperty.onChange {
            if (it != null) {
                (lyricsView.root as ScrollPane).vvalue = 0.0
                launch {
                    val lyricsDef = async { lyricsApi.getLyricsFor(it) }

                    launch(Dispatchers.JavaFx) {
                        lyricsProperty.set("processing...")
                    }

                    val lyrics = lyricsDef.await()

                    //we can only update properties in javafx thread
                    launch(Dispatchers.JavaFx) {
                        lyricsProperty.set(lyrics)
                    }
                }
            } else {
                launch(Dispatchers.JavaFx) {
                    lyricsProperty.set("Start playing a song!")
                }
            }
        }
    }

    override val coroutineContext: CoroutineContext = EmptyCoroutineContext + SupervisorJob()
}