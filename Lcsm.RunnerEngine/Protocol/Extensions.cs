using System.Text;
using System.Text.Json;
using Lcsm.RunnerEngine.Protocol.Models;
using Lcsm.RunnerEngine.Sockets;

namespace Lcsm.RunnerEngine.Protocol;

public static partial class Extensions
{
    public static async ValueTask SendRequest<T>(this IConnection connection, T packet,
        CancellationToken cancellationToken)
        where T : RequestPacket
    {
        var encodedPacket = Encoding.Default.GetBytes(JsonSerializer.Serialize(packet));
        await connection.SendAsync(encodedPacket, cancellationToken);
    }

    public static async ValueTask SendResponse<T>(this IConnection connection, T packet,
        CancellationToken cancellationToken)
        where T : ResponsePacket
    {
        var encodedPacket = Encoding.Default.GetBytes(JsonSerializer.Serialize(packet));
        await connection.SendAsync(encodedPacket, cancellationToken);
    }
    
    public static async ValueTask<RequestPacket?> ReceiveRequest(this IConnection connection,
        CancellationToken cancellationToken)
    {
        var packet = await connection.ReceiveAsync(cancellationToken);
        var data = Encoding.Default.GetString(packet);
        var basePacket = JsonSerializer.Deserialize<RequestPacket>(data);
        if (basePacket != null) basePacket.RawPacket = data;

        return basePacket;
    }

    public static async ValueTask<ResponsePacket?> ReceiveResponse(this IConnection connection,
        CancellationToken cancellationToken)
    {
        var packet = await connection.ReceiveAsync(cancellationToken);
        var data = Encoding.Default.GetString(packet);
        var basePacket = JsonSerializer.Deserialize<ResponsePacket>(data);
        if (basePacket != null) basePacket.RawPacket = data;

        return basePacket;
    }
    
    public static RequestPacket<T> DeserializeWith<T>(this RequestPacket packet)
    {
        ArgumentException.ThrowIfNullOrEmpty(packet.RawPacket);
        return JsonSerializer.Deserialize<RequestPacket<T>>(packet.RawPacket)!;
    }

    public static ResponsePacket<T> DeserializeWith<T>(this ResponsePacket packet)
    {
        ArgumentException.ThrowIfNullOrEmpty(packet.RawPacket);
        return JsonSerializer.Deserialize<ResponsePacket<T>>(packet.RawPacket)!;
    }
}