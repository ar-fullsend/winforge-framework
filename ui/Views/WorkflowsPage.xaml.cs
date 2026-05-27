using System.Collections.Specialized;
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
        Loaded   += WorkflowsPage_Loaded;
        Unloaded += WorkflowsPage_Unloaded;
    }

    private async void WorkflowsPage_Loaded(object sender, RoutedEventArgs e)
    {
        // Keep the empty-state overlay in sync with the Workflows collection.
        ViewModel.Workflows.CollectionChanged += OnWorkflowsCollectionChanged;
        UpdateWorkflowEmptyState();

        await ViewModel.LoadWorkflowsCommand.ExecuteAsync(null);
    }

    private void WorkflowsPage_Unloaded(object sender, RoutedEventArgs e)
    {
        ViewModel.Workflows.CollectionChanged -= OnWorkflowsCollectionChanged;
        ViewModel.Dispose();
    }

    private void OnWorkflowsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        UpdateWorkflowEmptyState();
    }

    private void UpdateWorkflowEmptyState()
    {
        WorkflowEmptyState.Visibility = ViewModel.Workflows.Count == 0
            ? Visibility.Visible
            : Visibility.Collapsed;
    }
}
