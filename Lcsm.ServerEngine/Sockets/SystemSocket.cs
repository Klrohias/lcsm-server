using System.Net.Sockets;

namespace Lcsm.ServerEngine.Sockets;

public class SystemSocket(Socket wrappedSocket) : ISocket
{
    public async Task<IConnection> AcceptConnectionAsync(CancellationToken cancellationToken)
    {
        var conn = await wrappedSocket.AcceptAsync(cancellationToken);
        return new SystemConnection(conn);
    }
    
    private class SystemConnection(Socket wrappedSocket) : IConnection
    {
        public async ValueTask<byte[]> ReceiveAsync(CancellationToken cancellationToken)
        {
            // receive length
            var rawLengthPacket = new byte[4];
            await wrappedSocket.ReceiveAsync(rawLengthPacket.AsMemory(), cancellationToken);

            var length = BitConverter.ToInt32(rawLengthPacket);
            
            // receive packet
            var packet = new byte[length];
            await wrappedSocket.ReceiveAsync(packet.AsMemory(), cancellationToken);
            
            return packet;
        }

        public async ValueTask SendAsync(byte[] packet, CancellationToken cancellationToken)
        {
            await wrappedSocket.SendAsync(packet, cancellationToken);
        }

        public void Close()
        {
            wrappedSocket.Close();
        }
    }
}