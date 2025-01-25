using Lcsm.DataModels;
using Lcsm.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]")]
public class UserController(ITokenService tokenService) : ControllerBase
{
    [HttpPost]
    [Route("Authenticate")]
    public async Task<IActionResult> Authenticate([FromBody] AuthenticateRequestDto dto)
    {
        return Ok(new AuthenticateResponseDto
        {
            AccessToken = tokenService.IssueToken("qingyi", "Administrator")
        });
    }
    
    [HttpGet]
    [Authorize]
    public IActionResult Get()
    {
        return Ok("Successful");
    }
}