using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lcsm.Migrations
{
    /// <inheritdoc />
    public partial class AddRunnerName : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "Name",
                table: "Runners",
                type: "TEXT",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "Name",
                table: "Runners");
        }
    }
}
