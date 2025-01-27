using AutoMapper;
using Lcsm.Database;
using Lcsm.Database.Schema;
using Lcsm.DataModels;
using Lcsm.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.Controllers;

[ApiController]
[Route("[controller]")]
public class UserController(IUserService userService, LcsmDbContext dbContext, IMapper mapper) : ControllerBase
{
    [HttpPost]
    [Route("Authenticate")]
    public async Task<IActionResult> Authenticate([FromBody] AuthenticateRequestDto dto)
    {
        var user = await userService.GetUser(dto.Username!, CancellationToken.None);
        if (user == null) return NotFound("User not found");

        var successful = BCrypt.Net.BCrypt.Verify(dto.Password, user.Password);
        if (!successful)
        {
            return Unauthorized("Invalid cred");
        }

        return Ok(new AuthenticateResponseDto
        {
            AccessToken = userService.IssueToken(user.Username, user.IsAdministrator ? "Administrator" : "User")
        });
    }

    [HttpPut]
    public async Task<IActionResult> Create([FromBody] UserUpdateDto dto)
    {
        var userCount = await dbContext.Users.CountAsync();

        // allow to add the first user when there are not any users in database
        if (userCount != 0 && !User.IsInRole("Administrator"))
        {
            return Unauthorized();
        }

        // we use bcrypt to store user password
        var password = BCrypt.Net.BCrypt.HashPassword(dto.Password);

        var newUser = new User
        {
            Username = dto.Username,
            Password = password,
            IsAdministrator = (dto.IsAdministrator ?? false) || userCount == 0,
            AllowedInstances = []
        };

        await userService.AddUser(newUser, CancellationToken.None);
        return Ok();
    }

    [HttpGet]
    [Authorize]
    public async Task<IActionResult> Get(CancellationToken cancellationToken)
    {
        var username = User.Identity?.Name ?? "";
        if (string.IsNullOrEmpty(username)) return NotFound("User not found");
        var user = await userService.GetUser(username, cancellationToken);
        if (user == null) return NotFound("User not found");

        return Ok(mapper.Map<User>(user));
    }
}
