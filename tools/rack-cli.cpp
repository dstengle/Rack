/*
 * VCV Rack CLI Client
 * Command-line interface for interacting with VCV Rack HTTP API
 */

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <algorithm>
#include <sstream>
#include <iomanip>
#include <curl/curl.h>
#include <jansson.h>

// Configuration
static std::string API_BASE = "http://localhost:8080";

// HTTP response structure
struct HttpResponse {
	std::string body;
	long statusCode;
	bool success;
};

// Callback for libcurl to write response data
static size_t WriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
	((std::string*)userp)->append((char*)contents, size * nmemb);
	return size * nmemb;
}

// Make HTTP GET request
HttpResponse httpGet(const std::string& url) {
	HttpResponse response;
	CURL* curl = curl_easy_init();

	if (curl) {
		curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
		curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
		curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response.body);
		curl_easy_setopt(curl, CURLOPT_TIMEOUT, 10L);

		CURLcode res = curl_easy_perform(curl);
		curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response.statusCode);

		response.success = (res == CURLE_OK && response.statusCode == 200);
		curl_easy_cleanup(curl);
	}

	return response;
}

// Parse JSON response
json_t* parseJson(const HttpResponse& response) {
	if (!response.success) {
		std::cerr << "Error: HTTP request failed (status " << response.statusCode << ")" << std::endl;
		if (!response.body.empty()) {
			std::cerr << "Response: " << response.body << std::endl;
		}
		return nullptr;
	}

	json_error_t error;
	json_t* root = json_loads(response.body.c_str(), 0, &error);

	if (!root) {
		std::cerr << "Error: Failed to parse JSON: " << error.text << std::endl;
		return nullptr;
	}

	return root;
}

// Print table separator
void printSeparator(const std::vector<int>& widths) {
	std::cout << "+";
	for (int w : widths) {
		std::cout << std::string(w + 2, '-') << "+";
	}
	std::cout << std::endl;
}

// Print table row
void printRow(const std::vector<std::string>& cells, const std::vector<int>& widths) {
	std::cout << "|";
	for (size_t i = 0; i < cells.size(); i++) {
		std::cout << " " << std::left << std::setw(widths[i]) << cells[i] << " |";
	}
	std::cout << std::endl;
}

// Command: list-modules
int cmdListModules() {
	std::cout << "Fetching modules from " << API_BASE << "..." << std::endl << std::endl;

	HttpResponse response = httpGet(API_BASE + "/api/modules");
	json_t* root = parseJson(response);
	if (!root) return 1;

	json_t* modulesArray = json_object_get(root, "modules");
	if (!modulesArray || !json_is_array(modulesArray)) {
		std::cerr << "Error: Invalid response format" << std::endl;
		json_decref(root);
		return 1;
	}

	size_t numModules = json_array_size(modulesArray);
	if (numModules == 0) {
		std::cout << "No modules in the patch." << std::endl;
		json_decref(root);
		return 0;
	}

	std::cout << "Found " << numModules << " module(s) in the patch:" << std::endl << std::endl;

	// Table headers
	std::vector<int> widths = {6, 20, 20, 15, 15};
	std::vector<std::string> headers = {"ID", "Plugin", "Model", "Position", "Size"};

	printSeparator(widths);
	printRow(headers, widths);
	printSeparator(widths);

	// Print each module
	for (size_t i = 0; i < numModules; i++) {
		json_t* module = json_array_get(modulesArray, i);

		json_t* idJ = json_object_get(module, "id");
		json_t* pluginSlugJ = json_object_get(module, "pluginSlug");
		json_t* modelNameJ = json_object_get(module, "modelName");
		json_t* posJ = json_object_get(module, "pos");
		json_t* sizeJ = json_object_get(module, "size");

		std::ostringstream id, plugin, model, pos, size;

		if (idJ) id << json_integer_value(idJ);
		if (pluginSlugJ) plugin << json_string_value(pluginSlugJ);
		if (modelNameJ) model << json_string_value(modelNameJ);

		if (posJ) {
			json_t* xJ = json_object_get(posJ, "x");
			json_t* yJ = json_object_get(posJ, "y");
			if (xJ && yJ) {
				pos << std::fixed << std::setprecision(0)
				    << json_number_value(xJ) << "," << json_number_value(yJ);
			}
		}

		if (sizeJ) {
			json_t* xJ = json_object_get(sizeJ, "x");
			json_t* yJ = json_object_get(sizeJ, "y");
			if (xJ && yJ) {
				size << std::fixed << std::setprecision(0)
				     << json_number_value(xJ) << "x" << json_number_value(yJ);
			}
		}

		std::vector<std::string> row = {
			id.str(), plugin.str(), model.str(), pos.str(), size.str()
		};
		printRow(row, widths);
	}

	printSeparator(widths);

	json_decref(root);
	return 0;
}

