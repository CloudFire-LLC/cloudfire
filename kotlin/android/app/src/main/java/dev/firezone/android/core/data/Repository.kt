/* Licensed under Apache 2.0 (C) 2024 Firezone, Inc. */
package dev.firezone.android.core.data

import android.content.Context
import android.content.SharedPreferences
import dev.firezone.android.BuildConfig
import dev.firezone.android.core.data.model.Config
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import java.security.MessageDigest
import javax.inject.Inject

internal class Repository
    @Inject
    constructor(
        private val context: Context,
        private val coroutineDispatcher: CoroutineDispatcher,
        private val sharedPreferences: SharedPreferences,
    ) {
        fun getConfigSync(): Config {
            return Config(
                sharedPreferences.getString(AUTH_BASE_URL_KEY, null)
                    ?: BuildConfig.AUTH_BASE_URL,
                sharedPreferences.getString(API_URL_KEY, null)
                    ?: BuildConfig.API_URL,
                sharedPreferences.getString(LOG_FILTER_KEY, null)
                    ?: BuildConfig.LOG_FILTER,
            )
        }

        fun getConfig(): Flow<Config> =
            flow {
                emit(getConfigSync())
            }.flowOn(coroutineDispatcher)

        fun getDefaultConfigSync(): Config =
            Config(
                BuildConfig.AUTH_BASE_URL,
                BuildConfig.API_URL,
                BuildConfig.LOG_FILTER,
            )

        fun getDefaultConfig(): Flow<Config> =
            flow {
                emit(getDefaultConfigSync())
            }.flowOn(coroutineDispatcher)

        fun saveSettings(value: Config): Flow<Unit> =
            flow {
                emit(
                    sharedPreferences
                        .edit()
                        .putString(AUTH_BASE_URL_KEY, value.authBaseUrl)
                        .putString(API_URL_KEY, value.apiUrl)
                        .putString(LOG_FILTER_KEY, value.logFilter)
                        .apply(),
                )
            }.flowOn(coroutineDispatcher)

        fun getDeviceIdSync(): String? = sharedPreferences.getString(DEVICE_ID_KEY, null)

        fun getFavorites(): Flow<HashSet<String>> =
            flow {
                val set = sharedPreferences.getStringSet(FAVORITE_RESOURCES_KEY, null)
                emit(HashSet(set))
            }.flowOn(coroutineDispatcher)

        fun saveFavorites(value: HashSet<String>): Flow<Unit> =
            flow {
                emit(sharedPreferences.edit().putStringSet(FAVORITE_RESOURCES_KEY, value).apply())
            }.flowOn(coroutineDispatcher)

        fun getToken(): Flow<String?> =
            flow {
                emit(sharedPreferences.getString(TOKEN_KEY, null))
            }.flowOn(coroutineDispatcher)

        fun getTokenSync(): String? = sharedPreferences.getString(TOKEN_KEY, null)

        fun getStateSync(): String? = sharedPreferences.getString(STATE_KEY, null)

        fun getActorName(): Flow<String?> =
            flow {
                emit(getActorNameSync())
            }.flowOn(coroutineDispatcher)

        fun getActorNameSync(): String? =
            sharedPreferences.getString(ACTOR_NAME_KEY, null)?.let {
                if (it.isNotEmpty()) "Signed in as $it" else "Signed in"
            }

        fun getNonceSync(): String? = sharedPreferences.getString(NONCE_KEY, null)

        fun saveDeviceIdSync(value: String): Unit =
            sharedPreferences
                .edit()
                .putString(DEVICE_ID_KEY, value)
                .apply()

        fun saveNonce(value: String): Flow<Unit> =
            flow {
                emit(saveNonceSync(value))
            }.flowOn(coroutineDispatcher)

        fun saveNonceSync(value: String) = sharedPreferences.edit().putString(NONCE_KEY, value).apply()

        fun saveState(value: String): Flow<Unit> =
            flow {
                emit(saveStateSync(value))
            }.flowOn(coroutineDispatcher)

        fun saveStateSync(value: String) = sharedPreferences.edit().putString(STATE_KEY, value).apply()

        fun saveToken(value: String): Flow<Unit> =
            flow {
                val nonce = sharedPreferences.getString(NONCE_KEY, "").orEmpty()
                emit(
                    sharedPreferences
                        .edit()
                        .putString(TOKEN_KEY, nonce.plus(value))
                        .apply(),
                )
            }.flowOn(coroutineDispatcher)

        fun saveActorName(value: String): Flow<Unit> =
            flow {
                emit(
                    sharedPreferences
                        .edit()
                        .putString(ACTOR_NAME_KEY, value)
                        .apply(),
                )
            }.flowOn(coroutineDispatcher)

        fun validateState(value: String): Flow<Boolean> =
            flow {
                val state = sharedPreferences.getString(STATE_KEY, "").orEmpty()
                emit(MessageDigest.isEqual(state.toByteArray(), value.toByteArray()))
            }.flowOn(coroutineDispatcher)

        fun clearToken() {
            sharedPreferences.edit().apply {
                remove(TOKEN_KEY)
                apply()
            }
        }

        fun clearNonce() {
            sharedPreferences.edit().apply {
                remove(NONCE_KEY)
                apply()
            }
        }

        fun clearState() {
            sharedPreferences.edit().apply {
                remove(STATE_KEY)
                apply()
            }
        }

        fun clearActorName() {
            sharedPreferences.edit().apply {
                remove(ACTOR_NAME_KEY)
                apply()
            }
        }

        companion object {
            private const val AUTH_BASE_URL_KEY = "authBaseUrl"
            private const val ACTOR_NAME_KEY = "actorName"
            private const val API_URL_KEY = "apiUrl"
            private const val FAVORITE_RESOURCES_KEY = "favoriteResources"
            private const val LOG_FILTER_KEY = "logFilter"
            private const val TOKEN_KEY = "token"
            private const val NONCE_KEY = "nonce"
            private const val STATE_KEY = "state"
            private const val DEVICE_ID_KEY = "deviceId"
        }
    }
