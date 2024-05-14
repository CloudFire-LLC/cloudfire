defmodule Web.Live.Settings.IdentityProviders.NewTest do
  use Web.ConnCase, async: true

  setup do
    Domain.Config.put_env_override(:outbound_email_adapter_configured?, true)

    account = Fixtures.Accounts.create_account(features: %{idp_sync: false})
    actor = Fixtures.Actors.create_actor(type: :account_admin_user, account: account)

    {provider, bypass} = Fixtures.Auth.start_and_create_openid_connect_provider(account: account)

    identity = Fixtures.Auth.create_identity(account: account, actor: actor, provider: provider)

    %{
      account: account,
      actor: actor,
      openid_connect_provider: provider,
      bypass: bypass,
      identity: identity
    }
  end

  test "redirects to sign in page for unauthorized user", %{account: account, conn: conn} do
    path = ~p"/#{account}/settings/identity_providers/new"

    assert live(conn, path) ==
             {:error,
              {:redirect,
               %{
                 to: ~p"/#{account}?#{%{redirect_to: path}}",
                 flash: %{"error" => "You must sign in to access this page."}
               }}}
  end

  test "renders available options", %{
    account: account,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/settings/identity_providers/new")

    assert has_element?(lv, "#idp-option-google_workspace")
    assert html =~ "Google Workspace"

    assert has_element?(lv, "#idp-option-microsoft_entra")
    assert html =~ "Microsoft Entra"

    assert has_element?(lv, "#idp-option-okta")
    assert html =~ "Okta"

    assert has_element?(lv, "#idp-option-openid_connect")
    assert html =~ "OpenID Connect"
  end

  test "next step for non-idp-sync plans is OIDC form", %{
    account: account,
    identity: identity,
    conn: conn
  } do
    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/settings/identity_providers/new")

    lv
    |> element("#idp-option-google_workspace")
    |> render_click()

    assert_redirect(
      lv,
      ~p"/#{account}/settings/identity_providers/openid_connect/new?provider=google_workspace"
    )
  end

  test "next step for idp-sync plans is to custom adapter form", %{
    account: account,
    identity: identity,
    conn: conn
  } do
    Domain.Config.feature_flag_override(:idp_sync, true)

    {:ok, account} =
      Domain.Accounts.update_account(account, %{
        features: %{
          idp_sync: true
        }
      })

    {:ok, lv, _html} =
      conn
      |> authorize_conn(identity)
      |> live(~p"/#{account}/settings/identity_providers/new")

    lv
    |> element("#idp-option-google_workspace")
    |> render_click()

    assert_redirect(
      lv,
      ~p"/#{account}/settings/identity_providers/google_workspace/new"
    )
  end
end
