defmodule Web.HomeControllerTest do
  use Web.ConnCase, async: true

  describe "home/2" do
    test "renders the form to find the account sign in page", %{conn: conn} do
      conn = get(conn, ~p"/")
      html = response(conn, 200)

      assert html =~ "Account ID or Slug"
      assert html =~ "Go to Sign In page"
    end

    test "renders recently used account", %{conn: conn} do
      accounts = [
        Fixtures.Accounts.create_account(),
        Fixtures.Accounts.create_account()
      ]

      conn = get(conn, ~p"/")
      html = response(conn, 200)

      for account <- accounts do
        refute html =~ account.name
        refute html =~ ~p"/#{account.slug}"
      end

      account_ids =
        accounts
        |> Enum.map(& &1.id)
        |> :erlang.term_to_binary()

      %{resp_cookies: %{"fz_recent_account_ids" => %{value: value}}} =
        %{build_conn() | secret_key_base: Web.Endpoint.config(:secret_key_base)}
        |> put_resp_cookie("fz_recent_account_ids", account_ids, sign: true, secure: true)

      conn =
        build_conn()
        |> put_req_cookie("fz_recent_account_ids", value)
        |> get(~p"/")

      html = response(conn, 200)

      for account <- accounts do
        assert html =~ account.name
        assert html =~ ~p"/#{account.slug}"
      end
    end
  end

  describe "redirect_to_sign_in/2" do
    test "redirects to the sign in page", %{conn: conn} do
      id = Ecto.UUID.generate()
      conn = post(conn, ~p"/", %{"account_id_or_slug" => id, "as" => "client"})
      assert redirected_to(conn) == ~p"/#{id}?as=client"
    end

    test "downcases account slug on redirect", %{conn: conn} do
      conn = post(conn, ~p"/", %{"account_id_or_slug" => "FOO", "as" => "client"})
      assert redirected_to(conn) == ~p"/foo?as=client"
    end

    test "puts an error flash when slug is invalid", %{conn: conn} do
      conn = post(conn, ~p"/", %{"account_id_or_slug" => "?1", "as" => "client"})
      assert redirected_to(conn) == ~p"/?as=client"
      assert conn.assigns.flash["error"] == "Account ID or Slug contains invalid characters"
    end
  end
end
