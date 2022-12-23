defmodule FzHttp.Devices.DeviceSetting do
  use FzHttp, :schema
  alias FzHttp.Devices

  @primary_key false
  embedded_schema do
    field :ip, :string
    field :ip6, :string
    field :user_id, Ecto.UUID
  end

  def parse(device_or_device_as_map) do
    %__MODULE__{
      ip: Devices.decode(device_or_device_as_map.ipv4),
      ip6: Devices.decode(device_or_device_as_map.ipv6),
      user_id: device_or_device_as_map.user_id
    }
  end
end
