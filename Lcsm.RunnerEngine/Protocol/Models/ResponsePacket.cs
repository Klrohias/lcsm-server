using System.Text.Json.Serialization;

namespace Lcsm.RunnerEngine.Protocol.Models;

public class ResponsePacket
{
    [JsonIgnore] public string? RawPacket { get; set; }
    public string? Echo { get; set; }
    public bool Error { get; set; }
    public string? Message { get; set; }
}

public class ResponsePacket<T> : ResponsePacket
{
    public T? Data { get; set; }
}