using Lcsm.RunnerEngine.Database.Schema;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.RunnerEngine.Database;

public interface IDbContext : IDisposable
{
    public DbSet<Instance> Instances { get; }
    public DbContext GetWrappedContext();
}