defmodule Web.Clients.Show do
  use Web, :live_view
  import Web.Policies.Components
  alias Domain.{Clients, Flows, Config}

  def mount(%{"id" => id}, _session, socket) do
    with {:ok, client} <- Clients.fetch_client_by_id(id, socket.assigns.subject, preload: :actor),
         {:ok, flows} <-
           Flows.list_flows_for(client, socket.assigns.subject,
             preload: [gateway: [:group], policy: [:resource, :actor_group]]
           ) do
      :ok = Clients.subscribe_for_clients_presence_in_account(client.account_id)

      socket =
        assign(
          socket,
          client: client,
          flows: flows,
          flow_activities_enabled?: Config.flow_activities_enabled?()
        )

      {:ok, socket}
    else
      {:error, _reason} -> raise Web.LiveErrors.NotFoundError
    end
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/clients"}>Clients</.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/clients/#{@client.id}"}>
        <%= @client.name %>
      </.breadcrumb>
    </.breadcrumbs>

    <.section>
      <:title>
        Client Details
        <span :if={not is_nil(@client.deleted_at)} class="text-red-600">(deleted)</span>
      </:title>
      <:action :if={is_nil(@client.deleted_at)}>
        <.edit_button navigate={~p"/#{@account}/clients/#{@client}/edit"}>
          Edit Client
        </.edit_button>
      </:action>
      <:content>
        <.vertical_table id="client">
          <.vertical_table_row>
            <:label>Identifier</:label>
            <:value><%= @client.id %></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Name</:label>
            <:value><%= @client.name %></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Status</:label>
            <:value><.connection_status schema={@client} /></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Owner</:label>
            <:value>
              <.link
                navigate={~p"/#{@account}/actors/#{@client.actor.id}"}
                class={["font-medium", link_style()]}
              >
                <%= @client.actor.name %>
              </.link>
            </:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Created</:label>
            <:value>
              <.relative_datetime datetime={@client.inserted_at} />
            </:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Last Seen</:label>
            <:value>
              <.relative_datetime datetime={@client.last_seen_at} />
            </:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Last Seen Remote IP</:label>
            <:value>
              <.last_seen schema={@client} />
            </:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Client Version</:label>
            <:value><%= @client.last_seen_version %></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>User Agent</:label>
            <:value><%= @client.last_seen_user_agent %></:value>
          </.vertical_table_row>
        </.vertical_table>
      </:content>
    </.section>

    <.section>
      <:title>Activity</:title>
      <:help>
        Attempts by the actor using this client to access resources.
      </:help>
      <:content>
        <.table id="flows" rows={@flows} row_id={&"flows-#{&1.id}"}>
          <:col :let={flow} label="AUTHORIZED AT">
            <.relative_datetime datetime={flow.inserted_at} />
          </:col>
          <:col :let={flow} label="EXPIRES AT">
            <.relative_datetime datetime={flow.expires_at} />
          </:col>
          <:col :let={flow} label="REMOTE IP">
            <%= flow.client_remote_ip %>
          </:col>
          <:col :let={flow} label="POLICY">
            <.link
              navigate={~p"/#{@account}/policies/#{flow.policy_id}"}
              class={["font-medium", link_style()]}
            >
              <.policy_name policy={flow.policy} />
            </.link>
          </:col>
          <:col :let={flow} label="GATEWAY (IP)">
            <.link
              navigate={~p"/#{@account}/gateways/#{flow.gateway_id}"}
              class={["font-medium", link_style()]}
            >
              <%= flow.gateway.group.name %>-<%= flow.gateway.name %>
            </.link>
            (<%= flow.gateway_remote_ip %>)
          </:col>
          <:col :let={flow} :if={@flow_activities_enabled?} label="ACTIVITY">
            <.link navigate={~p"/#{@account}/flows/#{flow.id}"} class={["font-medium", link_style()]}>
              Show
            </.link>
          </:col>
          <:empty>
            <div class="text-center text-neutral-500 p-4">No activity to display.</div>
          </:empty>
        </.table>
      </:content>
    </.section>

    <.danger_zone :if={is_nil(@client.deleted_at)}>
      <:action>
        <.delete_button
          phx-click="delete"
          data-confirm={
            "Are you sure want to delete this client? " <>
            "User still will be able to create a new one by reconnecting to the Firezone."
          }
        >
          Delete Client
        </.delete_button>
      </:action>
      <:content></:content>
    </.danger_zone>
    """
  end

  def handle_info(
        %Phoenix.Socket.Broadcast{topic: "clients:" <> _account_id, payload: payload},
        socket
      ) do
    client = socket.assigns.client

    socket =
      cond do
        Map.has_key?(payload.joins, client.id) ->
          assign(socket, client: %{client | online?: true})

        Map.has_key?(payload.leaves, client.id) ->
          assign(socket, client: %{client | online?: false})

        true ->
          socket
      end

    {:noreply, socket}
  end

  def handle_event("delete", _params, socket) do
    {:ok, _client} = Clients.delete_client(socket.assigns.client, socket.assigns.subject)
    {:noreply, push_navigate(socket, to: ~p"/#{socket.assigns.account}/clients")}
  end
end
