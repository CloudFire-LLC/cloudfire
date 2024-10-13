defmodule Web.Settings.Account.Notifications.Edit do
  use Web, :live_view
  alias Domain.Accounts

  def mount(_params, _session, socket) do
    socket =
      assign(socket,
        page_title: "Edit Notifications",
        form: to_form(Accounts.change_account(socket.assigns.account))
      )

    {:ok, socket, temporary_assigns: [form: %Phoenix.HTML.Form{}]}
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/settings/account"}>
        Account Settings
      </.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/settings/account/edit"}>
        Edit Notifications
      </.breadcrumb>
    </.breadcrumbs>
    <.section>
      <:title>
        Edit Notifications
      </:title>
      <:content>
        <div class="max-w-2xl px-4 py-8 mx-auto lg:py-16">
          <h2 class="mb-4 text-xl text-neutral-900">Edit account notifications</h2>
          <.form for={@form} phx-change={:change} phx-submit={:submit}>
            <div class="grid gap-4 mb-4 sm:grid-cols-1 sm:gap-6 sm:mb-6">
              <.inputs_for :let={config} field={@form[:config]}>
                <.inputs_for :let={notifications} field={config[:notifications]}>
                  <table class="w-full text-sm text-left text-neutral-500">
                    <thead class="text-xs text-neutral-700 uppercase bg-neutral-50">
                      <tr>
                        <th scope="col" class="px-6 py-3 font-medium">
                          Notification
                        </th>
                        <th scope="col" class="px-6 py-3 font-medium text-center">
                          Enable
                        </th>
                        <th scope="col" class="px-6 py-3 font-medium text-center">
                          Disable
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr class="bg-white border-b">
                        <.inputs_for :let={outdated_gateway} field={notifications[:outdated_gateway]}>
                          <td scope="row" class="px-6 py-4 whitespace-nowrap">
                            Outdated Gateways
                          </td>
                          <td class="px-6 py-4 text-center">
                            <.input
                              class="mx-auto"
                              type="radio"
                              field={outdated_gateway[:enabled]}
                              value="true"
                              checked={outdated_gateway[:enabled].value == true}
                            />
                          </td>
                          <td class="px-6 py-4 text-center">
                            <.input
                              class="mx-auto"
                              type="radio"
                              field={outdated_gateway[:enabled]}
                              value="false"
                              checked={outdated_gateway[:enabled].value != true}
                            />
                          </td>
                        </.inputs_for>
                      </tr>
                    </tbody>
                  </table>
                </.inputs_for>
              </.inputs_for>
            </div>
            <.submit_button>
              Save
            </.submit_button>
          </.form>
        </div>
      </:content>
    </.section>
    """
  end

  def handle_event("change", %{"account" => attrs}, socket) do
    account = socket.assigns.account

    changeset =
      Accounts.change_account(account, attrs)
      |> Map.put(:action, :validate)

    socket = assign(socket, form: to_form(changeset))

    {:noreply, socket}
  end

  def handle_event("submit", %{"account" => attrs}, socket) do
    with {:ok, account} <-
           Accounts.update_account(socket.assigns.account, attrs, socket.assigns.subject) do
      {:noreply,
       push_navigate(socket,
         to: ~p"/#{account}/settings/account"
       )}
    else
      {:error, changeset} ->
        changeset = changeset |> Map.put(:action, :validate)
        {:noreply, assign(socket, form: to_form(changeset))}
    end
  end
end
