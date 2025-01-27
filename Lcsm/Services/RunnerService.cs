using Lcsm.Database;
using Lcsm.Database.Schema;
using Lcsm.RunnerEngine.Protocol;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.Services;

public class RunnerService(LcsmDbContext dbContext, IBuiltinRunnerService builtinRunnerService) : IRunnerService
{
    public async Task<RpcClient?> GetRpcClient(int runnerId, CancellationToken cancellationToken)
    {
        var runner = await GetRunner(runnerId, cancellationToken);
        
        if (runner?.SocketType == RunnerType.Builtin)
        {
            return new RpcClient(builtinRunnerService.Connection);
        }

        return null;
    }

    public Task<Runner?> GetRunner(int runnerId, CancellationToken cancellationToken)
    {
        return dbContext.Runners
            .FirstOrDefaultAsync(x => x.Id == runnerId, cancellationToken);
    }

    public async Task AddRunner(Runner runner, CancellationToken cancellationToken)
    {
        runner.Id = 0;
        await dbContext.Runners.AddAsync(runner, cancellationToken);
        await dbContext.SaveChangesAsync(cancellationToken);
    }

    public async Task UpdateRunner(Runner runner, CancellationToken cancellationToken)
    {
        var originalInstance = await dbContext.Runners
            .CountAsync(x => x.Id == runner.Id, cancellationToken);
        if (originalInstance == 0) return;

        dbContext.Runners.Update(runner);
        await dbContext.SaveChangesAsync(cancellationToken);
    }

    public async Task DeleteRunner(int runnerId, CancellationToken cancellationToken)
    {
        var runner = await dbContext.Runners
            .FirstOrDefaultAsync(x => x.Id == runnerId, cancellationToken);
        
        if (runner == null) return;
        dbContext.Runners.Remove(runner);
        
        await dbContext.SaveChangesAsync(cancellationToken);
    }

    public Task<List<Runner>> ListRunners(CancellationToken cancellationToken)
    {
        return dbContext.Runners.ToListAsync(cancellationToken);
    }
}