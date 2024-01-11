defmodule Web.Settings.IdentityProviders.GoogleWorkspace.Show do
  use Web, :live_view
  import Web.Settings.IdentityProviders.Components
  alias Domain.{Auth, Actors}

  def mount(%{"provider_id" => provider_id}, _session, socket) do
    with {:ok, provider} <-
           Auth.fetch_provider_by_id(provider_id, socket.assigns.subject,
             preload: [created_by_identity: [:actor]]
           ),
         {:ok, identities_count_by_provider_id} <-
           Auth.fetch_identities_count_grouped_by_provider_id(socket.assigns.subject),
         {:ok, groups_count_by_provider_id} <-
           Actors.fetch_groups_count_grouped_by_provider_id(socket.assigns.subject) do
      {:ok,
       assign(socket,
         provider: provider,
         identities_count_by_provider_id: identities_count_by_provider_id,
         groups_count_by_provider_id: groups_count_by_provider_id,
         page_title: "Identity Provider #{provider.name}"
       )}
    else
      _ -> raise Web.LiveErrors.NotFoundError
    end
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/settings/identity_providers"}>
        Identity Providers Settings
      </.breadcrumb>

      <.breadcrumb path={~p"/#{@account}/settings/identity_providers/google_workspace/#{@provider}"}>
        <%= @provider.name %>
      </.breadcrumb>
    </.breadcrumbs>

    <.section>
      <:title>
        Identity Provider <code><%= @provider.name %></code>
        <span :if={not is_nil(@provider.disabled_at)} class="text-primary-600">(disabled)</span>
        <span :if={not is_nil(@provider.deleted_at)} class="text-red-600">(deleted)</span>
      </:title>
      <:action :if={is_nil(@provider.deleted_at)}>
        <.edit_button navigate={
          ~p"/#{@account}/settings/identity_providers/google_workspace/#{@provider.id}/edit"
        }>
          Edit
        </.edit_button>
      </:action>
      <:action :if={is_nil(@provider.deleted_at)}>
        <%= if @provider.adapter_state["status"] != "pending_access_token" do %>
          <.button
            :if={not is_nil(@provider.disabled_at)}
            phx-click="enable"
            data-confirm="Are you sure want to enable this provider?"
          >
            Enable Identity Provider
          </.button>
        <% end %>
      </:action>
      <:action :if={is_nil(@provider.deleted_at)}>
        <.button
          style="primary"
          navigate={
            ~p"/#{@account.id}/settings/identity_providers/google_workspace/#{@provider}/redirect"
          }
          icon="hero-arrow-path"
        >
          Reconnect
        </.button>
      </:action>
      <:content>
        <.header>
          <:title>Details</:title>
        </.header>

        <.flash_group flash={@flash} />

        <div class="bg-white overflow-hidden">
          <.vertical_table id="provider">
            <.vertical_table_row>
              <:label>Name</:label>
              <:value><%= @provider.name %></:value>
            </.vertical_table_row>
            <.vertical_table_row>
              <:label>Status</:label>
              <:value>
                <.status provider={@provider} />
              </:value>
            </.vertical_table_row>

            <.vertical_table_row>
              <:label>Sync Status</:label>
              <:value>
                <.sync_status
                  account={@account}
                  provider={@provider}
                  identities_count_by_provider_id={@identities_count_by_provider_id}
                  groups_count_by_provider_id={@groups_count_by_provider_id}
                />
                <div
                  :if={
                    (is_nil(@provider.last_synced_at) and not is_nil(@provider.last_sync_error)) or
                      (@provider.last_syncs_failed > 3 and not is_nil(@provider.last_sync_error))
                  }
                  class="p-3 mt-2 border-l-4 border-red-500 bg-red-100 rounded-md"
                >
                  <p class="font-medium text-red-700">
                    IdP provider reported an error during the last sync:
                  </p>
                  <div class="flex items-center mt-1">
                    <span class="text-red-500 font-mono"><%= @provider.last_sync_error %></span>
                  </div>
                </div>
              </:value>
            </.vertical_table_row>

            <.vertical_table_row>
              <:label>Client ID</:label>
              <:value><%= @provider.adapter_config["client_id"] %></:value>
            </.vertical_table_row>
            <.vertical_table_row>
              <:label>Created</:label>
              <:value>
                <.created_by account={@account} schema={@provider} />
              </:value>
            </.vertical_table_row>
          </.vertical_table>
        </div>
      </:content>
    </.section>

    <.danger_zone :if={is_nil(@provider.deleted_at)}>
      <:action>
        <.button
          :if={is_nil(@provider.disabled_at)}
          style="warning"
          phx-click="disable"
          icon="hero-no-symbol"
          data-confirm="Are you sure want to disable this provider? Users will no longer be able to sign in with this provider and user / group sync will be paused."
        >
          Disable Identity Provider
        </.button>
        <.delete_button
          data-confirm="Are you sure want to delete this provider along with all related data?"
          phx-click="delete"
        >
          Delete Identity Provider
        </.delete_button>
      </:action>
      <:content></:content>
    </.danger_zone>
    """
  end

  def handle_event("delete", _params, socket) do
    {:ok, _provider} = Auth.delete_provider(socket.assigns.provider, socket.assigns.subject)

    {:noreply,
     push_navigate(socket, to: ~p"/#{socket.assigns.account}/settings/identity_providers")}
  end

  def handle_event("enable", _params, socket) do
    attrs = %{disabled_at: nil}
    {:ok, provider} = Auth.update_provider(socket.assigns.provider, attrs, socket.assigns.subject)

    {:ok, provider} =
      Auth.fetch_provider_by_id(provider.id, socket.assigns.subject,
        preload: [created_by_identity: [:actor]]
      )

    {:noreply, assign(socket, provider: provider)}
  end

  def handle_event("disable", _params, socket) do
    attrs = %{disabled_at: DateTime.utc_now()}
    {:ok, provider} = Auth.update_provider(socket.assigns.provider, attrs, socket.assigns.subject)

    {:ok, provider} =
      Auth.fetch_provider_by_id(provider.id, socket.assigns.subject,
        preload: [created_by_identity: [:actor]]
      )

    {:noreply, assign(socket, provider: provider)}
  end
end
