// ABOUTME: Keeps the unsent composer content in its retained SSH session.
// ABOUTME: Requires an explicit destination choice after the original input target changes.
package ai.spaceport.mobile

import uniffi.spaceport_mobile.InputTarget

internal class SessionDraft {
    var note = ""
    var file: DraftFile? = null
    var png: ByteArray? = null
    var destination: InputTarget? = null
        private set
    var label = ""
        private set

    fun present(target: InputTarget, name: String) {
        if (note.isEmpty() && png == null && file == null || destination == target) retarget(target, name)
    }

    fun retarget(target: InputTarget, name: String) {
        destination = target
        label = name
    }

    fun canSend(target: InputTarget?) = target != null && target == destination

    fun clear() {
        note = ""
        png = null
        file = null
        destination = null
        label = ""
    }
}
