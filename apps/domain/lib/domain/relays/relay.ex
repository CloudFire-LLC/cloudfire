defmodule Domain.Relays.Relay do
  use Domain, :schema

  schema "relays" do
    field :ipv4, Domain.Types.IP
    field :ipv6, Domain.Types.IP

    field :port, :integer, default: 3478

    field :last_seen_user_agent, :string
    field :last_seen_remote_ip, Domain.Types.IP
    field :last_seen_version, :string
    field :last_seen_at, :utc_datetime_usec

    field :stamp_secret, :string, virtual: true

    belongs_to :account, Domain.Accounts.Account
    belongs_to :group, Domain.Relays.Group
    belongs_to :token, Domain.Relays.Token

    field :deleted_at, :utc_datetime_usec
    timestamps()
  end
end
