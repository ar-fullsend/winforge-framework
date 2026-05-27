using System.Buffers.Binary;
using System.IO.Pipes;
using System.Text;
using System.Text.Json;

namespace WinForgeShell.Services;

/// <summary>
/// Manages the two Windows named pipes that connect the C# shell to the Rust backend.
///
/// Protocol framing (both pipes):
///   [4 bytes LE uint32 length][UTF-8 JSON body of that length]
///
/// Command pipe  ("winforge-shell-cmd"):
///   C# sends commands, Rust sends back exactly one response per command.
///
/// Event pipe    ("winforge-shell-evt"):
///   Rust pushes events unsolicited; C# reads them in a background loop.
/// </summary>
public sealed class IpcService : IDisposable
{
    // -----------------------------------------------------------------
    // Pipe names
    // -----------------------------------------------------------------

    private const string CmdPipeName = "winforge-shell-cmd";
    private const string EvtPipeName = "winforge-shell-evt";
    private const int    ConnectTimeoutMs = 5_000;

    // -----------------------------------------------------------------
    // Pipe streams
    // -----------------------------------------------------------------

    private NamedPipeClientStream? _cmdPipe;
    private NamedPipeClientStream? _evtPipe;

    // Serialise all command-pipe traffic so concurrent callers don't interleave frames.
    private readonly SemaphoreSlim _cmdLock = new(1, 1);

    // -----------------------------------------------------------------
    // Public events & properties
    // -----------------------------------------------------------------

    /// <summary>Fires on the thread-pool when a push event arrives on the event pipe.</summary>
    public event EventHandler<PushEventMessage>? EventReceived;

    /// <summary>Fires <c>true</c> when both pipes connect, <c>false</c> when either disconnects.</summary>
    public event EventHandler<bool>? ConnectionChanged;

    /// <summary><c>true</c> while both pipes are open and connected.</summary>
    public bool IsConnected { get; private set; }

    // -----------------------------------------------------------------
    // JSON options
    // -----------------------------------------------------------------

    private static readonly JsonSerializerOptions s_jsonOptions = new()
    {
        PropertyNameCaseInsensitive = true
    };

    // -----------------------------------------------------------------
    // Connect
    // -----------------------------------------------------------------

    /// <summary>
    /// Opens both named pipes (5 s timeout each) and starts the background
    /// event-receive loop.  Throws on failure so the caller can retry.
    /// </summary>
    public async Task ConnectAsync(CancellationToken ct)
    {
        // Dispose any previous streams
        DisposeStreams();

        var cmd = new NamedPipeClientStream(".", CmdPipeName,
            PipeDirection.InOut, PipeOptions.Asynchronous);

        var evt = new NamedPipeClientStream(".", EvtPipeName,
            PipeDirection.In, PipeOptions.Asynchronous);

        await cmd.ConnectAsync(ConnectTimeoutMs, ct).ConfigureAwait(false);
        await evt.ConnectAsync(ConnectTimeoutMs, ct).ConfigureAwait(false);

        _cmdPipe = cmd;
        _evtPipe = evt;

        IsConnected = true;
        ConnectionChanged?.Invoke(this, true);

        // Start background loop — not awaited; runs until disconnect or cancel
        _ = ReceiveEventsLoopAsync(ct);
    }

    // -----------------------------------------------------------------
    // Send command / receive response
    // -----------------------------------------------------------------

