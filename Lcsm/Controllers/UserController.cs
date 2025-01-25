using Lcsm.DataModels;
using Lcsm.Services;
using Microsoft.AspNetCore.Mvc;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]")]
public class UserController(IUserService userService) : ControllerBase
{
    [HttpPost]
    [Route("Authenticate")]
    public async Task<IActionResult> Authenticate([FromBody] AuthenticateRequestDto dto)
    {
        return Ok(dto.Username);
    }
    
    [HttpGet]
    public IActionResult Get()
    {
        return Ok("hello, world");
    }
}