using Lcsm.Database;
using Lcsm.Database.Schema;
using Lcsm.DataModels;
using Lcsm.ServerEngine.Protocol;
using Lcsm.ServerEngine.ServerManagement.Schema;
using Lcsm.Services;
using Microsoft.EntityFrameworkCore;
using NLog;
using NLog.Web;

namespace Lcsm;

class Program
{
    private static void Main(string[] args)
    {
        // NLog: early load
        var logger = LogManager.Setup().LoadConfigurationFromAppSettings().GetCurrentClassLogger();
        logger.Debug("init main");

        try
        {
            // ASP.NET Core: start building
            var builder = WebApplication.CreateBuilder(args);

            builder.Services.AddControllers();
            builder.Services.AddOpenApi();

            ConfigureServices(builder.Services);
            ConfigureAutoMapper(builder.Services);
          
            // NLog: Configure
            builder.Logging.ClearProviders();
            builder.Host.UseNLog();

            // Swagger: developing
            if (builder.Environment.IsDevelopment())
            {
                builder.Services.AddSwaggerGen();
            }
            
            // EFCore: add database
            builder.Services
                .AddDbContext<LcsmDbContext>(x =>
                    x.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
            
            var app = builder.Build();

            // Configure the HTTP request pipeline.
            if (app.Environment.IsDevelopment())
            {
                app.MapOpenApi();
                app.UseSwagger();
                app.UseSwaggerUI();
            }

            app.MapControllers();
            app.UseHttpsRedirection();
            
            app.Run();
        } catch (Exception exception)
        {
            // NLog: catch setup errors
            logger.Error(exception, "Stopped program because of exception");
            throw;
        }
        finally
        {
            // NLog: Shutdown
            LogManager.Shutdown();
        }

    }

    private static void ConfigureAutoMapper(IServiceCollection service)
    {
        service.AddAutoMapper(x =>
        {
            x.CreateMap<RunnerUpdateDto, Runner>();
            x.CreateMap<InstanceUpdateDto, Instance>();
        });
    }

    private static void ConfigureServices(IServiceCollection service)
    {
        service.AddScoped<IUserService, UserService>();
        service.AddScoped<IRunnerService, RunnerService>();
        service.AddSingleton<IBuiltinRunnerService, BuiltinRunnerService>();
    }
}
