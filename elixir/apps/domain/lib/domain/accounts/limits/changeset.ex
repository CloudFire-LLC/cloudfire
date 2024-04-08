defmodule Domain.Accounts.Limits.Changeset do
  use Domain, :changeset
  alias Domain.Accounts.Limits

  @fields ~w[users_count
             monthly_active_users_count
             service_accounts_count
             gateway_groups_count
             account_admin_users_count]a

  def changeset(limits \\ %Limits{}, attrs) do
    limits
    |> cast(attrs, @fields)
    |> validate_number(:users_count, greater_than_or_equal_to: 0)
    |> validate_number(:monthly_active_users_count, greater_than_or_equal_to: 0)
    |> validate_number(:service_accounts_count, greater_than_or_equal_to: 0)
    |> validate_number(:gateway_groups_count, greater_than_or_equal_to: 0)
    |> validate_number(:account_admin_users_count, greater_than_or_equal_to: 0)
  end
end
