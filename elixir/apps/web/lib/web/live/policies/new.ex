defmodule Web.Policies.New do
  use Web, :live_view
  alias Domain.{Resources, Actors, Policies}

  def mount(params, _session, socket) do
    # TODO: unify this dropdown and the one we use for live table filters
    resources = Resources.all_resources!(socket.assigns.subject, preload: [:gateway_groups])
    # TODO: unify this dropdown and the one we use for live table filters
    actor_groups = Actors.all_groups!(socket.assigns.subject, preload: :provider)
    form = to_form(Policies.new_policy(%{}, socket.assigns.subject))

    socket =
      assign(socket,
        resources: resources,
        actor_groups: actor_groups,
        params: Map.take(params, ["site_id"]),
        resource_id: params["resource_id"],
        actor_group_id: params["actor_group_id"],
        page_title: "New Policy",
        form: form
      )

    {:ok, socket, temporary_assigns: [form: %Phoenix.HTML.Form{}]}
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/policies"}>Policies</.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/policies/new"}>Add Policy</.breadcrumb>
    </.breadcrumbs>
    <.section>
      <:title>
        Add Policy
      </:title>
      <:content>
        <div class="max-w-2xl px-4 py-8 mx-auto lg:py-16">
          <h2 class="mb-4 text-xl text-neutral-900">Policy details</h2>
          <div
            :if={@actor_groups == []}
            class={[
              "p-4 text-sm flash-error",
              "text-red-800 bg-red-50"
            ]}
            role="alert"
          >
            <p class="text-sm leading-6">
              <.icon name="hero-exclamation-circle-mini" class="h-4 w-4" />
              You have no groups to create policies for. You can create a group <.link
                navigate={~p"/#{@account}/groups/new"}
                class={link_style()}
              >here</.link>.
            </p>
          </div>

          <.form :if={@actor_groups != []} for={@form} phx-submit="submit" phx-change="validate">
            <.base_error form={@form} field={:base} />
            <div class="grid gap-4 mb-4 sm:grid-cols-1 sm:gap-6 sm:mb-6">
              <.input
                :if={not is_nil(@actor_group_id)}
                type="hidden"
                field={@form[:actor_group_id]}
                value={@actor_group_id}
              />
              <.input
                field={@form[:actor_group_id]}
                label="Group"
                type="group_select"
                options={Web.Groups.Components.select_options(@actor_groups)}
                value={@actor_group_id || @form[:actor_group_id].value}
                disabled={not is_nil(@actor_group_id)}
                required
              />

              <.input
                :if={not is_nil(@resource_id)}
                type="hidden"
                field={@form[:resource_id]}
                value={@resource_id}
              />
              <.input
                field={@form[:resource_id]}
                label="Resource"
                type="select"
                options={
                  Enum.map(@resources, fn resource ->
                    group_names = resource.gateway_groups |> Enum.map(& &1.name)

                    [
                      key: "#{resource.name} - #{Enum.join(group_names, ",")}",
                      value: resource.id
                    ]
                  end)
                }
                value={@resource_id || @form[:resource_id].value}
                disabled={not is_nil(@resource_id)}
                required
              />
              <.input
                field={@form[:description]}
                type="textarea"
                label="Description"
                placeholder="Enter a reason for creating a policy here"
                phx-debounce="300"
              />
            </div>

            <.submit_button phx-disable-with="Creating Policy..." class="w-full">
              Create Policy
            </.submit_button>
          </.form>
        </div>
      </:content>
    </.section>
    """
  end

  def handle_event("validate", %{"policy" => policy_params}, socket) do
    form =
      policy_params
      |> put_default_policy_params(socket)
      |> Policies.new_policy(socket.assigns.subject)
      |> Map.put(:action, :validate)
      |> to_form()

    {:noreply, assign(socket, form: form)}
  end

  def handle_event("submit", %{"policy" => policy_params}, socket) do
    policy_params = put_default_policy_params(policy_params, socket)

    with {:ok, policy} <- Policies.create_policy(policy_params, socket.assigns.subject) do
      if site_id = socket.assigns.params["site_id"] do
        {:noreply,
         push_navigate(socket, to: ~p"/#{socket.assigns.account}/sites/#{site_id}?#resources")}
      else
        {:noreply, push_navigate(socket, to: ~p"/#{socket.assigns.account}/policies/#{policy}")}
      end
    else
      {:error, %Ecto.Changeset{} = changeset} ->
        form = to_form(changeset)
        {:noreply, assign(socket, form: form)}
    end
  end

  defp put_default_policy_params(attrs, socket) do
    if resource_id = socket.assigns.resource_id do
      Map.put(attrs, "resource_id", resource_id)
    else
      attrs
    end
  end
end
