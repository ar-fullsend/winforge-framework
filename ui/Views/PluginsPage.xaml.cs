using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using WinForgeShell.ViewModels;

namespace WinForgeShell.Views;

public sealed partial class PluginsPage : Page
{
    public PluginsViewModel ViewModel { get; } = new PluginsViewModel();

    public PluginsPage()
    {
        InitializeComponent();
        Loaded += PluginsPage_Loaded;
    }

    private async void PluginsPage_Loaded(object sender, RoutedEventArgs e)
    {
        await ViewModel.LoadPluginsCommand.ExecuteAsync(null);
    }
}
