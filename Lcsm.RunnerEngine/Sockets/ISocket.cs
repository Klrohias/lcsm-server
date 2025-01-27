namespace Lcsm.RunnerEngine.Sockets;

public interface ISocket
{
    public Task<IConnection> AcceptConnectionAsync(CancellationToken cancellationToken);
}
