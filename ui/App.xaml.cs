using Microsoft.UI.Xaml;
using WinForgeShell.Services;

namespace WinForgeShell;

public partial class App : Application
{
    /// <summary>Static reference to the running App instance.</summary>
    public new static App Current => (App)Application.Current;

    /// <summary>The singleton IPC service shared across all ViewModels.</summary>
    public IpcService IpcService { get; } = new IpcService();

    /// <summary>Reference to the main window, set during OnLaunched.</summary>
    public MainWindow? MainWindow { get; private set; }

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        MainWindow = new MainWindow();
        MainWindow.Activate();
    }
}
