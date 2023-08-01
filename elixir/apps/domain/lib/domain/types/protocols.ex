defimpl String.Chars, for: Postgrex.INET do
  def to_string(%Postgrex.INET{} = inet), do: Domain.Types.INET.to_string(inet)
end

defimpl Jason.Encoder, for: Postgrex.INET do
  def encode(%Postgrex.INET{} = struct, opts) do
    Jason.Encode.string("#{struct}", opts)
  end
end

defimpl String.Chars, for: Domain.Types.IPPort do
  def to_string(%Domain.Types.IPPort{} = ip_port), do: Domain.Types.IPPort.to_string(ip_port)
end
