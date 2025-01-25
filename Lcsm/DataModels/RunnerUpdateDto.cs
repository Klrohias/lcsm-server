using Lcsm.Database.Schema;

namespace Lcsm.DataModels;

public class RunnerUpdateDto
{
    public RunnerType? SocketType { get; set; } = RunnerType.Builtin;

    public string? Name { get; set; }
    
    public string? SocketUri { get; set; } = "";
    
    public string? Description { get; set; }
}