    /// <summary>
    /// Serialises <paramref name="command"/> with the 4-byte LE length prefix,
    /// writes it to the command pipe, reads back exactly one framed response,
    /// and returns it deserialised as <typeparamref name="TResponse"/>.
    /// </summary>
    public async Task<TResponse?> SendCommandAsync<TResponse>(object command, CancellationToken ct = default)
    {
        if (_cmdPipe is null || !_cmdPipe.IsConnected)
            throw new InvalidOperationException("Command pipe is not connected.");

        await _cmdLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            // Serialize and write
            string json = JsonSerializer.Serialize(command, s_jsonOptions);
            await WriteFrameAsync(_cmdPipe, json, ct).ConfigureAwait(false);

            // Read response
            string responseJson = await ReadFrameAsync(_cmdPipe, ct).ConfigureAwait(false);
            return JsonSerializer.Deserialize<TResponse>(responseJson, s_jsonOptions);
        }
        finally
        {
            _cmdLock.Release();
        }
    }

    // -----------------------------------------------------------------
    // Background event-receive loop
    // -----------------------------------------------------------------

    private async Task ReceiveEventsLoopAsync(CancellationToken ct)
    {
        try
        {
            while (!ct.IsCancellationRequested && _evtPipe is { IsConnected: true })
            {
                string json = await ReadFrameAsync(_evtPipe, ct).ConfigureAwait(false);

                PushEventMessage? msg = JsonSerializer.Deserialize<PushEventMessage>(
                    json, s_jsonOptions);

                if (msg is not null)
                    EventReceived?.Invoke(this, msg);
            }
        }
        catch (OperationCanceledException)
        {
            // Shutdown requested — exit silently
        }
        catch
        {
            // Pipe broke — signal disconnection
        }
        finally
        {
            if (IsConnected)
            {
                IsConnected = false;
                ConnectionChanged?.Invoke(this, false);
            }
        }
    }

    // -----------------------------------------------------------------
    // Framing helpers
    // -----------------------------------------------------------------

    /// <summary>
    /// Reads one frame: 4-byte LE uint32 length → reads that many bytes → returns UTF-8 string.
    /// </summary>
    private static async Task<string> ReadFrameAsync(Stream stream, CancellationToken ct)
    {
        // Read 4-byte length prefix
        byte[] lenBuf = new byte[4];
        await stream.ReadExactlyAsync(lenBuf, 0, 4, ct).ConfigureAwait(false);
        uint length = BinaryPrimitives.ReadUInt32LittleEndian(lenBuf);

        // Read body
        byte[] body = new byte[length];
        await stream.ReadExactlyAsync(body, 0, (int)length, ct).ConfigureAwait(false);

        return Encoding.UTF8.GetString(body);
    }

    /// <summary>
    /// Writes one frame: 4-byte LE uint32 length + UTF-8 JSON body.
    /// </summary>
    private static async Task WriteFrameAsync(Stream stream, string json, CancellationToken ct)
    {
        byte[] body = Encoding.UTF8.GetBytes(json);
        byte[] lenBuf = new byte[4];
        BinaryPrimitives.WriteUInt32LittleEndian(lenBuf, (uint)body.Length);

        // Write length prefix then body in two separate calls to avoid
        // allocating a combined buffer.
        await stream.WriteAsync(lenBuf, ct).ConfigureAwait(false);
        await stream.WriteAsync(body, ct).ConfigureAwait(false);
        await stream.FlushAsync(ct).ConfigureAwait(false);
    }

    // -----------------------------------------------------------------
    // Helpers for callers that want raw envelope discrimination
    // -----------------------------------------------------------------

    /// <summary>
    /// Sends a command and returns the raw JSON string of the response,
    /// useful when the caller wants to inspect the <c>kind</c> field before
    /// choosing which type to deserialise into.
    /// </summary>
    public async Task<string> SendCommandRawAsync(object command, CancellationToken ct = default)
    {
        if (_cmdPipe is null || !_cmdPipe.IsConnected)
            throw new InvalidOperationException("Command pipe is not connected.");

        await _cmdLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            string json = JsonSerializer.Serialize(command, s_jsonOptions);
            await WriteFrameAsync(_cmdPipe, json, ct).ConfigureAwait(false);
            return await ReadFrameAsync(_cmdPipe, ct).ConfigureAwait(false);
        }
        finally
        {
            _cmdLock.Release();
        }
    }

    // -----------------------------------------------------------------
    // Cleanup
    // -----------------------------------------------------------------

    private void DisposeStreams()
    {
        try { _cmdPipe?.Dispose(); } catch { /* ignore */ }
        try { _evtPipe?.Dispose(); } catch { /* ignore */ }
        _cmdPipe = null;
        _evtPipe = null;
    }

    public void Dispose()
    {
        IsConnected = false;
        DisposeStreams();
        _cmdLock.Dispose();
    }
}
