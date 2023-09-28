/* Licensed under Apache 2.0 (C) 2023 Firezone, Inc. */
package dev.firezone.android.tunnel.data

import android.content.SharedPreferences
import com.squareup.moshi.Moshi
import com.squareup.moshi.adapter
import dev.firezone.android.tunnel.data.TunnelRepository.Companion.CONFIG_KEY
import dev.firezone.android.tunnel.data.TunnelRepository.Companion.RESOURCES_KEY
import dev.firezone.android.tunnel.data.TunnelRepository.Companion.ROUTES_KEY
import dev.firezone.android.tunnel.data.TunnelRepository.Companion.STATE_KEY
import dev.firezone.android.tunnel.model.Resource
import dev.firezone.android.tunnel.model.Tunnel
import dev.firezone.android.tunnel.model.TunnelConfig
import java.lang.Exception
import javax.inject.Inject

@OptIn(ExperimentalStdlibApi::class)
class TunnelRepositoryImpl @Inject constructor(
    private val sharedPreferences: SharedPreferences,
    private val moshi: Moshi,
) : TunnelRepository {

    private val lock = Any()

    override fun addListener(callback: SharedPreferences.OnSharedPreferenceChangeListener) {
        sharedPreferences.registerOnSharedPreferenceChangeListener(callback)
    }

    override fun removeListener(callback: SharedPreferences.OnSharedPreferenceChangeListener) {
        sharedPreferences.unregisterOnSharedPreferenceChangeListener(callback)
    }

    override fun get(): Tunnel? = synchronized(lock) {
        return try {
            Tunnel(
                config = requireNotNull(getConfig()),
                state = getState(),
                routes = getRoutes(),
                resources = getResources(),
            )
        } catch (e: Exception) {
            null
        }
    }

    override fun setConfig(config: TunnelConfig) {
        synchronized(lock) {
            val json = moshi.adapter<TunnelConfig>().toJson(config)
            sharedPreferences.edit().putString(CONFIG_KEY, json).apply()
        }
    }

    override fun getConfig(): TunnelConfig? = synchronized(lock) {
        val json = sharedPreferences.getString(CONFIG_KEY, "{}") ?: "{}"
        return moshi.adapter<TunnelConfig>().fromJson(json)
    }

    override fun setState(state: Tunnel.State) {
        synchronized(lock) {
            sharedPreferences.edit().putString(STATE_KEY, state.name).apply()
        }
    }

    override fun getState(): Tunnel.State {
        val json = sharedPreferences.getString(STATE_KEY, null)
        return json?.let { Tunnel.State.valueOf(it) } ?: Tunnel.State.Down
    }

    override fun setResources(resources: List<Resource>) {
        synchronized(lock) {
            val json = moshi.adapter<List<Resource>>().toJson(resources)
            sharedPreferences.edit().putString(RESOURCES_KEY, json).apply()
        }
    }

    override fun getResources(): List<Resource> {
        synchronized(lock) {
            val json = sharedPreferences.getString(RESOURCES_KEY, "[]") ?: "[]"
            return moshi.adapter<List<Resource>>().fromJson(json) ?: emptyList()
        }
    }

    override fun addRoute(route: String) {
        synchronized(lock) {
            getRoutes().toMutableList().run {
                add(route)
                val json = moshi.adapter<List<String>>().toJson(this)
                sharedPreferences.edit().putString(ROUTES_KEY, json).apply()
            }
        }
    }

    override fun removeRoute(route: String) {
        synchronized(lock) {
            getRoutes().toMutableList().run {
                remove(route)
                val json = moshi.adapter<List<String>>().toJson(this)
                sharedPreferences.edit().putString(ROUTES_KEY, json).apply()
            }
        }
    }

    override fun getRoutes(): List<String> = synchronized(lock) {
        val json = sharedPreferences.getString(ROUTES_KEY, "[]") ?: "[]"
        return moshi.adapter<List<String>>().fromJson(json) ?: emptyList()
    }

    override fun clear() {
        synchronized(lock) {
            sharedPreferences.edit().clear().apply()
        }
    }
}
