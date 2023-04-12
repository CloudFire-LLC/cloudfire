defmodule API.Gateway.Channel do
  use API, :channel
  alias Domain.Gateways.Presence

  @impl true
  def join("gateway", _payload, socket) do
    send(self(), :after_join)
    {:ok, socket}
  end

  @impl true
  def handle_info(:after_join, socket) do
    {:ok, _} =
      Presence.track(socket, socket.assigns.gateway.id, %{
        online_at: System.system_time(:second)
      })

    {:noreply, socket}
  end
end
