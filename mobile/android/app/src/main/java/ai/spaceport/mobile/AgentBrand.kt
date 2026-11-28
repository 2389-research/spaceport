// ABOUTME: Resolves the canonical agent artwork packaged from Spaceport's shared assets.
// ABOUTME: Limits lookup to known agent resource names and lets the caller draw a fallback.
package ai.spaceport.mobile

import android.content.Context
import android.graphics.drawable.Drawable

internal val brandIdentity = Regex("[a-z0-9-]{1,32}")

internal fun agentBrand(context: Context, identity: String?): Drawable? {
    return agentBrandResources[identity]?.let(context::getDrawable)
}
