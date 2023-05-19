defmodule Domain.ActorsTest do
  use Domain.DataCase, async: true
  import Domain.Actors
  alias Domain.Auth
  alias Domain.Actors
  alias Domain.{AccountsFixtures, AuthFixtures, ActorsFixtures}

  describe "fetch_count_by_role/0" do
    setup do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      %{
        account: account,
        actor: actor,
        subject: subject
      }
    end

    test "returns correct count of not deleted actors by role", %{
      account: account,
      subject: subject
    } do
      assert fetch_count_by_role(:admin, subject) == 1
      assert fetch_count_by_role(:unprivileged, subject) == 0

      ActorsFixtures.create_actor(role: :admin)
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      assert {:ok, _actor} = delete_actor(actor, subject)
      assert fetch_count_by_role(:admin, subject) == 1
      assert fetch_count_by_role(:unprivileged, subject) == 0

      ActorsFixtures.create_actor(role: :admin, account: account)
      assert fetch_count_by_role(:admin, subject) == 2
      assert fetch_count_by_role(:unprivileged, subject) == 0

      ActorsFixtures.create_actor(role: :unprivileged)
      ActorsFixtures.create_actor(role: :unprivileged, account: account)
      assert fetch_count_by_role(:admin, subject) == 2
      assert fetch_count_by_role(:unprivileged, subject) == 1

      for _ <- 1..5, do: ActorsFixtures.create_actor(role: :unprivileged, account: account)
      assert fetch_count_by_role(:admin, subject) == 2
      assert fetch_count_by_role(:unprivileged, subject) == 6
    end

    test "returns error when subject can not view actors", %{subject: subject} do
      subject = AuthFixtures.remove_permissions(subject)

      assert fetch_count_by_role(:foo, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "fetch_actor_by_id/2" do
    test "returns error when actor is not found" do
      subject = AuthFixtures.create_subject()
      assert fetch_actor_by_id(Ecto.UUID.generate(), subject) == {:error, :not_found}
    end

    test "returns error when id is not a valid UUID" do
      subject = AuthFixtures.create_subject()
      assert fetch_actor_by_id("foo", subject) == {:error, :not_found}
    end

    test "returns own actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert {:ok, returned_actor} = fetch_actor_by_id(actor.id, subject)
      assert returned_actor.id == actor.id
    end

    test "returns non own actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      actor = ActorsFixtures.create_actor(account: account)

      assert {:ok, returned_actor} = fetch_actor_by_id(actor.id, subject)
      assert returned_actor.id == actor.id
    end

    test "returns error when actor is in another account" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      actor = ActorsFixtures.create_actor()

      assert fetch_actor_by_id(actor.id, subject) == {:error, :not_found}
    end

    test "returns error when subject can not view actors" do
      subject = AuthFixtures.create_subject()
      subject = AuthFixtures.remove_permissions(subject)

      assert fetch_actor_by_id("foo", subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "fetch_actor_by_id/1" do
    test "returns error when actor is not found" do
      assert fetch_actor_by_id(Ecto.UUID.generate()) == {:error, :not_found}
    end

    test "returns error when id is not a valid UUIDv4" do
      assert fetch_actor_by_id("foo") == {:error, :not_found}
    end

    test "returns actor" do
      actor = ActorsFixtures.create_actor(role: :admin)
      assert {:ok, returned_actor} = fetch_actor_by_id(actor.id)
      assert returned_actor.id == actor.id
    end
  end

  describe "fetch_actor_by_id!/1" do
    test "raises when actor is not found" do
      assert_raise(Ecto.NoResultsError, fn ->
        fetch_actor_by_id!(Ecto.UUID.generate())
      end)
    end

    test "raises when id is not a valid UUIDv4" do
      assert_raise(Ecto.Query.CastError, fn ->
        assert fetch_actor_by_id!("foo")
      end)
    end

    test "returns actor" do
      actor = ActorsFixtures.create_actor(role: :admin)
      assert returned_actor = fetch_actor_by_id!(actor.id)
      assert returned_actor.id == actor.id
    end
  end

  describe "list_actors/2" do
    test "returns empty list when there are not actors" do
      subject =
        %Auth.Subject{
          identity: nil,
          actor: %{id: Ecto.UUID.generate()},
          account: %{id: Ecto.UUID.generate()},
          context: nil,
          expires_at: nil,
          permissions: MapSet.new()
        }
        |> AuthFixtures.set_permissions([
          Actors.Authorizer.manage_actors_permission()
        ])

      assert list_actors(subject) == {:ok, []}
      assert list_actors(subject, hydrate: []) == {:ok, []}
    end

    test "returns list of actors in all roles" do
      account = AccountsFixtures.create_account()
      actor1 = ActorsFixtures.create_actor(account: account, role: :admin)
      actor2 = ActorsFixtures.create_actor(account: account, role: :unprivileged)
      ActorsFixtures.create_actor(role: :unprivileged)

      identity1 = AuthFixtures.create_identity(account: account, actor: actor1)
      subject = AuthFixtures.create_subject(identity1)

      assert {:ok, actors} = list_actors(subject)
      assert length(actors) == 2
      assert Enum.sort(Enum.map(actors, & &1.id)) == Enum.sort([actor1.id, actor2.id])
    end

    test "returns error when subject can not view actors" do
      subject = AuthFixtures.create_subject()
      subject = AuthFixtures.remove_permissions(subject)

      assert list_actors(subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "create_actor/4" do
    setup do
      account = AccountsFixtures.create_account()
      provider = AuthFixtures.create_email_provider(account: account)
      provider_identifier = AuthFixtures.random_provider_identifier(provider)

      %{
        account: account,
        provider: provider,
        provider_identifier: provider_identifier
      }
    end

    test "returns changeset error when required attrs are missing", %{
      provider: provider,
      provider_identifier: provider_identifier
    } do
      assert {:error, changeset} = create_actor(provider, provider_identifier, %{})
      refute changeset.valid?

      assert errors_on(changeset) == %{
               role: ["can't be blank"],
               type: ["can't be blank"]
             }
    end

    test "returns error on invalid attrs", %{
      provider: provider,
      provider_identifier: provider_identifier
    } do
      attrs = ActorsFixtures.actor_attrs(role: :foo, type: :bar)

      assert {:error, changeset} = create_actor(provider, provider_identifier, attrs)
      refute changeset.valid?

      assert errors_on(changeset) == %{
               role: ["is invalid"],
               type: ["is invalid"]
             }
    end

    test "returns error on duplicate provider_identifier", %{
      provider: provider
    } do
      provider_identifier = AuthFixtures.random_provider_identifier(provider)
      attrs = ActorsFixtures.actor_attrs()
      assert {:ok, _actor} = create_actor(provider, provider_identifier, attrs)
      assert {:error, changeset} = create_actor(provider, provider_identifier, attrs)
      assert errors_on(changeset) == %{provider_identifier: ["has already been taken"]}
    end

    test "creates an actor in given role", %{
      provider: provider
    } do
      for role <- [:admin, :unprivileged] do
        attrs = ActorsFixtures.actor_attrs(role: role)
        provider_identifier = AuthFixtures.random_provider_identifier(provider)
        assert {:ok, actor} = create_actor(provider, provider_identifier, attrs)
        assert actor.role == role
      end
    end

    test "creates an actor in given type", %{
      provider: provider
    } do
      for type <- [:user, :service_account] do
        attrs = ActorsFixtures.actor_attrs(type: type)
        provider_identifier = AuthFixtures.random_provider_identifier(provider)
        assert {:ok, actor} = create_actor(provider, provider_identifier, attrs)
        assert actor.type == type
      end
    end

    test "creates an actor and identity", %{
      provider: provider,
      provider_identifier: provider_identifier
    } do
      attrs = ActorsFixtures.actor_attrs()

      assert {:ok, actor} = create_actor(provider, provider_identifier, attrs)

      assert actor.type == attrs.type
      assert actor.role == attrs.role
      assert is_nil(actor.disabled_at)
      assert is_nil(actor.deleted_at)

      assert identity = Repo.one(Domain.Auth.Identity)
      assert identity.provider_id == provider.id
      assert identity.provider_identifier == provider_identifier
      assert identity.actor_id == actor.id
      assert identity.account_id == provider.account_id

      assert %{"sign_in_token_created_at" => _, "sign_in_token_hash" => _} =
               identity.provider_state

      assert identity.provider_virtual_state == nil

      assert is_nil(identity.deleted_at)
    end
  end

  describe "create_actor/5" do
    setup do
      account = AccountsFixtures.create_account()
      provider = AuthFixtures.create_email_provider(account: account)
      provider_identifier = AuthFixtures.random_provider_identifier(provider)

      %{
        account: account,
        provider: provider,
        provider_identifier: provider_identifier
      }
    end

    test "returns error when subject can not create actors", %{
      account: account,
      provider: provider,
      provider_identifier: provider_identifier
    } do
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()
        |> AuthFixtures.remove_permissions()

      assert create_actor(provider, provider_identifier, %{}, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end

    test "returns error when subject tries to create an account in another account", %{
      provider: provider,
      provider_identifier: provider_identifier
    } do
      subject = AuthFixtures.create_subject()
      assert create_actor(provider, provider_identifier, %{}, subject) == {:error, :unauthorized}
    end

    test "returns error when subject is trying to create an actor with a privilege escalation", %{
      account: account,
      provider: provider,
      provider_identifier: provider_identifier
    } do
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()

      admin_permissions = subject.permissions
      required_permissions = [Actors.Authorizer.manage_actors_permission()]

      subject =
        subject
        |> AuthFixtures.remove_permissions()
        |> AuthFixtures.set_permissions(required_permissions)

      missing_permissions =
        MapSet.difference(admin_permissions, MapSet.new(required_permissions))
        |> MapSet.to_list()

      attrs = %{type: :user, role: :admin}

      assert create_actor(provider, provider_identifier, attrs, subject) ==
               {:error, {:unauthorized, privilege_escalation: missing_permissions}}

      attrs = %{"type" => "user", "role" => "admin"}

      assert create_actor(provider, provider_identifier, attrs, subject) ==
               {:error, {:unauthorized, privilege_escalation: missing_permissions}}
    end
  end

  describe "change_actor_role/3" do
    setup do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      %{
        account: account,
        actor: actor,
        subject: subject
      }
    end

    test "allows admin to change other actors role", %{account: account, subject: subject} do
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      assert {:ok, %{role: :unprivileged}} = change_actor_role(actor, :unprivileged, subject)
      assert {:ok, %{role: :admin}} = change_actor_role(actor, :admin, subject)

      actor = ActorsFixtures.create_actor(role: :unprivileged, account: account)
      assert {:ok, %{role: :unprivileged}} = change_actor_role(actor, :unprivileged, subject)
      assert {:ok, %{role: :admin}} = change_actor_role(actor, :admin, subject)
    end

    test "returns error when subject can not manage roles", %{account: account} do
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()
        |> AuthFixtures.remove_permissions()

      assert change_actor_role(actor, :foo, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "disable_actor/2" do
    test "disables a given actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert {:ok, actor} = disable_actor(actor, subject)
      assert actor.disabled_at

      assert actor = Repo.get(Actors.Actor, actor.id)
      assert actor.disabled_at

      assert other_actor = Repo.get(Actors.Actor, other_actor.id)
      assert is_nil(other_actor.disabled_at)
    end

    test "returns error when trying to disable the last admin actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(account: account, role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert disable_actor(actor, subject) == {:error, :cant_disable_the_last_admin}
    end

    test "last admin check ignores admins in other accounts" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      ActorsFixtures.create_actor(role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert disable_actor(actor, subject) == {:error, :cant_disable_the_last_admin}
    end

    test "last admin check ignores disabled admins" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)
      {:ok, _other_actor} = disable_actor(other_actor, subject)

      assert disable_actor(actor, subject) == {:error, :cant_disable_the_last_admin}
    end

    test "returns error when trying to disable the last admin actor using a race condition" do
      for _ <- 0..50 do
        test_pid = self()

        spawn(fn ->
          allow_child_sandbox_access(test_pid)

          account = AccountsFixtures.create_account()

          actor_one = ActorsFixtures.create_actor(role: :admin, account: account)
          actor_two = ActorsFixtures.create_actor(role: :admin, account: account)

          subject_one = AuthFixtures.create_subject(actor_one)
          subject_two = AuthFixtures.create_subject(actor_two)

          for {actor, subject} <- [{actor_two, subject_one}, {actor_one, subject_two}] do
            spawn(fn ->
              allow_child_sandbox_access(test_pid)
              assert disable_actor(actor, subject) == {:error, :cant_disable_the_last_admin}
            end)
          end
        end)
      end
    end

    test "does not do anything when an actor is disabled twice" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert {:ok, _actor} = disable_actor(other_actor, subject)
      assert {:ok, other_actor} = disable_actor(other_actor, subject)
      assert {:ok, _actor} = disable_actor(other_actor, subject)
    end

    test "does not allow to disable actors in other accounts" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert disable_actor(other_actor, subject) == {:error, :not_found}
    end

    test "returns error when subject can not disable actors" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()
        |> AuthFixtures.remove_permissions()

      assert disable_actor(actor, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "enable_actor/2" do
    test "enables a given actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      {:ok, actor} = disable_actor(actor, subject)

      assert {:ok, actor} = enable_actor(actor, subject)
      assert actor.disabled_at

      assert actor = Repo.get(Actors.Actor, actor.id)
      assert actor.disabled_at

      assert other_actor = Repo.get(Actors.Actor, other_actor.id)
      assert is_nil(other_actor.disabled_at)
    end

    test "does not do anything when an actor is already enabled" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      {:ok, other_actor} = disable_actor(other_actor, subject)

      assert {:ok, _actor} = enable_actor(other_actor, subject)
      assert {:ok, other_actor} = enable_actor(other_actor, subject)
      assert {:ok, _actor} = enable_actor(other_actor, subject)
    end

    test "does not allow to enable actors in other accounts" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert enable_actor(other_actor, subject) == {:error, :not_found}
    end

    test "returns error when subject can not enable actors" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()
        |> AuthFixtures.remove_permissions()

      assert enable_actor(actor, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  describe "delete_actor/2" do
    test "deletes a given actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert {:ok, actor} = delete_actor(actor, subject)
      assert actor.deleted_at

      assert actor = Repo.get(Actors.Actor, actor.id)
      assert actor.deleted_at

      assert other_actor = Repo.get(Actors.Actor, other_actor.id)
      assert is_nil(other_actor.deleted_at)
    end

    test "returns error when trying to delete the last admin actor" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert delete_actor(actor, subject) == {:error, :cant_delete_the_last_admin}
    end

    test "last admin check ignores admins in other accounts" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      ActorsFixtures.create_actor(role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert delete_actor(actor, subject) == {:error, :cant_delete_the_last_admin}
    end

    test "last admin check ignores disabled admins" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)
      {:ok, _other_actor} = disable_actor(other_actor, subject)

      assert delete_actor(actor, subject) == {:error, :cant_delete_the_last_admin}
    end

    test "returns error when trying to delete the last admin actor using a race condition" do
      for _ <- 0..50 do
        test_pid = self()

        spawn(fn ->
          allow_child_sandbox_access(test_pid)

          account = AccountsFixtures.create_account()

          actor_one = ActorsFixtures.create_actor(role: :admin, account: account)
          actor_two = ActorsFixtures.create_actor(role: :admin, account: account)

          subject_one = AuthFixtures.create_subject(actor_one)
          subject_two = AuthFixtures.create_subject(actor_two)

          for {actor, subject} <- [{actor_two, subject_one}, {actor_one, subject_two}] do
            spawn(fn ->
              allow_child_sandbox_access(test_pid)
              assert delete_actor(actor, subject) == {:error, :cant_delete_the_last_admin}
            end)
          end
        end)
      end
    end

    test "does not allow to delete an actor twice" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin, account: account)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert {:ok, _actor} = delete_actor(other_actor, subject)
      assert delete_actor(other_actor, subject) == {:error, :not_found}
    end

    test "does not allow to delete actors in other accounts" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)
      other_actor = ActorsFixtures.create_actor(role: :admin)
      identity = AuthFixtures.create_identity(account: account, actor: actor)
      subject = AuthFixtures.create_subject(identity)

      assert delete_actor(other_actor, subject) == {:error, :not_found}
    end

    test "returns error when subject can not delete actors" do
      account = AccountsFixtures.create_account()
      actor = ActorsFixtures.create_actor(role: :admin, account: account)

      subject =
        AuthFixtures.create_identity(account: account, actor: actor)
        |> AuthFixtures.create_subject()
        |> AuthFixtures.remove_permissions()

      assert delete_actor(actor, subject) ==
               {:error,
                {:unauthorized,
                 [missing_permissions: [Actors.Authorizer.manage_actors_permission()]]}}
    end
  end

  defp allow_child_sandbox_access(parent_pid) do
    Ecto.Adapters.SQL.Sandbox.allow(Repo, parent_pid, self())
    # Allow is async call we need to break current process execution
    # to allow sandbox to be enabled
    :timer.sleep(10)
  end
end
