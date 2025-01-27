using System.ComponentModel.DataAnnotations;
using Lcsm.Database.Schema;

namespace Lcsm.DataModels;

public class RunnerUpdateDto
{
    [Required] public RunnerType? SocketType { get; set; } = RunnerType.Builtin;

    [Required] public string? Name { get; set; }

    public string? SocketUri { get; set; } = "";

    public string? Description { get; set; }
}