defmodule FzHttpWeb.SettingLive.SecurityTest do
  use FzHttpWeb.ConnCase, async: true

  alias FzHttpWeb.SettingLive.Security
  import FzHttp.SAMLConfigFixtures
  import Mox

  describe "authenticated mount" do
    test "loads the active sessions table", %{admin_conn: conn} do
      path = ~p"/settings/security"
      {:ok, _view, html} = live(conn, path)

      assert html =~ "<h4 class=\"title is-4\">Authentication</h4>"
    end

    test "selects the chosen option", %{admin_conn: conn} do
      path = ~p"/settings/security"
      {:ok, _view, html} = live(conn, path)
      assert html =~ ~s|<option selected="selected" value="0">Never</option>|

      FzHttp.Sites.get_site!() |> FzHttp.Sites.update_site(%{vpn_session_duration: 3_600})

      {:ok, _view, html} = live(conn, path)
      assert html =~ ~s|<option selected="selected" value="3600">Every Hour</option>|
    end
  end

  describe "unauthenticated mount" do
    test "redirects to not authorized", %{unauthed_conn: conn} do
      path = ~p"/settings/security"
      expected_path = ~p"/"

      assert {:error, {:redirect, %{to: ^expected_path}}} = live(conn, path)
    end
  end

  describe "session_duration_options/0" do
    @expected_durations [
      Never: 0,
      Once: 2_147_483_647,
      "Every Hour": 3_600,
      "Every Day": 86_400,
      "Every Week": 604_800,
      "Every 30 Days": 2_592_000,
      "Every 90 Days": 7_776_000
    ]

    test "displays the correct session duration integers" do
      assert Security.session_duration_options() == @expected_durations
    end
  end

  describe "toggles" do
    import FzHttp.ConfigurationsFixtures

    setup %{conf_key: key, conf_val: val} do
      stub_conf(key, val)
      {:ok, path: ~p"/settings/security"}
    end

    for {key, val} <- [
          {:local_auth_enabled, true},
          {:allow_unprivileged_device_management, true},
          {:allow_unprivileged_device_configuration, true},
          {:disable_vpn_on_oidc_error, true}
        ] do
      @tag conf_key: key, conf_val: val
      test "toggle #{key} when value in db is true", %{admin_conn: conn, path: path} do
        {:ok, view, _html} = live(conn, path)
        html = view |> element("input[phx-value-config=#{unquote(key)}}]") |> render()
        assert html =~ "checked"

        expect(Cache.Mock, :put!, fn unquote(key), v ->
          assert v == false
        end)

        view |> element("input[phx-value-config=#{unquote(key)}]") |> render_click()
      end
    end

    for {key, val} <- [
          {:local_auth_enabled, nil},
          {:allow_unprivileged_device_management, nil},
          {:allow_unprivileged_device_configuration, nil},
          {:disable_vpn_on_oidc_error, nil}
        ] do
      @tag conf_key: key, conf_val: val
      test "toggle #{key} when value in db is nil", %{admin_conn: conn, path: path} do
        {:ok, view, _html} = live(conn, path)
        html = view |> element("input[phx-value-config=#{unquote(key)}]") |> render()
        refute html =~ "checked"

        expect(Cache.Mock, :put!, fn unquote(key), v ->
          assert v == true
        end)

        view |> element("input[phx-value-config=#{unquote(key)}]") |> render_click()
      end
    end
  end

  describe "oidc configuration" do
    import FzHttp.ConfigurationsFixtures

    setup %{admin_conn: conn} do
      update_conf(%{
        openid_connect_providers: %{"test" => %{"label" => "test123"}},
        saml_identity_providers: %{}
      })

      path = ~p"/settings/security"
      {:ok, view, _html} = live(conn, path)
      [view: view]
    end

    test "click add button", %{view: view} do
      html =
        view
        |> element("a", "Add OpenID Connect Provider")
        |> render_click()

      assert html =~ ~s|<p class="modal-card-title">OIDC Configuration</p>|
    end

    test "click edit button", %{view: view} do
      html =
        view
        |> element("a", "Edit")
        |> render_click()

      assert html =~ ~s|<p class="modal-card-title">OIDC Configuration</p>|
      assert html =~ ~s|value="test123"|
    end

    test "validate", %{view: view} do
      expect(Cache.Mock, :put!, fn :openid_connect_providers, val ->
        assert val == %{"test" => %{"label" => "test123"}}
      end)

      view
      |> element("a", "Edit")
      |> render_click()

      html =
        view
        |> element("#oidc-form")
        |> render_submit(%{"label" => "updated"})

      # stays on the modal
      assert html =~ ~s|<p class="modal-card-title">OIDC Configuration</p>|
    end

    test "delete", %{view: view} do
      expect(Cache.Mock, :put!, fn :openid_connect_providers, val ->
        assert val == %{}
      end)

      view
      |> element("button", "Delete")
      |> render_click()
    end
  end

  describe "saml configuration" do
    import FzHttp.ConfigurationsFixtures

    setup %{admin_conn: conn} do
      # Security views use the DB config, not cached config, so update DB here for testing
      update_conf(%{
        openid_connect_providers: %{},
        saml_identity_providers: %{"test" => saml_attrs()}
      })

      path = ~p"/settings/security"
      {:ok, view, _html} = live(conn, path)
      [view: view]
    end

    test "click add button", %{view: view} do
      html =
        view
        |> element("a", "Add SAML Identity Provider")
        |> render_click()

      assert html =~ ~s|<p class="modal-card-title">SAML Configuration</p>|
    end

    test "click edit button", %{view: view} do
      html =
        view
        |> element("a", "Edit")
        |> render_click()

      assert html =~ ~s|<p class="modal-card-title">SAML Configuration</p>|
      assert html =~ ~s|entityID=&quot;http://localhost:8080/realms/firezone|
    end

    test "validate", %{view: view} do
      expect(Cache.Mock, :put!, fn :saml_identity_providers, val ->
        assert val == %{"test" => saml_attrs()}
      end)

      view
      |> element("a", "Edit")
      |> render_click()

      html =
        view
        |> element("#saml-form")
        |> render_submit(%{"metadata" => "updated"})

      # stays on the modal
      assert html =~ ~s|<p class="modal-card-title">SAML Configuration</p>|
    end

    test "delete", %{view: view} do
      expect(Cache.Mock, :put!, fn :saml_identity_providers, val ->
        assert val == %{}
      end)

      view
      |> element("button", "Delete")
      |> render_click()
    end
  end
end
