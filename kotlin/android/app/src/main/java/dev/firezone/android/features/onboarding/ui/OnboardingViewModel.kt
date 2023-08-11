package dev.firezone.android.features.onboarding.ui

import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.firezone.android.core.domain.preference.GetConfigUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.firezone.android.core.domain.preference.SavePortalUrlUseCase
import javax.inject.Inject
import kotlinx.coroutines.launch

@HiltViewModel
internal class OnboardingViewModel @Inject constructor(
    private val getConfigUseCase: GetConfigUseCase,
    private val savePortalUrlUseCase: SavePortalUrlUseCase,
) : ViewModel() {

    private val stateMutableLiveData = MutableLiveData<ViewState>()
    val stateLiveData: LiveData<ViewState> = stateMutableLiveData

    private val actionMutableLiveData = MutableLiveData<ViewAction>()
    val actionLiveData: LiveData<ViewAction> = actionMutableLiveData

    private var input = ""

    fun getPortalUrl() {
        viewModelScope.launch {
            getConfigUseCase().collect {
                actionMutableLiveData.postValue(
                    ViewAction.FillPortalUrl(it.portalUrl.orEmpty())
                )
            }
        }
    }

    fun onSaveOnboardingCompleted() {
        viewModelScope.launch {
            savePortalUrlUseCase(input).collect {
                actionMutableLiveData.postValue(ViewAction.NavigateToSignInFragment)
            }
        }
    }

    fun onValidateInput(input: String) {
        this.input = input
        stateMutableLiveData.postValue(
            ViewState().copy(
                isButtonEnabled = input.isEmpty().not()
            )
        )
    }

    internal sealed class ViewAction {
        object NavigateToSignInFragment : ViewAction()
        data class FillPortalUrl(val value: String) : ViewAction()
    }

    internal data class ViewState(
        val isButtonEnabled: Boolean = false,
    )
}
