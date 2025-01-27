using Lcsm.RunnerEngine;
using Microsoft.Extensions.Logging;

namespace Lcsm.Runner;

public class RunnerApplication(ILogger<RunnerApplication> logger)
{
    private EmbedEngine _embedEngine;
    public void Main()
    {
        logger.LogInformation("Hello, world");
    }
}