defmodule Web.Groups.Index do
  use Web, :live_view
  alias Domain.Actors

  def mount(_params, _session, socket) do
    socket =
      socket
      |> assign(page_title: "Groups")
      |> assign_live_table("groups",
        query_module: Actors.Group.Query,
        sortable_fields: [
          {:groups, :name},
          {:groups, :inserted_at}
        ],
        callback: &handle_groups_update!/2
      )

    {:ok, socket}
  end

  def handle_params(params, uri, socket) do
    socket = handle_live_tables_params(socket, params, uri)
    {:noreply, socket}
  end

  def handle_groups_update!(socket, list_opts) do
    list_opts = Keyword.put(list_opts, :preload, [:provider, created_by_identity: [:actor]])

    with {:ok, groups, metadata} <- Actors.list_groups(socket.assigns.subject, list_opts) do
      {:ok, group_actors} = Actors.peek_group_actors(groups, 3, socket.assigns.subject)

      assign(socket,
        groups: groups,
        groups_metadata: metadata,
        group_actors: group_actors
      )
    else
      {:error, :invalid_cursor} -> raise Web.LiveErrors.InvalidRequestError
      {:error, {:unknown_filter, _metadata}} -> raise Web.LiveErrors.InvalidRequestError
      {:error, {:invalid_type, _metadata}} -> raise Web.LiveErrors.InvalidRequestError
      {:error, {:invalid_value, _metadata}} -> raise Web.LiveErrors.InvalidRequestError
      {:error, _reason} -> raise Web.LiveErrors.NotFoundError
    end
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/groups"}><%= @page_title %></.breadcrumb>
    </.breadcrumbs>

    <.section>
      <:title>
        Groups
      </:title>
      <:action>
        <.add_button navigate={~p"/#{@account}/groups/new"}>
          Add Group
        </.add_button>
      </:action>
      <:content>
        <.flash_group flash={@flash} />
        <.live_table
          id="groups"
          rows={@groups}
          row_id={&"user-#{&1.id}"}
          filters={@filters_by_table_id["groups"]}
          filter={@filter_form_by_table_id["groups"]}
          ordered_by={@order_by_table_id["groups"]}
          metadata={@groups_metadata}
        >
          <:col :let={group} field={{:groups, :name}} label="NAME" class="w-2/4">
            <.link navigate={~p"/#{@account}/groups/#{group.id}"} class={[link_style()]}>
              <%= group.name %>
            </.link>

            <span :if={Actors.group_deleted?(group)} class="text-xs text-neutral-100">
              (deleted)
            </span>
          </:col>
          <:col :let={group} label="ACTORS">
            <.peek peek={Map.fetch!(@group_actors, group.id)}>
              <:empty>
                None
              </:empty>

              <:separator>
                <span class="pr-1">,</span>
              </:separator>

              <:item :let={actor}>
                <.link navigate={~p"/#{@account}/actors/#{actor}"} class={[link_style()]}>
                  <%= actor.name %>
                </.link>
              </:item>

              <:tail :let={count}>
                <span class="pl-1">
                  and <%= count %> more.
                </span>
              </:tail>
            </.peek>
          </:col>
          <:col :let={group} field={{:groups, :inserted_at}} label="SOURCE">
            <.created_by account={@account} schema={group} />
          </:col>
          <:empty>
            <div class="flex justify-center text-center text-neutral-500 p-4">
              <div class="w-auto pb-4">
                No groups to display.
                <.link class={[link_style()]} navigate={~p"/#{@account}/groups/new"}>
                  Add a group manually
                </.link>
                or
                <.link class={[link_style()]} navigate={~p"/#{@account}/settings/identity_providers"}>
                  go to settings
                </.link>
                to sync groups from an identity provider.
              </div>
            </div>
          </:empty>
        </.live_table>
      </:content>
    </.section>
    """
  end

  def handle_event(event, params, socket) when event in ["paginate", "order_by", "filter"],
    do: handle_live_table_event(event, params, socket)
end
