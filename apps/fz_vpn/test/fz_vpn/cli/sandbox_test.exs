defmodule FzVpn.CLI.SandboxTest do
  use ExUnit.Case, async: true

  import FzVpn.CLI

  @expected_returned ""

  test "setup" do
    assert cli().setup() == @expected_returned
  end

  test "teardown" do
    assert cli().teardown() == @expected_returned
  end

  test "exec!" do
    assert cli().exec!("dummy") == @expected_returned
  end

  test "set" do
    assert cli().set("dummy") == @expected_returned
  end

  test "show_latest_handshakes" do
    assert cli().show_latest_handshakes() == "4 seconds ago"
  end

  test "show_persistent_keepalive" do
    assert cli().show_persistent_keepalive() == "every 25 seconds"
  end

  test "show_transfer" do
    assert cli().show_transfer() == "4.60 MiB received, 59.21 MiB sent"
  end
end
