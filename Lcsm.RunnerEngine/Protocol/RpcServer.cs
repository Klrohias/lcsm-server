using Lcsm.RunnerEngine.Protocol.Models;
using Lcsm.RunnerEngine.Sockets;
using Microsoft.Extensions.Logging;

namespace Lcsm.RunnerEngine.Protocol;

public abstract class RpcServer(ISocket managedSocket, ILogger? logger)
{
    public bool Started => _cancellationTokenSource != null;
    
    private CancellationTokenSource? _cancellationTokenSource;
    
    public void Start()
    {
        if (Started) return;
        
        _cancellationTokenSource = new CancellationTokenSource();
        ServeInternal(_cancellationTokenSource.Token);
    }

    public void Stop()
    {
        if (!Started) return;
        
        _cancellationTokenSource?.CancelAsync()
            .GetAwaiter().GetResult();
        _cancellationTokenSource = null;
    }

    private async void ServeInternal(CancellationToken cancellationToken)
    {
        // accept connection until stop
        while (Started && !cancellationToken.IsCancellationRequested)
        {
            try
            {
                var newConn = await managedSocket.AcceptConnectionAsync(cancellationToken);
                HandleConnectionInternal(newConn, cancellationToken);
            }
            catch (Exception ex)
            {
                logger?.LogError(ex, "Error occured when accepting new connection");
            }
        }
    }

    private async void HandleConnectionInternal(IConnection connection, CancellationToken cancellationToken)
    {
        // disconnect when the connection is unavailable
        try
        {
            while (Started && !cancellationToken.IsCancellationRequested)
            {
                await HandleInvocationInternal(connection, cancellationToken);
            }
        }
        catch (Exception ex)
        {
            logger?.LogError(ex, "Connection become unavailable, will disconnect");
        }
        finally
        {
            connection.Close();
        }
    }

    private async ValueTask HandleInvocationInternal(IConnection connection, CancellationToken cancellationToken)
    {
        // receive a packet and process
        RequestPacket? basePacket = null;
        
        try
        {
            // receive packet
            basePacket = await connection.ReceiveRequest(cancellationToken);
            if (basePacket == null) return;

            // process the packet
            var response = await HandleInvocation(basePacket, cancellationToken);

            // respond
            await connection.SendResponse(new ResponsePacket<object>
            {
                Error = false,
                Message = null,
                Echo = basePacket.Echo,
                Data = response,
            }, cancellationToken);
        }
        catch (Exception ex)
        {
            logger?.LogError(ex, "Error while process client packet");
            if (basePacket == null) return;
            
            await connection.SendResponse(new ResponsePacket
            {
                Error = true,
                Message = ex.Message,
                Echo = basePacket.Echo,
            }, cancellationToken);
        }
    }

    protected abstract ValueTask<object?> HandleInvocation(RequestPacket rawPacket,
        CancellationToken cancellationToken);
}