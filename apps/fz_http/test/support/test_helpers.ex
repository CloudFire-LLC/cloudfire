defmodule FzHttp.TestHelpers do
  @moduledoc """
  Test setup helpers
  """

  alias FzHttp.{
    ConnectivityChecksFixtures,
    DevicesFixtures,
    Repo,
    RulesFixtures,
    Users,
    Users.User,
    UsersFixtures
  }

  def restore_env(key, val, cb) do
    old = Application.fetch_env!(:fz_http, key)
    Application.put_env(:fz_http, key, val)
    cb.(fn -> Application.put_env(:fz_http, key, old) end)
  end

  def clear_users do
    Repo.delete_all(User)
  end

  def create_device(tags) do
    device =
      if tags[:unauthed] || is_nil(tags[:user_id]) do
        DevicesFixtures.device()
      else
        DevicesFixtures.device(%{user_id: tags[:user_id]})
      end

    {:ok, device: device}
  end

  def create_other_user_device(_) do
    user_id = UsersFixtures.user(%{role: :unprivileged, email: "other_user@test"}).id

    device =
      DevicesFixtures.device(%{
        user_id: user_id,
        name: "other device",
        public_key: "other-pubkey"
      })

    {:ok, other_device: device}
  end

  def create_connectivity_checks(_tags) do
    connectivity_checks =
      Enum.map(1..5, fn _i ->
        ConnectivityChecksFixtures.connectivity_check_fixture()
      end)

    {:ok, connectivity_checks: connectivity_checks}
  end

  def create_devices(tags) do
    user_id =
      if tags[:unathed] || is_nil(tags[:user_id]) do
        UsersFixtures.user().id
      else
        tags[:user_id]
      end

    devices =
      Enum.map(1..5, fn num ->
        DevicesFixtures.device(%{
          name: "device #{num}",
          public_key: "#{num}",
          user_id: user_id
        })
      end)

    {:ok, devices: devices}
  end

  def create_user(tags) do
    user =
      if tags[:unprivileged] do
        UsersFixtures.user(%{role: :unprivileged})
      else
        UsersFixtures.user()
      end

    {:ok, user: user}
  end

  def create_accept_rule(_) do
    rule = RulesFixtures.rule(%{action: :accept})
    {:ok, rule: rule}
  end

  def create_drop_rule(_) do
    rule = RulesFixtures.rule(%{action: :drop})
    {:ok, rule: rule}
  end

  def create_rule(_) do
    rule = RulesFixtures.rule(%{})
    {:ok, rule: rule}
  end

  def create_rule6(_) do
    rule = RulesFixtures.rule6(%{})
    {:ok, rule6: rule}
  end

  def create_rule4(_) do
    rule = RulesFixtures.rule4(%{})
    {:ok, rule4: rule}
  end

  @doc """
  XXX: Mimic a more realistic setup.
  """
  def create_rules(_) do
    rules =
      1..5
      |> Enum.map(fn num ->
        destination = "#{num}.#{num}.#{num}.0/24"
        RulesFixtures.rule(%{destination: destination})
      end)

    {:ok, rules: rules}
  end

  def create_user_with_valid_sign_in_token(_) do
    {:ok, user: %User{} = UsersFixtures.user(Users.sign_in_keys())}
  end

  def create_user_with_expired_sign_in_token(_) do
    expired = DateTime.add(DateTime.utc_now(), -1 * 86_401)
    params = %{Users.sign_in_keys() | sign_in_token_created_at: expired}
    {:ok, user: %User{} = UsersFixtures.user(params)}
  end

  def create_users(%{count: count}) do
    users =
      Enum.map(1..count, fn i ->
        UsersFixtures.user(%{email: "userlist#{i}@test"})
      end)

    {:ok, users: users}
  end

  def create_users(_), do: create_users(%{count: 5})

  def clear_users(_) do
    {count, _result} = Repo.delete_all(User)
    {:ok, count: count}
  end
end
