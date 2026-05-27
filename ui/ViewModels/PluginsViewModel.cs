using System.Collections.ObjectModel;
using System.Collections.Specialized;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using WinForgeShell.Services;

namespace WinForgeShell.ViewModels;

public sealed partial class PluginsViewModel : ObservableObject
{
    // ---------------------------------------------------------------
    // Observable state
    // ---------------------------------------------------------------

    [ObservableProperty]
    private ObservableCollection<PluginItem> _plugins = new();

    /// <summary>
    /// Visible when the list is empty; Collapsed otherwise.
    /// Bound to the empty-state overlay in the view.
    /// </summary>
    public Visibility IsEmpty =>
        Plugins.Count == 0 ? Visibility.Visible : Visibility.Collapsed;

    // ---------------------------------------------------------------
    // Infrastructure
    // ---------------------------------------------------------------

    private readonly IpcService _ipc;
    private readonly DispatcherQueue _dispatcherQueue;

    public PluginsViewModel()
    {
        _ipc = ((App)global::Microsoft.UI.Xaml.Application.Current).IpcService;
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();

        // Raise IsEmpty when the collection itself changes
        Plugins.CollectionChanged += OnPluginsCollectionChanged;
    }

    private void OnPluginsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        OnPropertyChanged(nameof(IsEmpty));
    }

    // ---------------------------------------------------------------
    // Commands
    // ---------------------------------------------------------------

    [RelayCommand]
    private async Task LoadPluginsAsync()
    {
        try
        {
            var cmd = new ListPluginsCommand();
            var response = await _ipc.SendCommandAsync<PluginsResponse>(cmd);

            _dispatcherQueue.TryEnqueue(() =>
            {
                Plugins.Clear();
                if (response?.List is not null)
                {
                    foreach (var info in response.List)
                        Plugins.Add(PluginItem.FromInfo(info));
                }
            });
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"LoadPlugins error: {ex.Message}");
        }
    }
}
