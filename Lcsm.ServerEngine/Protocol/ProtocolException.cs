namespace Lcsm.ServerEngine.Protocol;

public class ProtocolException : Exception
{
    public string PacketData { get; }

    public ProtocolException(string packetData) : base("Internal Server Exception")
    {
        PacketData = packetData;
    }
}