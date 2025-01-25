using AutoMapper;
using Lcsm.ServerEngine.Protocol;
using Lcsm.ServerEngine.ServerManagement.Schema;

namespace Lcsm.ServerEngine;

static class ModelMapper
{
    public static Mapper Mapper = new(new MapperConfiguration(x =>
    {
        x.CreateMap<InstanceUpdatePacket, Instance>();
        x.CreateMap<Instance, InstanceUpdatePacket>();
    }));
}