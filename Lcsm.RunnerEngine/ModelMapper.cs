using AutoMapper;
using Lcsm.RunnerEngine.Database.Schema;
using Lcsm.RunnerEngine.Protocol;
using Lcsm.RunnerEngine.Protocol.Models;

namespace Lcsm.RunnerEngine;

static class ModelMapper
{
    public static Mapper Mapper = new(new MapperConfiguration(x =>
    {
        x.CreateMap<InstanceDto, Instance>();
        x.CreateMap<Instance, InstanceDto>();
    }));
}