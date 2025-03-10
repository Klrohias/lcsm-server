using Lcsm.RunnerEngine.Protocol;
using Lcsm.RunnerEngine.Sockets;

namespace Lcsm.Services;

public interface IBuiltinRunnerService
{
    public IConnection Connection { get; }

    public void Start();

    public void Stop();
    
    public bool Started { get; }
}