// ABOUTME: Packages the Kotlin application, generated bindings, and Android native libraries.
// ABOUTME: Builds the shared Rust controller for both supported ABIs before Android compilation.
import com.android.ide.common.vectordrawable.Svg2Vector
import com.android.ide.common.vectordrawable.VdPreview
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val mobile = rootDir.parentFile
val generated = mobile.resolve(".build/android")

val brandResources = layout.buildDirectory.dir("generated/agent-brand-res")
val brandSources = layout.buildDirectory.dir("generated/agent-brand-kotlin")
val generateAgentBrands = tasks.register("generateAgentBrands") {
    val marks = fileTree(mobile.parentFile.resolve("assets/icons")) { include("agent-*.svg", "agent-*.png") }
    inputs.files(marks)
    outputs.dir(brandResources)
    outputs.dir(brandSources)
    doLast {
        val drawables = brandResources.get().dir("drawable-nodpi").asFile
        drawables.deleteRecursively()
        drawables.mkdirs()
        val bindings = mutableListOf<String>()
        for (mark in marks.sortedBy { it.name }) {
            val name = mark.nameWithoutExtension.replace('-', '_')
            val identity = mark.nameWithoutExtension.removePrefix("agent-")
            require(identity.matches(Regex("[a-z0-9-]{1,32}")))
            bindings.add("    \"$identity\" to R.drawable.$name,")
            if (mark.extension == "png") {
                mark.copyTo(drawables.resolve("$name.png"))
            } else {
                val svg = temporaryDir.resolve(mark.name)
                svg.writeText(mark.readText().replace("currentColor", "#ffffff"))
                val vector = ByteArrayOutputStream()
                val errors = Svg2Vector.parseSvgToXml(svg.toPath(), vector)
                check(errors.isEmpty()) { "${mark.name}: $errors" }
                val detail = StringBuilder()
                val pixels = VdPreview.getPreviewFromVectorXml(
                    VdPreview.TargetSize.createFromMaxDimension(88), vector.toString(Charsets.UTF_8), detail,
                )
                check(pixels != null && detail.isEmpty()) { "${mark.name}: $detail" }
                ImageIO.write(pixels, "png", drawables.resolve("$name.png"))
            }
        }
        val source = brandSources.get().asFile
        source.mkdirs()
        source.resolve("AgentBrandResources.kt").writeText(
            "package ai.spaceport.mobile\n\ninternal val agentBrandResources = mapOf(\n" +
                bindings.joinToString("\n") + "\n)\n"
        )
    }
}

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
        res.srcDir(brandResources)
        java.srcDir(brandSources)
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
tasks.named("preBuild") { dependsOn(buildMobileNative, generateAgentBrands) }

dependencies {
    implementation("androidx.activity:activity:1.13.0")
    implementation("androidx.annotation:annotation:1.11.0")
    implementation("net.java.dev.jna:jna:5.19.1@aar")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("androidx.test:runner:1.7.0")
}
