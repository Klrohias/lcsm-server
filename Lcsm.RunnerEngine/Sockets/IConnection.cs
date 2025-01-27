namespace Lcsm.RunnerEngine.Sockets;

public interface IConnection
{
    public ValueTask<byte[]> ReceiveAsync(CancellationToken cancellationToken);
    public ValueTask SendAsync(byte[] packet, CancellationToken cancellationToken);
    public void Close();
}