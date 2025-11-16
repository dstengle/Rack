#!/usr/bin/env python3
"""
Test script for VCV Rack HTTP API

This script demonstrates the usage of the HTTP API by creating a simple patch
with multiple modules and connecting them together.

Usage:
    python test_http_api.py [--host localhost] [--port 8080]
"""

import requests
import json
import argparse
import sys
import time


class RackAPI:
    """Client for VCV Rack HTTP API"""

    def __init__(self, host='localhost', port=8080):
        self.base_url = f"http://{host}:{port}"

    def get_plugins(self):
        """Get all available plugins"""
        response = requests.get(f"{self.base_url}/api/plugins")
        response.raise_for_status()
        return response.json()

    def get_models(self):
        """Get all available module models"""
        response = requests.get(f"{self.base_url}/api/models")
        response.raise_for_status()
        return response.json()

    def get_modules(self):
        """Get all modules in the current patch"""
        response = requests.get(f"{self.base_url}/api/modules")
        response.raise_for_status()
        return response.json()

    def get_module(self, module_id):
        """Get details about a specific module"""
        response = requests.get(f"{self.base_url}/api/modules/{module_id}")
        response.raise_for_status()
        return response.json()

    def add_module(self, plugin_slug, model_slug, x=0, y=0):
        """Add a module to the patch"""
        data = {
            "pluginSlug": plugin_slug,
            "modelSlug": model_slug,
            "pos": {"x": x, "y": y}
        }
        response = requests.post(f"{self.base_url}/api/modules", json=data)
        response.raise_for_status()
        return response.json()

    def delete_module(self, module_id):
        """Delete a module from the patch"""
        response = requests.delete(f"{self.base_url}/api/modules/{module_id}")
        response.raise_for_status()
        return response.json()

    def get_cables(self):
        """Get all cables in the patch"""
        response = requests.get(f"{self.base_url}/api/cables")
        response.raise_for_status()
        return response.json()

    def add_cable(self, output_module_id, output_id, input_module_id, input_id):
        """Create a cable connection"""
        data = {
            "outputModuleId": output_module_id,
            "outputId": output_id,
            "inputModuleId": input_module_id,
            "inputId": input_id
        }
        response = requests.post(f"{self.base_url}/api/cables", json=data)
        response.raise_for_status()
        return response.json()

    def delete_cable(self, cable_id):
        """Delete a cable"""
        response = requests.delete(f"{self.base_url}/api/cables/{cable_id}")
        response.raise_for_status()
        return response.json()

    def find_model(self, plugin_slug, model_slug):
        """Find a specific model"""
        models = self.get_models()
        for model in models['models']:
            if model['pluginSlug'] == plugin_slug and model['slug'] == model_slug:
                return model
        return None


def test_basic_operations(api):
    """Test basic API operations"""
    print("=" * 60)
    print("Testing Basic API Operations")
    print("=" * 60)

    # Test 1: Get plugins
    print("\n1. Getting available plugins...")
    plugins = api.get_plugins()
    print(f"   Found {len(plugins['plugins'])} plugins")
    for plugin in plugins['plugins'][:3]:  # Show first 3
        print(f"   - {plugin['brand']}: {len(plugin['models'])} models")

    # Test 2: Get models
    print("\n2. Getting available models...")
    models = api.get_models()
    print(f"   Found {len(models['models'])} models")
    for model in models['models'][:3]:  # Show first 3
        print(f"   - {model['fullName']}: {model['description']}")

    # Test 3: Get current modules
    print("\n3. Getting current modules in patch...")
    modules = api.get_modules()
    print(f"   Found {len(modules['modules'])} modules")

    print("\n✓ Basic operations test passed!")


def test_module_management(api):
    """Test module creation and deletion"""
    print("\n" + "=" * 60)
    print("Testing Module Management")
    print("=" * 60)

    created_modules = []

    try:
        # Test 1: Create a VCO
        print("\n1. Creating VCO module...")
        vco = api.add_module("Fundamental", "VCO-1", x=0, y=0)
        created_modules.append(vco['id'])
        print(f"   Created VCO with ID {vco['id']}")

        # Test 2: Create a VCF
        print("\n2. Creating VCF module...")
        vcf = api.add_module("Fundamental", "VCF", x=150, y=0)
        created_modules.append(vcf['id'])
        print(f"   Created VCF with ID {vcf['id']}")

        # Test 3: Get module details
        print("\n3. Getting VCO details...")
        vco_details = api.get_module(vco['id'])
        print(f"   VCO has {len(vco_details['params'])} parameters")
        print(f"   VCO has {len(vco_details['inputs'])} inputs")
        print(f"   VCO has {len(vco_details['outputs'])} outputs")

        # Show some parameters
        if vco_details['params']:
            print("\n   Parameters:")
            for param in vco_details['params'][:3]:
                print(f"     - {param['name']}: {param['displayValue']}")

        # Test 4: List all modules
        print("\n4. Listing all modules...")
        modules = api.get_modules()
        print(f"   Total modules: {len(modules['modules'])}")
        for mod in modules['modules']:
            print(f"     - {mod['modelName']} at ({mod['pos']['x']}, {mod['pos']['y']})")

        print("\n✓ Module management test passed!")

    finally:
        # Cleanup
        print("\n5. Cleaning up created modules...")
        for module_id in created_modules:
            try:
                api.delete_module(module_id)
                print(f"   Deleted module {module_id}")
            except:
                pass


