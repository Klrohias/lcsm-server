using System.ComponentModel.DataAnnotations;

namespace Lcsm.DataModels;

public class UserUpdateDto
{
    [Required]
    [StringLength(48, MinimumLength = 1)]
    public required string Username { get; set; }
    
    [Required]
    public required string Password { get; set; }
    
    public bool? IsAdministrator { get; set; }
}