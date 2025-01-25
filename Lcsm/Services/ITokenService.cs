namespace Lcsm.Services;

public interface ITokenService
{
    public string IssueToken(string username, string role);
}