// Command: show-module <id>
int cmdShowModule(const std::string& moduleIdStr) {
	std::cout << "Fetching module details for ID " << moduleIdStr << "..." << std::endl << std::endl;

	HttpResponse moduleResponse = httpGet(API_BASE + "/api/modules/" + moduleIdStr);
	json_t* moduleRoot = parseJson(moduleResponse);
	if (!moduleRoot) return 1;

	// Print module info
	json_t* idJ = json_object_get(moduleRoot, "id");
	json_t* pluginSlugJ = json_object_get(moduleRoot, "pluginSlug");
	json_t* modelSlugJ = json_object_get(moduleRoot, "modelSlug");
	json_t* modelNameJ = json_object_get(moduleRoot, "modelName");

	std::cout << "Module Information:" << std::endl;
	std::cout << "  ID:          " << (idJ ? std::to_string(json_integer_value(idJ)) : "N/A") << std::endl;
	std::cout << "  Plugin:      " << (pluginSlugJ ? json_string_value(pluginSlugJ) : "N/A") << std::endl;
	std::cout << "  Model Slug:  " << (modelSlugJ ? json_string_value(modelSlugJ) : "N/A") << std::endl;
	std::cout << "  Model Name:  " << (modelNameJ ? json_string_value(modelNameJ) : "N/A") << std::endl;
	std::cout << std::endl;

	// Print parameters
	json_t* paramsArray = json_object_get(moduleRoot, "params");
	if (paramsArray && json_is_array(paramsArray)) {
		size_t numParams = json_array_size(paramsArray);
		if (numParams > 0) {
			std::cout << "Parameters (" << numParams << "):" << std::endl;
			std::vector<int> paramWidths = {4, 25, 15, 15, 20};
			std::vector<std::string> paramHeaders = {"ID", "Name", "Value", "Range", "Display"};

			printSeparator(paramWidths);
			printRow(paramHeaders, paramWidths);
			printSeparator(paramWidths);

			for (size_t i = 0; i < numParams; i++) {
				json_t* param = json_array_get(paramsArray, i);
				json_t* paramIdJ = json_object_get(param, "id");
				json_t* nameJ = json_object_get(param, "name");
				json_t* valueJ = json_object_get(param, "value");
				json_t* minJ = json_object_get(param, "minValue");
				json_t* maxJ = json_object_get(param, "maxValue");
				json_t* displayJ = json_object_get(param, "displayValue");

				std::ostringstream id, name, value, range, display;

				if (paramIdJ) id << json_integer_value(paramIdJ);
				if (nameJ) name << json_string_value(nameJ);
				if (valueJ) value << std::fixed << std::setprecision(3) << json_number_value(valueJ);

				if (minJ && maxJ) {
					range << std::fixed << std::setprecision(1)
					      << json_number_value(minJ) << " to " << json_number_value(maxJ);
				}

				if (displayJ) display << json_string_value(displayJ);

				std::vector<std::string> row = {
					id.str(), name.str(), value.str(), range.str(), display.str()
				};
				printRow(row, paramWidths);
			}

			printSeparator(paramWidths);
			std::cout << std::endl;
		}
	}

	// Print inputs
	json_t* inputsArray = json_object_get(moduleRoot, "inputs");
	if (inputsArray && json_is_array(inputsArray)) {
		size_t numInputs = json_array_size(inputsArray);
		if (numInputs > 0) {
			std::cout << "Inputs (" << numInputs << "):" << std::endl;
			std::vector<int> ioWidths = {4, 30, 10, 10};
			std::vector<std::string> ioHeaders = {"ID", "Name", "Channels", "Connected"};

			printSeparator(ioWidths);
			printRow(ioHeaders, ioWidths);
			printSeparator(ioWidths);

			for (size_t i = 0; i < numInputs; i++) {
				json_t* input = json_array_get(inputsArray, i);
				json_t* inputIdJ = json_object_get(input, "id");
				json_t* nameJ = json_object_get(input, "name");
				json_t* channelsJ = json_object_get(input, "channels");
				json_t* connectedJ = json_object_get(input, "connected");

				std::ostringstream id, name, channels, connected;

				if (inputIdJ) id << json_integer_value(inputIdJ);
				if (nameJ) name << json_string_value(nameJ);
				if (channelsJ) channels << json_integer_value(channelsJ);
				if (connectedJ) connected << (json_is_true(connectedJ) ? "Yes" : "No");

				std::vector<std::string> row = {
					id.str(), name.str(), channels.str(), connected.str()
				};
				printRow(row, ioWidths);
			}

			printSeparator(ioWidths);
			std::cout << std::endl;
		}
	}

	// Print outputs
	json_t* outputsArray = json_object_get(moduleRoot, "outputs");
	if (outputsArray && json_is_array(outputsArray)) {
		size_t numOutputs = json_array_size(outputsArray);
		if (numOutputs > 0) {
			std::cout << "Outputs (" << numOutputs << "):" << std::endl;
			std::vector<int> ioWidths = {4, 30, 10, 10};
			std::vector<std::string> ioHeaders = {"ID", "Name", "Channels", "Connected"};

			printSeparator(ioWidths);
			printRow(ioHeaders, ioWidths);
			printSeparator(ioWidths);

			for (size_t i = 0; i < numOutputs; i++) {
				json_t* output = json_array_get(outputsArray, i);
				json_t* outputIdJ = json_object_get(output, "id");
				json_t* nameJ = json_object_get(output, "name");
				json_t* channelsJ = json_object_get(output, "channels");
				json_t* connectedJ = json_object_get(output, "connected");

				std::ostringstream id, name, channels, connected;

				if (outputIdJ) id << json_integer_value(outputIdJ);
				if (nameJ) name << json_string_value(nameJ);
				if (channelsJ) channels << json_integer_value(channelsJ);
				if (connectedJ) connected << (json_is_true(connectedJ) ? "Yes" : "No");

				std::vector<std::string> row = {
					id.str(), name.str(), channels.str(), connected.str()
				};
				printRow(row, ioWidths);
			}

			printSeparator(ioWidths);
			std::cout << std::endl;
		}
	}

	// Get cables to show connections
	HttpResponse cablesResponse = httpGet(API_BASE + "/api/cables");
	json_t* cablesRoot = parseJson(cablesResponse);
	if (cablesRoot) {
		json_t* cablesArray = json_object_get(cablesRoot, "cables");
		if (cablesArray && json_is_array(cablesArray)) {
			// Find connections involving this module
			std::vector<json_t*> incomingCables;
			std::vector<json_t*> outgoingCables;

			int64_t moduleId = idJ ? json_integer_value(idJ) : -1;

			for (size_t i = 0; i < json_array_size(cablesArray); i++) {
				json_t* cable = json_array_get(cablesArray, i);
				json_t* inputModuleIdJ = json_object_get(cable, "inputModuleId");
				json_t* outputModuleIdJ = json_object_get(cable, "outputModuleId");

				if (inputModuleIdJ && json_integer_value(inputModuleIdJ) == moduleId) {
					incomingCables.push_back(cable);
				}
				if (outputModuleIdJ && json_integer_value(outputModuleIdJ) == moduleId) {
					outgoingCables.push_back(cable);
				}
			}

			// Print incoming connections
			if (!incomingCables.empty()) {
				std::cout << "Incoming Connections (" << incomingCables.size() << "):" << std::endl;
				std::vector<int> connWidths = {8, 15, 12, 12};
				std::vector<std::string> connHeaders = {"Cable ID", "From Module", "Output Port", "To Port"};

				printSeparator(connWidths);
				printRow(connHeaders, connWidths);
				printSeparator(connWidths);

				for (json_t* cable : incomingCables) {
					json_t* cableIdJ = json_object_get(cable, "id");
					json_t* outputModuleIdJ = json_object_get(cable, "outputModuleId");
					json_t* outputIdJ = json_object_get(cable, "outputId");
					json_t* inputIdJ = json_object_get(cable, "inputId");

					std::ostringstream cableId, fromModule, outputPort, toPort;

					if (cableIdJ) cableId << json_integer_value(cableIdJ);
					if (outputModuleIdJ) fromModule << json_integer_value(outputModuleIdJ);
					if (outputIdJ) outputPort << json_integer_value(outputIdJ);
					if (inputIdJ) toPort << json_integer_value(inputIdJ);

					std::vector<std::string> row = {
						cableId.str(), fromModule.str(), outputPort.str(), toPort.str()
					};
					printRow(row, connWidths);
				}

				printSeparator(connWidths);
				std::cout << std::endl;
			}

			// Print outgoing connections
			if (!outgoingCables.empty()) {
				std::cout << "Outgoing Connections (" << outgoingCables.size() << "):" << std::endl;
				std::vector<int> connWidths = {8, 12, 13, 12};
				std::vector<std::string> connHeaders = {"Cable ID", "From Port", "To Module", "Input Port"};

				printSeparator(connWidths);
				printRow(connHeaders, connWidths);
				printSeparator(connWidths);

				for (json_t* cable : outgoingCables) {
					json_t* cableIdJ = json_object_get(cable, "id");
					json_t* outputIdJ = json_object_get(cable, "outputId");
					json_t* inputModuleIdJ = json_object_get(cable, "inputModuleId");
					json_t* inputIdJ = json_object_get(cable, "inputId");

					std::ostringstream cableId, fromPort, toModule, inputPort;

					if (cableIdJ) cableId << json_integer_value(cableIdJ);
					if (outputIdJ) fromPort << json_integer_value(outputIdJ);
					if (inputModuleIdJ) toModule << json_integer_value(inputModuleIdJ);
					if (inputIdJ) inputPort << json_integer_value(inputIdJ);

					std::vector<std::string> row = {
						cableId.str(), fromPort.str(), toModule.str(), inputPort.str()
					};
					printRow(row, connWidths);
				}

				printSeparator(connWidths);
				std::cout << std::endl;
			}

			if (incomingCables.empty() && outgoingCables.empty()) {
				std::cout << "No connections to/from this module." << std::endl;
			}
		}

		json_decref(cablesRoot);
	}

	json_decref(moduleRoot);
	return 0;
}

