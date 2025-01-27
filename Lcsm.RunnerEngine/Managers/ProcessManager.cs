using System.Diagnostics;

namespace Lcsm.RunnerEngine.Managers;

public class ProcessManager
{
    private readonly Dictionary<int, Process> _childProcesses = [];
    private readonly object _syncRoot = new();

    private async void SuperviseProcessInternal(int instanceId, Process process)
    {
        await process.WaitForExitAsync();
        lock (_syncRoot)
        {
            _childProcesses.Remove(instanceId);
        } 
    }

    public bool IsRunning(int instanceId)
    {
        lock (_syncRoot)
        {
            return _childProcesses.ContainsKey(instanceId);
        }
    }

    public void StartProcess(int instanceId, string launchCommand, string workingDirectory)
    {
        lock (_syncRoot)
        {
            if (_childProcesses.ContainsKey(instanceId))
            {
                return;
            }
            
            var processStartInfo = new ProcessStartInfo
            {
                WorkingDirectory = workingDirectory,
                RedirectStandardInput = true,
                RedirectStandardError = true,
                RedirectStandardOutput = true,
                CreateNoWindow = true,
            };

            var splitCommand = launchCommand.Split(' ', 2);
            if (splitCommand.Length > 1)
            {
                processStartInfo.Arguments = splitCommand[1];
            }
            processStartInfo.FileName = splitCommand[0];

            var process = Process.Start(processStartInfo);
            
            ArgumentNullException.ThrowIfNull(process);
            _childProcesses.Add(instanceId, process);
            SuperviseProcessInternal(instanceId, process);
        }
    }

    public void TerminateProcess(int instanceId)
    {
        lock (_syncRoot)
        {
            if (_childProcesses.Remove(instanceId, out var process))
            {
                process.Kill();
            }
        }
    }
}