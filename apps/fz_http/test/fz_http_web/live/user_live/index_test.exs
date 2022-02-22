defmodule FzHttpWeb.UserLive.IndexTest do
  use FzHttpWeb.ConnCase, async: true

  alias FzHttp.Users

  describe "authenticated user list" do
    setup [:create_tunnels, :create_users]

    test "includes the created user email in the list", %{
      authed_conn: conn,
      tunnels: _tunnels,
      users: users
    } do
      path = Routes.user_index_path(conn, :index)
      {:ok, _view, html} = live(conn, path)

      for user <- users do
        assert html =~ user.email
      end
    end

    test "includes tunnel_counts in the list", %{
      authed_conn: conn,
      tunnels: _tunnels,
      users: _users
    } do
      path = Routes.user_index_path(conn, :index)
      {:ok, _view, html} = live(conn, path)

      for user <- Users.list_users(:with_tunnel_counts) do
        assert html =~ "<td id=\"user-#{user.id}-tunnel-count\">#{user.tunnel_count}</td>"
      end
    end

    test "navigates to user show", %{authed_conn: conn, users: users} do
      path = Routes.user_index_path(conn, :index)
      {:ok, view, _html} = live(conn, path)
      user = List.first(users)

      view
      |> element("a", user.email)
      |> render_click()

      assert_patched(view, Routes.user_show_path(conn, :show, user))
    end
  end

  describe "unauthenticated user list" do
    setup :create_users

    test "redirects to sign in", %{unauthed_conn: conn} do
      path = Routes.user_index_path(conn, :index)
      expected_path = Routes.session_path(conn, :new)
      assert {:error, {:redirect, %{to: ^expected_path}}} = live(conn, path)
    end
  end

  describe "create user" do
    setup :create_user

    @valid_user_attrs %{
      "user" => %{
        "email" => "testemail@localhost",
        "password" => "new_password",
        "password_confirmation" => "new_password"
      }
    }

    @invalid_user_attrs %{
      "user" => %{
        "email" => "invalid",
        "password" => "short",
        "new_password" => "short"
      }
    }

    test "successfully creates user", %{authed_conn: conn} do
      path = Routes.user_index_path(conn, :new)
      {:ok, view, _html} = live(conn, path)

      view
      |> element("form#user-form")
      |> render_submit(@valid_user_attrs)

      {new_path, flash} = assert_redirect(view)
      assert flash["info"] == "User created successfully."
      user = Users.get_user!(email: @valid_user_attrs["user"]["email"])
      assert new_path == Routes.user_show_path(conn, :show, user)
    end

    test "renders errors", %{authed_conn: conn} do
      path = Routes.user_index_path(conn, :new)
      {:ok, view, _html} = live(conn, path)

      new_view =
        view
        |> element("form#user-form")
        |> render_submit(@invalid_user_attrs)

      assert new_view =~ "has invalid format"
      assert new_view =~ "should be at least 12 character(s)"
    end
  end

  describe "add user modal" do
    setup :create_users

    test "shows the modal", %{authed_conn: conn} do
      path = Routes.user_index_path(conn, :index)
      {:ok, view, _html} = live(conn, path)

      modal =
        view
        |> element("a", "Add User")
        |> render_click()

      assert modal =~ "<p class=\"modal-card-title\">Add User</p>"
    end
  end
end
