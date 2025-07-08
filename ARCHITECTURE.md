# Codex-rs Architecture

## Overview

Codex-rs is a modular AI coding assistant with a flexible client-server architecture that supports multiple deployment modes and communication patterns.

## Core Architecture

### Protocol-Based Design

The system uses a **Submission Queue (SQ) / Event Queue (EQ)** pattern for communication:

- **Client → Server**: Operations (`Op` enum) like UserInput, ExecApproval
- **Server → Client**: Events (`EventMsg` enum) like AgentMessage, ExecApprovalRequest

This protocol is transport-agnostic and can work over:
- In-process channels
- stdin/stdout pipes  
- TCP sockets
- IPC mechanisms

### High-Level Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   TUI Mode  │     │  Exec Mode  │     │  MCP Mode   │
│ (Interactive)     │  (Headless) │     │  (Server)   │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┴───────────────────┘
                           │
                    ┌──────▼──────┐
                    │  Core Logic │
                    │  (Protocol) │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐      ┌──────▼──────┐   ┌──────▼──────┐
   │Sandbox  │      │MCP Clients  │   │Model Providers│
   └─────────┘      └─────────────┘   └─────────────┘
```

## Key Components

### 1. Entry Points

- **`codex` (cli)** - Main multitool with subcommands:
  - Interactive TUI mode (default)
  - `exec` - Non-interactive execution
  - `mcp` - Run as MCP server
  - `proto` - Protocol mode for stdin/stdout communication
  - `login` - Authentication
  - `config` - Configuration management
  - `debug` - Sandbox debugging

### 2. Core Library (`core/`)

- Business logic and protocol definitions
- Model provider abstractions
- Configuration management
- MCP connection management
- Execution and sandboxing logic

### 3. User Interfaces

- **TUI (`tui/`)** - Terminal UI with Ratatui
- **Exec (`exec/`)** - Headless CLI for automation
- **Protocol mode** - Raw protocol communication

### 4. Security & Sandboxing

- **Linux Sandbox (`linux-sandbox/`)** - Landlock and seccomp
- **macOS Sandbox** - Seatbelt sandboxing
- **Execution Policy (`execpolicy/`)** - Configurable approval policies

### 5. MCP (Model Context Protocol) Support

Codex can act as both client and server:

**As MCP Client:**
- Connects to external MCP servers
- Aggregates tools from multiple servers
- Uses qualified names: `server__NAME__tool`

**As MCP Server:**
- Runs via `codex mcp`
- JSON-RPC over stdin/stdout
- Exposes Codex functionality to other tools

## Communication Flow

### Protocol Messages

```rust
// Client → Server Operations
pub enum Op {
    UserInput { user_input: String },
    ConfigureSession { session: SessionConfig },
    ExecApproval { approved: bool },
    TaskComplete(TaskId),
    // ...
}

// Server → Client Events  
pub enum EventMsg {
    AgentMessage(ChatCompletionResponseStream),
    ExecApprovalRequest { cmd: String },
    TaskComplete,
    Error(String),
    // ...
}
```

### Session Management

- Sessions track conversation state
- Response IDs enable conversation threading
- Support for multiple concurrent sessions

## Deployment Modes

### 1. Interactive TUI (Default)
```bash
codex
```
Full-screen terminal interface with chat-like interaction.

### 2. Headless Execution
```bash
codex exec "refactor this function"
```
Non-interactive mode for automation and scripting.

### 3. MCP Server
```bash
codex mcp
```
Exposes Codex as an MCP server for integration with other tools.

### 4. Protocol Mode
```bash
codex proto
```
Raw protocol communication over stdin/stdout.

## Configuration System

- TOML configuration files (`config.toml`)
- Configuration profiles
- Environment variable overrides
- CLI flag overrides
- Custom approval predicates via external scripts

## Architecture Benefits

1. **Modularity** - Clear separation between UI, logic, and transport
2. **Flexibility** - Multiple deployment modes for different use cases
3. **Security** - Built-in sandboxing and approval mechanisms
4. **Extensibility** - MCP support for external tool integration
5. **Portability** - Cross-platform with platform-specific implementations

## Additional Components

- `ansi-escape/` - ANSI escape sequence handling
- `apply-patch/` - Code patching functionality
- `common/` - Shared utilities  
- `login/` - Authentication logic
- `mcp-types/` - MCP type definitions
- `mcp-client/` - MCP client implementation

## Future Architecture Considerations

The protocol-based design enables:
- Remote operation over network
- Web or GUI interfaces
- Integration with IDEs
- Distributed execution
- Multi-user collaboration
