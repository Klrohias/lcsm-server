using System.ComponentModel.DataAnnotations;
using Microsoft.EntityFrameworkCore;

namespace Lcsm.Database.Schema;

[PrimaryKey(nameof(Id))]
public class User
{
    public int Id { get; set; }

    [StringLength(32)] public string Username { get; set; }

    public string Password { get; set; }

    public List<int> AllowedInstances { get; set; }
    
    public bool IsAdministrator { get; set; }
}