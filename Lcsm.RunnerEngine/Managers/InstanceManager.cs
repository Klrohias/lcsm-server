using Lcsm.RunnerEngine.Database;
using Lcsm.RunnerEngine.Database.Schema;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.RunnerEngine.Managers;

public class InstanceManager
{
    private readonly IDbContextProvider _dbProvider;
    private readonly string _dataDirectory;

    public InstanceManager(IDbContextProvider dbProvider, string dataDirectory)
    {
        ArgumentNullException.ThrowIfNull(dbProvider);
        ArgumentException.ThrowIfNullOrWhiteSpace(dataDirectory);
        
        _dbProvider = dbProvider;
        _dataDirectory = dataDirectory;
    }

    public string GetInstancePath(Instance instance)
    {
        return instance.WorkingDirectory ?? Path.Combine(_dataDirectory, "Instances", instance.Id.ToString());
    }

    public Task<List<Instance>> ListInstancesAsync(CancellationToken cancellationToken)
    {
        using var dbContext = _dbProvider.Create();
        
        // partially load instance data
        return dbContext.Instances
            .Select(x => new Instance { Id = x.Id, Name = x.Name })
            .ToListAsync(cancellationToken: cancellationToken);
    }

    public async Task CreateInstanceAsync(Instance options, CancellationToken cancellationToken)
    {
        using var dbContext = _dbProvider.Create();
        var wrappedContext = dbContext.GetWrappedContext();
        
        // add into db
        var instance = new Instance
        {
            Id = 0,
            WorkingDirectory = options.WorkingDirectory,
            Name = options.Name,
            LaunchCommand = options.LaunchCommand
        };

        var entry = await dbContext.Instances.AddAsync(instance, cancellationToken);
        await wrappedContext.SaveChangesAsync(cancellationToken);
        
        // create default working directory
        if (string.IsNullOrWhiteSpace(options.WorkingDirectory))
        {
            var defaultPath = GetInstancePath(entry.Entity);
            if (Directory.Exists(defaultPath))
            {
                Directory.Delete(defaultPath);
            }
            
            Directory.CreateDirectory(defaultPath);
        }
    }

    public async Task DeleteInstanceAsync(int instanceId, CancellationToken cancellationToken)
    {
        using var dbContext = _dbProvider.Create();
        var wrappedContext = dbContext.GetWrappedContext();
        
        var instance = await dbContext.Instances
            .FirstOrDefaultAsync(x => x.Id == instanceId, cancellationToken);
        if (instance == null) return;

        dbContext.Instances.Remove(instance);
        await wrappedContext.SaveChangesAsync(cancellationToken);
    }
    
    public async Task UpdateInstanceAsync(Instance instance, CancellationToken cancellationToken)
    {
        using var dbContext = _dbProvider.Create();
        var wrappedContext = dbContext.GetWrappedContext();

        var originalInstance = await dbContext.Instances
            .CountAsync(x => x.Id == instance.Id, cancellationToken);
        if (originalInstance == 0) return;
        
        dbContext.Instances.Update(instance);
        await wrappedContext.SaveChangesAsync(cancellationToken);
    }

    public async Task<Instance> GetInstanceAsync(int instanceId, CancellationToken cancellationToken)
    {
        using var dbContext = _dbProvider.Create();
        
        var instance = await dbContext.Instances
            .FirstAsync(x => x.Id == instanceId, cancellationToken);

        return instance;
    }
}
