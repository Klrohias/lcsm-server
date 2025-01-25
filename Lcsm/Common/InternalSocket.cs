using System.Threading.Tasks.Dataflow;
using Lcsm.ServerEngine.Protocol;
using Lcsm.ServerEngine.Sockets;

namespace Lcsm.Common;

public class InternalSocket : ISocket
{
    public IConnection ClientConnection { get; }
    
    private readonly Connection _serverConnection;
    private readonly BufferBlock<byte[]> _rx = new();
    private readonly BufferBlock<byte[]> _tx = new();

    public InternalSocket()
    {
        _serverConnection = new Connection(_rx, _tx);
        ClientConnection = new Connection(_tx, _rx);
    }

    public Task<IConnection> AcceptConnectionAsync(CancellationToken cancellationToken)
    {
        // accept only once
        if (_serverConnection.Accepted)
        {
            var taskCompletionSource = new TaskCompletionSource<IConnection>();
            cancellationToken.Register(x
                => ((TaskCompletionSource<IConnection>)x!).TrySetCanceled(cancellationToken), taskCompletionSource);

            return taskCompletionSource.Task;
        }

        _serverConnection.Open();
        return Task.FromResult<IConnection>(_serverConnection);
    }

    private class Connection(BufferBlock<byte[]> rx, BufferBlock<byte[]> tx) : IConnection
    {
        public bool Accepted { get; private set; }

        public void Open()
        {
            Accepted = true;
        }

        public async ValueTask<byte[]> ReceiveAsync(CancellationToken cancellationToken)
        {
            return await rx.ReceiveAsync(cancellationToken);
        }

        public async ValueTask SendAsync(byte[] packet, CancellationToken cancellationToken)
        {
            await tx.SendAsync(packet, cancellationToken);
        }

        public void Close()
        {
            Accepted = false;
        }
    }
}
