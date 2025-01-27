using Lcsm.Database;
using Lcsm.RunnerEngine.Database;
using Lcsm.RunnerEngine.Database.Schema;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.Common;

public class LcsmDatabaseProvider(IServiceScopeFactory scopeFactory) : IDbContextProvider
{
    public IDbContext Create()
    {
        var scope = scopeFactory.CreateScope();
        var dbContext = scope.ServiceProvider.GetService<LcsmDbContext>();
        return new LcsmDatabaseWrapper(dbContext!, scope);
    }

    private class LcsmDatabaseWrapper(LcsmDbContext dbContext, IDisposable scope) : IDbContext
    {
        public void Dispose()
        {
            scope.Dispose();
        }

        public DbSet<Instance> Instances => dbContext.Instances;

        public DbContext GetWrappedContext()
        {
            return dbContext;
        }
    }
}