// Structure to hold cable info with module details
struct CableInfo {
	int64_t cableId;
	int64_t outputModuleId;
	std::string outputModuleName;
	int outputId;
	std::string outputName;
	int64_t inputModuleId;
	std::string inputModuleName;
	int inputId;
	std::string inputName;
	bool isAudio;  // Heuristic based on port names
};

// Determine if a port is likely audio based on its name
bool isLikelyAudioPort(const std::string& name) {
	std::string lower = name;
	std::transform(lower.begin(), lower.end(), lower.begin(), ::tolower);

	// Audio keywords
	if (lower.find("audio") != std::string::npos) return true;
	if (lower.find("output") != std::string::npos && lower.find("cv") == std::string::npos) return true;
	if (lower.find("signal") != std::string::npos) return true;
	if (lower.find("sine") != std::string::npos) return true;
	if (lower.find("saw") != std::string::npos) return true;
	if (lower.find("square") != std::string::npos) return true;
	if (lower.find("triangle") != std::string::npos) return true;
	if (lower.find("mix") != std::string::npos) return true;
	if (lower.find("left") != std::string::npos) return true;
	if (lower.find("right") != std::string::npos) return true;
	if (lower.find("in ") != std::string::npos) return true;
	if (lower.find("out ") != std::string::npos) return true;

	// Control/CV keywords (NOT audio)
	if (lower.find("cv") != std::string::npos) return false;
	if (lower.find("gate") != std::string::npos) return false;
	if (lower.find("trigger") != std::string::npos) return false;
	if (lower.find("clock") != std::string::npos) return false;
	if (lower.find("mod") != std::string::npos) return false;
	if (lower.find("fm") != std::string::npos) return false;
	if (lower.find("pitch") != std::string::npos) return false;
	if (lower.find("v/oct") != std::string::npos) return false;

	// Default to audio if no specific CV indicators
	return true;
}

