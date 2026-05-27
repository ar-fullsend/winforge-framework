using System.Text.Json;
using System.Text.Json.Serialization;

namespace WinForgeShell.Services;

// ---------------------------------------------------------------------------
// Shared envelope — every message (command, response, event) carries id+kind
// ---------------------------------------------------------------------------

/// <summary>
/// Base envelope shared by all messages on both pipes.
/// Deserialize this first to read the <see cref="Kind"/>, then re-deserialize
/// into the appropriate strongly-typed subclass.
/// </summary>
public sealed class IpcEnvelope
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;
}

// ---------------------------------------------------------------------------
// Commands  (C# → Rust, command pipe)
// ---------------------------------------------------------------------------

public sealed class PingCommand
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = Guid.NewGuid().ToString();

    [JsonPropertyName("kind")]
    public string Kind => "Ping";
}

public sealed class GetStatusCommand
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = Guid.NewGuid().ToString();

    [JsonPropertyName("kind")]
    public string Kind => "GetStatus";
}

public sealed class ListPluginsCommand
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = Guid.NewGuid().ToString();

    [JsonPropertyName("kind")]
    public string Kind => "ListPlugins";
}

public sealed class ListWorkflowsCommand
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = Guid.NewGuid().ToString();

    [JsonPropertyName("kind")]
    public string Kind => "ListWorkflows";

    [JsonPropertyName("dir")]
    public string Dir { get; init; } = "workflows";
}

public sealed class RunWorkflowCommand
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = Guid.NewGuid().ToString();

    [JsonPropertyName("kind")]
    public string Kind => "RunWorkflow";

    [JsonPropertyName("path")]
    public required string Path { get; init; }

    [JsonPropertyName("context")]
    public Dictionary<string, JsonElement> Context { get; init; } = new();
}

// ---------------------------------------------------------------------------
// Responses (Rust → C#, command pipe)
// ---------------------------------------------------------------------------

public sealed class PongResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;
}

public sealed class StatusResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    [JsonPropertyName("uptime_secs")]
    public long UptimeSecs { get; init; }

    [JsonPropertyName("plugin_count")]
    public int PluginCount { get; init; }

    [JsonPropertyName("running_workflows")]
    public int RunningWorkflows { get; init; }
}

public sealed class PluginInfo
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; init; } = string.Empty;

    [JsonPropertyName("description")]
    public string? Description { get; init; }

    [JsonPropertyName("capabilities")]
    public List<string> Capabilities { get; init; } = new();
}

public sealed class PluginsResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    [JsonPropertyName("list")]
    public List<PluginInfo> List { get; init; } = new();
}

public sealed class WorkflowInfo
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; init; } = string.Empty;

    [JsonPropertyName("description")]
    public string? Description { get; init; }

    [JsonPropertyName("path")]
    public string Path { get; init; } = string.Empty;

    [JsonPropertyName("step_count")]
    public int StepCount { get; init; }
}

public sealed class WorkflowsResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    [JsonPropertyName("list")]
    public List<WorkflowInfo> List { get; init; } = new();
}

public sealed class WorkflowStartedResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    /// <summary>The workflow run ID (separate from the message envelope ID).</summary>
    [JsonPropertyName("workflow_id")]
    public string WorkflowId { get; init; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; init; } = string.Empty;
}

public sealed class ErrorResponse
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    [JsonPropertyName("message")]
    public string Message { get; init; } = string.Empty;
}

// ---------------------------------------------------------------------------
// Push events (Rust → C#, event pipe)
// ---------------------------------------------------------------------------

/// <summary>
/// Discriminated union for all push events.  After reading the <see cref="Kind"/>
/// the appropriate strongly-typed properties are populated.
/// </summary>
public sealed class PushEventMessage
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("kind")]
    public string Kind { get; init; } = string.Empty;

    // --- WorkflowStepStarted / WorkflowStepCompleted / WorkflowCompleted ---

    [JsonPropertyName("workflow_id")]
    public string? WorkflowId { get; init; }

    [JsonPropertyName("step_id")]
    public string? StepId { get; init; }

    // "succeeded" | "failed" | "completed" | …
    [JsonPropertyName("status")]
    public string? Status { get; init; }

    // --- Log ---

    [JsonPropertyName("level")]
    public string? Level { get; init; }

    [JsonPropertyName("message")]
    public string? Message { get; init; }

    // --- PluginLoaded ---

    [JsonPropertyName("name")]
    public string? Name { get; init; }

    [JsonPropertyName("version")]
    public string? Version { get; init; }

    /// <summary>
    /// Human-readable summary of the event for display in the event log.
    /// </summary>
    public string Summary() => Kind switch
    {
        "WorkflowStepStarted"   => $"Step started: workflow={WorkflowId} step={StepId}",
        "WorkflowStepCompleted" => $"Step completed: workflow={WorkflowId} step={StepId} status={Status}",
        "WorkflowCompleted"     => $"Workflow completed: workflow={WorkflowId} status={Status}",
        "Log"                   => $"[{Level?.ToUpperInvariant()}] {Message}",
        _                       => Kind
    };
}

// ---------------------------------------------------------------------------
// View-model surface types (used by ViewModels / Views)
// ---------------------------------------------------------------------------

/// <summary>Thin wrapper around <see cref="WorkflowInfo"/> for list binding.</summary>
public sealed class WorkflowItem
{
    public string Name        { get; init; } = string.Empty;
    public string Version     { get; init; } = string.Empty;
    public string? Description { get; init; }
    public string Path        { get; init; } = string.Empty;
    public int    StepCount   { get; init; }

    public string DisplayLabel => $"{Name}  ({StepCount} steps)";

    public static WorkflowItem FromInfo(WorkflowInfo info) => new()
    {
        Name        = info.Name,
        Version     = info.Version,
        Description = info.Description,
        Path        = info.Path,
        StepCount   = info.StepCount
    };
}

/// <summary>Thin wrapper around <see cref="PluginInfo"/> for list binding.</summary>
public sealed class PluginItem
{
    public string Name         { get; init; } = string.Empty;
    public string Version      { get; init; } = string.Empty;
    public string? Description { get; init; }
    public string Capabilities { get; init; } = string.Empty;

    public static PluginItem FromInfo(PluginInfo info) => new()
    {
        Name         = info.Name,
        Version      = info.Version,
        Description  = info.Description,
        Capabilities = string.Join(", ", info.Capabilities)
    };
}
