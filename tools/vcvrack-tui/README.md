# VCV Rack Terminal Client

A terminal user interface (TUI) client for controlling VCV Rack patches via the HTTP API.

## Features

- **Module Management**: Add, remove, and inspect modules in your patch
- **Cable Management**: Create and remove cable connections between modules
- **Smart Completion**: Tab completion for commands, modules, and ports
- **Human-Friendly Names**: Modules get auto-assigned names like `VCO-1-1`, `VCF-1`, etc.
- **Real-Time Updates**: Automatic refresh from the VCV Rack server

## Prerequisites

- VCV Rack running with the HTTP API enabled (`--httpapi` flag)
- Rust toolchain for building from source

## Building

```bash
cd tools/vcvrack-tui
cargo build --release
```

The binary will be at `target/release/vcvrack-tui`.

## Usage

### Starting VCV Rack with HTTP API

```bash
./Rack --httpapi        # Default port 8080
./Rack --httpapi=9000   # Custom port
```

### Running the TUI Client

```bash
# Connect to default localhost:8080
vcvrack-tui

# Connect to custom host/port
vcvrack-tui --host localhost --port 9000
```

### Command Line Options

```
Options:
  -H, --host <HOST>   Server hostname [default: localhost]
  -p, --port <PORT>   Server port [default: 8080]
  -c, --config <FILE> Config file path
  --no-color          Disable colors
  --help              Show help
  --version           Show version
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `add <module>` | Add a module to the patch | `add VCV VCO-1` |
| `remove <name>` | Remove a module | `remove VCO-1-1` |
| `connect <src:port> <dst:port>` | Create a cable | `connect VCO-1-1:Sine VCF-1:In` |
| `disconnect <src:port> <dst:port>` | Remove a cable | `disconnect VCO-1-1:Sine VCF-1:In` |
| `list [modules\|cables]` | List modules or cables | `list cables` |
| `show <name>` | Show module details | `show VCO-1-1` |
| `help` | Show help | `help` |
| `refresh` | Refresh from server | `refresh` |
| `quit` / `exit` | Exit the application | `quit` |

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `?` / `F1` | Show help |
| `Tab` | Complete / Cycle completions |
| `Up` / `Down` | Navigate completions or command history |
| `Enter` | Execute command or select completion |
| `Esc` | Close popup / Clear input |
| `Ctrl+C` | Exit |
| `Ctrl+R` | Refresh from server |
| `Ctrl+U` | Clear input line |

## Module Naming

Modules are automatically named using the model name plus an instance number:

- First `VCO-1` added → `VCO-1-1`
- Second `VCO-1` added → `VCO-1-2`
- First `VCF` added → `VCF-1`

Use these friendly names instead of numeric IDs when referencing modules.

## Configuration

Config file location: `~/.config/vcvrack-tui/config.toml`

```toml
[server]
host = "localhost"
port = 8080
timeout_ms = 5000

[ui]
refresh_interval_ms = 1000
color_theme = "dark"
show_help_hints = true

[completion]
auto_suggest = true
fuzzy_match = true
max_suggestions = 10

[layout]
module_spacing = 100
```

## Example Session

```
> add VCV VCO-1
Added VCO-1-1 (VCV VCO-1)

> add VCV VCF
Added VCF-1 (VCV VCF)

> connect VCO-1-1:Sine VCF-1:In
Connected VCO-1-1:Sine -> VCF-1:In

> show VCF-1
[Shows VCF-1 details with all inputs/outputs]

> list cables
CABLES (1)
  VCO-1-1:Sine output  ->  VCF-1:In

> quit
```

## API Limitations

Based on the current VCV Rack HTTP API:

- **No parameter setting**: Parameters are read-only
- **No patch save/load**: Use VCV Rack's native patch management
- **No undo/redo**: Not supported by the API

## License

GPL-3.0 (same as VCV Rack)
