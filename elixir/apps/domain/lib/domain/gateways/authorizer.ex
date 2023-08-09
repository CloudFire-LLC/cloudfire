defmodule Domain.Gateways.Authorizer do
  use Domain.Auth.Authorizer
  alias Domain.Gateways.{Gateway, Group}

  def manage_gateways_permission, do: build(Gateway, :manage)
  def connect_gateways_permission, do: build(Gateway, :connect)

  @impl Domain.Auth.Authorizer

  def list_permissions_for_role(:account_admin_user) do
    [
      manage_gateways_permission()
    ]
  end

  def list_permissions_for_role(_) do
    [
      connect_gateways_permission()
    ]
  end

  @impl Domain.Auth.Authorizer
  def for_subject(queryable, %Subject{} = subject) do
    cond do
      has_permission?(subject, connect_gateways_permission()) ->
        # TODO: evaluate the resource policy for the subject
        by_account_id(queryable, subject)

      has_permission?(subject, manage_gateways_permission()) ->
        by_account_id(queryable, subject)
    end
  end

  defp by_account_id(queryable, subject) do
    cond do
      Ecto.Query.has_named_binding?(queryable, :groups) ->
        Group.Query.by_account_id(queryable, subject.account.id)

      Ecto.Query.has_named_binding?(queryable, :gateways) ->
        Gateway.Query.by_account_id(queryable, subject.account.id)
    end
  end
end
