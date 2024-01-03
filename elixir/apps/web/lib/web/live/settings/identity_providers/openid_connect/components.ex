defmodule Web.Settings.IdentityProviders.OpenIDConnect.Components do
  use Web, :component_library

  def provider_form(assigns) do
    ~H"""
    <div class="max-w-2xl px-4 py-8 mx-auto lg:py-12">
      <.form for={@form} phx-change={:change} phx-submit={:submit}>
        <.step>
          <:title>Step 1. Create OAuth application in your identity provider</:title>
          <:content>
            <p class="mb-4">
              Ensure the following scopes are added to the OAuth application:
            </p>
            <.code_block
              :for={scope <- [:openid, :email, :profile]}
              id={"scope-#{scope}"}
              class="w-full text-xs mb-4 whitespace-pre-line rounded"
              phx-no-format
            ><%= scope %></.code_block>
            <p class="mb-4">
              Ensure the OAuth application has the following redirect URLs whitelisted:
            </p>
            <.code_block
              :for={
                {type, redirect_url} <- [
                  sign_in: url(~p"/#{@account.id}/sign_in/providers/#{@id}/handle_callback"),
                  connect:
                    url(
                      ~p"/#{@account.id}/settings/identity_providers/openid_connect/#{@id}/handle_callback"
                    )
                ]
              }
              id={"redirect_url-#{type}"}
              class="w-full text-xs mb-4 whitespace-pre-line rounded"
              phx-no-format
            ><%= redirect_url %></.code_block>
          </:content>
        </.step>

        <.step>
          <:title>Step 2. Configure Firezone</:title>
          <:content>
            <.base_error form={@form} field={:base} />

            <div class="grid gap-4 mb-4 sm:grid-cols-1 sm:gap-6 sm:mb-6">
              <div>
                <.input
                  label="Name"
                  autocomplete="off"
                  field={@form[:name]}
                  placeholder="Name this identity provider"
                  required
                />
                <p class="mt-2 text-xs text-neutral-500">
                  A human-friendly name for this identity provider. This will be displayed to end-users.
                </p>
              </div>

              <.inputs_for :let={adapter_config_form} field={@form[:adapter_config]}>
                <%= # TODO: Can these be removed? %>
                <.input type="hidden" field={adapter_config_form[:response_type]} value="code" />
                <.input type="hidden" field={adapter_config_form[:scope]} />

                <div>
                  <.input
                    label="Client ID"
                    autocomplete="off"
                    field={adapter_config_form[:client_id]}
                    placeholder="Client ID from your identity provider"
                    required
                  />
                </div>

                <div>
                  <.input
                    label="Client secret"
                    autocomplete="off"
                    field={adapter_config_form[:client_secret]}
                    placeholder="Client secret from your identity provider"
                    required
                  />
                </div>

                <div>
                  <.input
                    label="Discovery Document URI"
                    autocomplete="off"
                    field={adapter_config_form[:discovery_document_uri]}
                    placeholder="https://example.com/.well-known/openid-configuration"
                    required
                  />
                  <p class="mt-2 text-xs text-neutral-500">
                    The URI to the OpenID Connect discovery document for your identity provider.
                  </p>
                </div>
              </.inputs_for>
            </div>

            <.submit_button>
              Connect Identity Provider
            </.submit_button>
          </:content>
        </.step>
      </.form>
    </div>
    """
  end
end
