namespace Lcsm.RunnerEngine.Protocol.Models;

public static class PacketAction
{
    public const string Empty = "";

    public const string ListInstances = nameof(ListInstances);
    public const string GetInstance = nameof(GetInstance);
    public const string CreateInstance = nameof(CreateInstance);
    public const string DeleteInstance = nameof(DeleteInstance);
    public const string UpdateInstance = nameof(UpdateInstance);

    public const string StartInstance = nameof(StartInstance);
    public const string StopInstance = nameof(StopInstance);
    public const string TerminateInstance = nameof(TerminateInstance);
    public const string TerminalInput = nameof(TerminalInput);
}
