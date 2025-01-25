using Microsoft.EntityFrameworkCore;

namespace Lcsm.Database.Schema;

[PrimaryKey(nameof(Id))]
public class Runner
{
    public int Id { get; set; }

    public RunnerType SocketType { get; set; } = RunnerType.Builtin;

    public string? Name { get; set; }
    
    public string SocketUri { get; set; } = "";
    
    public string? Description { get; set; }
}
