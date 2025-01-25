using AutoMapper;
using Lcsm.Database.Schema;
using Lcsm.DataModels;
using Lcsm.Services;
using Microsoft.AspNetCore.Mvc;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]")]
public class RunnersController(IRunnerService runnerService, IMapper mapper) : ControllerBase
{
    [HttpGet]
    public async Task<IActionResult> List()
    {
        return Ok(await runnerService.ListRunners(CancellationToken.None));
    }

    [HttpPut]
    public async Task<IActionResult> Create([FromBody] RunnerUpdateDto dto)
    {
        await runnerService.AddRunner(mapper.Map<Runner>(dto), CancellationToken.None);
        return Ok();
    }
    
    [HttpDelete("{runnerId:int}")]
    public async Task<IActionResult> Delete([FromRoute] int runnerId)
    {
        await runnerService.DeleteRunner(runnerId, CancellationToken.None);
        return Ok();
    }
    
    [HttpGet("{runnerId:int}")]
    public async Task<IActionResult> Get([FromRoute] int runnerId)
    {
        return Ok(await runnerService.GetRunner(runnerId, CancellationToken.None));
    }
    
    [HttpPost("{runnerId:int}")]
    public async Task<IActionResult> Update([FromRoute] int runnerId, [FromBody] RunnerUpdateDto dto)
    {
        var originalRunner = await runnerService.GetRunner(runnerId, CancellationToken.None);
        await runnerService.UpdateRunner(mapper.Map(dto, originalRunner)!, CancellationToken.None);
        
        return Ok();
    }
}