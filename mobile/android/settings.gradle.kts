// ABOUTME: Resolves Android plugins and dependencies from their official repositories.
// ABOUTME: Keeps the Android client independent of desktop and Apple release builds.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "SpaceportMobile"
include(":app")
