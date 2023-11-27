defmodule Web.Sites.Show do
  use Web, :live_view
  import Web.Sites.Components
  alias Domain.{Gateways, Resources}

  def mount(%{"id" => id}, _session, socket) do
    with {:ok, group} <-
           Gateways.fetch_group_by_id(id, socket.assigns.subject,
             preload: [
               connections: [:resource],
               created_by_identity: [:actor]
             ]
           ),
         {:ok, gateways} <-
           Gateways.list_connected_gateways_for_group(group, socket.assigns.subject,
             preload: [token: [created_by_identity: [:actor]]]
           ),
         resources =
           group.connections
           |> Enum.reject(&is_nil(&1.resource))
           |> Enum.map(& &1.resource),
         {:ok, resource_actor_groups_peek} <-
           Resources.peek_resource_actor_groups(resources, 3, socket.assigns.subject) do
      :ok = Gateways.subscribe_for_gateways_presence_in_group(group)

      {:ok,
       assign(socket,
         group: group,
         gateways: gateways,
         resource_actor_groups_peek: resource_actor_groups_peek
       )}
    else
      {:error, _reason} -> raise Web.LiveErrors.NotFoundError
    end
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/sites"}>Sites</.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/sites/#{@group}"}>
        <%= @group.name %>
      </.breadcrumb>
    </.breadcrumbs>

    <.section>
      <:title>
        Site: <code><%= @group.name %></code>
      </:title>
      <:action>
        <.edit_button navigate={~p"/#{@account}/sites/#{@group}/edit"}>
          Edit Site
        </.edit_button>
      </:action>

      <:content>
        <.vertical_table id="group">
          <.vertical_table_row>
            <:label>Name</:label>
            <:value><%= @group.name %></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Data Routing</:label>
            <:value><%= pretty_print_routing(@group.routing) %></:value>
          </.vertical_table_row>
          <.vertical_table_row>
            <:label>Created</:label>
            <:value>
              <.created_by account={@account} schema={@group} />
            </:value>
          </.vertical_table_row>
        </.vertical_table>
      </:content>
    </.section>

    <.section>
      <:title>
        Online Gateways
        <.link class={["text-sm", link_style()]} navigate={~p"/#{@account}/sites/#{@group}/gateways"}>
          see all <.icon name="hero-arrow-right" class="w-2 h-2" />
        </.link>
      </:title>
      <:action>
        <.add_button navigate={~p"/#{@account}/sites/#{@group}/new_token"}>
          Deploy
        </.add_button>
      </:action>
      <:content>
        <div class="relative overflow-x-auto">
          <.table id="gateways" rows={@gateways}>
            <:col :let={gateway} label="INSTANCE">
              <.link
                navigate={~p"/#{@account}/gateways/#{gateway.id}"}
                class="font-medium text-blue-600 hover:underline"
              >
                <%= gateway.name %>
              </.link>
            </:col>
            <:col :let={gateway} label="REMOTE IP">
              <code>
                <%= gateway.last_seen_remote_ip %>
              </code>
            </:col>
            <:col :let={gateway} label="TOKEN CREATED AT">
              <.created_by account={@account} schema={gateway.token} />
            </:col>
            <:col :let={gateway} label="STATUS">
              <.connection_status schema={gateway} />
            </:col>
            <:empty>
              <div class="flex flex-col items-center justify-center text-center text-slate-500 p-4">
                <div class="pb-4">
                  No gateways to display.
                </div>
                <div class="pb-4">
                  <.add_button navigate={~p"/#{@account}/sites/#{@group}/new_token"}>
                    Deploy a Gateway
                  </.add_button>
                </div>
                <div>
                  <p>
                    Deploy gateways to terminate connections to your site's resources. All gateways deployed within a site must be able to reach all its resources.
                  </p>
                </div>
              </div>
            </:empty>
          </.table>
        </div>
      </:content>
    </.section>

    <.section>
      <:title>
        Resources
      </:title>
      <:action>
        <.add_button navigate={~p"/#{@account}/resources/new?site_id=#{@group}"}>
          Create
        </.add_button>
      </:action>
      <:content>
        <div class="relative overflow-x-auto">
          <.table
            id="resources"
            rows={Enum.reject(@group.connections, &is_nil(&1.resource))}
            row_item={& &1.resource}
          >
            <:col :let={resource} label="NAME">
              <.link
                navigate={~p"/#{@account}/resources/#{resource}?site_id=#{@group}"}
                class="font-medium text-blue-600 hover:underline"
              >
                <%= resource.name %>
              </.link>
            </:col>
            <:col :let={resource} label="ADDRESS">
              <%= resource.address %>
            </:col>
            <:col :let={resource} label="Authorized groups">
              <.peek peek={Map.fetch!(@resource_actor_groups_peek, resource.id)}>
                <:empty>
                  None,
                  <.link
                    class={link_style() ++ ["px-1"]}
                    navigate={~p"/#{@account}/policies/new?resource_id=#{resource}&site_id=#{@group}"}
                  >
                    create a Policy
                  </.link>
                  to grant access.
                </:empty>

                <:item :let={group}>
                  <.link
                    class={link_style()}
                    navigate={~p"/#{@account}/groups/#{group.id}?site_id=#{@group}"}
                  >
                    <.badge>
                      <%= group.name %>
                    </.badge>
                  </.link>
                </:item>

                <:tail :let={count}>
                  <span class="inline-block whitespace-nowrap">
                    and <%= count %> more.
                  </span>
                </:tail>
              </.peek>
            </:col>
            <:empty>
              <div class="flex flex-col items-center justify-center text-center text-slate-500 p-4">
                <div class="pb-4">
                  No resources to display.
                </div>
                <div class="pb-4">
                  <.add_button navigate={~p"/#{@account}/resources/new?site_id=#{@group}"}>
                    Create a Resource
                  </.add_button>
                </div>
                <div>
                  <p>
                    Resources are the endpoints that you want to make available to your clients.
                  </p>
                  <p>
                    Access to them is controlled by Policies.
                  </p>
                </div>
              </div>
            </:empty>
          </.table>
        </div>
      </:content>
    </.section>

    <.danger_zone>
      <:action>
        <.delete_button
          phx-click="delete"
          data-confirm="Are you sure want to delete this gateway group and disconnect all it's gateways?"
        >
          Delete Site
        </.delete_button>
      </:action>
      <:content></:content>
    </.danger_zone>
    """
  end

  def handle_info(%Phoenix.Socket.Broadcast{topic: "gateway_groups:" <> _account_id}, socket) do
    socket =
      push_navigate(socket, to: ~p"/#{socket.assigns.account}/sites/#{socket.assigns.group}")

    {:noreply, socket}
  end

  def handle_event("delete", _params, socket) do
    # TODO: make sure tokens are all deleted too!
    {:ok, _group} = Gateways.delete_group(socket.assigns.group, socket.assigns.subject)
    {:noreply, push_navigate(socket, to: ~p"/#{socket.assigns.account}/sites")}
  end
end
