using Lcsm.RunnerEngine.Database;
using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Managers;
using Lcsm.RunnerEngine.Protocol;
using Lcsm.RunnerEngine.Protocol.Models;
using Lcsm.RunnerEngine.Sockets;
using Microsoft.Extensions.Logging;
using static Lcsm.RunnerEngine.ModelMapper;

namespace Lcsm.RunnerEngine;

public class EmbedEngine
{
    public bool Started => _engineRpcServer.Started;
    
    private readonly InstanceManager _instanceManager;
    private readonly ProcessManager _processManager;
    private readonly EngineRpcServer _engineRpcServer;

    public EmbedEngine(ISocket managedSocket, string dataDirectory, IDbContextProvider dbProvider,
        ILogger? logger = null)
    {
        ArgumentNullException.ThrowIfNull(managedSocket);
        _engineRpcServer = new EngineRpcServer(managedSocket, logger, this);
        _instanceManager = new InstanceManager(dbProvider, dataDirectory);
        _processManager = new ProcessManager();
    }

    public void Start()
    {
        _engineRpcServer.Start();
    }
    
    public void Stop()
    {
        _engineRpcServer.Stop();
    }

    private InstanceDto ToInstanceDto(Instance instance)
    {
        var instanceDto = Mapper.Map<InstanceDto>(instance);
        instanceDto.IsRunning = _processManager.IsRunning(instance.Id);

        return instanceDto;
    }

    private async ValueTask<object?> HandlePacket(RequestPacket packet, CancellationToken cancellationToken)
    {
        switch (packet.Action)
        {
            case PacketAction.Empty:
            {
                break;
            }
            case PacketAction.ListInstances:
            {
                var instance = await _instanceManager.ListInstancesAsync(cancellationToken);
                return instance.Select(ToInstanceDto);
            }
            case PacketAction.CreateInstance:
            {
                // map to options and create
                var dto = packet.DeserializeWith<InstanceDto>().Data;
                await _instanceManager.CreateInstanceAsync(Mapper.Map<Instance>(dto), cancellationToken);
                break;
            }
            case PacketAction.DeleteInstance:
            {
                var instanceId = packet.DeserializeWith<int>().Data;
                await _instanceManager.DeleteInstanceAsync(instanceId, cancellationToken);
                break;
            }
            case PacketAction.GetInstance:
            {
                var instanceId = packet.DeserializeWith<int>().Data;
                return ToInstanceDto(await _instanceManager.GetInstanceAsync(instanceId, cancellationToken));
            }
            case PacketAction.UpdateInstance:
            {
                var dto = packet.DeserializeWith<InstanceDto>().Data;
                await _instanceManager.UpdateInstanceAsync(Mapper.Map<Instance>(dto), cancellationToken);
                break;
            }
            
            case PacketAction.StartInstance:
            {
                var instanceId = packet.DeserializeWith<int>().Data;
                var instance = await _instanceManager.GetInstanceAsync(instanceId, cancellationToken);
                _processManager.StartProcess(instanceId, instance.LaunchCommand,
                    _instanceManager.GetInstancePath(instance));
                break;
            }
            case PacketAction.StopInstance:
                break;
            case PacketAction.TerminateInstance:
            {
                var instanceId = packet.DeserializeWith<int>().Data;
                _processManager.TerminateProcess(instanceId);
                break;
            }

            default:
                throw new ArgumentOutOfRangeException(nameof(packet.Action), packet.Action, null);
        }
        return null;
    }
    
    private class EngineRpcServer(ISocket managedSocket, ILogger? logger, EmbedEngine embedEngine) : RpcServer(managedSocket, logger)
    {
        protected override ValueTask<object?> HandleInvocation(RequestPacket rawPacket,
            CancellationToken cancellationToken)
        {
            return embedEngine.HandlePacket(rawPacket, cancellationToken);
        }
    }
}