package lyrics

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import memory.CurrentTrack
import org.jsoup.Jsoup
import org.jsoup.internal.StringUtil
import org.jsoup.nodes.CDataNode
import org.jsoup.nodes.Element
import org.jsoup.nodes.Node
import org.jsoup.nodes.TextNode
import org.jsoup.select.NodeTraversor
import org.jsoup.select.NodeVisitor
import tornadofx.Controller
import tornadofx.Rest
import java.net.URL
import java.net.URLEncoder

class GeniusLyricsApi : LyricsApi, Controller() {
    private val api: Rest by inject()
    private val cache: LyricCache = LyricCache()

    init {
        api.baseURI = "https://genius.com/"
        api.engine.requestInterceptor = {
            it.addHeader(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.169 Safari/537.36"
            )
        }
    }

    override suspend fun getLyricsFor(meta: CurrentTrack): String = coroutineScope {
        val cached = cache.getCachedLyric(meta)

        if (cached != null) return@coroutineScope cached

        val resultsResponse = withContext(Dispatchers.IO) {
            api.get("/api/search/multi?q=${URLEncoder.encode(meta.toString(), "UTF-8")}")
        }

        val results = resultsResponse
            .one()
            .getJsonObject("response")
            .getJsonArray("sections")
            .getJsonObject(1) // songs section
            .getJsonArray("hits")

        if (results.size == 0) return@coroutineScope "No lyrics found on Genius! :-(".also {
            cache.addCachedLyric(
                meta,
                it
            )
        }

        val trackUrl = results.getJsonObject(0).getJsonObject("result").getJsonString("url").string

        val track = withContext(Dispatchers.IO) { Jsoup.parse(URL(trackUrl), 5000) }

        println(track.select(".Lyrics__Container-sc-1ynbvzw-8").text())

        val sb = StringUtil.borrowBuilder()
        for (element in track.select(".Lyrics__Container-sc-1ynbvzw-8")) {
            sb.append("\n")
            sb.append(element.text('\n'))
        }
        val lyric = StringUtil.releaseBuilder(sb).trim().ifEmpty { "[Instrumental]" }

        cache.addCachedLyric(meta, lyric)

        return@coroutineScope lyric
    }
}

// monkey patch the existing text function to allow for a custom delim instead of ' '
private fun Element.text(delim: Char): String {
    fun lastCharIsWhitespace(sb: StringBuilder): Boolean {
        return sb.isNotEmpty() && sb[sb.length - 1] == ' '
    }

    fun preserveWhitespace(node: Node?): Boolean {
        // looks only at this element and five levels up, to prevent recursion & needless stack searches
        if (node is Element) {
            var el = node as Element?
            var i = 0
            do {
                if (el!!.tag().preserveWhitespace()) return true
                el = el.parent()
                i++
            } while (i < 6 && el != null)
        }
        return false
    }

    fun appendNormalisedText(accum: StringBuilder, textNode: TextNode) {
        val text = textNode.wholeText
        if (preserveWhitespace(textNode.parentNode()) || textNode is CDataNode) accum.append(text) else StringUtil.appendNormalisedWhitespace(
            accum,
            text,
            lastCharIsWhitespace(accum)
        )
    }

    val accum: StringBuilder = StringUtil.borrowBuilder()
    NodeTraversor.traverse(object : NodeVisitor {
        override fun head(node: Node, depth: Int) {
            if (node is TextNode) {
                appendNormalisedText(accum, node)
            } else if (node is Element) {
                if (accum.isNotEmpty() &&
                    (node.isBlock || node.tag().name == "br") &&
                    !lastCharIsWhitespace(accum)
                ) accum.append(delim)
            }
        }

        override fun tail(node: Node, depth: Int) {
            // make sure there is a space between block tags and immediately following text nodes <div>One</div>Two should be "One Two".
            if (node is Element) {
                if (node.isBlock && node.nextSibling() is TextNode && !lastCharIsWhitespace(accum)) accum.append(delim)
            }
        }
    }, this)

    return StringUtil.releaseBuilder(accum).trim { it <= ' ' }
}