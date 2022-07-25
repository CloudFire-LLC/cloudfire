defmodule FzHttpWeb.UserSocket do
  use Phoenix.Socket

  alias FzHttp.Users

  require Logger

  ## Channels
  # channel "room:*", FzHttpWeb.RoomChannel
  channel "notification:session", FzHttpWeb.NotificationChannel

  # Socket params are passed from the client and can
  # be used to verify and authenticate a user. After
  # verification, you can put default assigns into
  # the socket that will be set for all channels, ie
  #
  #     {:ok, assign(socket, :user_id, verified_user_id)}
  #
  # To deny connection, return `:error`.
  #
  # See `Phoenix.Token` documentation for examples in
  # performing token verification on connect.
  def connect(%{"token" => token}, socket, connect_info) do
    ip = get_ip_address(connect_info)

    case Phoenix.Token.verify(socket, "user auth", token, max_age: 86_400) do
      {:ok, user_id} ->
        {:ok,
         socket
         |> assign(:current_user, Users.get_user!(user_id))
         |> assign(:remote_ip, ip)}

      {:error, _} ->
        :error
    end
  end

  # Socket id's are topics that allow you to identify all sockets for a given user:
  #
  #     def id(socket), do: "user_socket:#{socket.assigns.user_id}"
  #
  # Would allow you to broadcast a "disconnect" event and terminate
  # all active sockets and channels for a given user:
  #
  #     FzHttpWeb.Endpoint.broadcast("user_socket:#{user.id}", "disconnect", %{})
  #
  # Returning `nil` makes this socket anonymous.
  # def id(_socket), do: nil
  def id(socket), do: "user_socket:#{socket.assigns.current_user.id}"

  defp get_ip_address(%{x_headers: headers_list}) when length(headers_list) > 0 do
    header =
      Enum.find(headers_list, fn {key, _val} -> key == "x-real-ip" end) ||
        Enum.find(headers_list, fn {key, _val} -> key == "x-forwarded-for" end)

    case header do
      {_key, value} -> value
      _ -> nil
    end
  end

  defp get_ip_address(%{peer_data: %{address: address}}) do
    convert_ip(address)

    address
    |> Tuple.to_list()
    |> Enum.join(".")
  end

  # IPv4
  defp convert_ip({_, _, _, _} = address) do
    address
    |> Tuple.to_list()
    |> Enum.join(".")
  end

  # IPv6
  defp convert_ip(address) do
    address
    |> Tuple.to_list()
    |> Enum.join(":")
  end
end