// Command: list-connections [--sort-by module|type]
int cmdListConnections(const std::string& sortBy) {
	std::cout << "Fetching connections from " << API_BASE << "..." << std::endl << std::endl;

	// Get cables
	HttpResponse cablesResponse = httpGet(API_BASE + "/api/cables");
	json_t* cablesRoot = parseJson(cablesResponse);
	if (!cablesRoot) return 1;

	// Get modules to resolve module names
	HttpResponse modulesResponse = httpGet(API_BASE + "/api/modules");
	json_t* modulesRoot = parseJson(modulesResponse);
	if (!modulesRoot) {
		json_decref(cablesRoot);
		return 1;
	}

	// Build module ID to name map and get detailed info for port names
	std::map<int64_t, std::string> moduleNames;
	std::map<int64_t, json_t*> moduleDetails;

	json_t* modulesArray = json_object_get(modulesRoot, "modules");
	if (modulesArray && json_is_array(modulesArray)) {
		for (size_t i = 0; i < json_array_size(modulesArray); i++) {
			json_t* module = json_array_get(modulesArray, i);
			json_t* idJ = json_object_get(module, "id");
			json_t* modelNameJ = json_object_get(module, "modelName");

			if (idJ && modelNameJ) {
				int64_t id = json_integer_value(idJ);
				moduleNames[id] = json_string_value(modelNameJ);

				// Fetch detailed module info
				std::string moduleIdStr = std::to_string(id);
				HttpResponse detailResponse = httpGet(API_BASE + "/api/modules/" + moduleIdStr);
				json_t* detailRoot = parseJson(detailResponse);
				if (detailRoot) {
					// Store it (will be freed later)
					json_incref(detailRoot);
					moduleDetails[id] = detailRoot;
				}
			}
		}
	}

	// Process cables
	json_t* cablesArray = json_object_get(cablesRoot, "cables");
	if (!cablesArray || !json_is_array(cablesArray)) {
		std::cerr << "Error: Invalid cables response format" << std::endl;
		json_decref(cablesRoot);
		json_decref(modulesRoot);
		for (auto& pair : moduleDetails) {
			json_decref(pair.second);
		}
		return 1;
	}

	size_t numCables = json_array_size(cablesArray);
	if (numCables == 0) {
		std::cout << "No connections in the patch." << std::endl;
		json_decref(cablesRoot);
		json_decref(modulesRoot);
		for (auto& pair : moduleDetails) {
			json_decref(pair.second);
		}
		return 0;
	}

	// Build cable info list
	std::vector<CableInfo> cables;

	for (size_t i = 0; i < numCables; i++) {
		json_t* cable = json_array_get(cablesArray, i);

		json_t* cableIdJ = json_object_get(cable, "id");
		json_t* outputModuleIdJ = json_object_get(cable, "outputModuleId");
		json_t* outputIdJ = json_object_get(cable, "outputId");
		json_t* inputModuleIdJ = json_object_get(cable, "inputModuleId");
		json_t* inputIdJ = json_object_get(cable, "inputId");

		CableInfo info;
		info.cableId = cableIdJ ? json_integer_value(cableIdJ) : -1;
		info.outputModuleId = outputModuleIdJ ? json_integer_value(outputModuleIdJ) : -1;
		info.outputId = outputIdJ ? json_integer_value(outputIdJ) : -1;
		info.inputModuleId = inputModuleIdJ ? json_integer_value(inputModuleIdJ) : -1;
		info.inputId = inputIdJ ? json_integer_value(inputIdJ) : -1;

		// Get module names
		if (moduleNames.count(info.outputModuleId)) {
			info.outputModuleName = moduleNames[info.outputModuleId];
		} else {
			info.outputModuleName = "Module " + std::to_string(info.outputModuleId);
		}

		if (moduleNames.count(info.inputModuleId)) {
			info.inputModuleName = moduleNames[info.inputModuleId];
		} else {
			info.inputModuleName = "Module " + std::to_string(info.inputModuleId);
		}

		// Get port names
		info.outputName = "Port " + std::to_string(info.outputId);
		info.inputName = "Port " + std::to_string(info.inputId);

		if (moduleDetails.count(info.outputModuleId)) {
			json_t* detail = moduleDetails[info.outputModuleId];
			json_t* outputsArray = json_object_get(detail, "outputs");
			if (outputsArray && json_is_array(outputsArray)) {
				for (size_t j = 0; j < json_array_size(outputsArray); j++) {
					json_t* output = json_array_get(outputsArray, j);
					json_t* portIdJ = json_object_get(output, "id");
					json_t* nameJ = json_object_get(output, "name");
					if (portIdJ && json_integer_value(portIdJ) == info.outputId && nameJ) {
						info.outputName = json_string_value(nameJ);
						break;
					}
				}
			}
		}

		if (moduleDetails.count(info.inputModuleId)) {
			json_t* detail = moduleDetails[info.inputModuleId];
			json_t* inputsArray = json_object_get(detail, "inputs");
			if (inputsArray && json_is_array(inputsArray)) {
				for (size_t j = 0; j < json_array_size(inputsArray); j++) {
					json_t* input = json_array_get(inputsArray, j);
					json_t* portIdJ = json_object_get(input, "id");
					json_t* nameJ = json_object_get(input, "name");
					if (portIdJ && json_integer_value(portIdJ) == info.inputId && nameJ) {
						info.inputName = json_string_value(nameJ);
						break;
					}
				}
			}
		}

		// Determine if audio or control based on port names
		info.isAudio = isLikelyAudioPort(info.outputName) || isLikelyAudioPort(info.inputName);

		cables.push_back(info);
	}

	// Sort cables
	if (sortBy == "module") {
		std::sort(cables.begin(), cables.end(), [](const CableInfo& a, const CableInfo& b) {
			if (a.outputModuleId != b.outputModuleId)
				return a.outputModuleId < b.outputModuleId;
			if (a.inputModuleId != b.inputModuleId)
				return a.inputModuleId < b.inputModuleId;
			return a.cableId < b.cableId;
		});
	} else if (sortBy == "type") {
		std::sort(cables.begin(), cables.end(), [](const CableInfo& a, const CableInfo& b) {
			if (a.isAudio != b.isAudio)
				return a.isAudio;  // Audio first
			if (a.outputModuleId != b.outputModuleId)
				return a.outputModuleId < b.outputModuleId;
			return a.cableId < b.cableId;
		});
	}

	// Print cables
	std::cout << "Found " << numCables << " connection(s)";
	if (!sortBy.empty()) {
		std::cout << " (sorted by " << sortBy << ")";
	}
	std::cout << ":" << std::endl << std::endl;

	std::vector<int> cableWidths = {8, 6, 20, 25, 20, 25};
	std::vector<std::string> cableHeaders = {"Cable ID", "Type", "From Module", "Output Port", "To Module", "Input Port"};

	printSeparator(cableWidths);
	printRow(cableHeaders, cableWidths);
	printSeparator(cableWidths);

	for (const CableInfo& cable : cables) {
		std::ostringstream cableId, type, fromModule, outputPort, toModule, inputPort;

		cableId << cable.cableId;
		type << (cable.isAudio ? "Audio" : "CV");
		fromModule << cable.outputModuleName << " [" << cable.outputModuleId << "]";
		outputPort << cable.outputName;
		toModule << cable.inputModuleName << " [" << cable.inputModuleId << "]";
		inputPort << cable.inputName;

		std::vector<std::string> row = {
			cableId.str(), type.str(), fromModule.str(), outputPort.str(),
			toModule.str(), inputPort.str()
		};
		printRow(row, cableWidths);
	}

	printSeparator(cableWidths);

	// Print summary by type
	int audioCount = 0, cvCount = 0;
	for (const CableInfo& cable : cables) {
		if (cable.isAudio) audioCount++;
		else cvCount++;
	}

	std::cout << std::endl;
	std::cout << "Summary:" << std::endl;
	std::cout << "  Audio connections: " << audioCount << std::endl;
	std::cout << "  CV connections:    " << cvCount << std::endl;

	// Clean up
	json_decref(cablesRoot);
	json_decref(modulesRoot);
	for (auto& pair : moduleDetails) {
		json_decref(pair.second);
	}

	return 0;
}

