using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using WinForgeShell.Views;

namespace WinForgeShell;

public sealed partial class MainWindow : Window
{
    private readonly CancellationTokenSource _cts = new();

    public MainWindow()
    {
        InitializeComponent();
        Closed += MainWindow_Closed;
    }

    private void MainWindow_Closed(object sender, WindowEventArgs args)
    {
        _cts.Cancel();
        App.Current.IpcService.ConnectionChanged -= OnConnectionChanged;
    }

    // Called from the root Grid's Loaded event (wired in XAML).
    // Window itself does not have a Loaded event in WinUI 3.
    private async void RootGrid_Loaded(object sender, RoutedEventArgs e)
    {
        // Wire connection status updates
        App.Current.IpcService.ConnectionChanged += OnConnectionChanged;

        // Navigate to Workflows by default
        NavView.SelectedItem = NavView.MenuItems[0];
        ContentFrame.Navigate(typeof(WorkflowsPage));

        // Start IPC connection with retry loop
        await StartIpcWithRetryAsync(_cts.Token);
    }

    private async Task StartIpcWithRetryAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                DispatcherQueue.TryEnqueue(() =>
                    UpdateStatusBadge("Connecting...", "#f39c12"));

                await App.Current.IpcService.ConnectAsync(ct);
                // ConnectionChanged(true) fires from within IpcService on success.
                return;
            }
            catch (OperationCanceledException)
            {
                return;
            }
            catch
            {
                // Pipes not available yet — wait 3 s and retry
                DispatcherQueue.TryEnqueue(() =>
                    UpdateStatusBadge("Disconnected", "#95a5a6"));

                try
                {
                    await Task.Delay(3_000, ct);
                }
                catch (OperationCanceledException)
                {
                    return;
                }
            }
        }
    }

    private void OnConnectionChanged(object? sender, bool connected)
    {
        DispatcherQueue.TryEnqueue(() =>
        {
            if (connected)
            {
                UpdateStatusBadge("Connected", "#2ecc71");
            }
            else
            {
                UpdateStatusBadge("Disconnected", "#95a5a6");
                // Reconnect in the background
                _ = StartIpcWithRetryAsync(_cts.Token);
            }
        });
    }

    private void UpdateStatusBadge(string text, string hexColor)
    {
        StatusText.Text = text;
        StatusDot.Fill = new SolidColorBrush(ParseHexColor(hexColor));
    }

    private static Windows.UI.Color ParseHexColor(string hex)
    {
        hex = hex.TrimStart('#');
        return Windows.UI.Color.FromArgb(
            255,
            Convert.ToByte(hex[0..2], 16),
            Convert.ToByte(hex[2..4], 16),
            Convert.ToByte(hex[4..6], 16));
    }

    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.SelectedItem is not NavigationViewItem item)
            return;

        Type? pageType = item.Tag?.ToString() switch
        {
            "Workflows" => typeof(WorkflowsPage),
            "Plugins"   => typeof(PluginsPage),
            "Events"    => typeof(EventsPage),
            _           => null
        };

        if (pageType is not null && ContentFrame.CurrentSourcePageType != pageType)
            ContentFrame.Navigate(pageType);
    }
}
