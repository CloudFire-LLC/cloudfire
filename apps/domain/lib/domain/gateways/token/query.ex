defmodule Domain.Gateways.Token.Query do
  use Domain, :query

  def all do
    from(token in Domain.Gateways.Token, as: :token)
    |> where([token: token], is_nil(token.deleted_at))
  end

  def by_id(queryable \\ all(), id) do
    where(queryable, [token: token], token.id == ^id)
  end

  def by_group_id(queryable \\ all(), group_id) do
    where(queryable, [token: token], token.group_id == ^group_id)
  end
end
