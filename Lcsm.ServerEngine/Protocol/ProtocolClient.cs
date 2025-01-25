using System.Text;
using System.Text.Json;
using Lcsm.ServerEngine.ServerManagement.Schema;
using Lcsm.ServerEngine.Sockets;

namespace Lcsm.ServerEngine.Protocol;

public class ProtocolClient(IConnection connection)
{
    public TimeSpan Timeout { get; set; } = TimeSpan.FromSeconds(30);
    
    private readonly Dictionary<string, TaskCompletionSource<string>> _pendingRequests = new();

    private async ValueTask SendTextPacketInternal(string text, CancellationToken cancellationToken)
    {
        var bytes = Encoding.Default.GetBytes(text);
        await connection.SendAsync(bytes, cancellationToken);
    }

    private async void WaitResponseInternal()
    {
        var data = Encoding.Default.GetString(await connection.ReceiveAsync(CancellationToken.None));
        var packet = JsonSerializer.Deserialize<BasePacket>(data);
        var echo = packet?.Echo ?? "";
        if (_pendingRequests.TryGetValue(echo, out var taskCompletionSource))
        {
            if (packet!.Type == PacketType.Error)
            {
                taskCompletionSource.TrySetException(new ProtocolException(data));
            }
            else
            {
                taskCompletionSource.TrySetResult(data);
            }

            _pendingRequests.Remove(echo);
        }
    }

    private async Task<string> CreateRequestInternal(PacketType type, object? data = null,
        CancellationToken cancellationToken = default)
    {
        var echo = Guid.NewGuid().ToString();

        // request
        var taskCompletionSource = new TaskCompletionSource<string>();
        _pendingRequests.Add(echo, taskCompletionSource);

        var cancellationTokenSource = new CancellationTokenSource();
        var linkedCancellationTokenSource =
            CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, cancellationTokenSource.Token);
        cancellationTokenSource.CancelAfter(Timeout);
        var linkedToken = linkedCancellationTokenSource.Token;
        linkedToken.Register(x => ((TaskCompletionSource<string>)x!).TrySetCanceled(linkedToken), taskCompletionSource);
        
        var requestPacket = new BasePacket<object>
        {
            Echo = echo,
            Type = type,
            Data = data
        };
        
        await SendTextPacketInternal(JsonSerializer.Serialize(requestPacket), linkedToken);

        // wait for response
        WaitResponseInternal();

        var result = await taskCompletionSource.Task;

        return result;
    }

    public async Task<List<Instance>> ListInstances(CancellationToken cancellationToken)
    {
        var data = await CreateRequestInternal(PacketType.ListInstances, null, cancellationToken);

        return JsonSerializer.Deserialize<BasePacket<List<Instance>>>(data)?.Data!;
    }

    public async Task<Instance> GetInstance(int instanceId, CancellationToken cancellationToken)
    {
        var data = await CreateRequestInternal(PacketType.GetInstance, instanceId, cancellationToken);
        
        return JsonSerializer.Deserialize<BasePacket<Instance>>(data)?.Data!;
    }
    
    public async Task CreateInstance(Instance instance, CancellationToken cancellationToken)
    {
        await CreateRequestInternal(PacketType.CreateInstance, ModelMapper.Mapper.Map<InstanceUpdatePacket>(instance),
            cancellationToken);
    }
    
    public async Task UpdateInstance(Instance instance, CancellationToken cancellationToken)
    {
        await CreateRequestInternal(PacketType.UpdateInstance, ModelMapper.Mapper.Map<InstanceUpdatePacket>(instance),
            cancellationToken);
    }
    
    public async Task DeleteInstance(int instanceId, CancellationToken cancellationToken)
    {
        await CreateRequestInternal(PacketType.DeleteInstance, instanceId, cancellationToken);
    }
}