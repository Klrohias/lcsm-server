using Lcsm.Database.Schema;
using Lcsm.RunnerEngine.Protocol;

namespace Lcsm.Services;

public interface IRunnerService
{
    public Task<RpcClient?> GetRpcClient(int runnerId, CancellationToken cancellationToken);
    
    public Task<Runner?> GetRunner(int runnerId, CancellationToken cancellationToken);
    
    public Task AddRunner(Runner runner, CancellationToken cancellationToken);
    
    public Task UpdateRunner(Runner runner, CancellationToken cancellationToken);

    public Task DeleteRunner(int runnerId, CancellationToken cancellationToken);
    
    public Task<List<Runner>> ListRunners(CancellationToken cancellationToken);
}