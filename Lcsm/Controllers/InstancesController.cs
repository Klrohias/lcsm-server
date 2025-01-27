using AutoMapper;
using Lcsm.DataModels;
using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Protocol;
using Lcsm.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using SystemTextJsonPatch;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]/{runnerId:int}")]
public class InstancesController(IRunnerService runnerService, IMapper mapper) : ControllerBase
{
    [HttpGet]
    [Authorize]
    public async Task<IActionResult> List([FromRoute] int runnerId)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        return Ok(await protocolClient.ListInstances2(CancellationToken.None));
    }

    [HttpGet("{instanceId:int}")]
    [Authorize]
    public async Task<IActionResult> Get([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        return Ok(await protocolClient.GetInstance2(instanceId, CancellationToken.None));
    }
    
    [HttpPut("{instanceId:int}/RunningProcess")]
    [Authorize]
    public async Task<IActionResult> Start([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        await protocolClient.StartInstance(instanceId, CancellationToken.None);
        return Ok();
    }
    
    [HttpDelete("{instanceId:int}/RunningProcess")]
    [Authorize]
    public async Task<IActionResult> Terminate([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        await protocolClient.TerminateInstance(instanceId, CancellationToken.None);
        return Ok();
    }

    [HttpPut]
    [Authorize("Administrator")]
    public async Task<IActionResult> Create([FromRoute] int runnerId, [FromBody] InstanceUpdateDto options)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        await protocolClient.CreateInstance(mapper.Map<Instance>(options),
            CancellationToken.None);
        return Ok();
    }

    [HttpDelete("{instanceId:int}")]
    [Authorize("Administrator")]
    public async Task<IActionResult> Delete([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");
        
        await protocolClient.DeleteInstance(instanceId, CancellationToken.None);
        return Ok();
    }

    [HttpPost("{instanceId:int}")]
    [Authorize]
    public async Task<IActionResult> Update(
        [FromRoute] int runnerId,
        [FromRoute] int instanceId,
        [FromBody] JsonPatchDocument<Instance> patch)
    {
        var protocolClient = await runnerService.GetRpcClient(runnerId, CancellationToken.None);
        if (protocolClient == null) return NotFound("Instance not found");

        var instance = await protocolClient.GetInstance(instanceId, CancellationToken.None);
        patch.ApplyTo(instance);
        if (instanceId != instance.Id)
        {
            return Conflict("InstanceId has been changed");
        }
        
        await protocolClient.UpdateInstance(instance, CancellationToken.None);
        return Ok();
    }
}
