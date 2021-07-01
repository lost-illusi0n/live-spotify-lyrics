package lyrics

import memory.CurrentTrack

class LyricCache {
    private val lyricCache: LinkedHashMap<CurrentTrack, String> = LinkedHashMap(100)

    fun getCachedLyric(`for`: CurrentTrack): String? = lyricCache[`for`]

    fun addCachedLyric(`for`: CurrentTrack, value: String) {
        if (lyricCache.size == 100) {
            ArrayList(lyricCache.entries).removeLast()
        }
        lyricCache[`for`] = value
    }
}