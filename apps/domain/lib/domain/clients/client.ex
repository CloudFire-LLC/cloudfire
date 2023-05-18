defmodule Domain.Clients.Client do
  use Domain, :schema

  schema "clients" do
    field :external_id, :string

    field :name, :string

    field :public_key, :string

    field :ipv4, Domain.Types.IP
    field :ipv6, Domain.Types.IP

    field :last_seen_user_agent, :string
    field :last_seen_remote_ip, Domain.Types.IP
    field :last_seen_version, :string
    field :last_seen_at, :utc_datetime_usec

    belongs_to :account, Domain.Accounts.Account
    belongs_to :actor, Domain.Actors.Actor
    belongs_to :identity, Domain.Auth.Identity

    field :deleted_at, :utc_datetime_usec
    timestamps()
  end
end
