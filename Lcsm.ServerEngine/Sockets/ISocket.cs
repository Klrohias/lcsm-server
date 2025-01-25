namespace Lcsm.ServerEngine.Sockets;

public interface ISocket
{
    public Task<IConnection> AcceptConnectionAsync(CancellationToken cancellationToken);
}