using System.ComponentModel.DataAnnotations;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.ServerEngine.ServerManagement.Schema;

[PrimaryKey(nameof(Id))]
public class Instance
{
    public int Id { get; set; }

    [StringLength(48, MinimumLength = 1)] public string Name { get; set; } = "";

    [StringLength(512)] public string LaunchCommand { get; set; } = "";
    
    [StringLength(512)] public string? WorkingDirectory { get; set; }
}
