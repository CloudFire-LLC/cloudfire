defmodule API.GatewayJSON do
  alias API.Pagination
  alias Domain.Gateways

  @doc """
  Renders a list of Gateways.
  """
  def index(%{gateways: gateways, metadata: metadata}) do
    %{
      data: Enum.map(gateways, &data/1),
      metadata: Pagination.metadata(metadata)
    }
  end

  @doc """
  Render a single Gateway
  """
  def show(%{gateway: gateway}) do
    %{data: data(gateway)}
  end

  defp data(%Gateways.Gateway{} = gateway) do
    %{
      id: gateway.id,
      name: gateway.name,
      ipv4: gateway.ipv4,
      ipv6: gateway.ipv6,
      online: gateway.online?
    }
  end
end
