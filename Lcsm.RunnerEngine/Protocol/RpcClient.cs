using System.Text;
using System.Text.Json;
using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Protocol.Models;
using Lcsm.RunnerEngine.Sockets;

namespace Lcsm.RunnerEngine.Protocol;

public class RpcClient(IConnection connection)
{
    public TimeSpan Timeout { get; set; } = TimeSpan.FromSeconds(30);
    
    private readonly Dictionary<string, TaskCompletionSource<ResponsePacket>> _pendingRequests = new();

    private async void WaitResponseInternal()
    {
        var packet = await connection.ReceiveResponse(CancellationToken.None)!;
        var echo = packet?.Echo ?? "";
        
        if (_pendingRequests.TryGetValue(echo, out var taskCompletionSource))
        {
            if (packet!.Error)
            {
                taskCompletionSource.TrySetException(new RpcException(packet, packet.Message));
            }
            else
            {
                taskCompletionSource.TrySetResult(packet);
            }

            _pendingRequests.Remove(echo);
        }
    }

    public async Task<ResponsePacket> Request(string action, object? data = null,
        CancellationToken cancellationToken = default)
    {
        var echo = Guid.NewGuid().ToString();

        // prepare for receiving
        var taskCompletionSource = new TaskCompletionSource<ResponsePacket>();
        _pendingRequests.Add(echo, taskCompletionSource);

        // create token
        var cancellationTokenSource = new CancellationTokenSource();
        var linkedCancellationTokenSource =
            CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, cancellationTokenSource.Token);
        cancellationTokenSource.CancelAfter(Timeout);
        var linkedToken = linkedCancellationTokenSource.Token;
        linkedToken.Register(x => ((TaskCompletionSource<ResponsePacket>)x!)
            .TrySetCanceled(linkedToken), taskCompletionSource);

        // send
        await connection.SendRequest(new RequestPacket<object>
        {
            Echo = echo,
            Action = action,
            Data = data
        }, linkedToken);

        // wait for response
        WaitResponseInternal();

        var result = await taskCompletionSource.Task;
        return result;
    }
}