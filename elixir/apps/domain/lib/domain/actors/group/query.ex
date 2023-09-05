defmodule Domain.Actors.Group.Query do
  use Domain, :query

  def all do
    from(groups in Domain.Actors.Group, as: :groups)
    |> where([groups: groups], is_nil(groups.deleted_at))
  end

  def by_id(queryable \\ all(), id)

  def by_id(queryable, {:in, ids}) do
    where(queryable, [groups: groups], groups.id in ^ids)
  end

  def by_id(queryable, id) do
    where(queryable, [groups: groups], groups.id == ^id)
  end

  def by_account_id(queryable \\ all(), account_id) do
    where(queryable, [groups: groups], groups.account_id == ^account_id)
  end

  def by_provider_id(queryable \\ all(), provider_id) do
    where(queryable, [groups: groups], groups.provider_id == ^provider_id)
  end

  def by_not_empty_provider_id(queryable \\ all()) do
    where(queryable, [groups: groups], not is_nil(groups.provider_id))
  end

  def by_provider_identifier(queryable \\ all(), provider_identifier)

  def by_provider_identifier(queryable, {:in, provider_identifiers}) do
    where(queryable, [groups: groups], groups.provider_identifier in ^provider_identifiers)
  end

  def by_provider_identifier(queryable, provider_identifier) do
    where(queryable, [groups: groups], groups.provider_identifier == ^provider_identifier)
  end

  def group_by_provider_id(queryable \\ all()) do
    queryable
    |> group_by([groups: groups], groups.provider_id)
    |> where([groups: groups], not is_nil(groups.provider_id))
    |> select([groups: groups], %{
      provider_id: groups.provider_id,
      count: count(groups.id)
    })
  end

  def preload_few_actors_for_each_group(queryable \\ all(), limit) do
    queryable
    |> with_joined_memberships(limit)
    |> with_joined_actors()
    |> with_joined_actor_counts()
    |> select([groups: groups, actors: actors, actor_counts: actor_counts], %{
      id: groups.id,
      count: actor_counts.count,
      item: actors
    })
  end

  def with_joined_memberships(queryable) do
    join(queryable, :left, [groups: groups], memberships in assoc(groups, :memberships),
      as: :memberships
    )
  end

  def with_joined_memberships(queryable, limit) do
    subquery =
      Domain.Actors.Membership.Query.all()
      |> where([memberships: memberships], memberships.group_id == parent_as(:groups).id)
      # we need second join to exclude soft deleted actors before applying a limit
      |> join(:inner, [memberships: memberships], actors in ^Domain.Actors.Actor.Query.all(),
        on: actors.id == memberships.actor_id
      )
      |> select([memberships: memberships], memberships.actor_id)
      |> limit(^limit)

    join(queryable, :cross_lateral, [groups: groups], memberships in subquery(subquery),
      as: :memberships
    )
  end

  def with_joined_actor_counts(queryable) do
    subquery =
      Domain.Actors.Membership.Query.count_actors_by_group_id()
      |> where([memberships: memberships], memberships.group_id == parent_as(:groups).id)

    join(queryable, :cross_lateral, [groups: groups], actor_counts in subquery(subquery),
      as: :actor_counts
    )
  end

  def with_joined_actors(queryable \\ all()) do
    join(queryable, :left, [memberships: memberships], actors in ^Domain.Actors.Actor.Query.all(),
      on: actors.id == memberships.actor_id,
      as: :actors
    )
  end

  def lock(queryable \\ all()) do
    lock(queryable, "FOR UPDATE")
  end
end
