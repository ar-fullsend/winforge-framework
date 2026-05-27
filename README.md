# WinForge Framework

**WinForge** is a powerful, modular, and secure framework for building complex, enterprise-grade, and high-performance applications natively on Windows.

It combines the raw power and safety of Rust for systems-level components with the rich ecosystem and rapid development capabilities of C# and WinUI 3, while offering deep, first-class integration with the Windows operating system.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![.NET](https://img.shields.io/badge/.NET-9-512BD4?logo=dotnet)](https://dotnet.microsoft.com/)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![WinUI](https://img.shields.io/badge/WinUI-3-0078D4)](https://learn.microsoft.com/windows/apps/winui/)

---

## Vision & Philosophy

Modern Windows development often involves juggling multiple technologies, fighting platform quirks, and rebuilding common infrastructure for security, observability, and extensibility. **WinForge** aims to solve this by providing a cohesive, opinionated yet flexible foundation.

**Core Principles:**

- **Security by Default**: Capability-based security, least privilege, seamless integration with Windows security features (TPM, AppContainer, Credential Guard, code signing).
- **Performance Without Sacrifices**: Rust core for hot paths, zero-cost abstractions where possible, efficient IPC and shared memory.
- **True Modularity**: Hot-swappable, signed plugins with clear contracts. Support for polyglot extensions.
- **Observability First**: Everything is measurable and traceable out of the box using native Windows mechanisms + OpenTelemetry.
- **Developer Joy**: Excellent CLI, scaffolding, hot reload, strong typing across boundaries, and comprehensive documentation.
- **Windows-Native**: Not a cross-platform abstraction that ignores Windows strengths. We embrace ETW, Performance Counters, WinRT, COM interop, MSIX, Task Scheduler, and more.

WinForge is ideal for:
- Data center and infrastructure management tools
- Privacy and security desktop applications
- AI/Agent-powered productivity tools (on-device inference, workflow automation)
- Enterprise line-of-business applications requiring long-running workflows and robust auditing
- Complex hybrid desktop + service applications

---

## Architecture

WinForge follows a layered architecture with clear separation of concerns:

```mermaid
graph TD
    subgraph "Developer Experience"
        CLI[winforge CLI] --> Scaffolder[Scaffolder & Codegen]
        CLI --> HotReload[Hot Reload Dev Server]
    end

    subgraph "Core Runtime (Rust)"
        Runtime[Actor Runtime & Scheduler] --> Loader[Plugin Loader]
        Runtime --> EventBus[Typed Event Bus]
        Runtime --> Workflow[Workflow & Saga Engine]
        Runtime --> Interop[Windows Interop Layer]
        Loader --> Capability[Capability Enforcer]
    end

    subgraph "Application Layer (C#)"
        UI[WinUI 3 Shell / Pages]
        ViewModels[Reactive ViewModels]
        Services[Hosted Services]
    end

    subgraph "Extensibility"
        Plugins[Signed Plugins<br/>(Rust / C# / Python / PS) ]
        Scripts[Scripting Runtime]
    end

    subgraph "Windows Platform"
        ETW[ETW / Perf Counters]
        NamedPipes[Named Pipes + Auth]
        TPM[TPM 2.0 / Windows Hello]
        MSIX[MSIX Packaging]
        Services[Windows Services]
    end

    CLI --> Runtime
    Runtime --> UI
    Runtime --> Plugins
    Interop --> ETW
    Interop --> NamedPipes
    Capability --> Plugins
    Workflow --> EventBus
```

### Layers Explained

1. **Rust Core Runtime** (`winforge-core`)
   - Built with `tokio`, `tracing`, `windows-rs`.
   - Actor model for concurrency.
   - High-performance shared memory and named pipe transports.
   - Capability-based sandboxing using Windows Job Objects and AppContainer where applicable.
   - Workflow engine capable of persisting state for long-running processes.

2. **C# / WinUI Application Layer**
   - Modern MVVM with CommunityToolkit.
   - Reactive UI updates via EventBus subscriptions.
   - Hosted services for background work.
   - Seamless hosting of WebView2 for hybrid scenarios.

