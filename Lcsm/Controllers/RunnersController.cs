using AutoMapper;
using Lcsm.Database.Schema;
using Lcsm.DataModels;
using Lcsm.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using SystemTextJsonPatch;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]")]
public class RunnersController(IRunnerService runnerService, IMapper mapper) : ControllerBase
{
    [HttpGet]
    [Authorize]
    public async Task<IActionResult> List()
    {
        return Ok(await runnerService.ListRunners(CancellationToken.None));
    }

    [HttpPut]
    [Authorize("Administrator")]
    public async Task<IActionResult> Create([FromBody] RunnerUpdateDto dto)
    {
        await runnerService.AddRunner(mapper.Map<Runner>(dto), CancellationToken.None);
        return Ok();
    }
    
    [HttpDelete("{runnerId:int}")]
    [Authorize("Administrator")]
    public async Task<IActionResult> Delete([FromRoute] int runnerId)
    {
        await runnerService.DeleteRunner(runnerId, CancellationToken.None);
        return Ok();
    }
    
    [HttpGet("{runnerId:int}")]
    [Authorize]
    public async Task<IActionResult> Get([FromRoute] int runnerId)
    {
        return Ok(await runnerService.GetRunner(runnerId, CancellationToken.None));
    }
    
    [HttpPost("{runnerId:int}")]
    [Authorize("Administrator")]
    public async Task<IActionResult> Update([FromRoute] int runnerId, [FromBody] JsonPatchDocument<Runner> patch)
    {
        var originalRunner = await runnerService.GetRunner(runnerId, CancellationToken.None);
        if (originalRunner == null) return NotFound("Runner not found");
        
        patch.ApplyTo(originalRunner);
        await runnerService.UpdateRunner(originalRunner, CancellationToken.None);
        
        return Ok();
    }
}