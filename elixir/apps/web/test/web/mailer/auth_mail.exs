defmodule Web.Mailer.AuthEmailTest do
  use Web.ConnCase, async: true
  import Web.Mailer.AuthEmail

  setup do
    Domain.Config.put_env_override(:outbound_email_adapter_configured?, true)
    account = Fixtures.Accounts.create_account()
    provider = Fixtures.Auth.create_email_provider(account: account)

    admin_actor = Fixtures.Actors.create_actor(type: :account_admin_user, account: account)

    admin_identity =
      Fixtures.Auth.create_identity(account: account, provider: provider, actor: admin_actor)

    client_actor = Fixtures.Actors.create_actor(type: :account_user, account: account)

    client_identity =
      Fixtures.Auth.create_identity(account: account, provider: provider, actor: client_actor)

    %{
      account: account,
      provider: provider,
      admin_actor: admin_actor,
      admin_identity: admin_identity,
      client_actor: client_actor,
      client_identity: client_identity
    }
  end

  describe "new_user_email/3" do
    test "should contain relevant account and user info", %{
      account: account,
      provider: provider,
      admin_actor: admin_actor,
      admin_identity: admin_identity,
      client_identity: client_identity
    } do
      admin_subject =
        Fixtures.Auth.create_subject(
          account: account,
          provider: provider,
          identity: admin_identity,
          actor: admin_actor
        )

      email_body = new_user_email(account, client_identity, admin_subject)

      assert email_body.text_body =~ "Welcome to Firezone!"
      assert email_body.text_body =~ "#{admin_actor.name} invited you"
      assert email_body.text_body =~ account.name
      assert email_body.text_body =~ account.slug
    end
  end
end
