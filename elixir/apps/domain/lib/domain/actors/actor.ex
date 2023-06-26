defmodule Domain.Actors.Actor do
  use Domain, :schema

  schema "actors" do
    field :type, Ecto.Enum, values: [:account_user, :account_admin_user, :service_account]

    field :name, :string

    has_many :identities, Domain.Auth.Identity

    # belongs_to :group, Domain.Actors.Group
    belongs_to :account, Domain.Accounts.Account

    field :disabled_at, :utc_datetime_usec
    field :deleted_at, :utc_datetime_usec
    timestamps()
  end
end
