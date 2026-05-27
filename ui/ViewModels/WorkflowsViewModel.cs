using System.Collections.ObjectModel;
using System.Text.Json;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.UI.Dispatching;
using WinForgeShell.Services;

namespace WinForgeShell.ViewModels;

public sealed partial class WorkflowsViewModel : ObservableObject, IDisposable
{
    // ---------------------------------------------------------------
    // Observable state
    // ---------------------------------------------------------------

    [ObservableProperty]
    private ObservableCollection<WorkflowItem> _workflows = new();

    [ObservableProperty]
    [NotifyCanExecuteChangedFor(nameof(RunWorkflowCommand))]
    private WorkflowItem? _selectedWorkflow;

    /// <summary>
    /// Event log, newest entry at index 0 (top of list).
    /// Capped at 200 entries.
    /// </summary>
    [ObservableProperty]
    private ObservableCollection<string> _eventLog = new();

    [ObservableProperty]
    [NotifyCanExecuteChangedFor(nameof(RunWorkflowCommand))]
    private bool _isRunning;

    // ---------------------------------------------------------------
    // Infrastructure
    // ---------------------------------------------------------------

    private const int MaxLogEntries = 200;

    private readonly IpcService _ipc;
    private readonly DispatcherQueue _dispatcherQueue;

    public WorkflowsViewModel()
    {
        _ipc = ((App)global::Microsoft.UI.Xaml.Application.Current).IpcService;
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();

        _ipc.EventReceived += OnEventReceived;
    }

    // ---------------------------------------------------------------
    // Commands
    // ---------------------------------------------------------------

    [RelayCommand]
    private async Task LoadWorkflowsAsync()
    {
        try
        {
            var cmd = new ListWorkflowsCommand { Dir = "workflows" };
            var response = await _ipc.SendCommandAsync<WorkflowsResponse>(cmd);

            _dispatcherQueue.TryEnqueue(() =>
            {
                Workflows.Clear();
                if (response?.List is not null)
                {
                    foreach (var info in response.List)
                        Workflows.Add(WorkflowItem.FromInfo(info));
                }
            });
        }
        catch (Exception ex)
        {
            AppendLog($"[ERROR] LoadWorkflows failed: {ex.Message}");
        }
    }

    private bool CanRunWorkflow() =>
        SelectedWorkflow is not null && !IsRunning;

    [RelayCommand(CanExecute = nameof(CanRunWorkflow))]
    private async Task RunWorkflowAsync()
    {
        if (SelectedWorkflow is null || IsRunning)
            return;

        IsRunning = true;
        AppendLog($"[{Timestamp()}] Running workflow: {SelectedWorkflow.Name}");

        try
        {
            var cmd = new RunWorkflowCommand
            {
                Path    = SelectedWorkflow.Path,
                Context = new Dictionary<string, JsonElement>()
            };

            // Read the raw response so we can handle both WorkflowStarted and Error
            string raw = await _ipc.SendCommandRawAsync(cmd);
            var envelope = JsonSerializer.Deserialize<IpcEnvelope>(raw);

            if (envelope?.Kind == "WorkflowStarted")
            {
                var started = JsonSerializer.Deserialize<WorkflowStartedResponse>(raw);
                AppendLog($"[{Timestamp()}] Workflow started: {started?.Name} (run id={started?.WorkflowId})");
            }
            else if (envelope?.Kind == "Error")
            {
                var err = JsonSerializer.Deserialize<ErrorResponse>(raw);
                AppendLog($"[{Timestamp()}] Error: {err?.Message}");
            }
            else
            {
                AppendLog($"[{Timestamp()}] Unexpected response kind: {envelope?.Kind}");
            }
        }
        catch (Exception ex)
        {
            AppendLog($"[{Timestamp()}] [ERROR] RunWorkflow failed: {ex.Message}");
        }
        finally
        {
            _dispatcherQueue.TryEnqueue(() => IsRunning = false);
        }
    }

    // ---------------------------------------------------------------
    // Event subscription
    // ---------------------------------------------------------------

    private void OnEventReceived(object? sender, PushEventMessage evt)
    {
        string line = $"[{Timestamp()}] {evt.Summary()}";
        AppendLog(line);
    }

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    private void AppendLog(string line)
    {
        _dispatcherQueue.TryEnqueue(() =>
        {
            EventLog.Insert(0, line);
            while (EventLog.Count > MaxLogEntries)
                EventLog.RemoveAt(EventLog.Count - 1);
        });
    }

    private static string Timestamp() =>
        DateTime.Now.ToString("HH:mm:ss");

    public void Dispose()
    {
        _ipc.EventReceived -= OnEventReceived;
    }
}