// Command: list-plugins
int cmdListPlugins() {
	std::cout << "Fetching plugins from " << API_BASE << "..." << std::endl << std::endl;

	HttpResponse response = httpGet(API_BASE + "/api/plugins");
	json_t* root = parseJson(response);
	if (!root) return 1;

	// Extract plugins array from response object
	json_t* pluginsArray = json_object_get(root, "plugins");
	if (!pluginsArray || !json_is_array(pluginsArray)) {
		std::cerr << "Error: Invalid response format (expected 'plugins' array)" << std::endl;
		json_decref(root);
		return 1;
	}

	size_t numPlugins = json_array_size(pluginsArray);
	std::cout << "Found " << numPlugins << " plugin(s):" << std::endl << std::endl;

	// Table headers
	std::vector<int> widths = {25, 25, 12, 10};
	std::vector<std::string> headers = {"Plugin", "Brand", "Version", "Models"};

	printSeparator(widths);
	printRow(headers, widths);
	printSeparator(widths);

	// Print each plugin
	for (size_t i = 0; i < numPlugins; i++) {
		json_t* plugin = json_array_get(pluginsArray, i);

		json_t* slugJ = json_object_get(plugin, "slug");
		json_t* nameJ = json_object_get(plugin, "name");
		json_t* brandJ = json_object_get(plugin, "brand");
		json_t* versionJ = json_object_get(plugin, "version");
		json_t* modelsJ = json_object_get(plugin, "models");

		std::ostringstream slug, brand, version, models;

		if (slugJ) slug << json_string_value(slugJ);
		if (brandJ) brand << json_string_value(brandJ);
		else if (nameJ) brand << json_string_value(nameJ);
		if (versionJ) version << json_string_value(versionJ);
		if (modelsJ && json_is_array(modelsJ)) {
			models << json_array_size(modelsJ);
		}

		std::vector<std::string> row = {
			slug.str(), brand.str(), version.str(), models.str()
		};
		printRow(row, widths);
	}

	printSeparator(widths);

	json_decref(root);
	return 0;
}

