defmodule Web.Settings.IdentityProviders.SAML.Show do
  use Web, :live_view
  alias Domain.Auth

  def mount(%{"provider_id" => provider_id}, _session, socket) do
    with {:ok, provider} <-
           Auth.fetch_active_provider_by_id(provider_id, socket.assigns.subject,
             preload: [created_by_identity: [:actor]]
           ) do
      {:ok, assign(socket, provider: provider)}
    else
      _ -> raise Web.LiveErrors.NotFoundError
    end
  end

  def handle_event("delete", _params, socket) do
    {:ok, _provider} = Auth.delete_provider(socket.assigns.provider, socket.assigns.subject)
    {:noreply, redirect(socket, to: ~p"/#{socket.assigns.account}/settings/identity_providers")}
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs home_path={~p"/#{@account}/dashboard"}>
      <.breadcrumb path={~p"/#{@account}/settings/identity_providers"}>
        Identity Providers Settings
      </.breadcrumb>

      <.breadcrumb path={
        ~p"/#{@account}/settings/identity_providers/saml/DF43E951-7DFB-4921-8F7F-BF0F8D31FA89"
      }>
        <%= @provider.name %>
      </.breadcrumb>
    </.breadcrumbs>
    <.header>
      <:title>
        Viewing Identity Provider <code><%= @provider.name %></code>
      </:title>
      <:actions>
        <.edit_button navigate={
          ~p"/#{@account}/settings/identity_providers/saml/#{@provider.id}/edit"
        }>
          Edit Identity Provider
        </.edit_button>
      </:actions>
    </.header>
    <!-- Identity Provider details -->
    <.header>
      <:title>Details</:title>
    </.header>

    <.flash_group flash={@flash} />

    <div class="bg-white dark:bg-gray-800 overflow-hidden">
      <table class="w-full text-sm text-left text-gray-500 dark:text-gray-400">
        <tbody>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Name
            </th>
            <td class="px-6 py-4">
              <%= @provider.name %>
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Type
            </th>
            <td class="px-6 py-4">
              SAML 2.0
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Sign requests
            </th>
            <td class="px-6 py-4">
              Yes
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Sign metadata
            </th>
            <td class="px-6 py-4">
              Yes
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Require signed assertions
            </th>
            <td class="px-6 py-4">
              Yes
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Require signed envelopes
            </th>
            <td class="px-6 py-4">
              Yes
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Base URL
            </th>
            <td class="px-6 py-4">
              Yes
            </td>
          </tr>
          <tr class="border-b border-gray-200 dark:border-gray-700">
            <th
              scope="row"
              class="text-right px-6 py-4 font-medium text-gray-900 whitespace-nowrap bg-gray-50 dark:text-white dark:bg-gray-800"
            >
              Created
            </th>
            <td class="px-6 py-4">
              <.datetime datetime={@provider.inserted_at} /> by <.owner schema={@provider} />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <.header>
      <:title>
        Danger zone
      </:title>
      <:actions>
        <.delete_button
          data-confirm="Are you sure want to delete this provider along with all related data?"
          phx-click="delete"
        >
          Delete Identity Provider
        </.delete_button>
      </:actions>
    </.header>
    """
  end
end
