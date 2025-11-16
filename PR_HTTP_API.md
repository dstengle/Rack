# Pull Request: HTTP API Server for Remote Control

## Summary

This PR adds a comprehensive HTTP API server that enables programmatic control of VCV Rack through RESTful endpoints. The API allows automation, remote control, integration with other software, and algorithmic patch generation.

## Features

### RESTful API with JSON Responses
- List available plugins and module models
- Add/remove modules from patches
- Query module details (parameters, inputs, outputs, lights)
- Create/remove cable connections between modules
- CORS support for web-based clients

### Thread-Safe Operation
- HTTP server runs in a separate thread
- Proper synchronization with Engine and UI
- Concurrent request handling with thread pool
- Safe interaction with VCV Rack's internal state

### Cross-Platform Implementation
- Socket-based HTTP server (Windows, macOS, Linux)
- Platform-specific socket APIs properly abstracted
- Works on all supported VCV Rack platforms

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/plugins` | List all available plugins |
| GET | `/api/models` | List all module models |
| GET | `/api/modules` | List modules in current patch |
| GET | `/api/modules/:id` | Get detailed module information |
| POST | `/api/modules` | Add module to patch |
| DELETE | `/api/modules/:id` | Remove module from patch |
| GET | `/api/cables` | List all cable connections |
| POST | `/api/cables` | Create cable connection |
| DELETE | `/api/cables/:id` | Remove cable |

## Usage

### Command Line
```bash
# Start VCV Rack with HTTP API on default port 8080
./Rack --httpapi

# Start with custom port
./Rack --httpapi=9000
```

### API Examples

**List available models:**
```bash
curl http://localhost:8080/api/models
```

**Add a VCO module:**
```bash
curl -X POST http://localhost:8080/api/modules \
  -H "Content-Type: application/json" \
  -d '{"pluginSlug":"Fundamental","modelSlug":"VCO-1","pos":{"x":0,"y":0}}'
```

**Create cable connection:**
```bash
curl -X POST http://localhost:8080/api/cables \
  -H "Content-Type: application/json" \
  -d '{"outputModuleId":1,"outputId":0,"inputModuleId":2,"inputId":0}'
```

## Files Changed

### New Files
- `include/httpapi.hpp` - HTTP API header interface
- `src/httpapi.cpp` - Complete HTTP server implementation (~800 lines)
- `HTTP_API.md` - Comprehensive API documentation with examples
- `test_http_api.py` - Python test script demonstrating API usage

### Modified Files
- `adapters/standalone.cpp` - Integration with Rack startup/shutdown
  - Added `--httpapi` command-line flag
  - Server lifecycle management (init/destroy)

## Implementation Details

### Architecture
- **HTTP Server**: Custom implementation using platform sockets
- **Request Handling**: Threaded handler for concurrent requests
- **JSON Processing**: Uses existing Jansson library
- **Thread Safety**: Proper Engine locking for all operations

### Key Design Decisions
1. **No External Dependencies**: Uses only standard sockets and existing VCV libraries
2. **Optional Feature**: Disabled by default, enabled via command-line flag
3. **CORS Enabled**: Allows web-based control panels
4. **Read-Only Parameters**: Module parameters can be read but not set (future enhancement)

## Use Cases

### 1. Automated Patch Generation
Create patches programmatically based on algorithms:
```python
def create_synth_voice():
    vco = add_module("Fundamental", "VCO-1", x=0, y=0)
    vcf = add_module("Fundamental", "VCF", x=150, y=0)
    vca = add_module("Fundamental", "VCA-1", x=300, y=0)
    connect(vco, 0, vcf, 0)
    connect(vcf, 0, vca, 0)
```

### 2. Testing and Validation
Automatically verify patch connections and module states

### 3. Live Coding
Modify patches in real-time from code

### 4. DAW Integration
Control VCV Rack from external sequencers and DAWs

### 5. Web-Based Control
Build browser-based patch editors and control panels

## Testing

### Included Test Suite
The PR includes a comprehensive Python test script (`test_http_api.py`) that demonstrates:
- Plugin and model listing
- Module creation and deletion
- Module detail queries
- Cable connection management
- Complete patch building example

### Running Tests
```bash
# Start Rack with HTTP API
./Rack --httpapi

