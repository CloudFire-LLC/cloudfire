defmodule Domain.ApiTokens.ApiToken.Query do
  use Domain, :query

  def all do
    from(api_tokens in Domain.ApiTokens.ApiToken, as: :api_tokens)
  end

  def by_id(queryable \\ all(), id) do
    where(queryable, [api_tokens: api_tokens], api_tokens.id == ^id)
  end

  def by_actor_id(queryable \\ all(), actor_id) do
    where(queryable, [api_tokens: api_tokens], api_tokens.actor_id == ^actor_id)
  end

  def not_expired(queryable \\ all()) do
    where(queryable, [api_tokens: api_tokens], api_tokens.expires_at >= fragment("NOW()"))
  end
end
