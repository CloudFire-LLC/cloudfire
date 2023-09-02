defmodule Domain.Actors.Actor.Query do
  use Domain, :query

  def all do
    from(actors in Domain.Actors.Actor, as: :actors)
    |> where([actors: actors], is_nil(actors.deleted_at))
  end

  def by_id(queryable \\ all(), id)

  def by_id(queryable, {:in, ids}) do
    where(queryable, [actors: actors], actors.id in ^ids)
  end

  def by_id(queryable, {:not, id}) do
    where(queryable, [actors: actors], actors.id != ^id)
  end

  def by_id(queryable, id) do
    where(queryable, [actors: actors], actors.id == ^id)
  end

  def by_account_id(queryable \\ all(), account_id) do
    where(queryable, [actors: actors], actors.account_id == ^account_id)
  end

  def by_type(queryable \\ all(), type) do
    where(queryable, [actors: actors], actors.type == ^type)
  end

  def not_disabled(queryable \\ all()) do
    where(queryable, [actors: actors], is_nil(actors.disabled_at))
  end

  def preload_few_groups_for_each_actor(queryable \\ all(), limit) do
    queryable
    |> with_joined_memberships(limit)
    |> with_joined_groups()
    |> with_joined_group_counts()
    |> select([actors: actors, groups: groups, group_counts: group_counts], %{
      id: actors.id,
      count: group_counts.count,
      item: groups
    })
  end

  def with_joined_memberships(queryable, limit) do
    subquery =
      Domain.Actors.Membership.Query.all()
      |> where([memberships: memberships], memberships.actor_id == parent_as(:actors).id)
      # we need second join to exclude soft deleted actors before applying a limit
      |> join(:inner, [memberships: memberships], groups in ^Domain.Actors.Group.Query.all(),
        on: groups.id == memberships.group_id
      )
      |> select([memberships: memberships], memberships.group_id)
      |> limit(^limit)

    join(queryable, :cross_lateral, [actors: actors], memberships in subquery(subquery),
      as: :memberships
    )
  end

  def with_joined_group_counts(queryable) do
    subquery =
      Domain.Actors.Membership.Query.count_groups_by_actor_id()
      |> where([memberships: memberships], memberships.actor_id == parent_as(:actors).id)

    join(queryable, :cross_lateral, [actors: actors], group_counts in subquery(subquery),
      as: :group_counts
    )
  end

  def with_joined_groups(queryable \\ all()) do
    join(queryable, :left, [memberships: memberships], groups in ^Domain.Actors.Group.Query.all(),
      on: groups.id == memberships.group_id,
      as: :groups
    )
  end

  def lock(queryable \\ all()) do
    lock(queryable, "FOR UPDATE")
  end

  def with_assoc(queryable \\ all(), qual \\ :left, assoc) do
    with_named_binding(queryable, assoc, fn query, binding ->
      join(query, qual, [actors: actors], a in assoc(actors, ^binding), as: ^binding)
    end)
  end
end
