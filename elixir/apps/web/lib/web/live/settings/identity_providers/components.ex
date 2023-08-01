defmodule Web.Settings.IdentityProviders.Components do
  use Web, :component_library

  def status(%{provider: %{deleted_at: deleted_at}} = assigns) when not is_nil(deleted_at) do
    ~H"""
    <div class="flex items-center">
      <span class="w-3 h-3 bg-gray-500 rounded-full"></span>
      <span class="ml-3">
        Deleted
      </span>
    </div>
    """
  end

  def status(
        %{
          provider: %{
            disabled_at: disabled_at,
            adapter_state: %{"status" => "pending_access_token"}
          }
        } = assigns
      )
      when not is_nil(disabled_at) do
    ~H"""
    <div class="flex items-center">
      <span class="w-3 h-3 bg-red-500 rounded-full"></span>
      <span class="ml-3">
        Pending access token,
        <span :if={@provider.adapter_state["status"]}>
          <.link navigate={
            ~p"/#{@provider.account_id}/settings/identity_providers/google_workspace/#{@provider}/redirect"
          }>
            <button class={~w[
          text-white bg-primary-600 rounded-lg
          font-medium text-sm
          px-2 py-1 text-center
          hover:bg-primary-700
          focus:ring-4 focus:outline-none focus:ring-primary-300
          dark:bg-primary-600 dark:hover:bg-primary-700 dark:focus:ring-primary-800
          active:text-white/80
        ]}>reconnect identity provider</button>
          </.link>
        </span>
      </span>
    </div>
    """
  end

  def status(%{provider: %{disabled_at: disabled_at}} = assigns) when not is_nil(disabled_at) do
    ~H"""
    <div class="flex items-center">
      <span class="w-3 h-3 bg-gray-500 rounded-full"></span>
      <span class="ml-3">
        Disabled
      </span>
    </div>
    """
  end

  def status(assigns) do
    ~H"""
    <div class="flex items-center">
      <span class="w-3 h-3 bg-green-500 rounded-full"></span>
      <span class="ml-3">
        Active
      </span>
    </div>
    """
  end

  def adapter_name(:email), do: "Magic Link"
  def adapter_name(:userpass), do: "Username & Password"
  def adapter_name(:token), do: "API Access Token"
  def adapter_name(:workos), do: "WorkOS"
  def adapter_name(:google_workspace), do: "Google Workspace"
  def adapter_name(:openid_connect), do: "OpenID Connect"
  def adapter_name(:saml), do: "SAML 2.0"

  def view_provider(%{adapter: adapter} = provider) when adapter in [:email, :userpass, :token],
    do: ~p"/#{provider.account_id}/settings/identity_providers/system/#{provider}"

  def view_provider(%{adapter: :openid_connect} = provider),
    do: ~p"/#{provider.account_id}/settings/identity_providers/openid_connect/#{provider}"

  def view_provider(%{adapter: :google_workspace} = provider),
    do: ~p"/#{provider.account_id}/settings/identity_providers/google_workspace/#{provider}"

  def view_provider(%{adapter: :saml} = provider),
    do: ~p"/#{provider.account_id}/settings/identity_providers/saml/#{provider}"

  # def edit_provider(%{adapter: adapter} = provider) when adapter in [:email, :userpass, :token],
  #   do: ~p"/#{provider.account_id}/settings/identity_providers/system/#{provider}/edit"

  # def edit_provider(%{adapter: :openid_connect} = provider),
  #   do: ~p"/#{provider.account_id}/settings/identity_providers/openid_connect/#{provider}/edit"

  # def edit_provider(%{adapter: :google_workspace} = provider),
  #   do: ~p"/#{provider.account_id}/settings/identity_providers/google_workspace/#{provider}/edit"

  # def edit_provider(%{adapter: :saml} = provider),
  #   do: ~p"/#{provider.account_id}/settings/identity_providers/saml/#{provider}/edit"
end
