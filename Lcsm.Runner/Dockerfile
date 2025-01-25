FROM mcr.microsoft.com/dotnet/runtime:8.0 AS base
USER $APP_UID
WORKDIR /app

FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build
ARG BUILD_CONFIGURATION=Release
WORKDIR /src
COPY ["Lcsm.Runner/Lcsm.Runner.csproj", "Lcsm.Runner/"]
RUN dotnet restore "Lcsm.Runner/Lcsm.Runner.csproj"
COPY . .
WORKDIR "/src/Lcsm.Runner"
RUN dotnet build "Lcsm.Runner.csproj" -c $BUILD_CONFIGURATION -o /app/build

FROM build AS publish
ARG BUILD_CONFIGURATION=Release
RUN dotnet publish "Lcsm.Runner.csproj" -c $BUILD_CONFIGURATION -o /app/publish /p:UseAppHost=false

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["dotnet", "Lcsm.Runner.dll"]