// Command: list-models [plugin-slug]
int cmdListModels(const std::string& pluginSlug) {
	std::cout << "Fetching models from " << API_BASE << "..." << std::endl << std::endl;

	HttpResponse response = httpGet(API_BASE + "/api/plugins");
	json_t* root = parseJson(response);
	if (!root) return 1;

	// Extract plugins array from response object
	json_t* pluginsArray = json_object_get(root, "plugins");
	if (!pluginsArray || !json_is_array(pluginsArray)) {
		std::cerr << "Error: Invalid response format (expected 'plugins' array)" << std::endl;
		json_decref(root);
		return 1;
	}

	// Collect all models
	std::vector<std::tuple<std::string, std::string, std::string, std::string>> allModels;

	for (size_t i = 0; i < json_array_size(pluginsArray); i++) {
		json_t* plugin = json_array_get(pluginsArray, i);
		json_t* pluginSlugJ = json_object_get(plugin, "slug");
		json_t* brandJ = json_object_get(plugin, "brand");
		json_t* modelsJ = json_object_get(plugin, "models");

		if (!pluginSlugJ || !modelsJ || !json_is_array(modelsJ)) continue;

		std::string pSlug = json_string_value(pluginSlugJ);
		std::string brand = brandJ ? json_string_value(brandJ) : pSlug;

		// Filter by plugin if specified
		if (!pluginSlug.empty() && pSlug != pluginSlug) continue;

		for (size_t j = 0; j < json_array_size(modelsJ); j++) {
			json_t* model = json_array_get(modelsJ, j);
			json_t* modelSlugJ = json_object_get(model, "slug");
			json_t* nameJ = json_object_get(model, "name");
			json_t* descJ = json_object_get(model, "description");

			std::string modelSlug = modelSlugJ ? json_string_value(modelSlugJ) : "";
			std::string name = nameJ ? json_string_value(nameJ) : modelSlug;
			std::string desc = descJ ? json_string_value(descJ) : "";

			allModels.push_back(std::make_tuple(pSlug, brand, name, desc));
		}
	}

	if (allModels.empty()) {
		if (pluginSlug.empty()) {
			std::cout << "No models found." << std::endl;
		} else {
			std::cout << "No models found for plugin '" << pluginSlug << "'." << std::endl;
		}
		json_decref(root);
		return 0;
	}

	std::cout << "Found " << allModels.size() << " model(s)";
	if (!pluginSlug.empty()) {
		std::cout << " in plugin '" << pluginSlug << "'";
	}
	std::cout << ":" << std::endl << std::endl;

	// Table headers
	std::vector<int> widths = {20, 20, 25, 40};
	std::vector<std::string> headers = {"Plugin", "Brand", "Model", "Description"};

	printSeparator(widths);
	printRow(headers, widths);
	printSeparator(widths);

	// Print each model
	for (const auto& modelTuple : allModels) {
		std::vector<std::string> row = {
			std::get<0>(modelTuple),
			std::get<1>(modelTuple),
			std::get<2>(modelTuple),
			std::get<3>(modelTuple)
		};
		printRow(row, widths);
	}

	printSeparator(widths);

	json_decref(root);
	return 0;
}

