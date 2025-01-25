namespace Lcsm.ServerEngine.Protocol;

public enum PacketType
{
    Empty,
    
    ListInstances,
    GetInstance,
    CreateInstance,
    DeleteInstance,
    UpdateInstance,
    
    StartInstance,
    StopInstance,
    TerminalInstance,
    
    Error,
}