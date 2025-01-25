namespace Lcsm.ServerEngine.ServerManagement;

public interface IDbContextProvider
{
    public IDbContext Create();
}