// ABOUTME: Pins the Android and Kotlin build plugins for Spaceport Mobile.
// ABOUTME: Leaves application configuration and native generation to the app module.
plugins {
    id("com.android.application") version "8.11.1" apply false
    id("org.jetbrains.kotlin.android") version "2.2.21" apply false
}
