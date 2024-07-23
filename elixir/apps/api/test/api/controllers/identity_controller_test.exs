defmodule API.IdentityControllerTest do
  use API.ConnCase, async: true
  alias Domain.Auth.Identity

  setup do
    account = Fixtures.Accounts.create_account()
    actor = Fixtures.Actors.create_actor(type: :api_client, account: account)

    %{
      account: account,
      actor: actor
    }
  end

  describe "index/2" do
    test "returns error when not authorized", %{conn: conn, actor: actor} do
      conn = get(conn, "/actors/#{actor.id}/identities")
      assert json_response(conn, 401) == %{"error" => %{"reason" => "Unauthorized"}}
    end

    test "lists all identities for actor", %{conn: conn, account: account, actor: actor} do
      identities =
        for _ <- 1..3, do: Fixtures.Auth.create_identity(%{account: account, actor: actor})

      conn =
        conn
        |> authorize_conn(actor)
        |> put_req_header("content-type", "application/json")
        |> get("/actors/#{actor.id}/identities")

      assert %{
               "data" => data,
               "metadata" => %{
                 "count" => count,
                 "limit" => limit,
                 "next_page" => next_page,
                 "prev_page" => prev_page
               }
             } = json_response(conn, 200)

      assert count == 3
      assert limit == 50
      assert is_nil(next_page)
      assert is_nil(prev_page)

      data_ids = Enum.map(data, & &1["id"])
      identity_ids = Enum.map(identities, & &1.id)

      assert equal_ids?(data_ids, identity_ids)
    end

    test "lists resources with limit", %{conn: conn, account: account, actor: actor} do
      identities =
        for _ <- 1..3, do: Fixtures.Auth.create_identity(%{account: account, actor: actor})

      conn =
        conn
        |> authorize_conn(actor)
        |> put_req_header("content-type", "application/json")
        |> get("/actors/#{actor.id}/identities", limit: "2")

      assert %{
               "data" => data,
               "metadata" => %{
                 "count" => count,
                 "limit" => limit,
                 "next_page" => next_page,
                 "prev_page" => prev_page
               }
             } = json_response(conn, 200)

      assert limit == 2
      assert count == 3
      refute is_nil(next_page)
      assert is_nil(prev_page)

      data_ids = Enum.map(data, & &1["id"]) |> MapSet.new()
      identity_ids = Enum.map(identities, & &1.id) |> MapSet.new()

      assert MapSet.subset?(data_ids, identity_ids)
    end
  end

  describe "show/2" do
    test "returns error when not authorized", %{conn: conn, account: account, actor: actor} do
      identity = Fixtures.Auth.create_identity(%{account: account, actor: actor})
      conn = get(conn, "/actors/#{actor.id}/identities/#{identity.id}")
      assert json_response(conn, 401) == %{"error" => %{"reason" => "Unauthorized"}}
    end

    test "returns a single resource", %{conn: conn, account: account, actor: actor} do
      identity = Fixtures.Auth.create_identity(%{account: account, actor: actor})

      conn =
        conn
        |> authorize_conn(actor)
        |> put_req_header("content-type", "application/json")
        |> get("/actors/#{actor.id}/identities/#{identity.id}")

      assert json_response(conn, 200) == %{
               "data" => %{
                 "id" => identity.id,
                 "actor_id" => actor.id,
                 "provider_id" => identity.provider_id,
                 "provider_identifier" => identity.provider_identifier
               }
             }
    end
  end

  describe "create/2" do
    test "returns error when not authorized", %{conn: conn, actor: actor} do
      conn = post(conn, "/actors/#{actor.id}/identities", %{})
      assert json_response(conn, 401) == %{"error" => %{"reason" => "Unauthorized"}}
    end

    test "returns error on empty params/body", %{
      conn: conn,
      account: account,
      actor: api_actor
    } do
      actor = Fixtures.Actors.create_actor(account: account)

      conn =
        conn
        |> authorize_conn(api_actor)
        |> put_req_header("content-type", "application/json")
        |> post("/actors/#{actor.id}/identities")

      assert resp = json_response(conn, 400)
      assert resp == %{"error" => %{"reason" => "Bad Request"}}
    end

    test "returns error on invalid identity provider", %{
      conn: conn,
      account: account,
      actor: api_actor
    } do
      actor = Fixtures.Actors.create_actor(account: account)

      attrs = %{}

      conn =
        conn
        |> authorize_conn(api_actor)
        |> put_req_header("content-type", "application/json")
        |> post("/actors/#{actor.id}/identities",
          provider_id: "1234",
          identity: attrs
        )

      assert resp = json_response(conn, 404)
      assert resp == %{"error" => %{"reason" => "Not Found"}}
    end

    test "returns error on invalid identity attrs", %{
      conn: conn,
      account: account,
      actor: api_actor
    } do
      {oidc_provider, _bypass} =
        Fixtures.Auth.start_and_create_openid_connect_provider(account: account)

      actor = Fixtures.Actors.create_actor(account: account)

      attrs = %{}

      conn =
        conn
        |> authorize_conn(api_actor)
        |> put_req_header("content-type", "application/json")
        |> post("/actors/#{actor.id}/identities",
          provider_id: oidc_provider.id,
          identity: attrs
        )

      assert resp = json_response(conn, 422)

      assert resp ==
               %{
                 "error" => %{
                   "reason" => "Unprocessable Entity",
                   "validation_errors" => %{"provider_identifier" => ["can't be blank"]}
                 }
               }
    end

    test "creates a resource with valid attrs", %{
      conn: conn,
      account: account,
      actor: api_actor
    } do
      {oidc_provider, _bypass} =
        Fixtures.Auth.start_and_create_openid_connect_provider(account: account)

      actor = Fixtures.Actors.create_actor(account: account)

      attrs = %{"provider_identifier" => "foo@local"}

      conn =
        conn
        |> authorize_conn(api_actor)
        |> put_req_header("content-type", "application/json")
        |> post("/actors/#{actor.id}/identities",
          provider_id: oidc_provider.id,
          identity: attrs
        )

      assert resp = json_response(conn, 201)

      assert resp["data"]["provider_identifier"] == attrs["provider_identifier"]
      assert resp["data"]["provider_id"] == oidc_provider.id
      assert resp["data"]["actor_id"] == actor.id
    end
  end

  describe "delete/2" do
    test "returns error when not authorized", %{conn: conn, account: account, actor: actor} do
      identity = Fixtures.Auth.create_identity(%{account: account, actor: actor})
      conn = delete(conn, "/actors/#{actor.id}/identities/#{identity.id}")
      assert json_response(conn, 401) == %{"error" => %{"reason" => "Unauthorized"}}
    end

    test "deletes a resource", %{conn: conn, account: account, actor: actor} do
      identity = Fixtures.Auth.create_identity(%{account: account, actor: actor})

      conn =
        conn
        |> authorize_conn(actor)
        |> put_req_header("content-type", "application/json")
        |> delete("/actors/#{actor.id}/identities/#{identity.id}")

      assert json_response(conn, 200) == %{
               "data" => %{
                 "id" => identity.id,
                 "actor_id" => actor.id,
                 "provider_id" => identity.provider_id,
                 "provider_identifier" => identity.provider_identifier
               }
             }

      assert identity = Repo.get(Identity, identity.id)
      assert identity.deleted_at
    end
  end
end
