using System.Text.Json.Serialization;

namespace Lcsm.RunnerEngine.Protocol.Models;

public class RequestPacket
{
    [JsonIgnore] public string? RawPacket { get; set; }
    public string Echo { get; set; } = string.Empty;
    public string Action { get; set; } = PacketAction.Empty;
}

public class RequestPacket<T> : RequestPacket
{
    public T? Data { get; set; }
}