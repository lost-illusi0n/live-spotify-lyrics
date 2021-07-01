package ui

import javafx.geometry.Pos
import javafx.scene.Parent
import tornadofx.*
import java.awt.Toolkit

class SpotifyLyricsView : View() {
    override val root: Parent = borderpane {
        prefWidth = Toolkit.getDefaultToolkit().screenSize.width / 3.0
        prefHeight = Toolkit.getDefaultToolkit().screenSize.height / 2.0

        top<CurrentSongView>()

        top {
            hbox {
                alignment = Pos.CENTER

                add<CurrentSongView>()
            }
        }

        center<LyricsView>()

        bottom = label("made by lost")
    }
}

private const val LOREM_IPSUM = """
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Quisque nunc neque, pharetra vitae dignissim sit amet, convallis vitae arcu. Duis finibus consequat pulvinar. In erat justo, tempus vel enim et, ullamcorper blandit augue. Vestibulum vel sem at tortor pharetra dignissim at scelerisque lorem. Vestibulum mauris dui, dictum vitae urna sed, fermentum sagittis sapien. Nunc pharetra et justo ac sollicitudin. Curabitur vestibulum risus porttitor, sagittis sapien ut, euismod magna.
Cras blandit ipsum vitae rutrum condimentum. Cras blandit in diam eu dapibus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Cras tristique scelerisque mauris nec dictum. Suspendisse imperdiet venenatis velit, eu ultricies metus venenatis in. Nullam a justo ut erat condimentum semper in in neque. Donec diam lacus, mattis non varius non, finibus at orci. Donec dictum interdum augue, non luctus neque. Quisque cursus nisi eu lacus euismod mollis nec et nunc.
Maecenas non pretium metus, quis vestibulum ex. Cras pellentesque neque nec metus ultricies ultricies. In blandit mauris sed sodales mattis. Interdum et malesuada fames ac ante ipsum primis in faucibus. Morbi nisi mi, tristique feugiat mauris eu, efficitur porta turpis. Donec vitae tincidunt odio. Nam tristique rhoncus arcu, sit amet cursus sem consectetur eu. Donec sed nisl ipsum. Donec mollis turpis vitae augue vehicula tempus. Ut dapibus magna vestibulum, maximus sapien a, ultricies elit. Pellentesque suscipit iaculis mi sed aliquam. Morbi et ultricies nulla. Quisque efficitur nunc id tempor blandit.
Ut venenatis lectus magna, et faucibus nulla gravida nec. Mauris ultricies dui velit, a ornare dolor egestas sit amet. Proin sit amet purus ut augue vehicula ornare. Nunc blandit, lectus non feugiat aliquam, enim tortor lacinia nibh, eget condimentum nibh purus quis quam. Nulla erat libero, pulvinar vel pretium ac, mollis eget lacus. Duis posuere, ante et varius placerat, nibh arcu condimentum mi, ut sodales lacus dui sed quam. Aenean et posuere metus. Quisque leo elit, feugiat et mauris quis, semper venenatis nunc. Sed at odio vel lectus hendrerit viverra. Ut non quam consequat, feugiat orci eget, finibus urna. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse potenti.
Sed venenatis mauris mi, nec facilisis ligula molestie id. Etiam nec bibendum mauris. Donec varius dui mi, vitae feugiat eros cursus et. Donec convallis consectetur egestas. Cras sollicitudin volutpat augue, ac lacinia est semper et. Vestibulum vel hendrerit massa. Vivamus varius risus id turpis suscipit, in rhoncus tortor tincidunt. Pellentesque scelerisque vitae arcu eget tempor. In quis orci neque. Suspendisse vestibulum, purus et ultricies mollis, nisl lectus rutrum est, vitae commodo ante ligula in augue. Sed vitae pellentesque mauris, ut viverra neque. Phasellus vestibulum ligula vel dignissim congue. Aliquam erat volutpat. Nullam eu vestibulum mi, ut viverra nibh.
"""