// Command: get <endpoint>
int cmdGet(const std::string& endpoint) {
	// Ensure endpoint starts with /
	std::string path = endpoint;
	if (path.empty() || path[0] != '/') {
		path = "/" + path;
	}

	std::string url = API_BASE + path;
	std::cout << "GET " << url << std::endl << std::endl;

	HttpResponse response = httpGet(url);

	if (response.success) {
		std::cout << response.body << std::endl;
		return 0;
	} else {
		std::cerr << "Error: HTTP request failed (status " << response.statusCode << ")" << std::endl;
		if (!response.body.empty()) {
			std::cerr << "Response: " << response.body << std::endl;
		}
		return 1;
	}
}

// Print usage
void printUsage(const char* progName) {
	std::cout << "VCV Rack CLI Client - Command-line interface for VCV Rack HTTP API" << std::endl;
	std::cout << std::endl;
	std::cout << "Usage: " << progName << " [options] <command> [args]" << std::endl;
	std::cout << std::endl;
	std::cout << "Options:" << std::endl;
	std::cout << "  --host <host>       Set API host (default: localhost)" << std::endl;
	std::cout << "  --port <port>       Set API port (default: 8080)" << std::endl;
	std::cout << "  --help, -h          Show this help message" << std::endl;
	std::cout << std::endl;
	std::cout << "Commands:" << std::endl;
	std::cout << "  list-plugins        List all installed plugins" << std::endl;
	std::cout << "  list-models [slug]  List all models (optionally filter by plugin slug)" << std::endl;
	std::cout << "  list-modules        List all modules in the current patch" << std::endl;
	std::cout << "  show-module <id>    Show detailed information about a module" << std::endl;
	std::cout << "  list-connections    List all connections (cables) in the patch" << std::endl;
	std::cout << "    --sort-by module  Sort connections by module ID" << std::endl;
	std::cout << "    --sort-by type    Sort connections by type (audio/CV)" << std::endl;
	std::cout << "  get <endpoint>      Make a raw GET request to an API endpoint" << std::endl;
	std::cout << std::endl;
	std::cout << "Examples:" << std::endl;
	std::cout << "  " << progName << " list-plugins" << std::endl;
	std::cout << "  " << progName << " list-models" << std::endl;
	std::cout << "  " << progName << " list-models Fundamental" << std::endl;
	std::cout << "  " << progName << " list-modules" << std::endl;
	std::cout << "  " << progName << " show-module 1" << std::endl;
	std::cout << "  " << progName << " list-connections --sort-by type" << std::endl;
	std::cout << "  " << progName << " get /api/plugins" << std::endl;
	std::cout << "  " << progName << " --port 9000 list-plugins" << std::endl;
	std::cout << std::endl;
}

