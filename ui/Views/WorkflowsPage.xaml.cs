using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using WinForgeShell.ViewModels;

namespace WinForgeShell.Views;

public sealed partial class WorkflowsPage : Page
{
    public WorkflowsViewModel ViewModel { get; } = new WorkflowsViewModel();

    public WorkflowsPage()
    {
        InitializeComponent();
        Loaded += WorkflowsPage_Loaded;
        Unloaded += WorkflowsPage_Unloaded;
    }

    private async void WorkflowsPage_Loaded(object sender, RoutedEventArgs e)
    {
        await ViewModel.LoadWorkflowsCommand.ExecuteAsync(null);
    }

    private void WorkflowsPage_Unloaded(object sender, RoutedEventArgs e)
    {
        ViewModel.Dispose();
    }
}
