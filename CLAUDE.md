# CLAUDE.md - VCV Rack Codebase Guide for AI Assistants

This document provides comprehensive guidance for AI assistants working with the VCV Rack codebase.

## Table of Contents

1. [Overview](#overview)
2. [Repository Structure](#repository-structure)
3. [Build System](#build-system)
4. [Architecture](#architecture)
5. [Coding Conventions](#coding-conventions)
6. [Development Workflows](#development-workflows)
7. [Key Concepts](#key-concepts)
8. [Important Files](#important-files)
9. [Common Patterns](#common-patterns)
10. [Testing and Debugging](#testing-and-debugging)
11. [Resources](#resources)

---

## Overview

**VCV Rack** is a professional virtual Eurorack modular synthesizer platform written in C++. It consists of:

- **Rack**: The host application (this repository)
- **Plugins**: Loadable modules that extend functionality
- **Version**: 2.x (major version)
- **License**: Dual-licensed (GPLv3 for Free edition, commercial for Pro)
- **Platforms**: Windows, macOS (x64/ARM64), Linux

### Project Goals

- High-performance real-time audio processing
- Extensible plugin architecture
- Cross-platform compatibility
- Professional-grade UI/UX

### Important Contribution Policy

**VCV is unable to accept outside code contributions.** This is a read-only codebase for analysis and plugin development. See `.github/CONTRIBUTING.md` for alternative ways to contribute.

---

## Repository Structure

### Top-Level Directory Layout

```
/home/user/Rack/
├── src/                    # Implementation files (~100 .cpp files)
│   ├── app/               # UI application layer (ModuleWidget, PortWidget, etc.)
│   ├── widget/            # Widget base classes
│   ├── ui/                # UI controls (Button, Menu, TextField, etc.)
│   ├── window/            # Window system (OpenGL, event handling)
│   ├── engine/            # DSP engine & module infrastructure
│   ├── plugin/            # Plugin loading and management
│   ├── dsp/               # Digital signal processing utilities
│   ├── core/              # Built-in modules (Audio I/O, MIDI, Blank, Notes)
│   └── *.cpp              # Core utilities (asset, audio, color, common, etc.)
│
├── include/               # Public/private headers (~122 .hpp files)
│   ├── rack.hpp          # **MAIN PUBLIC API** - includes all public headers
│   ├── rack0.hpp         # Legacy API compatibility
│   ├── app/              # UI widget headers
│   ├── widget/           # Widget base classes & event system
│   ├── ui/               # UI control headers
│   ├── window/           # Window.hpp, Svg.hpp
│   ├── engine/           # Module, Cable, Port, Engine, Light, Param
│   ├── plugin/           # Plugin.hpp, Model.hpp, callbacks.hpp
│   ├── dsp/              # DSP algorithms (filters, FFT, resampler, etc.)
│   ├── simd/             # SIMD vector operations
│   └── *.hpp             # Utility headers (common, math, string, system, etc.)
│
├── dep/                   # External dependencies (bundled)
│   ├── include/          # Third-party headers
│   └── lib/              # Compiled static libraries
│
├── adapters/              # Platform adapters
│   └── standalone.cpp    # Application entry point (main())
│
├── res/                   # Resources
│   ├── fonts/            # DejaVu, Nunito, DSEG7, NotoEmoji, NotoSans
│   ├── ComponentLibrary/ # SVG UI components (knobs, ports, etc.)
│   └── Core/             # Built-in module panels
│
├── plugins/               # Third-party plugin storage directory
├── presets/               # User presets for core modules
├── translations/          # i18n files (German, Spanish, French, etc.)
├── docs/                  # Doxygen configuration
│
├── Makefile              # Main build system
├── plugin.mk             # Plugin SDK build template
├── compile.mk            # Compilation rules
├── arch.mk               # Architecture detection
├── dep.mk                # Dependency management
├── helper.py             # Python script for creating plugins
├── .astylerc             # Code formatting config (Artistic Style 3.1)
│
├── CHANGELOG.md          # Detailed version history
├── README.md             # Project overview
├── LICENSE.md            # Multi-license information
└── Core.json             # Metadata for built-in modules
```

### Source Directory Details

| Directory | Purpose | Key Files |
|-----------|---------|-----------|
| `src/app/` | UI application layer | ModuleWidget.cpp, RackWidget.cpp, Scene.cpp, PortWidget.cpp, CableWidget.cpp |
| `src/widget/` | Widget implementations | Widget.cpp, FramebufferWidget.cpp, TransformWidget.cpp |
| `src/ui/` | UI controls | Button.cpp, Menu.cpp, TextField.cpp, ScrollWidget.cpp |
| `src/window/` | Window system | Window.cpp |
| `src/engine/` | DSP engine | Engine.cpp, Module.cpp, Cable.cpp, Port.cpp |
| `src/plugin/` | Plugin management | plugin.cpp |
| `src/dsp/` | DSP utilities | filter.cpp, fft.cpp, resampler.cpp |
| `src/core/` | Built-in modules | Audio.cpp, MIDI_CV.cpp, Blank.cpp, Notes.cpp |
| `src/` (root) | Core utilities | asset.cpp, patch.cpp, settings.cpp, logger.cpp |

### Include Directory Details

| Directory | Purpose | Key Headers |
|-----------|---------|-------------|
| `include/engine/` | DSP engine API | Engine.hpp, Module.hpp, Port.hpp, Cable.hpp, Param.hpp |
| `include/plugin/` | Plugin API | Plugin.hpp, Model.hpp, callbacks.hpp |
| `include/dsp/` | DSP algorithms | filter.hpp, fft.hpp, resampler.hpp, midi.hpp, digital.hpp |
| `include/widget/` | Widget system | Widget.hpp, event.hpp, FramebufferWidget.hpp |
| `include/app/` | Application UI | ModuleWidget.hpp, PortWidget.hpp, RackWidget.hpp |
| `include/simd/` | SIMD operations | Vector.hpp, functions.hpp |
| `include/` (root) | Utilities | helpers.hpp, math.hpp, string.hpp, system.hpp, random.hpp |

---

## Build System

### Makefile Architecture

The build system uses **GNU Make** with a modular design:

- **Makefile** - Main build file, links library and standalone app
- **arch.mk** - CPU & OS detection (x86_64/ARM64 + Windows/Mac/Linux)
- **compile.mk** - Compilation rules (C++11, optimization flags)
- **dep.mk** - Dependency building (WGET, CMAKE, CONFIGURE)
- **plugin.mk** - Plugin SDK template for third-party developers

### Build Targets

```bash
# Core library (shared object)
make libRack.so        # Linux
make libRack.dylib     # macOS
make libRack.dll       # Windows

# Standalone application
make Rack              # Linux/macOS
make Rack.exe          # Windows

# Build both library and standalone
make all

# Dependencies
make dep               # Build all dependencies
make cleandep          # Clean dependencies

# Run targets
make run               # Run with development flag (-d)
make runr              # Run release mode
make debug             # Run in debugger (gdb/lldb)

# Performance & debugging
make perf              # Profile with perf and hotspot
make valgrind          # Memory check with valgrind

# Distribution (internal use)
make dist              # Create distribution package
make sdk               # Create SDK package
make package           # Create installer

# Cleanup
make clean             # Remove build artifacts
make cleandist         # Remove dist directory
```

### Dependencies

All dependencies are bundled in `dep/` directory:

**Audio:**
- RtAudio - Cross-platform audio I/O
- RtMidi - Cross-platform MIDI I/O
- libspeexdsp - Fixed-ratio resampler
- libsamplerate - Variable-ratio resampler

**Graphics:**
- GLFW - Window & input handling
- GLEW - OpenGL extensions
- NanoVG - 2D vector graphics (modified fork)
- NanoSVG - SVG parsing

**Data/Networking:**
- Jansson - JSON parsing
- libcurl - HTTP/HTTPS
- OpenSSL - Cryptography
- libarchive - Archive handling
- Zstandard - Compression for patches

**DSP:**
- PFFFT - Fast Fourier Transform
- TinyExpr - Math expression evaluation

**UI:**
- osdialog - Native file dialogs
- oui-blendish - Blendish UI theme
- Fuzzy Search Database - Module browser search

### Compilation Flags

**C++ Standard:** C++11 (`-std=c++11`)

**Optimization:**
```
-O3
-funsafe-math-optimizations
-fno-omit-frame-pointer
```

**Architecture:**
- x64: `-march=nehalem`
- ARM64: `-march=armv8-a+fp+simd`

**Platform-specific:**
- Linux: `-static-libstdc++ -static-libgcc -fno-gnu-unique`
- macOS: `-stdlib=libc++ -mmacosx-version-min=10.9`
- Windows: `-D_USE_MATH_DEFINES -municode`

---

## Architecture

### Core Architecture Patterns

#### 1. Module-Plugin System

**Two-layer plugin architecture:**

```
Plugin (container)
  ├── Model 1 (factory)
  │   ├── Module (DSP)
  │   └── ModuleWidget (UI)
  ├── Model 2
  └── Model 3
```

**Plugin struct** (`plugin/Plugin.hpp`):
- Manages collection of module models
- Metadata: slug, version, license, author, URLs
- Dynamically loaded via OS-specific handle (dlopen on Linux)

**Model struct** (`plugin/Model.hpp`):
- Factory pattern for creating Module/ModuleWidget instances
- Created via template helper: `createModel<TModule, TModuleWidget>(slug)`

**Module struct** (`engine/Module.hpp`):
- Base class for DSP processing
- Contains params, inputs, outputs, lights
- `process(const ProcessArgs& args)` - main DSP callback

#### 2. Scene Graph Architecture

Hierarchical 2D widget tree:

```
Scene (root)
  └── RackScrollWidget
      └── ZoomWidget
          └── RackWidget (main canvas)
              ├── ModuleWidget 1
              │   ├── SvgPanel
              │   ├── ParamWidget (Knob, Slider, Switch)
              │   ├── PortWidget (Input, Output)
              │   └── LightWidget (LED)
              ├── ModuleWidget 2
              └── CableWidget (connects ports)
```

#### 3. DSP Engine Threading

**Engine** (`engine/Engine.hpp`) uses reader-writer mutex pattern:

- `stepBlock()` - Share-locks for parallel DSP processing
- Module add/remove - Exclusive locks
- Worker thread pool for parallel module stepping
- `yieldWorkers()` - Hint for long-running operations

**Thread safety rules:**
- Engine operations are thread-safe
- Module `process()` runs on engine thread
- UI operations run on main thread
- Use `Module::fromJson()` for parameter restoration

#### 4. Event System

Custom event propagation (`widget/event.hpp`):

- Event types: MouseEvent, KeyEvent, DragEvent, SelectEvent, etc.
- Bubbling/capturing phases in widget hierarchy
- Widgets handle events by implementing event handlers
- Widgets request deletion via `requestDelete()` flag

#### 5. Rendering Pipeline

**Graphics stack:**
- NanoVG for 2D vector graphics
- OpenGL for hardware acceleration
- SVG widgets for panel graphics
- FramebufferWidget for off-screen rendering (performance optimization)
- OpenGlWidget for direct GPU access

### Namespace Organization

```cpp
namespace rack {
  namespace engine { Module, Cable, Port, Engine, ... }
  namespace plugin { Plugin, Model, ... }
  namespace widget { Widget, FramebufferWidget, ... }
  namespace ui { Menu, Button, TextField, ... }
  namespace app { ModuleWidget, PortWidget, Scene, ... }
  namespace dsp { filter, fft, resampler, ... }
  namespace math { clamp, eucMod, ... }
  namespace string { f, lowercase, trim, ... }
  namespace system { getTime, getThreadId, ... }
  namespace settings { Global settings variables }
  namespace random { Random number generators }
  // etc.
}
```

### Separation of Concerns

1. **DSP Layer** (`engine/`, `dsp/`) - Pure signal processing, no UI dependencies
2. **UI Layer** (`widget/`, `ui/`, `app/`) - Event handling, rendering, scene graph
3. **Plugin System** (`plugin/`) - Dynamic loading, model factories
4. **Core Utilities** - Math, string, system, asset management

---

## Coding Conventions

### Code Style

VCV Rack uses **Artistic Style 3.1** (`.astylerc`):

```
Style: Java (braces on same line)
Indent: Tabs (width 2)
Pointer/reference alignment: type* ptr, type& ref
Pad operators, commas, headers
```

**Auto-format command:**
```bash
astyle --suffix=none --options=.astylerc -r 'include/*' 'src/*'
```

### Naming Conventions

| Type | Convention | Examples |
|------|------------|----------|
| Classes/Structs | PascalCase | `Widget`, `Module`, `Engine`, `PortWidget` |
| Functions | camelCase | `stepBlock()`, `getSampleRate()`, `configParam()` |
| Member variables | camelCase | `paramQuantities`, `box`, `module` |
| Constants | UPPER_CASE | `RACK_GRID_WIDTH`, `PORT_MAX_CHANNELS` |
| Macros | UPPER_CASE | `PRIVATE`, `DEPRECATED`, `LENGTHOF()` |
| Namespaces | lowercase | `rack::engine`, `rack::dsp` |
| Files | PascalCase | `ModuleWidget.cpp`, `Engine.hpp` |

### Header Organization

**Public API rule:**
- **Plugins must only include `rack.hpp`**
- Direct inclusion of other headers is unsupported and may break

**rack.hpp structure:**
```cpp
#pragma once
#include <common.hpp>
#include <math.hpp>
#include <string.hpp>
// ... all public headers
#include <engine/Engine.hpp>
#include <engine/Module.hpp>
// ... etc.
```

**Private symbols:**
```cpp
#define PRIVATE __attribute__((error("Using internal Rack function")))

PRIVATE void internalFunction(); // Plugins get compile error
```

### File Structure

**Header files (.hpp):**
```cpp
#pragma once

#include <rack.hpp> // If needed

namespace rack {
namespace subsystem {

struct MyClass {
	// Members
	int value;

	// Methods
	void doSomething();
};

} // namespace subsystem
} // namespace rack
```

**Source files (.cpp):**
```cpp
#include <app/ModuleWidget.hpp> // Own header first
#include <app/Scene.hpp>        // Related headers
#include <context.hpp>          // System headers

namespace rack {
namespace app {

void ModuleWidget::doSomething() {
	// Implementation
}

} // namespace app
} // namespace rack
```

### Common Patterns

**Guard macros for platform-specific code:**
```cpp
#ifdef ARCH_LIN
	// Linux-specific code
#endif
#ifdef ARCH_MAC
	// macOS-specific code
#endif
#ifdef ARCH_WIN
	// Windows-specific code
#endif
```

**Deprecation:**
```cpp
DEPRECATED void oldFunction();
```

**Enum ranges:**
```cpp
enum ParamIds {
	FREQ_PARAM,
	FINE_PARAM,
	ENUMS(OCTAVE_PARAM, 3), // OCTAVE_PARAM+0, +1, +2
	NUM_PARAMS
};
```

---

## Development Workflows

### Git Workflow

**Commit message style:**
- Imperative mood, short descriptions
- Examples from recent commits:
  - "Update changelog."
  - "Make left/right keys go to start/end of selection if text is selected in TextField."
  - "Add spotlightBrightness and spotlightRadius to settings."

**Branch structure:**
- Development happens on feature branches
- Clean, linear history preferred

### Changelog Format

Version entries in `CHANGELOG.md` follow this structure:

```markdown
### 2.6.6 (2025-11-04)
- High-level changes
- Core
  - *Module Name*
    - Specific changes to module
- SDK
  - SDK-related changes
```

### Version Numbering

- Format: `MAJOR.MINOR.PATCH`
- Major version: 2
- Version extracted from git tags: `v2.6.6`

---

## Key Concepts

### 1. Modules

**Module** is the base class for all DSP processing units.

**Lifecycle:**
```cpp
struct MyModule : Module {
	// 1. Configuration (constructor)
	MyModule() {
		config(NUM_PARAMS, NUM_INPUTS, NUM_OUTPUTS, NUM_LIGHTS);
		configParam(FREQ_PARAM, 0.f, 1.f, 0.5f, "Frequency");
		configInput(AUDIO_INPUT, "Audio");
		configOutput(AUDIO_OUTPUT, "Audio");
		configLight(LED_LIGHT, "Status");
	}

	// 2. Processing (called every sample or block)
	void process(const ProcessArgs& args) override {
		float in = inputs[AUDIO_INPUT].getVoltage();
		float out = in * params[FREQ_PARAM].getValue();
		outputs[AUDIO_OUTPUT].setVoltage(out);
	}

	// 3. Serialization (optional)
	json_t* dataToJson() override {
		json_t* rootJ = json_object();
		json_object_set_new(rootJ, "customData", json_integer(myData));
		return rootJ;
	}

	void dataFromJson(json_t* rootJ) override {
		json_t* dataJ = json_object_get(rootJ, "customData");
		if (dataJ)
			myData = json_integer_value(dataJ);
	}
};
```

**Key members:**
- `params[]` - User-controllable parameters (knobs, sliders, switches)
- `inputs[]` - Input ports (audio/CV signals)
- `outputs[]` - Output ports
- `lights[]` - LEDs and light displays
- `leftExpander`, `rightExpander` - Adjacent module communication

### 2. ModuleWidget

**ModuleWidget** is the visual representation of a Module.

**Typical structure:**
```cpp
struct MyModuleWidget : ModuleWidget {
	MyModuleWidget(MyModule* module) {
		setModule(module);

		// Set panel
		setPanel(createPanel(asset::plugin(pluginInstance, "res/MyPanel.svg")));

		// Add screws
		addChild(createWidget<ScrewSilver>(Vec(0, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 15, 0)));

		// Add params (knobs, switches)
		addParam(createParamCentered<RoundBlackKnob>(
			mm2px(Vec(10, 20)), module, MyModule::FREQ_PARAM));

		// Add inputs
		addInput(createInputCentered<PJ301MPort>(
			mm2px(Vec(10, 40)), module, MyModule::AUDIO_INPUT));

		// Add outputs
		addOutput(createOutputCentered<PJ301MPort>(
			mm2px(Vec(10, 60)), module, MyModule::AUDIO_OUTPUT));

		// Add lights
		addChild(createLightCentered<MediumLight<RedLight>>(
			mm2px(Vec(10, 80)), module, MyModule::LED_LIGHT));
	}
};
```

### 3. Plugins

**Plugin initialization:**
```cpp
// Define plugin instance
Plugin* pluginInstance;

// Define models
Model* modelMyModule;
Model* modelAnotherModule;

// Initialize plugin
void init(Plugin* p) {
	pluginInstance = p;
	p->slug = "MyCompany-MyPlugin";
	p->version = "1.0.0";
	p->name = "My Plugin";
	p->brand = "MyCompany";

	// Add models
	p->addModel(modelMyModule = createModel<MyModule, MyModuleWidget>("MyModule"));
	p->addModel(modelAnotherModule = createModel<AnotherModule, AnotherModuleWidget>("AnotherModule"));
}
```

### 4. Polyphony

VCV Rack supports polyphonic cables (up to 16 channels):

```cpp
// Get number of channels
int channels = inputs[AUDIO_INPUT].getChannels();

// Get voltage from specific channel
float voltage = inputs[AUDIO_INPUT].getVoltage(c);

// Set output channels
outputs[AUDIO_OUTPUT].setChannels(channels);

// Set voltage for specific channel
outputs[AUDIO_OUTPUT].setVoltage(voltage, c);

// Process all channels
for (int c = 0; c < channels; c++) {
	float in = inputs[AUDIO_INPUT].getVoltage(c);
	float out = processChannel(in);
	outputs[AUDIO_OUTPUT].setVoltage(out, c);
}
```

### 5. Expanders

Modules can communicate with adjacent modules via expanders:

```cpp
// Define message struct
struct MyMessage {
	float data[16];
};

// In Module constructor
leftExpander.producerMessage = new MyMessage();
leftExpander.consumerMessage = new MyMessage();

// In process()
MyMessage* msgToLeft = (MyMessage*) leftExpander.module->rightExpander.producerMessage;
msgToLeft->data[0] = 1.23f;
leftExpander.module->rightExpander.messageFlipRequested = true;
```

### 6. DSP Utilities

**Filters:**
```cpp
#include <dsp/filter.hpp>

dsp::TRCFilter<float> filter;
filter.setCutoff(1000.f / args.sampleRate);
float out = filter.process(in);
```

**FFT:**
```cpp
#include <dsp/fft.hpp>

dsp::RealFFT fft(1024);
fft.rfft(input, output);
fft.irfft(output, input);
```

**Resampling:**
```cpp
#include <dsp/resampler.hpp>

dsp::SampleRateConverter<1> src;
src.setRates(44100.f, 48000.f);
int outputLen = src.process(input, &output);
```

### 7. SIMD Operations

Use SIMD for performance-critical code:

```cpp
#include <simd/functions.hpp>

using float_4 = simd::float_4;

float_4 x = {1.f, 2.f, 3.f, 4.f};
float_4 y = simd::sin(x);
float_4 z = simd::clamp(y, -1.f, 1.f);
```

---

## Important Files

### Entry Points

| File | Purpose |
|------|---------|
| `adapters/standalone.cpp` | Application main() entry point |
| `src/plugin.cpp` | Plugin loading and management |
| `src/engine/Engine.cpp` | DSP engine implementation |

### Core APIs

| File | Purpose |
|------|---------|
| `include/rack.hpp` | **Main public API** - include this in plugins |
| `include/engine/Engine.hpp` | DSP engine, threading, sample rate |
| `include/engine/Module.hpp` | Module base class |
| `include/plugin/Plugin.hpp` | Plugin container |
| `include/plugin/Model.hpp` | Module factory pattern |
| `include/plugin/callbacks.hpp` | Plugin entry points |
| `include/helpers.hpp` | Template factories for UI |

### DSP

| File | Purpose |
|------|---------|
| `include/dsp/filter.hpp` | IIR/FIR filters |
| `include/dsp/fft.hpp` | FFT via PFFFT |
| `include/dsp/resampler.hpp` | Sample rate conversion |
| `include/dsp/midi.hpp` | MIDI utilities |
| `include/dsp/digital.hpp` | Digital signal processing |

### UI Framework

| File | Purpose |
|------|---------|
| `include/widget/Widget.hpp` | Scene graph node base class |
| `include/widget/event.hpp` | Event system |
| `include/app/ModuleWidget.hpp` | Module visual representation |
| `include/app/RackWidget.hpp` | Main rack canvas |
| `include/app/PortWidget.hpp` | Port (jack) widget |
| `include/app/CableWidget.hpp` | Cable visualization |

### Utilities

| File | Purpose |
|------|---------|
| `include/math.hpp` | Math utilities (clamp, etc.) |
| `include/string.hpp` | String utilities |
| `include/system.hpp` | System utilities (files, threads) |
| `include/random.hpp` | Random number generation |
| `include/asset.hpp` | Resource loading |
| `include/settings.hpp` | User settings |

### Configuration

| File | Purpose |
|------|---------|
| `Core.json` | Built-in module metadata |
| `template.vcv` | Default patch template |
| `.astylerc` | Code formatting config |
| `plugin.json` | Plugin manifest (in plugins) |

---

## Common Patterns

### Creating a Module

Use `helper.py` to scaffold:

```bash
./helper.py createmodule MyModule
```

Or manually follow the pattern in `src/core/` modules.

### Helper Functions

VCV provides template helpers in `include/helpers.hpp`:

```cpp
// Create widgets
createWidget<TWidget>(pos)
createWidgetCentered<TWidget>(pos)

// Create module components
createParam<TParamWidget>(pos, module, paramId)
createInput<TPortWidget>(pos, module, inputId)
createOutput<TPortWidget>(pos, module, outputId)
createLight<TLightWidget>(pos, module, lightId)

// Create panel
createPanel(asset::plugin(pluginInstance, "res/Panel.svg"))

// Create model
createModel<TModule, TModuleWidget>("ModuleSlug")
```

### Coordinate Systems

**Grid units:**
- HP (Horizontal Pitch): 5.08mm Eurorack standard
- Use `RACK_GRID_WIDTH` constant (15 pixels = 1HP at default zoom)

**Conversion helpers:**
```cpp
math::Vec px = mm2px(math::Vec(10.16, 20.32)); // mm to pixels
```

**Widget positioning:**
- Use `box` member (math::Rect with pos and size)
- Origin (0,0) is top-left
- Position widgets relative to panel top-left

### JSON Serialization

**Module data:**
```cpp
json_t* MyModule::dataToJson() override {
	json_t* rootJ = json_object();
	json_object_set_new(rootJ, "myInt", json_integer(myIntValue));
	json_object_set_new(rootJ, "myFloat", json_real(myFloatValue));
	json_object_set_new(rootJ, "myBool", json_boolean(myBoolValue));
	json_object_set_new(rootJ, "myString", json_string(myStringValue.c_str()));
	return rootJ;
}

void MyModule::dataFromJson(json_t* rootJ) override {
	json_t* myIntJ = json_object_get(rootJ, "myInt");
	if (myIntJ)
		myIntValue = json_integer_value(myIntJ);
	// ... etc
}
```

**Plugin settings:**
```cpp
json_t* settingsToJson() {
	json_t* rootJ = json_object();
	// Save global plugin settings
	return rootJ;
}

void settingsFromJson(json_t* rootJ) {
	// Load global plugin settings
}
```

### Context Menu Items

Add right-click menu items to modules:

```cpp
struct MyModuleWidget : ModuleWidget {
	void appendContextMenu(Menu* menu) override {
		MyModule* module = dynamic_cast<MyModule*>(this->module);
		menu->addChild(new MenuSeparator);
		menu->addChild(createMenuLabel("My Custom Options"));

		menu->addChild(createBoolPtrMenuItem("Enable Feature", "", &module->featureEnabled));

		menu->addChild(createIndexPtrSubmenuItem("Mode",
			{"Mode A", "Mode B", "Mode C"},
			&module->mode));
	}
};
```

### Parameter Randomization

Control parameter randomization behavior:

```cpp
configParam(FREQ_PARAM, 0.f, 1.f, 0.5f, "Frequency");
paramQuantities[FREQ_PARAM]->randomizeEnabled = false; // Exclude from randomization
```

### Asset Loading

Load resources from plugin directory:

```cpp
// Load SVG
std::shared_ptr<Svg> svg = APP->window->loadSvg(asset::plugin(pluginInstance, "res/MyPanel.svg"));

// Load font
std::shared_ptr<Font> font = APP->window->loadFont(asset::plugin(pluginInstance, "res/MyFont.ttf"));
```

---

## Testing and Debugging

### Manual Testing

**Run standalone app:**
```bash
make run          # Development mode (-d flag)
make runr         # Release mode
```

**Development flags:**
- `-d` enables development features
- Logs written to `log.txt` in user directory

### Debugging

**With GDB (Linux/Windows):**
```bash
make debug
# or
gdb --args ./Rack -d
```

**With LLDB (macOS):**
```bash
make debug
# or
lldb -- ./Rack -d
```

**Debug output:**
```cpp
#include <common.hpp>

DEBUG("Value: %f", myValue);
INFO("Status: %s", myString.c_str());
WARN("Warning message");
```

### Memory Checking

**Valgrind (Linux):**
```bash
make valgrind
```

Suppression file: `valgrind.supp`

### Performance Profiling

**Linux perf + hotspot:**
```bash
make perf
```

### Code Style Checking

**Format code:**
```bash
astyle --suffix=none --options=.astylerc -r 'include/*' 'src/*'
```

### Module Testing Checklist

When creating/modifying modules:

1. ✓ Test all parameter ranges (min, max, default)
2. ✓ Test polyphonic cables (1-16 channels)
3. ✓ Test save/load (JSON serialization)
4. ✓ Test right-click menu items
5. ✓ Test module browser search tags
6. ✓ Test preset save/load
7. ✓ Test CPU usage (should be minimal when idle)
8. ✓ Test with different sample rates
9. ✓ Check for audio glitches/clicks
10. ✓ Verify expander communication (if applicable)

---

## Resources

### Official Documentation

- **Manual**: https://vcvrack.com/manual/
- **Plugin Development Tutorial**: https://vcvrack.com/manual/PluginDevelopmentTutorial
- **Building Guide**: https://vcvrack.com/manual/Building
- **API Documentation**: Generated from headers via Doxygen

### Community

- **VCV Community**: https://vcvrack.com/manual/Communities
- **Plugin Library**: https://library.vcvrack.com/
- **Support**: https://vcvrack.com/support

### Source Code

- **Rack Repository**: https://github.com/VCVRack/Rack
- **Fundamental Modules**: VCV Fundamental plugin (reference implementation)
- **Plugin Toolchain**: https://github.com/VCVRack/rack-plugin-toolchain

### Key Files to Reference

When working on specific areas, consult these example implementations:

**Built-in Modules (src/core/):**
- `Audio.cpp` - Audio I/O interface
- `MIDI_CV.cpp` - MIDI to CV conversion (complex module example)
- `Blank.cpp` - Simple module example

**UI Components (src/app/):**
- `ModuleWidget.cpp` - Module UI base class
- `RackWidget.cpp` - Main canvas implementation
- `PortWidget.cpp` - Port/jack implementation

**DSP (src/dsp/):**
- `filter.cpp` - Filter implementations
- `resampler.cpp` - Sample rate conversion

### External Libraries Documentation

- **NanoVG**: https://github.com/memononen/nanovg
- **GLFW**: https://www.glfw.org/documentation.html
- **Jansson**: https://jansson.readthedocs.io/
- **RtAudio**: https://www.music.mcgill.ca/~gary/rtaudio/
- **RtMidi**: https://www.music.mcgill.ca/~gary/rtmidi/

---

## AI Assistant Guidelines

When working with this codebase:

1. **Always include `rack.hpp`** - Never include individual headers in plugin code
2. **Respect the PRIVATE macro** - Don't use internal functions marked PRIVATE
3. **Follow the established patterns** - Use helper functions, follow naming conventions
4. **Separate DSP from UI** - Module handles DSP, ModuleWidget handles UI
5. **Thread safety** - Remember Engine runs on separate thread from UI
6. **Don't accept PRs** - This is a read-only codebase for analysis and plugin development
7. **Check CHANGELOG.md** - Understand recent changes and version history
8. **Use template helpers** - Leverage `createModel`, `createParam`, etc.
9. **Test polyphony** - Always consider 1-16 channel scenarios
10. **Profile performance** - Audio modules must be efficient

### Common Tasks

**Reading codebase:**
- Start with `include/rack.hpp` to see public API surface
- Check `src/core/` for reference module implementations
- Review `include/helpers.hpp` for common patterns

**Understanding a module:**
- Find its Model registration in plugin init
- Read Module class (DSP logic)
- Read ModuleWidget class (UI layout)
- Check `dataToJson/dataFromJson` for persistence

**Understanding DSP:**
- Check `include/dsp/` headers for algorithms
- Review `include/simd/` for performance patterns
- Look at `src/engine/Engine.cpp` for threading model

**Understanding UI:**
- Start with `include/widget/Widget.hpp` for base widget
- Check `include/widget/event.hpp` for event system
- Review `src/app/Scene.cpp` for scene graph structure

---

## Version Information

- **Rack Version**: 2.6.6 (as of 2025-11-04)
- **API Stability**: Public API in `rack.hpp` is stable within major version
- **ABI Compatibility**: Plugins must be recompiled for each minor version
- **C++ Standard**: C++11

---

## License

VCV Rack is dual-licensed:
- **Free Edition**: GPLv3 (see LICENSE-GPLv3.txt)
- **Pro Edition**: Commercial license

Plugins can use their own licenses but must be compatible with their target edition.

---

*This document was generated for AI assistant reference. For human developers, please refer to the official documentation at https://vcvrack.com/manual/*
