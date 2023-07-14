defmodule Web.Mailer do
  alias Swoosh.Mailer
  alias Swoosh.Email
  require Logger

  @doc """
  Delivers an email via configured Swoosh adapter.

  If adapter is not configured or is set to nil, the delivery will be ignored and
  function will return `{:ok, %{}}`.

  Notice: this code is copied from `Swoosh.Mailer.deliver/2` and modified to
  not send emails if adapter is not configured. This is needed to avoid
  custom adapter implementation that does nothing.
  """
  def deliver(email, config \\ []) do
    opts = Mailer.parse_config(:web, __MODULE__, [], config)
    metadata = %{email: email, config: config, mailer: __MODULE__}

    if opts[:adapter] do
      :telemetry.span([:swoosh, :deliver], metadata, fn ->
        case Mailer.deliver(email, opts) do
          {:ok, result} -> {{:ok, result}, Map.put(metadata, :result, result)}
          {:error, error} -> {{:error, error}, Map.put(metadata, :error, error)}
        end
      end)
    else
      Logger.info("Emails are not configured", email_subject: inspect(email.subject))
      {:ok, %{}}
    end
  end

  defp render_template(view, template, format, assigns) do
    heex = apply(view, String.to_atom("#{template}_#{format}"), [assigns])
    assigns = Keyword.merge(assigns, inner_content: heex)
    Phoenix.Template.render_to_string(view, "#{template}_#{format}", "html", assigns)
  end

  def render_body(%Swoosh.Email{} = email, view, template, assigns) do
    assigns = assigns ++ [email: email]

    email
    |> Email.html_body(render_template(view, template, "html", assigns))
    |> Email.text_body(render_template(view, template, "text", assigns))
  end

  def active? do
    mailer_config = Domain.Config.fetch_env!(:web, Web.Mailer)
    mailer_config[:from_email] && mailer_config[:adapter]
  end

  def default_email do
    # Fail hard if email not configured
    from_email =
      Domain.Config.fetch_env!(:web, Web.Mailer)
      |> Keyword.fetch!(:from_email)

    Email.new()
    |> Email.from(from_email)
  end
end
