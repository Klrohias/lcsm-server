using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Protocol.Models;

namespace Lcsm.RunnerEngine.Protocol;

public static partial class Extensions
{
    public static async Task<List<Instance>> ListInstances(this RpcClient self, CancellationToken cancellationToken)
    {
        var data = await self.Request(PacketAction.ListInstances, null, cancellationToken);
        return data.DeserializeWith<List<Instance>>().Data!;
    }
    
    public static async Task<List<InstanceDto>> ListInstances2(this RpcClient self, CancellationToken cancellationToken)
    {
        var data = await self.Request(PacketAction.ListInstances, null, cancellationToken);
        return data.DeserializeWith<List<InstanceDto>>().Data!;
    }

    public static async Task<Instance> GetInstance(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        var data = await self.Request(PacketAction.GetInstance, instanceId, cancellationToken);

        return data.DeserializeWith<Instance>().Data!;
    }
    
    public static async Task<InstanceDto> GetInstance2(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        var data = await self.Request(PacketAction.GetInstance, instanceId, cancellationToken);

        return data.DeserializeWith<InstanceDto>().Data!;
    }
    
    public static async Task CreateInstance(this RpcClient self, Instance instance, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.CreateInstance, ModelMapper.Mapper.Map<InstanceDto>(instance),
            cancellationToken);
    }
    
    public static async Task UpdateInstance(this RpcClient self, Instance instance, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.UpdateInstance, ModelMapper.Mapper.Map<InstanceDto>(instance),
            cancellationToken);
    }
    
    public static async Task DeleteInstance(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.DeleteInstance, instanceId, cancellationToken);
    }

    public static async Task StartInstance(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.StartInstance, instanceId, cancellationToken);
    }
    
    public static async Task StopInstance(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.StopInstance, instanceId, cancellationToken);
    }
    
    public static async Task TerminateInstance(this RpcClient self, int instanceId, CancellationToken cancellationToken)
    {
        await self.Request(PacketAction.TerminateInstance, instanceId, cancellationToken);
    }
}