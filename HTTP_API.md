# VCV Rack HTTP API Documentation

## Overview

The VCV Rack HTTP API allows programmatic control of VCV Rack through RESTful HTTP endpoints. This enables remote control, automation, integration with other software, and scripting of patch creation and manipulation.

## Starting the HTTP API Server

The HTTP API server is disabled by default and must be enabled via command-line flag:

```bash
# Start Rack with HTTP API on default port 8080
./Rack --httpapi

# Start with custom port
./Rack --httpapi=9000

# Short form with default port
./Rack -H

# Short form with custom port
./Rack -H9000
```

The server runs on `http://localhost:8080` (or your specified port) and accepts requests from any origin (CORS enabled).

## API Endpoints

All endpoints return JSON responses with appropriate HTTP status codes.

### Plugin Information

#### `GET /api/plugins`

List all available plugins and their modules.

**Response:**
```json
{
  "plugins": [
    {
      "slug": "Fundamental",
      "name": "VCV Fundamental",
      "brand": "VCV",
      "version": "2.6.4",
      "author": "VCV",
      "models": [
        {
          "slug": "VCO-1",
          "name": "VCO-1",
          "description": "Voltage-controlled oscillator"
        },
        ...
      ]
    },
    ...
  ]
}
```

#### `GET /api/models`

List all available module models from all plugins.

**Response:**
```json
{
  "models": [
    {
      "pluginSlug": "Fundamental",
      "pluginName": "VCV Fundamental",
      "slug": "VCO-1",
      "name": "VCO-1",
      "fullName": "VCV VCO-1",
      "description": "Voltage-controlled oscillator"
    },
    ...
  ]
}
```

### Module Management

#### `GET /api/modules`

List all modules currently in the patch.

**Response:**
```json
{
  "modules": [
    {
      "id": 1,
      "pluginSlug": "Fundamental",
      "modelSlug": "VCO-1",
      "modelName": "VCO-1",
      "pos": { "x": 0, "y": 0 },
      "size": { "x": 90, "y": 380 }
    },
    ...
  ]
}
```

#### `GET /api/modules/:id`

Get detailed information about a specific module including all parameters, inputs, outputs, and lights.

**Response:**
```json
{
  "id": 1,
  "pluginSlug": "Fundamental",
  "modelSlug": "VCO-1",
  "modelName": "VCO-1",
  "params": [
    {
      "id": 0,
      "value": 0.5,
      "name": "Frequency",
      "label": "FREQ",
      "unit": "Hz",
      "minValue": -4.0,
      "maxValue": 4.0,
      "defaultValue": 0.0,
      "displayValue": "261.626 Hz"
    },
    ...
  ],
  "inputs": [
    {
      "id": 0,
      "channels": 1,
      "connected": true,
      "name": "1V/octave pitch",
      "description": ""
    },
    ...
  ],
  "outputs": [
    {
      "id": 0,
      "channels": 1,
      "connected": true,
      "name": "Sine output",
      "description": ""
    },
    ...
  ],
  "lights": [
    {
      "id": 0,
      "value": 0.75,
      "name": "Frequency light"
    },
    ...
  ]
}
```

#### `POST /api/modules`

Add a new module to the patch.

**Request Body:**
```json
{
  "pluginSlug": "Fundamental",
  "modelSlug": "VCO-1",
  "pos": {
    "x": 100,
    "y": 0
  }
}
```

**Response (201 Created):**
```json
{
  "id": 5,
  "pluginSlug": "Fundamental",
  "modelSlug": "VCO-1"
}
```

#### `DELETE /api/modules/:id`

Remove a module from the patch.

**Response:**
```json
{
  "success": true
}
```

### Cable Management

#### `GET /api/cables`

List all cables (connections) in the patch.

**Response:**
```json
{
  "cables": [
    {
      "id": 100,
      "outputModuleId": 1,
      "outputId": 0,
      "inputModuleId": 2,
      "inputId": 0
    },
    ...
  ]
}
```

#### `POST /api/cables`

Create a new cable connection between two modules.

**Request Body:**
```json
{
  "outputModuleId": 1,
  "outputId": 0,
  "inputModuleId": 2,
  "inputId": 0
}
```

**Response (201 Created):**
```json
{
  "id": 101,
  "outputModuleId": 1,
  "outputId": 0,
  "inputModuleId": 2,
  "inputId": 0
}
```

#### `DELETE /api/cables/:id`

