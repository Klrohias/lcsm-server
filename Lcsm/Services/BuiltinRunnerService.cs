using Lcsm.Common;
using Lcsm.RunnerEngine;
using Lcsm.RunnerEngine.Protocol;
using Lcsm.RunnerEngine.Sockets;

namespace Lcsm.Services;

public class BuiltinRunnerService : IBuiltinRunnerService
{
    public bool Started => _embedEngine.Started;
    public IConnection Connection => _internalSocket.ClientConnection;

    private readonly EmbedEngine _embedEngine;
    private readonly InternalSocket _internalSocket = new();

    public BuiltinRunnerService(IServiceScopeFactory scopeFactory, IConfiguration configuration,
        ILogger<EmbedEngine> loggerForEmbedServer)
    {
        var databaseProvider = new LcsmDatabaseProvider(scopeFactory);

        _embedEngine = new EmbedEngine(
            managedSocket: _internalSocket,
            dataDirectory: configuration["DataDirectory"] ?? Path.Combine(AppContext.BaseDirectory, "./Data/"),
            dbProvider: databaseProvider,
            logger: loggerForEmbedServer);

        _embedEngine.Start();
    }

    public void Start()
    {
        _embedEngine.Start();
    }

    public void Stop()
    {
        _embedEngine.Stop();
    }
}
