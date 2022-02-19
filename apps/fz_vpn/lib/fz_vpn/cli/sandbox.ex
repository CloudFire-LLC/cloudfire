defmodule FzVpn.CLI.Sandbox do
  @moduledoc """
  Sandbox CLI environment for WireGuard CLI operations.
  """

  require Logger

  @wg_show """
  interface: wg-firezone
  public key: Kewtu/udoH+mZzcS0vixCXa8fiMNcurlNy+oQzLZiQk=
  private key: (hidden)
  listening port: 51820

  peer: 1RSUaL+er3+HJM7JW2u5uZDIFNNJkw2nV7dnZyOAK2k=
    endpoint: 73.136.58.38:55433
    allowed ips: 10.3.2.2/32, fd00:3:2::2/128
    latest handshake: 56 minutes, 14 seconds ago
    transfer: 1.21 MiB received, 39.30 MiB sent
  """
  @show_latest_handshakes "4 seconds ago"
  @show_persistent_keepalive "every 25 seconds"
  @show_transfer "4.60 MiB received, 59.21 MiB sent"
  @default_returned ""

  def interface_address, do: "eth0"
  def setup, do: @default_returned
  def teardown, do: @default_returned
  def pubkey(_privkey), do: rand_key()

  def exec!(_cmd) do
    @default_returned
  end

  def set(_conf_str) do
    @default_returned
  end

  def remove_peers do
    @wg_show
    |> String.split("\n")
    |> Enum.filter(fn line ->
      String.contains?(line, "peer")
    end)
    |> Enum.map(fn line ->
      String.replace_leading(line, "peer: ", "")
    end)
    |> Enum.each(fn pubkey ->
      remove_peer(pubkey)
    end)
  end

  def remove_peer(_pubkey) do
    @default_returned
  end

  def set_peer(_pubkey, _allowed_ips) do
    @default_returned
  end

  def show_latest_handshakes, do: @show_latest_handshakes
  def show_persistent_keepalive, do: @show_persistent_keepalive
  def show_transfer, do: @show_transfer

  # Generate extremely fake keys in Sandbox mode
  defp rand_key, do: :crypto.strong_rand_bytes(32) |> Base.encode64()
end