Remove a cable from the patch.

**Response:**
```json
{
  "success": true
}
```

## Error Responses

All errors return appropriate HTTP status codes with JSON error messages:

```json
{
  "error": "Module not found"
}
```

Common status codes:
- `200 OK` - Request successful
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request format or parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

## Usage Examples

### Python Example

```python
import requests
import json

# Base URL
base_url = "http://localhost:8080"

# List all available models
response = requests.get(f"{base_url}/api/models")
models = response.json()["models"]
print(f"Found {len(models)} module models")

# Add a VCO module
vco = requests.post(f"{base_url}/api/modules", json={
    "pluginSlug": "Fundamental",
    "modelSlug": "VCO-1",
    "pos": {"x": 0, "y": 0}
})
vco_id = vco.json()["id"]
print(f"Created VCO with ID {vco_id}")

# Add a VCA module
vca = requests.post(f"{base_url}/api/modules", json={
    "pluginSlug": "Fundamental",
    "modelSlug": "VCA-1",
    "pos": {"x": 100, "y": 0}
})
vca_id = vca.json()["id"]
print(f"Created VCA with ID {vca_id}")

# Get VCO details to find output ports
vco_details = requests.get(f"{base_url}/api/modules/{vco_id}").json()
print(f"VCO has {len(vco_details['outputs'])} outputs")

# Connect VCO output to VCA input
cable = requests.post(f"{base_url}/api/cables", json={
    "outputModuleId": vco_id,
    "outputId": 0,  # First output
    "inputModuleId": vca_id,
    "inputId": 0    # First input
})
cable_id = cable.json()["id"]
print(f"Created cable with ID {cable_id}")

# List all modules in the patch
modules = requests.get(f"{base_url}/api/modules").json()["modules"]
print(f"\nCurrent patch has {len(modules)} modules:")
for mod in modules:
    print(f"  - {mod['modelName']} (ID: {mod['id']})")
```

### cURL Examples

```bash
# List all plugins
curl http://localhost:8080/api/plugins

# List all models
curl http://localhost:8080/api/models

# Add a module
curl -X POST http://localhost:8080/api/modules \
  -H "Content-Type: application/json" \
  -d '{"pluginSlug":"Fundamental","modelSlug":"VCO-1","pos":{"x":0,"y":0}}'

# Get module details
curl http://localhost:8080/api/modules/1

# Create a cable
curl -X POST http://localhost:8080/api/cables \
  -H "Content-Type: application/json" \
  -d '{"outputModuleId":1,"outputId":0,"inputModuleId":2,"inputId":0}'

# List all cables
curl http://localhost:8080/api/cables

# Delete a module
curl -X DELETE http://localhost:8080/api/modules/1
```

### JavaScript/Node.js Example

```javascript
const axios = require('axios');

const BASE_URL = 'http://localhost:8080';

async function buildPatch() {
  try {
    // Get available models
    const models = await axios.get(`${BASE_URL}/api/models`);
    console.log(`Found ${models.data.models.length} models`);

    // Create VCO
    const vco = await axios.post(`${BASE_URL}/api/modules`, {
      pluginSlug: 'Fundamental',
      modelSlug: 'VCO-1',
      pos: { x: 0, y: 0 }
    });
    console.log(`Created VCO: ${vco.data.id}`);

    // Create VCF (filter)
    const vcf = await axios.post(`${BASE_URL}/api/modules`, {
      pluginSlug: 'Fundamental',
      modelSlug: 'VCF',
      pos: { x: 100, y: 0 }
    });
    console.log(`Created VCF: ${vcf.data.id}`);

    // Create VCA (amplifier)
    const vca = await axios.post(`${BASE_URL}/api/modules`, {
      pluginSlug: 'Fundamental',
      modelSlug: 'VCA-1',
      pos: { x: 200, y: 0 }
    });
    console.log(`Created VCA: ${vca.data.id}`);

    // Connect VCO -> VCF
    await axios.post(`${BASE_URL}/api/cables`, {
      outputModuleId: vco.data.id,
      outputId: 0,
      inputModuleId: vcf.data.id,
      inputId: 0
    });

    // Connect VCF -> VCA
    await axios.post(`${BASE_URL}/api/cables`, {
      outputModuleId: vcf.data.id,
      outputId: 0,
      inputModuleId: vca.data.id,
      inputId: 0
    });

    console.log('Patch created successfully!');
  } catch (error) {
    console.error('Error:', error.response?.data || error.message);
  }
}

buildPatch();
```

