package dev.firezone.android.core

import android.app.Application
import com.google.firebase.crashlytics.FirebaseCrashlytics
import dagger.hilt.android.HiltAndroidApp
import dev.firezone.android.BuildConfig

@HiltAndroidApp
class FirezoneApp : Application() {

    override fun onCreate() {
        super.onCreate()

        // Disable Crashlytics for debug builds
        FirebaseCrashlytics.getInstance().setCrashlyticsCollectionEnabled(!BuildConfig.DEBUG)
    }
}
