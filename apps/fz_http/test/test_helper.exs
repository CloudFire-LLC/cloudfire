Mox.defmock(OpenIDConnect.Mock, for: OpenIDConnect.MockBehaviour)
Mox.defmock(Cache.Mock, for: Cache.MockBehaviour)

ExUnit.start()
Ecto.Adapters.SQL.Sandbox.mode(FzHttp.Repo, :manual)
