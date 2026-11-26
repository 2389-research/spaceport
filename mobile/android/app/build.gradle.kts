// ABOUTME: Packages the Kotlin application, generated bindings, and Android native libraries.
// ABOUTME: Builds the shared Rust controller for both supported ABIs before Android compilation.
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val mobile = rootDir.parentFile
val generated = mobile.resolve(".build/android")

android {
    namespace = "ai.spaceport.mobile"
    compileSdk = 36
    ndkVersion = "28.2.13676358"
    // AGP 8.11's default. The release workflow reads this pin for apksigner and aapt2, whose output changes between versions.
    buildToolsVersion = "35.0.0"

    defaultConfig {
        applicationId = "ai.research2389.spaceport.mobile"
        minSdk = 26
        targetSdk = 36
        // Release builds take both from the v* tag (scripts/release/android-request.py).
        versionCode = providers.gradleProperty("spaceport.versionCode").orNull?.toInt() ?: 1
        versionName = providers.gradleProperty("spaceport.versionName").orNull ?: "0.1"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk { abiFilters += setOf("arm64-v8a", "x86_64") }
    }
    // The release key lives only in the android-release environment's secrets. Without it,
    // assembleRelease leaves the APK unsigned, and the release workflow refuses to publish it.
    val keystore = System.getenv("SPACEPORT_ANDROID_KEYSTORE")
    if (keystore != null) {
        signingConfigs.create("release") {
            storeFile = file(keystore)
            storePassword = System.getenv("SPACEPORT_ANDROID_KEYSTORE_PASSWORD")
            keyAlias = System.getenv("SPACEPORT_ANDROID_KEY_ALIAS")
            keyPassword = System.getenv("SPACEPORT_ANDROID_KEY_PASSWORD")
        }
    }
    buildTypes {
        debug { applicationIdSuffix = ".debug" }
        release { signingConfigs.findByName("release")?.let { signingConfig = it } }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // JNA distributes prebuilt dispatch libraries, including legacy ABIs the NDK cannot strip.
    packaging.jniLibs.keepDebugSymbols += "**/libjnidispatch.so"
    sourceSets["main"].apply {
        java.srcDir(generated.resolve("kotlin"))
        jniLibs.srcDir(generated.resolve("jniLibs"))
    }
}

kotlin { jvmToolchain(17) }

val buildMobileNative = tasks.register<Exec>("buildMobileNative") {
    description = "Build the Rust controller and generate its Kotlin bindings."
    val sdk = androidComponents.sdkComponents.sdkDirectory
    val ndk = android.ndkVersion
    val api = android.defaultConfig.minSdk
    doFirst {
        commandLine(
            rootDir.resolve("build-native"),
            sdk.get().asFile.resolve("ndk/$ndk"),
            api.toString()
        )
    }
}
tasks.named("preBuild") { dependsOn(buildMobileNative) }

dependencies {
    implementation("androidx.activity:activity:1.13.0")
    implementation("androidx.annotation:annotation:1.11.0")
    implementation("net.java.dev.jna:jna:5.19.1@aar")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("androidx.test:runner:1.7.0")
}
