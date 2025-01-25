using Lcsm.ServerEngine.ServerManagement.Schema;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.ServerEngine.ServerManagement;

public interface IDbContext : IDisposable
{
    public DbSet<Instance> Instances { get; }
    public DbContext GetWrappedContext();
}