import Config

###############################
##### Domain ##################
###############################

config :domain, Domain.Repo,
  pool_size: 10,
  show_sensitive_data_on_connection_error: false

###############################
##### Web #####################
###############################

config :web, Web.Endpoint,
  cache_static_manifest: "priv/static/cache_manifest.json",
  server: true

###############################
##### API #####################
###############################

config :api, API.Endpoint, server: true

###############################
##### Third-party configs #####
###############################

config :phoenix, :filter_parameters, [
  "password",
  "secret",
  "token",
  "public_key",
  "private_key",
  "preshared_key"
]

# Do not print debug messages in production and handle all
# other reports by Elixir Logger with JSON back-end so that.
# we can parse them in log analysys tools.
# Notice: SASL reports turned off because of their verbosity.
# Notice: Log level can be overriden on production with LOG_LEVEL environment variable.
config :logger,
  backends: [LoggerJSON],
  utc_log: true,
  level: :info,
  handle_sasl_reports: false,
  handle_otp_reports: true

config :logger_json, :backend,
  json_encoder: Jason,
  formatter: LoggerJSON.Formatters.GoogleCloudLogger,
  metadata: :all

config :logger, level: :info

config :swoosh, local: false
