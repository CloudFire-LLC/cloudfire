defmodule FzHttpWeb.Auth.JSON.Authentication do
  @moduledoc """
  API Authentication implementation module for Guardian.
  """
  use Guardian, otp_app: :fz_http

  alias FzHttp.{
    ApiTokens,
    Users
  }

  @impl Guardian
  def subject_for_token(user, _claims) do
    {:ok, to_string(user.id)}
  end

  @impl Guardian
  def resource_from_claims(%{"jti" => api_token_id}) do
    with {:ok, api_token} <- ApiTokens.get_api_token(api_token_id),
         false <- ApiTokens.expired?(api_token),
         {:ok, user} <- Users.get_user(api_token.user_id) do
      {:ok, user}
    else
      _ ->
        {:error, :resource_not_found}
    end
  end
end
