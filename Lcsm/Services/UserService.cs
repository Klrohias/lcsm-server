using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;
using Lcsm.Database;
using Lcsm.Database.Schema;
using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Tokens;

namespace Lcsm.Services;

public class UserService(IConfiguration configuration, LcsmDbContext dbContext) : IUserService
{
    public string IssueToken(string username, string role)
    {
        var claims = new[]
        {
            new Claim(ClaimTypes.Name, username),
            new Claim(ClaimTypes.Role, role)
        };
        
        var issuerSigningKey =
            Encoding.Default.GetBytes(configuration["JwtSettings:IssuerSigningKey"] ?? "default");

        var issuer = configuration.GetSection("JwtSettings:ValidIssuers").Get<string[]>()?.FirstOrDefault() ?? "";
        var audience = configuration.GetSection("JwtSettings:ValidAudiences").Get<string[]>()?.FirstOrDefault() ?? "";;
        var cred = new SigningCredentials(new SymmetricSecurityKey(issuerSigningKey), SecurityAlgorithms.HmacSha256);

        var token = new JwtSecurityToken(
            issuer: issuer,
            audience: audience,
            claims: claims,
            notBefore: DateTime.Now,
            expires: DateTime.Now.AddSeconds(3600),
            signingCredentials: cred
        );

        return new JwtSecurityTokenHandler().WriteToken(token);
    }

    public Task<User?> GetUser(string username, CancellationToken cancellationToken)
    {
        return dbContext.Users.FirstOrDefaultAsync(x => x.Username == username, cancellationToken);
    }

    public async Task AddUser(User user, CancellationToken cancellationToken)
    {
        await dbContext.Users.AddAsync(user, cancellationToken);
        await dbContext.SaveChangesAsync(cancellationToken);
    }
}