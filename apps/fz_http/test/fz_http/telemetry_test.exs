defmodule FzHttp.TelemetryTest do
  use FzHttp.DataCase, async: true

  alias FzHttp.Telemetry

  describe "user" do
    setup :create_user

    test "count" do
      ping_data = Telemetry.ping_data()

      assert ping_data[:user_count] == 1
    end

    test "count mfa", %{user: user} do
      {:ok, [user: other_user]} = create_user(%{})
      {:ok, _method} = create_method(user, type: :totp)
      {:ok, _method} = create_method(other_user, type: :portable)
      ping_data = Telemetry.ping_data()

      assert ping_data[:users_with_mfa] == 2
      assert ping_data[:users_with_mfa_totp] == 1
    end
  end

  describe "device" do
    setup [:create_devices, :create_other_user_device]

    test "count" do
      ping_data = Telemetry.ping_data()

      assert ping_data[:device_count] == 6
    end

    test "max count for users" do
      ping_data = Telemetry.ping_data()

      assert ping_data[:max_devices_for_users] == 5
    end
  end

  describe "auth" do
    test "count openid providers" do
      ping_data = Telemetry.ping_data()

      assert ping_data[:openid_providers] == 7
    end

    test "disable vpn on oidc error enabled" do
      stub_conf(:disable_vpn_on_oidc_error, true)

      ping_data = Telemetry.ping_data()

      assert ping_data[:disable_vpn_on_oidc_error]
    end

    test "disable vpn on oidc error disabled" do
      stub_conf(:disable_vpn_on_oidc_error, false)

      ping_data = Telemetry.ping_data()

      refute ping_data[:disable_vpn_on_oidc_error]
    end

    test "local authentication enabled" do
      stub_conf(:local_auth_enabled, true)

      ping_data = Telemetry.ping_data()

      assert ping_data[:local_authentication]
    end

    test "local authentication disabled" do
      stub_conf(:local_auth_enabled, false)

      ping_data = Telemetry.ping_data()

      refute ping_data[:local_authentication]
    end

    test "unprivileged device management enabled" do
      stub_conf(:allow_unprivileged_device_management, true)

      ping_data = Telemetry.ping_data()

      assert ping_data[:unprivileged_device_management]
    end

    test "unprivileged device configuration enabled" do
      stub_conf(:allow_unprivileged_device_configuration, true)

      ping_data = Telemetry.ping_data()

      assert ping_data[:unprivileged_device_configuration]
    end

    test "unprivileged device configuration disabled" do
      stub_conf(:allow_unprivileged_device_configuration, false)

      ping_data = Telemetry.ping_data()

      refute ping_data[:unprivileged_device_configuration]
    end
  end

  describe "database" do
    test "local hostname" do
      FzHttp.Config.maybe_put_env_override(FzHttp.Repo, hostname: "localhost")

      ping_data = Telemetry.ping_data()

      refute ping_data[:external_database]
    end

    test "local url" do
      FzHttp.Config.maybe_put_env_override(FzHttp.Repo, url: "postgres://127.0.0.1")

      ping_data = Telemetry.ping_data()

      refute ping_data[:external_database]
    end

    test "external hostname" do
      FzHttp.Config.maybe_put_env_override(FzHttp.Repo, hostname: "firezone.dev")

      ping_data = Telemetry.ping_data()

      assert ping_data[:external_database]
    end

    test "external url" do
      FzHttp.Config.maybe_put_env_override(FzHttp.Repo, url: "postgres://firezone.dev")

      ping_data = Telemetry.ping_data()

      assert ping_data[:external_database]
    end
  end

  describe "email" do
    test "outbound set" do
      FzHttp.Config.maybe_put_env_override(FzHttpWeb.Mailer, from_email: "test@firezone.dev")

      ping_data = Telemetry.ping_data()

      assert ping_data[:outbound_email]
    end

    test "outbound unset" do
      FzHttp.Config.maybe_put_env_override(FzHttpWeb.Mailer, from_email: nil)

      ping_data = Telemetry.ping_data()

      refute ping_data[:outbound_email]
    end
  end
end
