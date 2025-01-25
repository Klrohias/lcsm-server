using System.Text;
using System.Text.Json;
using Lcsm.ServerEngine.Protocol;
using Lcsm.ServerEngine.ServerManagement;
using Lcsm.ServerEngine.ServerManagement.Schema;
using Lcsm.ServerEngine.Sockets;
using Microsoft.Extensions.Logging;

namespace Lcsm.ServerEngine;

public class EmbedServer
{
    public bool Started { get; private set; }
    
    private readonly ISocket _managedSocket;
    private readonly object _syncRoot = new();
    private readonly ILogger<EmbedServer>? _logger;
    private readonly InstanceManager _instanceManager;
    
    private CancellationTokenSource? _cancellationTokenSource;
    private Task? _serverTask;

    public EmbedServer(ISocket managedSocket, string dataDirectory, IDbContextProvider dbProvider,
        ILogger<EmbedServer>? logger = null)
    {
        ArgumentNullException.ThrowIfNull(managedSocket);

        _managedSocket = managedSocket;
        _logger = logger;
        _instanceManager = new InstanceManager(dbProvider, dataDirectory);
    }

    public void Start()
    {
        lock (_syncRoot)
        {
            if (Started)
            {
                return;
            }

            Started = true;
        }

        _cancellationTokenSource = new CancellationTokenSource();
        _serverTask = ServeInternal(_cancellationTokenSource.Token);
    }

    public void Stop()
    {
        lock (_syncRoot)
        {
            if (!Started)
            {
                return;
            }

            Started = false;
        }

        _cancellationTokenSource?.Cancel();
        _serverTask?.GetAwaiter().GetResult();
    }

    public async Task EmbedServe()
    {
        lock (_syncRoot)
        {
            if (Started)
            {
                return;
            }

            Started = true;
        }

        await ServeInternal(CancellationToken.None);

        lock (_syncRoot)
        {
            Started = false;
        }
    }

    private async Task ServeInternal(CancellationToken cancellationToken)
    {
        while (Started && !cancellationToken.IsCancellationRequested)
        {
            var newConn = await _managedSocket.AcceptConnectionAsync(cancellationToken);
            ConnectionWorkerInternal(newConn, cancellationToken);
        }
    }

    private async ValueTask SendBasePacketInternal<T>(IConnection connection, T packet,
        CancellationToken cancellationToken)
        where T : BasePacket
    {
        var encodedResponsePacket = Encoding.Default.GetBytes(JsonSerializer.Serialize(packet));
        await connection.SendAsync(encodedResponsePacket, cancellationToken);
    }

    private async ValueTask<(BasePacket?, string)> ReceiveBasePacketInternal(IConnection connection, CancellationToken cancellationToken)
    {
        var packet = await connection.ReceiveAsync(cancellationToken);
        var data = Encoding.Default.GetString(packet);
        var basePacket = JsonSerializer.Deserialize<BasePacket>(data);

        return (basePacket, data);
    }

    private async void ConnectionWorkerInternal(IConnection connection, CancellationToken cancellationToken)
    {
        try
        {
            while (Started && !cancellationToken.IsCancellationRequested)
            {
                BasePacket? basePacket = null;
                try
                {
                    // receive packet
                    (basePacket, var data) = await ReceiveBasePacketInternal(connection, cancellationToken);
                    if (basePacket == null) return;

                    // process the packet
                    var response = await HandlePacket(basePacket.Type, data, cancellationToken);

                    // respond
                    await SendBasePacketInternal(connection, new BasePacket<object>
                    {
                        Data = response,
                        Echo = basePacket.Echo,
                        Type = basePacket.Type
                    }, cancellationToken);
                }
                catch (Exception ex)
                {
                    _logger?.LogError(ex, "Error while process client packet");
                    
                    if (basePacket == null) continue;
                    
                    // respond exception
                    await SendBasePacketInternal(connection, new BasePacket<object>
                    {
                        Data = null,
                        Echo = basePacket.Echo,
                        Type = PacketType.Error
                    }, cancellationToken);
                }
            }
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Problems in connection");
        }
        finally
        {
            connection.Close();
        }
    }

    private async Task<object?> HandlePacket(PacketType packetType, string data, CancellationToken cancellationToken)
    {
        switch (packetType)
        {
            case PacketType.Empty:
            {
                break;
            }
            case PacketType.ListInstances:
            {
                return await _instanceManager.ListInstancesAsync(cancellationToken);
            }
            case PacketType.CreateInstance:
            {
                // map to options and create
                var packet = JsonSerializer.Deserialize<BasePacket<InstanceUpdatePacket>>(data)?.Data!;
                await _instanceManager.CreateInstanceAsync(ModelMapper.Mapper.Map<Instance>(packet), cancellationToken);
                break;
            }
            case PacketType.DeleteInstance:
            {
                var instanceId = JsonSerializer.Deserialize<BasePacket<int>>(data)?.Data ?? 0;
                await _instanceManager.DeleteInstanceAsync(instanceId, cancellationToken);
                break;
            }
            case PacketType.StartInstance:
                break;
            case PacketType.StopInstance:
                break;
            case PacketType.TerminalInstance:
                break;
            case PacketType.GetInstance:
            {
                var instanceId = JsonSerializer.Deserialize<BasePacket<int>>(data)?.Data ?? 0;
                return await _instanceManager.GetInstanceAsync(instanceId, cancellationToken);
            }
            case PacketType.UpdateInstance:
            {
                var packet = JsonSerializer.Deserialize<BasePacket<InstanceUpdatePacket>>(data)?.Data!;
                await _instanceManager.UpdateInstanceAsync(ModelMapper.Mapper.Map<Instance>(packet), cancellationToken);
                break;
            }

            case PacketType.Error:
            default:
                throw new ArgumentOutOfRangeException(nameof(packetType), packetType, null);
        }
        return null;
    }
}