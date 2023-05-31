defmodule Domain.NetworkFixtures do
  alias Domain.Repo
  alias Domain.Network
  alias Domain.AccountsFixtures

  def address_attrs(attrs \\ %{}) do
    attrs = Enum.into(attrs, %{account_id: nil, address: nil, type: nil})

    {account, attrs} =
      Map.pop_lazy(attrs, :account, fn ->
        AccountsFixtures.create_account()
      end)

    {:ok, inet} = Domain.Types.INET.cast(attrs.address)
    type = type(inet.address)
    %{attrs | address: inet, type: type, account_id: account.id}
  end

  defp type(tuple) when tuple_size(tuple) == 4, do: :ipv4
  defp type(tuple) when tuple_size(tuple) == 8, do: :ipv6

  def create_address(attrs \\ %{}) do
    %Network.Address{}
    |> struct(address_attrs(attrs))
    |> Repo.insert!()
  end
end
