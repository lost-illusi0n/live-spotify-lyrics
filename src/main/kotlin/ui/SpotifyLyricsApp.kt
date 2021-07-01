package ui

import tornadofx.App
import tornadofx.UIComponent

class SpotifyLyricsApp : App(SpotifyLyricsView::class) {
    private val currentSongController: CurrentSongController by inject()

    override fun onBeforeShow(view: UIComponent) {
        super.onBeforeShow(view)
        view.titleProperty.bind(currentSongController.currentTrackLabelProperty)
    }
}