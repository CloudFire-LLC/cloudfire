defmodule Web.Live.Relays.ShowTest do
  use Web.ConnCase, async: true

  setup do
    account = Fixtures.Accounts.create_account()
    actor = Fixtures.Actors.create_actor(type: :account_admin_user, account: account)
    identity = Fixtures.Auth.create_identity(account: account, actor: actor)
    subject = Fixtures.Auth.create_subject(account: account, actor: actor, identity: identity)

    relay = Fixtures.Relays.create_relay(account: account, actor: actor, identity: identity)
    relay = Repo.preload(relay, :group)

    %{
      account: account,
      actor: actor,
      identity: identity,
      subject: subject,
      relay: relay
    }
  end

  test "redirects to sign in page for unauthorized user", %{
    account: account,
    relay: relay,
    conn: conn
  } do
    assert live(conn, ~p"/#{account}/relays/#{relay}") ==
             {:error,
              {:redirect,
               %{
                 to: ~p"/#{account}",
                 flash: %{"error" => "You must log in to access this page."}
               }}}
  end

  test "renders not found error when relay is deleted", %{
    account: account,
    relay: relay,
    identity: identity,
    conn: conn
  } do
    relay = Fixtures.Relays.delete_relay(relay)

    assert_raise Web.LiveErrors.NotFoundError, fn ->
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/relays/#{relay}")
    end
  end

  test "renders breadcrumbs item", %{
    account: account,
    relay: relay,
    identity: identity,
    conn: conn
  } do
    {:ok, _lv, html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/relays/#{relay}")

    assert item = Floki.find(html, "[aria-label='Breadcrumb']")
    breadcrumbs = String.trim(Floki.text(item))
    assert breadcrumbs =~ "Relay Instance Groups"
    assert breadcrumbs =~ relay.group.name
    assert breadcrumbs =~ to_string(relay.ipv4)
  end

  test "renders relay details", %{
    account: account,
    relay: relay,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/relays/#{relay}")

    table =
      lv
      |> element("#relay")
      |> render()
      |> vertical_table_to_map()

    assert table["instance group name"] =~ relay.group.name
    assert table["last seen"]
    assert table["location"] =~ to_string(relay.last_seen_remote_ip)
    assert table["remote ipv4"] =~ to_string(relay.ipv4)
    assert table["remote ipv6"] =~ to_string(relay.ipv6)
    assert table["status"] =~ "Offline"
    assert table["user agent"] =~ relay.last_seen_user_agent
    assert table["version"] =~ relay.last_seen_version
  end

  test "renders relay status", %{
    account: account,
    relay: relay,
    identity: identity,
    conn: conn
  } do
    :ok = Domain.Relays.connect_relay(relay, "foo")

    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/relays/#{relay}")

    table =
      lv
      |> element("#relay")
      |> render()
      |> vertical_table_to_map()

    assert table["status"] =~ "Online"
  end

  test "allows deleting relays", %{
    account: account,
    relay: relay,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/relays/#{relay}")

    assert lv
           |> element("button", "Delete Relay")
           |> render_click() ==
             {:error, {:redirect, %{to: ~p"/#{account}/relay_groups/#{relay.group}"}}}

    assert Repo.get(Domain.Relays.Relay, relay.id).deleted_at
  end
end
