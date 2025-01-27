using System.ComponentModel.DataAnnotations;

namespace Lcsm.DataModels;

public class InstanceUpdateDto
{
    [Required]
    [StringLength(48, MinimumLength = 1)]
    public string? Name { get; set; } = "Untitled";

    [StringLength(512)] public string? LaunchCommand { get; set; } = "";

    [StringLength(512)] public string? WorkingDirectory { get; set; }
}