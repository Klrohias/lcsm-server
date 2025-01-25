using Lcsm.Database.Schema;
using Lcsm.ServerEngine.Protocol;

namespace Lcsm.Services;

public interface IRunnerService
{
    public Task<ProtocolClient> GetProtocolClient(int runnerId, CancellationToken cancellationToken);
    
    public Task<Runner?> GetRunner(int runnerId, CancellationToken cancellationToken);
    
    public Task AddRunner(Runner runner, CancellationToken cancellationToken);
    
    public Task UpdateRunner(Runner runner, CancellationToken cancellationToken);

    public Task DeleteRunner(int runnerId, CancellationToken cancellationToken);
    
    public Task<List<Runner>> ListRunners(CancellationToken cancellationToken);
}