3. **Plugin System**
   - Manifest-driven with cryptographic signing.
   - Capability declarations (e.g., `filesystem.read`, `gpu.compute`, `network.http`).
   - Dynamic loading with version compatibility checks.
   - Cross-language via well-defined FFI or gRPC boundaries.

4. **Workflow Orchestration**
   - Declarative definition (YAML/JSON or fluent C# API).
   - Supports sequential, parallel, conditional, retry, compensation (saga), and human approval steps.
   - Durable execution with checkpointing.

---

## Technology Stack

| Layer              | Technologies                                      | Key Libraries / Crates                          |
|--------------------|---------------------------------------------------|-------------------------------------------------|
| **Core**           | Rust 1.85+                                        | `windows-rs`, `tokio`, `tracing`, `serde`       |
| **UI**             | C# 12 / .NET 9, WinUI 3, WinAppSDK                | CommunityToolkit.Mvvm, WinUIEx, Uno (optional)  |
| **Scripting**      | Python, PowerShell 7+                             | pythonnet / PyO3, System.Management.Automation  |
| **IPC**            | Named Pipes, gRPC, QUIC                           | `tonic`, `prost`, MsQuic                        |
| **Data**           | SQLite, LiteDB, SQL Server LocalDB                | Microsoft.Data.Sqlite, Entity Framework Core    |
| **Observability**  | ETW, OpenTelemetry, Performance Counters          | `opentelemetry`, Serilog + ETW sink             |
| **Packaging**      | MSIX, App Installer, self-updating                | Windows App SDK packaging, differential updates |
| **Security**       | TPM 2.0, Code Signing, AppContainer               | Windows.Security, rustls or schannel            |
| **AI / Compute**   | ONNX Runtime + DirectML, NPU support              | Microsoft.ML.OnnxRuntime.DirectML               |

---

## Getting Started

### Prerequisites

- **Windows 11** (recommended) or Windows 10 version 22H2+
- Visual Studio 2022 (with "Desktop development with C++" and ".NET desktop development") or VS Code + extensions
- [.NET 9 SDK](https://dotnet.microsoft.com/download/dotnet/9.0)
- [Rust](https://rustup.rs/) (stable)
- Windows App SDK (latest)
- PowerShell 7+ (for full scripting support)
- Git

### 1. Install the WinForge CLI (from source for now)

```bash
git clone https://github.com/ar-fullsend/winforge-framework.git
cd winforge-framework
# Build the CLI tool
cargo build -p winforge-cli --release
# Add to PATH or use directly
```

### 2. Create a New Project

```bash
winforge new MySecureApp --template enterprise-desktop --features "workflow,ai,security"
cd MySecureApp
```

This scaffolds:
- A Rust workspace (`core/`, `plugins/`, `shared/`)
- A C# solution with WinUI 3 project
- Sample capability-declared plugin
- Example durable workflow
- Pre-configured GitHub Actions for building, testing, signing, and MSIX creation
- Comprehensive documentation in `/docs`

### 3. Run in Development Mode

```bash
winforge dev
```

This starts the hot-reload environment, watches for changes in Rust and C# code, and provides a rich logging dashboard.

---

## Core Concepts & How It Works

### Plugins & Capabilities

Every extension declares its required capabilities in a manifest. The runtime enforces them strictly.

**Example `plugin.manifest.json`:**

```json
{
  "id": "com.acme.datavault",
  "name": "Secure Data Vault Plugin",
  "version": "0.9.0",
  "author": "Acme Corp",
  "capabilities": [
    "filesystem.read",
    "filesystem.write",
    "crypto.encrypt",
    "ai.inference.local"
  ],
  "entrypoint": {
    "rust": "datavault_plugin.dll",
    "csharp": "Acme.DataVault.Plugin"
  },
  "permissions": {
    "allowed_paths": ["%APPDATA%\\WinForge\\DataVault"]
  }
}
```

At load time, the Capability Enforcer creates a restricted execution context. Violations are logged via ETW and can terminate the plugin.

### The Event Bus

A high-performance, typed event bus is the backbone of communication.

```rust
// In Rust core
#[derive(Event)]
struct DataProcessedEvent {
    record_id: u64,
    checksum: String,
}

// Publish
event_bus.publish(DataProcessedEvent { ... }).await;
```

C# side subscribes easily:

```csharp
[EventHandler]
public partial class DataProcessedHandler : IEventHandler<DataProcessedEvent>
{
    public Task HandleAsync(DataProcessedEvent evt, CancellationToken ct)
    {
        // Update UI reactively
        return Task.CompletedTask;
    }
}
```

### Workflow Engine

Define complex, long-running processes that survive restarts.

**`workflows/data-pipeline.workflow.yaml`:**

```yaml
name: SecureDataPipeline
version: "1.0"
steps:
  - id: validate_input
    type: plugin
    plugin_id: com.acme.datavault
    action: validate

  - id: ai_analyze
    type: ai_inference
    model: "local/privacy-guard-v2.onnx"
    input: "${{ steps.validate_input.output }}"

  - id: store
    type: plugin
    plugin_id: com.acme.datavault
    action: encrypt_and_store
    compensation: rollback_storage

on_failure:
  - notify_admin
  - compensate: rollback_storage
```

The engine handles checkpointing to local durable storage, automatic retries with backoff, and distributed tracing across steps.

### Windows Service Hosting

WinForge applications can be hosted as proper Windows Services with graceful shutdown, recovery policies, and integration with Service Control Manager.

```csharp
// In Program.cs or service host
builder.Services.AddWinForgeHostedService<MyBackgroundOrchestrator>();
```

### Observability

- All internal operations emit structured ETW events.
- OpenTelemetry traces and metrics are collected automatically.
- A built-in dashboard (WinUI or web via WebView2) shows real-time metrics, plugin health, and workflow status.
- Easy export to Azure Monitor, Prometheus, or ELK.

---

## Example Use Cases

1. **Privacy-First Desktop Agent**
   On-device content analysis, screen region protection helpers, local model inference, and secure local vault — all while respecting strict capability boundaries.

2. **Infrastructure Operations Tool**
   Multi-site technical visit planner + executor with offline capability, robust logging of every action, hardware interaction via Windows APIs, and workflow approval gates.

3. **Enterprise Workflow Automation**
   Long-running business processes (document processing, approval chains, data reconciliation) with full audit trail and compensation logic.

4. **AI-Augmented Productivity Suite**
   Local NPU-accelerated inference, agentic workflows, and deep integration with Windows Shell and notifications.

---

## Project Structure (Scaffolded)

```
MySecureApp/
├── winforge.toml          # Project manifest
├── Cargo.toml             # Rust workspace
├── src/
│   ├── core/                # Rust core crates
│   ├── plugins/
│   └── shared/
├── MySecureApp.UI/        # C# WinUI 3 project
├── workflows/             # Declarative workflow definitions
├── plugins/               # Plugin source + manifests
├── docs/
├── .github/workflows/     # Build, test, package, sign, release
├── tests/
└── README.md
```

---

## Roadmap

- **v0.1 (Current)**: Core runtime, basic plugin loader, CLI scaffolding, Event Bus, Rust + C# interop foundation.
- **v0.2**: Full workflow engine with durability, WinUI 3 reference implementation, ETW + OpenTelemetry integration.
- **v0.3**: Advanced security (AppContainer enforcement, TPM-backed keys), Python/PowerShell hosting, MSIX packaging templates.
- **v0.4**: Visual workflow designer (WinUI-based), hot-reload improvements, performance profiling tools.
- **v1.0**: Production hardened, comprehensive documentation, plugin marketplace support, NPU/DirectML first-class experience.

See the [GitHub Projects](https://github.com/ar-fullsend/winforge-framework/projects) board and open issues for detailed tasks.

---

## Contributing

Contributions are highly encouraged! We especially value:

- Windows platform expertise and API bindings
- Performance optimizations in the Rust core
- New plugin examples and templates
- Improvements to the workflow DSL and engine
- Documentation, tutorials, and sample applications
- Testing infrastructure (property-based, UI automation, fuzzing)

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Before submitting a PR, please open an issue to discuss significant changes.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

Built with appreciation for the Windows developer community, the Rust systems programming ecosystem, and the .NET team for making WinUI 3 and the App SDK excellent.

Special thanks to all contributors and early adopters.

---

*WinForge — Empowering developers to build the future of Windows software, one secure module at a time.*
