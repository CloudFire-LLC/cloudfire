package dev.firezone.android.core.di

import android.app.Application
import android.content.Context
import android.content.SharedPreferences
import android.content.res.Resources
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import dev.firezone.android.features.session.backend.BootShutdownReceiver
import dev.firezone.android.features.session.backend.SessionManager
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher

internal const val ENCRYPTED_SHARED_PREFERENCES = "encryptedSharedPreferences"

@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    internal fun provideContext(app: Application): Context = app.applicationContext

    @Provides
    internal fun provideResources(app: Application): Resources = app.resources

    @Provides
    internal fun provideEncryptedSharedPreferences(app: Application): SharedPreferences =
        EncryptedSharedPreferences.create(
            app.applicationContext,
            ENCRYPTED_SHARED_PREFERENCES,
            MasterKey.Builder(app.applicationContext)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build(),
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )

    @Provides
    internal fun provideSessionManager(
        sharedPreferences: SharedPreferences
    ): SessionManager = SessionManager(sharedPreferences)

    @Provides
    internal fun provideBroadcastReceiver(
        @MainImmediateDispatcher coroutineDispatcher: CoroutineDispatcher,
        sessionManager: SessionManager,
    ): BootShutdownReceiver = BootShutdownReceiver(coroutineDispatcher, sessionManager)
}
