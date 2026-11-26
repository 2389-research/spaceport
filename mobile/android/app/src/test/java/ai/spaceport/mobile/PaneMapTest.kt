// ABOUTME: Tests how Herdr's reported pane cells scale into the map drawn in Go To.
// ABOUTME: Covers a two-column split and refuses an empty grid.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test
import uniffi.spaceport_mobile.MobilePaneMap
import uniffi.spaceport_mobile.MobilePaneRect

class PaneMapTest {
    @Test fun tilesScaleCellsToTheViewAndKeepPaneIds() {
        val map = MobilePaneMap(80u, 24u, listOf(
            MobilePaneRect("left", 0u, 0u, 40u, 24u),
            MobilePaneRect("top", 40u, 0u, 40u, 12u),
            MobilePaneRect("bottom", 40u, 12u, 40u, 12u),
        ))
        assertEquals(
            listOf(PaneTile("left", 0f, 0f, 200f, 120f), PaneTile("top", 200f, 0f, 400f, 60f),
                PaneTile("bottom", 200f, 60f, 400f, 120f)),
            paneTiles(map, 400f, 120f),
        )
    }

    @Test fun emptyGridHasNoTiles() {
        assertEquals(emptyList<PaneTile>(), paneTiles(MobilePaneMap(0u, 24u, emptyList()), 400f, 120f))
    }
}
