using System.Text;
using Lcsm.Database;
using Lcsm.Database.Schema;
using Lcsm.DataModels;
using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Protocol;
using Lcsm.RunnerEngine.Protocol.Models;
using Lcsm.Services;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Tokens;
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
            
            // LCSM: configure controllers and openapi
            builder.Services.AddControllers();
            builder.Services.AddOpenApi();

            ConfigureServices(builder.Services);
            ConfigureAutoMapper(builder.Services);
            
            // LCSM: configure auth
            builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme)
                .AddJwtBearer(options =>
                {
                    var issuerSigningKey =
                        Encoding.Default.GetBytes(builder.Configuration["JwtSettings:IssuerSigningKey"] ?? "default");
                    options.TokenValidationParameters = new TokenValidationParameters
                    {
                        ValidateIssuerSigningKey = true,
                        ValidIssuers = builder.Configuration.GetSection("JwtSettings:ValidIssuers").Get<string[]>(),
                        ValidAudiences = builder.Configuration.GetSection("JwtSettings:ValidAudiences").Get<string[]>(),
                        IssuerSigningKey = new SymmetricSecurityKey(issuerSigningKey)
                    };
                });

            builder.Services.AddAuthorization(x =>
            {
                x.AddPolicy("Administrator", y => y.RequireRole("Administrator").Build());
            });
          
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

            // LCSM: configure middlewares
            if (app.Environment.IsDevelopment())
            {
                app.MapOpenApi();
                app.UseSwagger();
                app.UseSwaggerUI();
            }
            
            app.UseAuthentication();
            app.UseAuthorization();
            
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
            x.CreateMap<InstanceDto, Instance>();
            x.CreateMap<User, User>()
                .ForMember(x => x.Password, opt => opt.Ignore());
        });
    }

    private static void ConfigureServices(IServiceCollection service)
    {
        service.AddScoped<IUserService, UserService>();
        service.AddScoped<IRunnerService, RunnerService>();
        service.AddSingleton<IBuiltinRunnerService, BuiltinRunnerService>();
    }
}
