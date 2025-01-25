using Lcsm.Common;
using Lcsm.ServerEngine;
using Lcsm.ServerEngine.Protocol;
using Lcsm.ServerEngine.Sockets;

namespace Lcsm.Services;

public class BuiltinRunnerService : IBuiltinRunnerService
{
    public bool Started => _embedServer.Started;
    public IConnection Connection => _internalSocket.ClientConnection;
    
    private readonly EmbedServer _embedServer;
    private readonly InternalSocket _internalSocket = new();

    public BuiltinRunnerService(IServiceScopeFactory scopeFactory, IConfiguration configuration,
        ILogger<EmbedServer> loggerForEmbedServer)
    {
        var databaseProvider = new LcsmDatabaseProvider(scopeFactory);

        _embedServer = new EmbedServer(
            managedSocket: _internalSocket,
            dataDirectory: configuration["DataDirectory"] ?? Path.Combine(AppContext.BaseDirectory, "./Data/"),
            dbProvider: databaseProvider,
            logger: loggerForEmbedServer);

        _embedServer.Start();
    }

    public void Start()
    {
        _embedServer.Start();
    }

    public void Stop()
    {
        _embedServer.Stop();
    }
}