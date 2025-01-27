namespace Lcsm.RunnerEngine.Database;

public interface IDbContextProvider
{
    public IDbContext Create();
}