int main(int argc, char* argv[]) {
	// Initialize libcurl
	curl_global_init(CURL_GLOBAL_DEFAULT);

	// Parse arguments
	std::vector<std::string> args;
	std::string host = "localhost";
	int port = 8080;

	for (int i = 1; i < argc; i++) {
		std::string arg = argv[i];

		if (arg == "--help" || arg == "-h") {
			printUsage(argv[0]);
			curl_global_cleanup();
			return 0;
		} else if (arg == "--host") {
			if (i + 1 < argc) {
				host = argv[++i];
			} else {
				std::cerr << "Error: --host requires an argument" << std::endl;
				curl_global_cleanup();
				return 1;
			}
		} else if (arg == "--port") {
			if (i + 1 < argc) {
				port = std::atoi(argv[++i]);
			} else {
				std::cerr << "Error: --port requires an argument" << std::endl;
				curl_global_cleanup();
				return 1;
			}
		} else {
			args.push_back(arg);
		}
	}

	// Build API base URL
	API_BASE = "http://" + host + ":" + std::to_string(port);

	// Parse command
	if (args.empty()) {
		std::cerr << "Error: No command specified" << std::endl;
		std::cerr << "Use --help for usage information" << std::endl;
		curl_global_cleanup();
		return 1;
	}

	std::string command = args[0];
	int result = 0;

	if (command == "list-plugins") {
		result = cmdListPlugins();
	} else if (command == "list-models") {
		if (args.size() >= 2) {
			result = cmdListModels(args[1]);
		} else {
			result = cmdListModels("");
		}
	} else if (command == "get") {
		if (args.size() < 2) {
			std::cerr << "Error: get requires an endpoint" << std::endl;
			result = 1;
		} else {
			result = cmdGet(args[1]);
		}
	} else if (command == "list-modules") {
		result = cmdListModules();
	} else if (command == "show-module") {
		if (args.size() < 2) {
			std::cerr << "Error: show-module requires a module ID" << std::endl;
			result = 1;
		} else {
			result = cmdShowModule(args[1]);
		}
	} else if (command == "list-connections") {
		std::string sortBy = "";
		if (args.size() >= 3 && args[1] == "--sort-by") {
			sortBy = args[2];
			if (sortBy != "module" && sortBy != "type") {
				std::cerr << "Error: --sort-by must be 'module' or 'type'" << std::endl;
				result = 1;
			} else {
				result = cmdListConnections(sortBy);
			}
		} else {
			result = cmdListConnections("");
		}
	} else {
		std::cerr << "Error: Unknown command '" << command << "'" << std::endl;
		std::cerr << "Use --help for usage information" << std::endl;
		result = 1;
	}

	// Cleanup
	curl_global_cleanup();

	return result;
}
