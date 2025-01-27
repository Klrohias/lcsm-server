using Lcsm.Runner;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using NLog;
using NLog.Extensions.Logging;

var logger = LogManager.GetCurrentClassLogger();
try
{
    using var servicesProvider = new ServiceCollection()
        .AddTransient<RunnerApplication>()
        .AddLogging(loggingBuilder =>
        {
            loggingBuilder.ClearProviders();
            loggingBuilder.AddNLog();
        }).BuildServiceProvider();

    var runner = servicesProvider.GetRequiredService<RunnerApplication>();
    runner.Main();
}
catch (Exception ex)
{
    logger.Error(ex, "Stopped program because of exception");
    throw;
}
finally
{
    LogManager.Shutdown();
}