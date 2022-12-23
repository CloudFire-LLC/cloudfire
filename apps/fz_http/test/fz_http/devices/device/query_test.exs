defmodule FzHttp.Devices.Device.QueryTest do
  use FzHttp.DataCase, async: true
  import FzHttp.Devices.Device.Query
  alias FzHttp.DevicesFixtures

  describe "next_available_address/3" do
    test "selects available IPv4 in CIDR range at the offset" do
      cidr = string_to_inet("10.3.2.0/29")
      gateway_ip = string_to_inet("10.3.2.0")
      offset = 3

      queryable = next_available_address(cidr, offset, [gateway_ip])

      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 3}}
    end

    test "skips addresses taken by the gateway" do
      cidr = string_to_inet("10.3.2.0/29")
      gateway_ip = string_to_inet("10.3.2.3")
      offset = 3

      queryable = next_available_address(cidr, offset, [gateway_ip])

      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 4}}
    end

    test "forward scans available address after offset it it's assigned to a device" do
      cidr = string_to_inet("10.3.2.0/29")
      gateway_ip = string_to_inet("10.3.2.0")
      offset = 3

      queryable = next_available_address(cidr, offset, [gateway_ip])

      DevicesFixtures.device(%{ipv4: "10.3.2.3"})
      DevicesFixtures.device(%{ipv4: "10.3.2.4"})
      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 5}}

      DevicesFixtures.device(%{ipv4: "10.3.2.5"})
      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 6}}
    end

    test "backward scans available address if forward scan found not available IPs" do
      cidr = string_to_inet("10.3.2.0/29")
      gateway_ip = string_to_inet("10.3.2.0")
      offset = 5

      queryable = next_available_address(cidr, offset, [gateway_ip])

      DevicesFixtures.device(%{ipv4: "10.3.2.5"})
      DevicesFixtures.device(%{ipv4: "10.3.2.6"})
      # Notice: end of range is 10.3.2.7
      # but it's a broadcast address that we don't allow to assign
      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 4}}

      DevicesFixtures.device(%{ipv4: "10.3.2.4"})
      assert Repo.one(queryable) == %Postgrex.INET{address: {10, 3, 2, 3}}
    end

    test "selects nothing when CIDR range is exhausted" do
      cidr = string_to_inet("10.3.2.0/30")
      gateway_ip = string_to_inet("10.3.2.1")
      offset = 1

      DevicesFixtures.device(%{ipv4: "10.3.2.2"})
      queryable = next_available_address(cidr, offset, [gateway_ip])
      assert is_nil(Repo.one(queryable))

      DevicesFixtures.device(%{ipv4: "10.3.2.1"})
      queryable = next_available_address(cidr, offset, [])
      assert is_nil(Repo.one(queryable))

      # Notice: real start of range is 10.3.2.0,
      # but it's a typical gateway address that we don't allow to assign
    end

    test "prevents two concurrent transactions from acquiring the same address" do
      cidr = string_to_inet("10.3.2.0/29")
      gateway_ip = string_to_inet("10.3.2.3")
      offset = 3

      queryable = next_available_address(cidr, offset, [gateway_ip])

      test_pid = self()

      spawn(fn ->
        Ecto.Adapters.SQL.Sandbox.unboxed_run(Repo, fn ->
          Repo.transaction(fn ->
            ip = Repo.one(queryable)
            send(test_pid, {:ip, ip})
            Process.sleep(200)
          end)
        end)
      end)

      ip1 = Repo.one(queryable)
      assert_receive {:ip, ip2}, 1_000

      assert Enum.sort([ip1, ip2]) ==
               Enum.sort([
                 %Postgrex.INET{address: {10, 3, 2, 4}},
                 %Postgrex.INET{address: {10, 3, 2, 5}}
               ])
    end

    test "selects available IPv6 in CIDR range at the offset" do
      cidr = string_to_inet("fd00::3:2:0/120")
      gateway_ip = string_to_inet("fd00::3:2:3")
      offset = 3

      queryable = next_available_address(cidr, offset, [gateway_ip])

      assert Repo.one(queryable) == %Postgrex.INET{address: {64_768, 0, 0, 0, 0, 3, 2, 4}}
    end

    test "selects nothing when IPv6 CIDR range is exhausted" do
      cidr = string_to_inet("fd00::3:2:0/126")
      gateway_ip = string_to_inet("fd00::3:2:1")
      offset = 3

      DevicesFixtures.device(%{ipv6: "fd00::3:2:2"})

      queryable = next_available_address(cidr, offset, [gateway_ip])
      assert is_nil(Repo.one(queryable))
    end
  end

  defp string_to_inet(string) do
    {:ok, inet} = EctoNetwork.INET.cast(string)
    inet
  end
end
