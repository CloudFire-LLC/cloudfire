defmodule Domain.Accounts.Account.Query do
  use Domain, :query

  def all do
    from(account in Domain.Accounts.Account, as: :account)
    # |> where([account: account], is_nil(account.deleted_at))
  end

  def by_id(queryable \\ all(), id) do
    where(queryable, [account: account], account.id == ^id)
  end

  def by_slug(queryable \\ all(), slug) do
    where(queryable, [account: account], account.slug == ^slug)
  end
end
