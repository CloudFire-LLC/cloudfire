plugins {
    id("org.mozilla.rust-android-gradle.rust-android")
    id("com.android.library")
    id("kotlin-android")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "dev.firezone.connlib"
    compileSdk = 33

    defaultConfig {
        minSdk = 29
        targetSdk = 33
        consumerProguardFiles("consumer-rules.pro")
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    externalNativeBuild {
        cmake {
            version = "3.22.1"
        }
    }
    ndkVersion = "25.2.9519653"
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility(JavaVersion.VERSION_1_8)
        targetCompatibility(JavaVersion.VERSION_1_8)
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    publishing {
        singleVariant("release")
    }
    sourceSets["main"].jniLibs {
        srcDir("jniLibs")
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.7.0")
    implementation("androidx.test.ext:junit-gtest:1.0.0-alpha01")
    implementation("com.android.ndk.thirdparty:googletest:1.11.0-beta-1")
    implementation(fileTree(mapOf("dir" to "libs", "include" to listOf("*.jar"))))
    implementation("org.jetbrains.kotlin:kotlin-stdlib:1.7.21")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.3")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.4.0")
}

apply(plugin = "org.mozilla.rust-android-gradle.rust-android")

fun copyJniShared(task: Task, buildType: String) = task.apply {
    outputs.upToDateWhen { false }

    val jniTargets = mapOf(
        "armv7-linux-androideabi" to "armeabi-v7a",
        "aarch64-linux-android" to "arm64-v8a",
        "i686-linux-android" to "x86",
        "x86_64-linux-android" to "x86_64",
    )

    jniTargets.forEach { entry ->
        val soFile = File(
            project.projectDir.parentFile.parentFile.parentFile.parentFile,
            "target/${entry.key}/${buildType}/libconnlib.so"
        )
        val targetDir = File(project.projectDir, "/jniLibs/${entry.value}").apply {
            if (!exists()) {
                mkdirs()
            }
        }

        copy {
            with(copySpec {
                from(soFile)
            })
            into(targetDir)
        }
    }
}

cargo {
    prebuiltToolchains = true
    verbose = true
    module  = "../"
    libname = "connlib"
    targets = listOf("arm", "arm64", "x86", "x86_64")
    features {
        if (System.getenv("CONNLIB_MOCK") != null) {
            defaultAnd(listOf("mock").toTypedArray())
        }
    }
}

tasks.register("copyJniSharedObjectsDebug") {
    copyJniShared(this, "debug")
}

tasks.register("copyJniSharedObjectsRelease") {
    copyJniShared(this, "release")
}

tasks.whenTaskAdded {
    if (name.startsWith("javaPreCompile")) {
        val newTasks = arrayOf (
            tasks.named("cargoBuild"),
            if (name.endsWith("Debug")) {
                tasks.named("copyJniSharedObjectsDebug")
            } else {
                tasks.named("copyJniSharedObjectsRelease")
            }
        )
        dependsOn(*newTasks)
    }
}
