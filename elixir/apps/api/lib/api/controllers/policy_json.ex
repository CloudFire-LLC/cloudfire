defmodule API.PolicyJSON do
  alias API.Pagination
  alias Domain.Policies

  @doc """
  Renders a list of Policies.
  """
  def index(%{policies: policies, metadata: metadata}) do
    %{
      data: Enum.map(policies, &data/1),
      metadata: Pagination.metadata(metadata)
    }
  end

  @doc """
  Render a single Policy
  """
  def show(%{policy: policy}) do
    %{data: data(policy)}
  end

  defp data(%Policies.Policy{} = policy) do
    %{
      id: policy.id,
      actor_group_id: policy.actor_group_id,
      resource_id: policy.resource_id,
      description: policy.description
    }
  end
end
