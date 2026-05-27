using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using WinForgeShell.Services;

namespace WinForgeShell.Views;

public sealed partial class EventsPage : Page
{
    private readonly List<string> _events = new();

    public EventsPage()
    {
        InitializeComponent();
        Loaded   += EventsPage_Loaded;
        Unloaded += EventsPage_Unloaded;
    }

    private void EventsPage_Loaded(object sender, RoutedEventArgs e)
    {
        App.Current.IpcService.EventReceived += OnEventReceived;
        EmptyState.Visibility = _events.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    private void EventsPage_Unloaded(object sender, RoutedEventArgs e)
    {
        App.Current.IpcService.EventReceived -= OnEventReceived;
    }

    private void OnEventReceived(object? sender, PushEventMessage msg)
    {
        DispatcherQueue.TryEnqueue(() =>
        {
            string line = $"[{DateTime.Now:HH:mm:ss}]  {msg.Kind,-28} {FormatPayload(msg)}";
            _events.Insert(0, line);

            // Keep at most 500 entries to avoid unbounded memory growth.
            if (_events.Count > 500)
                _events.RemoveAt(_events.Count - 1);

            EventsItemsControl.ItemsSource = null;
            EventsItemsControl.ItemsSource = _events;
            EmptyState.Visibility = Visibility.Collapsed;
        });
    }

    private static string FormatPayload(PushEventMessage msg) => msg.Kind switch
    {
        "WorkflowStepStarted"   => $"workflow={msg.WorkflowId?[..8]}… step={msg.StepId}",
        "WorkflowStepCompleted" => $"workflow={msg.WorkflowId?[..8]}… step={msg.StepId} [{msg.Status}]",
        "WorkflowCompleted"     => $"workflow={msg.WorkflowId?[..8]}… [{msg.Status}]",
        "PluginLoaded"          => $"{msg.Name} v{msg.Version}",
        "Log"                   => $"[{msg.Level?.ToUpperInvariant()}] {msg.Message}",
        _                       => string.Empty,
    };

    private void ClearButton_Click(object sender, RoutedEventArgs e)
    {
        _events.Clear();
        EventsItemsControl.ItemsSource = null;
        EmptyState.Visibility = Visibility.Visible;
    }
}
