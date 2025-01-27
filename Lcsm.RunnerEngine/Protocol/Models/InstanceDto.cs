using System.ComponentModel.DataAnnotations;

namespace Lcsm.RunnerEngine.Protocol.Models;

public class InstanceDto
{
    public int? Id { get; set; }
    
    [StringLength(48, MinimumLength = 1)] public string? Name { get; set; } = "Untitled";

    [StringLength(512)] public string? LaunchCommand { get; set; } = "";
    
    [StringLength(512)] public string? WorkingDirectory { get; set; }
    
    public bool IsRunning { get; set; }
}
