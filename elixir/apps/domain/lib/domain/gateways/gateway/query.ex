defmodule Domain.Gateways.Gateway.Query do
  use Domain, :query

  def all do
    from(gateways in Domain.Gateways.Gateway, as: :gateways)
    |> where([gateways: gateways], is_nil(gateways.deleted_at))
  end

  def by_id(queryable \\ all(), id) do
    where(queryable, [gateways: gateways], gateways.id == ^id)
  end

  def by_ids(queryable \\ all(), ids) do
    where(queryable, [gateways: gateways], gateways.id in ^ids)
  end

  def by_user_id(queryable \\ all(), user_id) do
    where(queryable, [gateways: gateways], gateways.user_id == ^user_id)
  end

  def by_account_id(queryable \\ all(), account_id) do
    where(queryable, [gateways: gateways], gateways.account_id == ^account_id)
  end

  def by_resource_id(queryable \\ all(), resource_id) do
    queryable
    |> with_joined_connections()
    |> where([connections: connections], connections.resource_id == ^resource_id)
  end

  def returning_all(queryable \\ all()) do
    select(queryable, [gateways: gateways], gateways)
  end

  def with_joined_connections(queryable \\ all()) do
    with_named_binding(queryable, :connections, fn queryable, binding ->
      queryable
      |> join(
        :inner,
        [gateways: gateways],
        connections in ^Domain.Resources.Connection.Query.all(),
        on: connections.gateway_group_id == gateways.group_id,
        as: ^binding
      )
    end)
  end

  def with_preloaded_user(queryable \\ all()) do
    with_named_binding(queryable, :user, fn queryable, binding ->
      queryable
      |> join(:inner, [gateways: gateways], user in assoc(gateways, ^binding), as: ^binding)
      |> preload([gateways: gateways, user: user], user: user)
    end)
  end
end
