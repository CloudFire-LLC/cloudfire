defmodule Web.PageComponents do
  use Phoenix.Component
  use Web, :verified_routes
  import Web.CoreComponents

  slot :title, required: true, doc: "The title of the section to be displayed"
  slot :action, required: false, doc: "A slot for action to the right from title"

  slot :content, required: true, doc: "A slot for content of the section" do
    attr :flash, :any, doc: "The flash to be displayed above the content"
  end

  slot :help, required: false, doc: "A slot for help text to be displayed above the content"

  def section(assigns) do
    ~H"""
    <div class="mb-6 bg-white overflow-hidden shadow mx-5 rounded border px-6 pb-6">
      <.header>
        <:title>
          <%= render_slot(@title) %>
        </:title>

        <:actions :if={not Enum.empty?(@action)}>
          <%= for action <- @action do %>
            <%= render_slot(action) %>
          <% end %>
        </:actions>
      </.header>

      <p :for={help <- @help} class="px-1 pb-3">
        <%= render_slot(help) %>
      </p>

      <section :for={content <- @content} class="section-body">
        <div :if={Map.get(content, :flash)} class="mb-4">
          <.flash kind={:info} flash={Map.get(content, :flash)} style="wide" />
          <.flash kind={:error} flash={Map.get(content, :flash)} style="wide" />
        </div>
        <%= render_slot(content) %>
      </section>
    </div>
    """
  end

  slot :action, required: false, doc: "A slot for action to the right of the title"

  slot :content, required: false, doc: "A slot for content of the section" do
    attr :flash, :any, doc: "The flash to be displayed above the content"
  end

  def danger_zone(assigns) do
    ~H"""
    <.section :if={length(@action) > 0}>
      <:title>Danger Zone</:title>

      <:action :for={action <- @action} :if={not Enum.empty?(@action)}>
        <%= render_slot(action) %>
      </:action>

      <:content :for={content <- @content}>
        <%= render_slot(content) %>
      </:content>
    </.section>
    """
  end

  def link_style do
    [
      "text-accent-500",
      "hover:underline"
    ]
  end
end
