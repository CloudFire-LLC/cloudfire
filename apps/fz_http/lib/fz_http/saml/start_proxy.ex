defmodule FzHttp.SAML.StartProxy do
  @moduledoc """
  This proxy starts Samly.Provider with proper configs
  """

  def child_spec(arg) do
    %{id: __MODULE__, start: {__MODULE__, :start_link, [arg]}}
  end

  def start_link(:test) do
    start_link(nil)
  end

  def start_link(_) do
    providers = FzHttp.Configurations.get!(:saml_identity_providers)
    samly = Samly.Provider.start_link()

    FzHttp.Config.fetch_env!(:samly, Samly.Provider)
    |> set_service_provider()
    |> set_identity_providers(providers)
    |> refresh()

    samly
  end

  def set_service_provider(samly_configs) do
    entity_id = FzHttp.Config.fetch_env!(:fz_http, :saml_entity_id)
    keyfile = FzHttp.Config.fetch_env!(:fz_http, :saml_keyfile_path)
    certfile = FzHttp.Config.fetch_env!(:fz_http, :saml_certfile_path)

    # Only one service provider definition: us.
    Keyword.put(samly_configs, :service_providers, [
      %{
        id: "firezone",
        entity_id: entity_id,
        certfile: certfile,
        keyfile: keyfile
      }
    ])
  end

  def set_identity_providers(samly_configs, providers) do
    external_url = FzHttp.Config.fetch_env!(:fz_http, :external_url)

    identity_providers =
      providers
      |> Enum.map(fn provider ->
        # XXX We should not set default values here, instead they should be part
        # of the changeset and always valid in database
        %{
          id: provider.id,
          sp_id: "firezone",
          metadata: provider.metadata,
          base_url: provider.base_url || Path.join(external_url, "/auth/saml"),
          sign_requests: provider.sign_requests,
          sign_metadata: provider.sign_metadata,
          signed_assertion_in_resp: provider.signed_assertion_in_resp,
          signed_envelopes_in_resp: provider.signed_envelopes_in_resp
        }
      end)

    Keyword.put(samly_configs, :identity_providers, identity_providers)
  end

  def refresh(samly_configs) do
    FzHttp.Config.put_env(:samly, Samly.Provider, samly_configs)
    Samly.Provider.refresh_providers()
  end

  # XXX: This should be removed when the configurations singleton record is removed.
  #
  # Needed to prevent the test suite from recursively restarting this module as
  # it put!()'s mock data
  if Mix.env() == :test do
    def restart, do: :ignore
  else
    def restart do
      :ok = Supervisor.terminate_child(FzHttp.Supervisor, __MODULE__)
      Supervisor.restart_child(FzHttp.Supervisor, __MODULE__)
    end
  end
end