def test_cable_management(api):
    """Test cable creation and deletion"""
    print("\n" + "=" * 60)
    print("Testing Cable Management")
    print("=" * 60)

    created_modules = []
    created_cables = []

    try:
        # Create test modules
        print("\n1. Creating test modules...")
        vco = api.add_module("Fundamental", "VCO-1", x=0, y=0)
        created_modules.append(vco['id'])
        print(f"   Created VCO {vco['id']}")

        vca = api.add_module("Fundamental", "VCA-1", x=150, y=0)
        created_modules.append(vca['id'])
        print(f"   Created VCA {vca['id']}")

        # Test 2: Create a cable
        print("\n2. Connecting VCO output to VCA input...")
        cable = api.add_cable(
            output_module_id=vco['id'],
            output_id=0,
            input_module_id=vca['id'],
            input_id=0
        )
        created_cables.append(cable['id'])
        print(f"   Created cable {cable['id']}")

        # Test 3: List all cables
        print("\n3. Listing all cables...")
        cables = api.get_cables()
        print(f"   Total cables: {len(cables['cables'])}")
        for cab in cables['cables']:
            print(f"     - Cable {cab['id']}: Module {cab['outputModuleId']}[{cab['outputId']}] "
                  f"-> Module {cab['inputModuleId']}[{cab['inputId']}]")

        print("\n✓ Cable management test passed!")

    finally:
        # Cleanup
        print("\n4. Cleaning up...")
        for cable_id in created_cables:
            try:
                api.delete_cable(cable_id)
                print(f"   Deleted cable {cable_id}")
            except:
                pass
        for module_id in created_modules:
            try:
                api.delete_module(module_id)
                print(f"   Deleted module {module_id}")
            except:
                pass


def build_example_patch(api):
    """Build a complete example patch"""
    print("\n" + "=" * 60)
    print("Building Example Patch: Simple Synthesizer Voice")
    print("=" * 60)

    created_modules = []
    created_cables = []

    try:
        # Create modules
        print("\n1. Creating modules...")

        vco = api.add_module("Fundamental", "VCO-1", x=0, y=0)
        created_modules.append(vco['id'])
        print(f"   ✓ VCO-1 (ID: {vco['id']})")

        vcf = api.add_module("Fundamental", "VCF", x=150, y=0)
        created_modules.append(vcf['id'])
        print(f"   ✓ VCF (ID: {vcf['id']})")

        vca = api.add_module("Fundamental", "VCA-1", x=300, y=0)
        created_modules.append(vca['id'])
        print(f"   ✓ VCA-1 (ID: {vca['id']})")

        scope = api.add_module("Fundamental", "Scope", x=450, y=0)
        created_modules.append(scope['id'])
        print(f"   ✓ Scope (ID: {scope['id']})")

        # Create signal chain
        print("\n2. Creating signal chain: VCO -> VCF -> VCA -> Scope")

        cable1 = api.add_cable(vco['id'], 0, vcf['id'], 0)
        created_cables.append(cable1['id'])
        print(f"   ✓ VCO -> VCF (Cable: {cable1['id']})")

        cable2 = api.add_cable(vcf['id'], 0, vca['id'], 0)
        created_cables.append(cable2['id'])
        print(f"   ✓ VCF -> VCA (Cable: {cable2['id']})")

        cable3 = api.add_cable(vca['id'], 0, scope['id'], 0)
        created_cables.append(cable3['id'])
        print(f"   ✓ VCA -> Scope (Cable: {cable3['id']})")

        # Display patch summary
        print("\n3. Patch Summary:")
        modules = api.get_modules()
        cables = api.get_cables()
        print(f"   Total modules: {len(modules['modules'])}")
        print(f"   Total cables: {len(cables['cables'])}")

        print("\n✓ Example patch created successfully!")
        print("\n   The patch should now be visible in VCV Rack.")
        print("   Press Enter to clean up the patch...")
        input()

    finally:
        # Cleanup
        print("\nCleaning up patch...")
        for cable_id in created_cables:
            try:
                api.delete_cable(cable_id)
            except:
                pass
        for module_id in created_modules:
            try:
                api.delete_module(module_id)
            except:
                pass
        print("   ✓ Cleanup complete")


def main():
    parser = argparse.ArgumentParser(description='Test VCV Rack HTTP API')
    parser.add_argument('--host', default='localhost', help='VCV Rack host (default: localhost)')
    parser.add_argument('--port', type=int, default=8080, help='VCV Rack HTTP API port (default: 8080)')
    parser.add_argument('--test', choices=['basic', 'modules', 'cables', 'patch', 'all'],
                        default='all', help='Which test to run (default: all)')

    args = parser.parse_args()

    # Create API client
    api = RackAPI(host=args.host, port=args.port)

    # Test connection
    print(f"Connecting to VCV Rack at {api.base_url}...")
    try:
        api.get_plugins()
        print("✓ Connected successfully!\n")
    except requests.exceptions.ConnectionError:
        print(f"\n✗ Error: Could not connect to VCV Rack at {api.base_url}")
        print("\nMake sure:")
        print("  1. VCV Rack is running")
        print("  2. HTTP API is enabled (use --httpapi flag)")
        print("  3. The port matches (default is 8080)")
        sys.exit(1)
    except Exception as e:
        print(f"\n✗ Error: {e}")
        sys.exit(1)

    # Run tests
    try:
        if args.test in ['basic', 'all']:
            test_basic_operations(api)

        if args.test in ['modules', 'all']:
            test_module_management(api)

        if args.test in ['cables', 'all']:
            test_cable_management(api)

        if args.test in ['patch', 'all']:
            build_example_patch(api)

        print("\n" + "=" * 60)
        print("All tests completed successfully! ✓")
        print("=" * 60)

    except Exception as e:
        print(f"\n✗ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
