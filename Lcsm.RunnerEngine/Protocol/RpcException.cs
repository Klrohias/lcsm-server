using Lcsm.RunnerEngine.Protocol.Models;

namespace Lcsm.RunnerEngine.Protocol;

public class RpcException(ResponsePacket packet, string? message) : Exception(message)
{
    public ResponsePacket Packet { get; } = packet;
}