## Use Cases

### 1. Automated Patch Generation

Create patches programmatically based on algorithms or musical rules:

```python
def create_poly_synth(num_voices=4):
    """Create a polyphonic synthesizer with multiple voices"""
    voices = []

    for i in range(num_voices):
        # Create VCO, VCF, VCA for each voice
        vco = add_module("Fundamental", "VCO-1", x=i*150, y=0)
        vcf = add_module("Fundamental", "VCF", x=i*150, y=100)
        vca = add_module("Fundamental", "VCA-1", x=i*150, y=200)

        # Connect signal chain
        connect(vco, 0, vcf, 0)
        connect(vcf, 0, vca, 0)

        voices.append({"vco": vco, "vcf": vcf, "vca": vca})

    return voices
```

### 2. Patch Testing and Validation

Automatically test patches to ensure they're wired correctly:

```python
def validate_patch():
    """Check that all inputs are connected"""
    modules = get_modules()

    for module in modules:
        details = get_module_details(module['id'])
        for input_port in details['inputs']:
            if not input_port['connected']:
                print(f"Warning: {module['modelName']} input {input_port['name']} is unconnected")
```

### 3. Live Coding / Algorithmic Composition

Modify patches in real-time from code:

```python
import time
import math

def animate_patch():
    """Animate module positions"""
    modules = get_modules()

    for t in range(100):
        for i, module in enumerate(modules):
            x = 100 + 50 * math.sin(t * 0.1 + i)
            y = 200 + 50 * math.cos(t * 0.1 + i)
            update_module_position(module['id'], x, y)

        time.sleep(0.1)
```

### 4. Integration with DAWs/Sequencers

Control VCV Rack from external software:

```python
def sync_with_daw(bpm, bar):
    """Synchronize with external DAW"""
    # Add/remove modules based on DAW arrangement
    if bar == 1:
        add_module("Fundamental", "VCO-1", x=0, y=0)
    elif bar == 8:
        add_module("Fundamental", "VCF", x=150, y=0)
```

## CORS Support

The HTTP API includes CORS headers to allow requests from web browsers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

This enables web-based control panels and browser-based patch editors.

## Thread Safety

The HTTP API server runs in a separate thread from the main VCV Rack application. All operations that modify the Engine or RackWidget are thread-safe and use appropriate locking mechanisms to prevent race conditions.

## Performance Considerations

- The HTTP API server uses a thread pool to handle multiple concurrent requests
- Each request creates a new thread for handling to avoid blocking
- Module and cable operations are performed on the Engine thread with proper synchronization
- For bulk operations, consider batching requests to minimize overhead

## Security Considerations

**Warning:** The HTTP API server is intended for local use and automation. It:

- Binds to all interfaces (0.0.0.0) by default
- Has no authentication mechanism
- Allows full control over the Rack patch

**Recommendations:**
- Only enable the HTTP API when needed
- Use a firewall to restrict access to localhost only in untrusted networks
- Consider using an SSH tunnel for remote access instead of exposing the port directly

## Limitations

- Cannot set parameter values (read-only parameter access)
- Cannot control audio/MIDI devices
- Cannot save/load patches (use VCV Rack's native patch management)
- Module widgets are positioned but may be adjusted by Rack's layout algorithm
- No parameter automation or smooth parameter changes via API

## Future Enhancements

Potential future additions to the HTTP API:

- Parameter value setting and automation
- Patch save/load endpoints
- Sample rate and buffer size control
- Real-time parameter value streaming via WebSocket
- Module preset management
- Undo/redo operations
- Clipboard operations (copy/paste modules)
- Module browser and search
- Performance metrics and monitoring

## Troubleshooting

### Server won't start
- Check that the port is not already in use
- Try a different port with `--httpapi=PORT`
- Check firewall settings

### Module creation fails
- Verify the plugin slug and model slug are correct
- Check that the plugin is installed and loaded
- Use `/api/models` to see available models

### Cable creation fails
- Verify both modules exist with `/api/modules`
- Check that port IDs are valid with `/api/modules/:id`
- Ensure you're connecting output to input (not output to output)

### JSON parsing errors
- Ensure request Content-Type is `application/json`
- Validate JSON syntax
- Check that all required fields are present

## Contact and Support

For issues, feature requests, or questions about the HTTP API, please refer to the VCV Rack manual and community forums.
