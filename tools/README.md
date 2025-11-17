# VCV Rack CLI Client

![Build CLI Client](https://github.com/dstengle/Rack/workflows/Build%20CLI%20Client/badge.svg)

A command-line interface for interacting with the VCV Rack HTTP API.

## Download Pre-built Binaries

Pre-built binaries are available from GitHub Actions for all supported platforms:

1. Go to the [Actions tab](https://github.com/dstengle/Rack/actions/workflows/build-cli-client.yml)
2. Click on the latest successful workflow run
3. Download the artifact for your platform:
   - **Linux (x64)**: `rack-cli-linux-x64.tar.gz`
   - **macOS (x86_64)**: `rack-cli-macos-x86_64.tar.gz`
   - **macOS (ARM64)**: `rack-cli-macos-arm64.tar.gz`
   - **Windows (x64)**: `rack-cli-windows-x64.zip`

For releases, binaries are attached to the release page.

### Extract and Run

#### Linux / macOS
```bash
tar -xzf rack-cli-*.tar.gz
chmod +x rack-cli
./rack-cli --help
```

#### Windows
Extract `rack-cli-windows-x64.zip` and run:
```powershell
.\rack-cli.exe --help
```

## Building from Source

### Prerequisites

- C++ compiler (g++ or clang++)
- libcurl development libraries
- jansson development libraries

### Build with Make

From the root Rack directory:

```bash
# Build dependencies first (if not already built)
make dep

# Build the CLI client
make cli
```

The executable will be created at `tools/rack-cli` (or `tools/rack-cli.exe` on Windows).

### Manual Build

If you prefer to build manually:

```bash
# Linux
g++ -o tools/rack-cli tools/rack-cli.cpp \
    -Iinclude -Idep/include \
    -static-libstdc++ -static-libgcc \
    dep/lib/libcurl.a dep/lib/libssl.a dep/lib/libcrypto.a dep/lib/libjansson.a \
    -lpthread -ldl

# macOS
clang++ -o tools/rack-cli tools/rack-cli.cpp \
    -Iinclude -Idep/include \
    -stdlib=libc++ \
    dep/lib/libcurl.a dep/lib/libssl.a dep/lib/libcrypto.a dep/lib/libjansson.a \
    -lpthread -ldl \
    -framework CoreFoundation -framework Security
```

## Usage

First, start VCV Rack with the HTTP API enabled:

```bash
./Rack --httpapi
# or with custom port
./Rack --httpapi=9000
```

Then use the CLI client to interact with the running instance:

### List all modules in the patch

```bash
./tools/rack-cli list-modules
```

Example output:
```
Found 3 module(s) in the patch:

+--------+----------------------+----------------------+-----------------+-----------------+
| ID     | Plugin               | Model                | Position        | Size            |
+--------+----------------------+----------------------+-----------------+-----------------+
| 1      | Fundamental          | VCO-1                | 0,0             | 90x380          |
| 2      | Fundamental          | VCF                  | 100,0           | 75x380          |
| 3      | Fundamental          | VCA-1                | 200,0           | 60x380          |
+--------+----------------------+----------------------+-----------------+-----------------+
```

### Show detailed information about a specific module

```bash
./tools/rack-cli show-module 1
```

Example output:
```
Module Information:
  ID:          1
  Plugin:      Fundamental
  Model Slug:  VCO-1
  Model Name:  VCO-1

Parameters (8):
+------+---------------------------+-----------------+-----------------+----------------------+
| ID   | Name                      | Value           | Range           | Display              |
+------+---------------------------+-----------------+-----------------+----------------------+
| 0    | Frequency                 | 0.000           | -4.0 to 4.0     | 261.626 Hz           |
| 1    | Fine frequency            | 0.000           | -1.0 to 1.0     | 0.000 Hz             |
| 2    | Linear FM depth           | 0.000           | 0.0 to 1.0      | 0.0 %                |
...
+------+---------------------------+-----------------+-----------------+----------------------+

Inputs (4):
+------+--------------------------------+------------+------------+
| ID   | Name                           | Channels   | Connected  |
+------+--------------------------------+------------+------------+
| 0    | 1V/octave pitch                | 1          | Yes        |
| 1    | Linear FM                      | 1          | No         |
...
+------+--------------------------------+------------+------------+

Outputs (4):
+------+--------------------------------+------------+------------+
| ID   | Name                           | Channels   | Connected  |
+------+--------------------------------+------------+------------+
| 0    | Sine output                    | 1          | Yes        |
| 1    | Triangle output                | 1          | No         |
...
+------+--------------------------------+------------+------------+

Incoming Connections (1):
+----------+-----------------+--------------+-----------+
| Cable ID | From Module     | Output Port  | To Port   |
+----------+-----------------+--------------+-----------+
| 100      | 5               | 0            | 0         |
+----------+-----------------+--------------+-----------+

Outgoing Connections (1):
+----------+--------------+---------------+-------------+
| Cable ID | From Port    | To Module     | Input Port  |
+----------+--------------+---------------+-------------+
| 101      | 0            | 2             | 0           |
+----------+--------------+---------------+-------------+
```

### List all connections (cables)

```bash
# List all connections
./tools/rack-cli list-connections

# Sort by module ID
./tools/rack-cli list-connections --sort-by module

# Sort by connection type (Audio/CV)
./tools/rack-cli list-connections --sort-by type
```

Example output:
```
Found 5 connection(s) (sorted by type):

+----------+--------+----------------------+---------------------------+----------------------+---------------------------+
| Cable ID | Type   | From Module          | Output Port               | To Module            | Input Port                |
+----------+--------+----------------------+---------------------------+----------------------+---------------------------+
| 100      | Audio  | VCO-1 [1]            | Sine output               | VCF [2]              | Audio input               |
| 101      | Audio  | VCF [2]              | Lowpass output            | VCA-1 [3]            | Level input               |
| 102      | Audio  | VCA-1 [3]            | Audio output              | Audio-8 [4]          | Left input                |
| 103      | CV     | MIDI-CV [5]          | V/Oct                     | VCO-1 [1]            | 1V/octave pitch           |
| 104      | CV     | MIDI-CV [5]          | Gate                      | VCA-1 [3]            | Gate input                |
+----------+--------+----------------------+---------------------------+----------------------+---------------------------+

Summary:
  Audio connections: 3
  CV connections:    2
```

### Custom host and port

By default, the CLI connects to `localhost:8080`. You can specify a different host and port:

```bash
./tools/rack-cli --host 192.168.1.100 --port 9000 list-modules
```

### Help

```bash
./tools/rack-cli --help
```

## Connection Type Detection

The CLI client attempts to classify connections as either "Audio" or "CV" (Control Voltage) based on port names:

- **Audio ports** typically have names containing: "audio", "output", "signal", "sine", "saw", "square", "triangle", "mix", "left", "right", "in", "out"
- **CV ports** typically have names containing: "cv", "gate", "trigger", "clock", "mod", "fm", "pitch", "v/oct"

The classification is heuristic and may not be 100% accurate for all modules.

## Testing with Docker

To test the CLI client in the Docker build environment:

```bash
# Build the Docker image
./docker-build.sh build-image

# Build dependencies
./docker-build.sh deps

# Build Rack with HTTP API
./docker-build.sh build

# Build the CLI client
./docker-build.sh exec make cli

# In one terminal, run Rack with HTTP API
./docker-build.sh run

# In another terminal, test the CLI client
./docker-build.sh exec ./tools/rack-cli list-modules
```

## API Endpoints Used

The CLI client uses the following VCV Rack HTTP API endpoints:

- `GET /api/modules` - List all modules
- `GET /api/modules/:id` - Get module details
- `GET /api/cables` - List all connections

For more information about the HTTP API, see `HTTP_API.md` in the root directory.

## Troubleshooting

### Connection refused

Make sure VCV Rack is running with the HTTP API enabled:
```bash
./Rack --httpapi
```

### Port already in use

If port 8080 is already in use, specify a different port:
```bash
# Start Rack on port 9000
./Rack --httpapi=9000

# Connect CLI to port 9000
./tools/rack-cli --port 9000 list-modules
```

### No modules found

If you see "No modules in the patch", it means the current patch is empty. Try:
1. Opening an existing patch in Rack
2. Adding some modules manually in Rack
3. Then running the CLI client again

## Development

The CLI client is a standalone C++ program that:
- Uses libcurl for HTTP requests
- Uses jansson for JSON parsing
- Formats output as ASCII tables
- Supports cross-platform builds (Linux, macOS, Windows)

Source code: `tools/rack-cli.cpp`

## License

The CLI client is part of the VCV Rack project and follows the same licensing as VCV Rack.
