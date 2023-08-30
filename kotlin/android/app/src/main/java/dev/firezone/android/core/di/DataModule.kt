package dev.firezone.android.core.di

import android.content.SharedPreferences
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import dev.firezone.android.core.data.AuthRepository
import dev.firezone.android.core.data.AuthRepositoryImpl
import dev.firezone.android.core.data.PreferenceRepository
import dev.firezone.android.core.data.PreferenceRepositoryImpl
import kotlinx.coroutines.CoroutineDispatcher

@Module
@InstallIn(SingletonComponent::class)
class DataModule {
    @Provides
    internal fun provideAuthRepository(
        @IoDispatcher coroutineDispatcher: CoroutineDispatcher,
        sharedPreferences: SharedPreferences
    ): AuthRepository = AuthRepositoryImpl(coroutineDispatcher, sharedPreferences)

    @Provides
    internal fun providePreferenceRepository(
        @IoDispatcher coroutineDispatcher: CoroutineDispatcher,
        sharedPreferences: SharedPreferences
    ): PreferenceRepository = PreferenceRepositoryImpl(coroutineDispatcher, sharedPreferences)
}
