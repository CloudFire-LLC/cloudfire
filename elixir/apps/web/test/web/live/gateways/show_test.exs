defmodule Web.Live.Gateways.ShowTest do
  use Web.ConnCase, async: true

  setup do
    account = Fixtures.Accounts.create_account()
    actor = Fixtures.Actors.create_actor(type: :account_admin_user, account: account)
    identity = Fixtures.Auth.create_identity(account: account, actor: actor)
    subject = Fixtures.Auth.create_subject(account: account, actor: actor, identity: identity)

    gateway = Fixtures.Gateways.create_gateway(account: account, actor: actor, identity: identity)
    gateway = Repo.preload(gateway, :group)

    %{
      account: account,
      actor: actor,
      identity: identity,
      subject: subject,
      gateway: gateway
    }
  end

  test "redirects to sign in page for unauthorized user", %{
    account: account,
    gateway: gateway,
    conn: conn
  } do
    assert live(conn, ~p"/#{account}/gateways/#{gateway}") ==
             {:error,
              {:redirect,
               %{
                 to: ~p"/#{account}",
                 flash: %{"error" => "You must log in to access this page."}
               }}}
  end

  test "renders not found error when gateway is deleted", %{
    account: account,
    gateway: gateway,
    identity: identity,
    conn: conn
  } do
    gateway = Fixtures.Gateways.delete_gateway(gateway)

    assert_raise Web.LiveErrors.NotFoundError, fn ->
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")
    end
  end

  test "renders breadcrumbs item", %{
    account: account,
    gateway: gateway,
    identity: identity,
    conn: conn
  } do
    {:ok, _lv, html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")

    assert item = Floki.find(html, "[aria-label='Breadcrumb']")
    breadcrumbs = String.trim(Floki.text(item))
    assert breadcrumbs =~ "Sites"
    assert breadcrumbs =~ gateway.group.name
    assert breadcrumbs =~ gateway.name
  end

  test "renders gateway details", %{
    account: account,
    gateway: gateway,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")

    table =
      lv
      |> element("#gateway")
      |> render()
      |> vertical_table_to_map()

    assert table["site"] =~ gateway.group.name
    assert table["name"] =~ gateway.name
    assert table["last seen"]
    assert table["last seen remote ip"] =~ to_string(gateway.last_seen_remote_ip)
    assert table["status"] =~ "Offline"
    assert table["user agent"] =~ gateway.last_seen_user_agent
    assert table["version"] =~ gateway.last_seen_version
  end

  test "renders gateway status", %{
    account: account,
    gateway: gateway,
    identity: identity,
    conn: conn
  } do
    :ok = Domain.Gateways.connect_gateway(gateway)

    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")

    table =
      lv
      |> element("#gateway")
      |> render()
      |> vertical_table_to_map()

    assert table["status"] =~ "Online"
  end

  test "renders logs table", %{
    account: account,
    identity: identity,
    gateway: gateway,
    conn: conn
  } do
    flow =
      Fixtures.Flows.create_flow(
        account: account,
        gateway: gateway
      )

    flow =
      Repo.preload(flow, client: [:actor], gateway: [:group], policy: [:actor_group, :resource])

    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")

    [row] =
      lv
      |> element("#flows")
      |> render()
      |> table_to_map()

    assert row["authorized at"]
    assert row["expires at"]
    assert row["remote ip"] == to_string(gateway.last_seen_remote_ip)
    assert row["policy"] =~ flow.policy.actor_group.name
    assert row["policy"] =~ flow.policy.resource.name

    assert row["client, actor (ip)"] =~ flow.client.name
    assert row["client, actor (ip)"] =~ "owned by #{flow.client.actor.name}"
    assert row["client, actor (ip)"] =~ to_string(flow.client_remote_ip)
  end

  test "allows deleting gateways", %{
    account: account,
    gateway: gateway,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/gateways/#{gateway}")

    lv
    |> element("button", "Delete Gateway")
    |> render_click()

    assert_redirected(lv, ~p"/#{account}/sites/#{gateway.group}")

    assert Repo.get(Domain.Gateways.Gateway, gateway.id).deleted_at
  end
end
