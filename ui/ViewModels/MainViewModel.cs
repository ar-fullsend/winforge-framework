using CommunityToolkit.Mvvm.ComponentModel;

namespace WinForgeShell.ViewModels;

/// <summary>
/// Tracks overall connection state for the title-bar status badge.
/// </summary>
public sealed partial class MainViewModel : ObservableObject
{
    [ObservableProperty]
    private bool _isConnected;

    [ObservableProperty]
    private string _connectionStatus = "Disconnected";

    [ObservableProperty]
    private string _connectionColor = "#95a5a6";

    partial void OnIsConnectedChanged(bool value)
    {
        if (value)
        {
            ConnectionStatus = "Connected";
            ConnectionColor  = "#2ecc71";
        }
        else
        {
            ConnectionStatus = "Disconnected";
            ConnectionColor  = "#95a5a6";
        }
    }

    /// <summary>
    /// Call when the app transitions to a "connecting" state.
    /// </summary>
    public void SetConnecting()
    {
        IsConnected      = false;
        ConnectionStatus = "Connecting...";
        ConnectionColor  = "#f39c12";
    }
}
