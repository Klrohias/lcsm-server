using System.ComponentModel.DataAnnotations;

namespace Lcsm.DataModels;

public class AuthenticateRequestDto
{
    [Required]
    public string? Username { get; set; }
    
    [Required]
    public string? Password { get; set; }
}