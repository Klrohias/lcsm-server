using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;
using Lcsm.Database;
using Microsoft.IdentityModel.Tokens;
using JwtRegisteredClaimNames = Microsoft.IdentityModel.JsonWebTokens.JwtRegisteredClaimNames;

namespace Lcsm.Services;

public class TokenService(IConfiguration configuration) : ITokenService
{
    public string IssueToken(string username, string role)
    {
        var claims = new[]
        {
            new Claim(JwtRegisteredClaimNames.Sub, username),
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
}