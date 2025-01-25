namespace Lcsm.ServerEngine.Protocol;

public class BasePacket
{
    public string Echo { get; set; } = string.Empty;
    public PacketType Type { get; set; } = PacketType.Empty;
}

public class BasePacket<T> : BasePacket
{
    public T? Data { get; set; }
}