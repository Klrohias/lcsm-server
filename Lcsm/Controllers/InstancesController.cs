using AutoMapper;
using Lcsm.DataModels;
using Lcsm.ServerEngine.ServerManagement.Schema;
using Lcsm.Services;
using Microsoft.AspNetCore.Mvc;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]/{runnerId:int}")]
public class InstancesController(IRunnerService runnerService, IMapper mapper) : ControllerBase
{
    [HttpGet]
    public async Task<IActionResult> List([FromRoute] int runnerId)
    {
        var protocolClient = await runnerService.GetProtocolClient(runnerId, CancellationToken.None);
        
        return Ok(await protocolClient.ListInstances(CancellationToken.None));
    }

    [HttpGet("{instanceId:int}")]
    public async Task<IActionResult> Get([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetProtocolClient(runnerId, CancellationToken.None);
        
        return Ok(await protocolClient.GetInstance(instanceId, CancellationToken.None));
    }

    [HttpPut]
    public async Task<IActionResult> Create([FromRoute] int runnerId, [FromBody] InstanceUpdateDto options)
    {
        var protocolClient = await runnerService.GetProtocolClient(runnerId, CancellationToken.None);
        
        await protocolClient.CreateInstance(mapper.Map<Instance>(options),
            CancellationToken.None);
        return Ok();
    }

    [HttpDelete("{instanceId:int}")]
    public async Task<IActionResult> Delete([FromRoute] int runnerId, [FromRoute] int instanceId)
    {
        var protocolClient = await runnerService.GetProtocolClient(runnerId, CancellationToken.None);
        
        await protocolClient.DeleteInstance(instanceId, CancellationToken.None);
        return Ok();
    }

    [HttpPost("{instanceId:int}")]
    public async Task<IActionResult> Update(
        [FromRoute] int runnerId,
        [FromRoute] int instanceId,
        [FromBody] InstanceUpdateDto options)
    {
        var protocolClient = await runnerService.GetProtocolClient(runnerId, CancellationToken.None);

        var instance = await protocolClient.GetInstance(instanceId, CancellationToken.None);
        var modifiedInstance = mapper.Map(options, instance);

        await protocolClient.UpdateInstance(modifiedInstance, CancellationToken.None);

        return Ok();
    }
}
