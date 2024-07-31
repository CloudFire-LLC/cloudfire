/* Licensed under Apache 2.0 (C) 2024 Firezone, Inc. */
package dev.firezone.android.features.session.ui

import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.firezone.android.core.data.Repository
import dev.firezone.android.tunnel.TunnelService.Companion.State
import dev.firezone.android.tunnel.model.Resource
import javax.inject.Inject

@HiltViewModel
internal class SessionViewModel
    @Inject
    constructor() : ViewModel() {
        @Inject
        internal lateinit var repo: Repository
        private val _serviceStatusLiveData = MutableLiveData<State>()
        private val _resourcesLiveData = MutableLiveData<List<Resource>>(emptyList())

        private val favoriteResources: HashSet<String> = HashSet()
        val serviceStatusLiveData: MutableLiveData<State>
            get() = _serviceStatusLiveData
        val resourcesLiveData: MutableLiveData<List<Resource>>
            get() = _resourcesLiveData

        fun clearActorName() = repo.clearActorName()
        fun getActorName() = repo.getActorNameSync()

        fun getFavoriteResources() = repo.getFavoritesSync()
        fun addFavoriteResource(id: String) {
            val favorites = repo.getFavoritesSync()
            favorites.add(id)
            repo.saveFavoritesSync(favorites)
        }
        fun removeFavoriteResource(id: String) {
            val favorites = repo.getFavoritesSync()
            favorites.remove(id)
            repo.saveFavoritesSync(favorites)
        }

        fun clearToken() = repo.clearToken()
    }
