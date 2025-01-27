using Lcsm.Database.Schema;

namespace Lcsm.Services;

public interface IUserService
{
    public string IssueToken(string username, string role);

    public Task<User?> GetUser(string username, CancellationToken cancellationToken);

    public Task AddUser(User user, CancellationToken cancellationToken);
}