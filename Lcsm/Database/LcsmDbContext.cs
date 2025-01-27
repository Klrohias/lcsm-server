using Lcsm.Database.Schema;
using Lcsm.RunnerEngine.Database.Schema;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.Database;

public class LcsmDbContext(DbContextOptions<LcsmDbContext> options) : DbContext(options)
{
    public required DbSet<Instance> Instances { get; set; }
    public required DbSet<User> Users { get; set; }
    public required DbSet<Runner> Runners { get; set; }
}