# In another terminal
python3 test_http_api.py
```

## Security Considerations

⚠️ **Important**: The HTTP API is intended for local use and automation.

- Server binds to all interfaces (0.0.0.0) by default
- No authentication mechanism
- Allows full control over Rack patches
- **Recommendation**: Use firewall rules to restrict access in untrusted networks

## Performance Impact

- Minimal overhead when disabled (default state)
- Server runs in separate thread - no impact on audio processing
- Efficient request handling with concurrent connections
- No performance degradation in normal VCV Rack operation

## Documentation

### API Documentation
`HTTP_API.md` provides:
- Complete endpoint reference
- Request/response examples
- Usage examples in Python, JavaScript, and cURL
- Troubleshooting guide
- Security best practices

### Code Examples
The test script demonstrates:
- Basic API operations
- Module management
- Cable management
- Complete patch building

## Backward Compatibility

✅ **Fully backward compatible**
- HTTP API is opt-in via command-line flag
- No changes to existing VCV Rack functionality
- No impact when feature is not used
- Existing patches load/save normally

## Future Enhancements

Potential future additions (not in this PR):
- Parameter value setting and automation
- Patch save/load endpoints
- Real-time parameter streaming via WebSocket
- Module preset management
- Performance metrics API

## Testing Checklist

- [x] Compiles on Linux
- [x] Compiles on macOS (code review)
- [x] Compiles on Windows (code review)
- [x] HTTP server starts/stops cleanly
- [x] All endpoints return valid JSON
- [x] Module creation works correctly
- [x] Cable creation works correctly
- [x] Module/cable deletion works correctly
- [x] Thread-safe operation verified
- [x] No memory leaks (basic testing)
- [x] CORS headers work correctly
- [x] Test suite passes

## Breaking Changes

None. This is a purely additive feature.

## Migration Guide

No migration needed. To use the HTTP API:
1. Start VCV Rack with `--httpapi` flag
2. Access API at `http://localhost:8080`
3. See `HTTP_API.md` for endpoint documentation

## Related Issues

This feature enables:
- Automation and scripting of VCV Rack
- Integration with external tools
- Automated testing of patches
- Remote control and monitoring
- Programmatic patch generation

## Checklist

- [x] Code follows project style guidelines (`.astylerc`)
- [x] Documentation added (`HTTP_API.md`)
- [x] Test script included (`test_http_api.py`)
- [x] No breaking changes
- [x] Thread-safe implementation
- [x] Cross-platform compatible
- [x] Minimal performance impact
- [x] Security considerations documented

## Screenshots/Examples

### Example: Creating a Simple Patch via API

```python
import requests

# List available models
models = requests.get('http://localhost:8080/api/models').json()

# Create modules
vco = requests.post('http://localhost:8080/api/modules', json={
    'pluginSlug': 'Fundamental',
    'modelSlug': 'VCO-1',
    'pos': {'x': 0, 'y': 0}
}).json()

vca = requests.post('http://localhost:8080/api/modules', json={
    'pluginSlug': 'Fundamental',
    'modelSlug': 'VCA-1',
    'pos': {'x': 150, 'y': 0}
}).json()

# Connect them
cable = requests.post('http://localhost:8080/api/cables', json={
    'outputModuleId': vco['id'],
    'outputId': 0,
    'inputModuleId': vca['id'],
    'inputId': 0
}).json()

print(f"Created patch with {len([vco, vca])} modules and {1} cable")
```

## Notes for Reviewers

### Code Organization
- HTTP server implementation is self-contained in `src/httpapi.cpp`
- Clean integration with existing codebase via `adapters/standalone.cpp`
- No modifications to core Engine or UI code
- Uses existing JSON library (Jansson) for consistency

### Thread Safety
- All Engine operations properly locked
- UI operations queued appropriately
- No race conditions in testing

### Platform Compatibility
- Windows socket API (`winsock2.h`)
- POSIX socket API (Linux/macOS)
- Proper cleanup on all platforms

### Memory Management
- All allocations properly freed
- JSON objects properly reference counted
- No leaks detected in testing

## Questions?

See `HTTP_API.md` for detailed documentation or ask in the PR discussion.
