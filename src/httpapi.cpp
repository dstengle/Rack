#include <httpapi.hpp>
#include <context.hpp>
#include <plugin.hpp>
#include <engine/Engine.hpp>
#include <app/Scene.hpp>
#include <app/RackWidget.hpp>
#include <app/ModuleWidget.hpp>
#include <app/CableWidget.hpp>
#include <app/PortWidget.hpp>
#include <settings.hpp>
#include <system.hpp>
#include <string.hpp>

#include <thread>
#include <atomic>
#include <sstream>
#include <cstring>

#ifdef ARCH_WIN
	#include <winsock2.h>
	#include <ws2tcpip.h>
	#pragma comment(lib, "ws2_32.lib")
	typedef SOCKET socket_t;
	#define CLOSE_SOCKET closesocket
	#define SOCKET_ERROR_VALUE SOCKET_ERROR
	#define INVALID_SOCKET_VALUE INVALID_SOCKET
#else
	#include <sys/socket.h>
	#include <netinet/in.h>
	#include <arpa/inet.h>
	#include <unistd.h>
	#include <fcntl.h>
	typedef int socket_t;
	#define CLOSE_SOCKET close
	#define SOCKET_ERROR_VALUE -1
	#define INVALID_SOCKET_VALUE -1
#endif


namespace rack {
namespace httpapi {


static std::thread serverThread;
static std::atomic<bool> running{false};
static socket_t serverSocket = INVALID_SOCKET_VALUE;
static int serverPort = 8080;


struct HttpRequest {
	std::string method;
	std::string path;
	std::string query;
	std::map<std::string, std::string> headers;
	std::string body;
};


struct HttpResponse {
	int statusCode = 200;
	std::string statusText = "OK";
	std::map<std::string, std::string> headers;
	std::string body;

	HttpResponse() {
		headers["Content-Type"] = "application/json";
		headers["Access-Control-Allow-Origin"] = "*";
	}

	std::string toString() {
		std::ostringstream ss;
		ss << "HTTP/1.1 " << statusCode << " " << statusText << "\r\n";
		headers["Content-Length"] = std::to_string(body.length());
		for (const auto& pair : headers) {
			ss << pair.first << ": " << pair.second << "\r\n";
		}
		ss << "\r\n";
		ss << body;
		return ss.str();
	}
};


static void sendResponse(socket_t client, const HttpResponse& response) {
	std::string responseStr = response.toString();
	send(client, responseStr.c_str(), (int)responseStr.length(), 0);
}


static void sendJsonResponse(socket_t client, json_t* rootJ, int statusCode = 200) {
	HttpResponse response;
	response.statusCode = statusCode;
	response.statusText = (statusCode == 200) ? "OK" :
	                      (statusCode == 201) ? "Created" :
	                      (statusCode == 400) ? "Bad Request" :
	                      (statusCode == 404) ? "Not Found" :
	                      (statusCode == 500) ? "Internal Server Error" : "Error";

	if (rootJ) {
		char* jsonStr = json_dumps(rootJ, JSON_COMPACT);
		response.body = jsonStr;
		free(jsonStr);
		json_decref(rootJ);
	}

	sendResponse(client, response);
}


static void sendErrorResponse(socket_t client, int statusCode, const std::string& message) {
	json_t* errorJ = json_object();
	json_object_set_new(errorJ, "error", json_string(message.c_str()));
	sendJsonResponse(client, errorJ, statusCode);
}


static HttpRequest parseRequest(const std::string& requestStr) {
	HttpRequest request;
	std::istringstream iss(requestStr);
	std::string line;

	// Parse request line
	if (std::getline(iss, line)) {
		std::istringstream requestLine(line);
		std::string fullPath;
		requestLine >> request.method >> fullPath;

		// Split path and query
		size_t queryPos = fullPath.find('?');
		if (queryPos != std::string::npos) {
			request.path = fullPath.substr(0, queryPos);
			request.query = fullPath.substr(queryPos + 1);
		} else {
			request.path = fullPath;
		}
	}

	// Parse headers
	while (std::getline(iss, line) && line != "\r" && !line.empty()) {
		size_t colonPos = line.find(':');
		if (colonPos != std::string::npos) {
			std::string key = line.substr(0, colonPos);
			std::string value = line.substr(colonPos + 1);
			// Trim whitespace
			value.erase(0, value.find_first_not_of(" \t\r"));
			value.erase(value.find_last_not_of(" \t\r") + 1);
			request.headers[key] = value;
		}
	}

	// Parse body
	std::string bodyContent;
	while (std::getline(iss, line)) {
		bodyContent += line;
		if (iss.peek() != EOF)
			bodyContent += "\n";
	}
	request.body = bodyContent;

	return request;
}


// API endpoint handlers

static void handleGetPlugins(socket_t client) {
	json_t* pluginsJ = json_array();

	for (plugin::Plugin* p : plugin::plugins) {
		json_t* pluginJ = json_object();
		json_object_set_new(pluginJ, "slug", json_string(p->slug.c_str()));
		json_object_set_new(pluginJ, "name", json_string(p->name.c_str()));
		json_object_set_new(pluginJ, "brand", json_string(p->getBrand().c_str()));
		json_object_set_new(pluginJ, "version", json_string(p->version.c_str()));
		json_object_set_new(pluginJ, "author", json_string(p->author.c_str()));

		json_t* modelsJ = json_array();
		for (plugin::Model* m : p->models) {
			if (!m->hidden) {
				json_t* modelJ = json_object();
				json_object_set_new(modelJ, "slug", json_string(m->slug.c_str()));
				json_object_set_new(modelJ, "name", json_string(m->name.c_str()));
				json_object_set_new(modelJ, "description", json_string(m->description.c_str()));
				json_array_append_new(modelsJ, modelJ);
			}
		}
		json_object_set_new(pluginJ, "models", modelsJ);

		json_array_append_new(pluginsJ, pluginJ);
	}

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "plugins", pluginsJ);
	sendJsonResponse(client, responseJ);
}


static void handleGetModels(socket_t client) {
	json_t* modelsJ = json_array();

	for (plugin::Plugin* p : plugin::plugins) {
		for (plugin::Model* m : p->models) {
			if (!m->hidden) {
				json_t* modelJ = json_object();
				json_object_set_new(modelJ, "pluginSlug", json_string(p->slug.c_str()));
				json_object_set_new(modelJ, "pluginName", json_string(p->name.c_str()));
				json_object_set_new(modelJ, "slug", json_string(m->slug.c_str()));
				json_object_set_new(modelJ, "name", json_string(m->name.c_str()));
				json_object_set_new(modelJ, "fullName", json_string(m->getFullName().c_str()));
				json_object_set_new(modelJ, "description", json_string(m->description.c_str()));
				json_array_append_new(modelsJ, modelJ);
			}
		}
	}

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "models", modelsJ);
	sendJsonResponse(client, responseJ);
}


static void handleGetModules(socket_t client) {
	if (!APP || !APP->scene || !APP->scene->rack) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	json_t* modulesJ = json_array();

	std::vector<app::ModuleWidget*> moduleWidgets = APP->scene->rack->getModules();
	for (app::ModuleWidget* mw : moduleWidgets) {
		if (!mw->module)
			continue;

		json_t* moduleJ = json_object();
		json_object_set_new(moduleJ, "id", json_integer(mw->module->id));

		if (mw->module->model) {
			json_object_set_new(moduleJ, "pluginSlug", json_string(mw->module->model->plugin->slug.c_str()));
			json_object_set_new(moduleJ, "modelSlug", json_string(mw->module->model->slug.c_str()));
			json_object_set_new(moduleJ, "modelName", json_string(mw->module->model->name.c_str()));
		}

		json_t* posJ = json_object();
		json_object_set_new(posJ, "x", json_real(mw->box.pos.x));
		json_object_set_new(posJ, "y", json_real(mw->box.pos.y));
		json_object_set_new(moduleJ, "pos", posJ);

		json_t* sizeJ = json_object();
		json_object_set_new(sizeJ, "x", json_real(mw->box.size.x));
		json_object_set_new(sizeJ, "y", json_real(mw->box.size.y));
		json_object_set_new(moduleJ, "size", sizeJ);

		json_array_append_new(modulesJ, moduleJ);
	}

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "modules", modulesJ);
	sendJsonResponse(client, responseJ);
}


static void handleGetModuleDetails(socket_t client, int64_t moduleId) {
	if (!APP || !APP->engine) {
		sendErrorResponse(client, 500, "Engine not initialized");
		return;
	}

	engine::Module* module = APP->engine->getModule(moduleId);
	if (!module) {
		sendErrorResponse(client, 404, "Module not found");
		return;
	}

	json_t* moduleJ = json_object();
	json_object_set_new(moduleJ, "id", json_integer(module->id));

	if (module->model) {
		json_object_set_new(moduleJ, "pluginSlug", json_string(module->model->plugin->slug.c_str()));
		json_object_set_new(moduleJ, "modelSlug", json_string(module->model->slug.c_str()));
		json_object_set_new(moduleJ, "modelName", json_string(module->model->name.c_str()));
	}

	// Parameters
	json_t* paramsJ = json_array();
	for (size_t i = 0; i < module->params.size(); i++) {
		json_t* paramJ = json_object();
		json_object_set_new(paramJ, "id", json_integer(i));
		json_object_set_new(paramJ, "value", json_real(module->params[i].getValue()));

		if (i < module->paramQuantities.size() && module->paramQuantities[i]) {
			engine::ParamQuantity* pq = module->paramQuantities[i];
			json_object_set_new(paramJ, "name", json_string(pq->name.c_str()));
			json_object_set_new(paramJ, "label", json_string(pq->label.c_str()));
			json_object_set_new(paramJ, "unit", json_string(pq->unit.c_str()));
			json_object_set_new(paramJ, "minValue", json_real(pq->minValue));
			json_object_set_new(paramJ, "maxValue", json_real(pq->maxValue));
			json_object_set_new(paramJ, "defaultValue", json_real(pq->defaultValue));
			json_object_set_new(paramJ, "displayValue", json_string(pq->getDisplayValueString().c_str()));
		}

		json_array_append_new(paramsJ, paramJ);
	}
	json_object_set_new(moduleJ, "params", paramsJ);

	// Inputs
	json_t* inputsJ = json_array();
	for (size_t i = 0; i < module->inputs.size(); i++) {
		json_t* inputJ = json_object();
		json_object_set_new(inputJ, "id", json_integer(i));
		json_object_set_new(inputJ, "channels", json_integer(module->inputs[i].getChannels()));
		json_object_set_new(inputJ, "connected", json_boolean(module->inputs[i].isConnected()));

		if (i < module->inputInfos.size() && module->inputInfos[i]) {
			json_object_set_new(inputJ, "name", json_string(module->inputInfos[i]->name.c_str()));
			json_object_set_new(inputJ, "description", json_string(module->inputInfos[i]->description.c_str()));
		}

		json_array_append_new(inputsJ, inputJ);
	}
	json_object_set_new(moduleJ, "inputs", inputsJ);

	// Outputs
	json_t* outputsJ = json_array();
	for (size_t i = 0; i < module->outputs.size(); i++) {
		json_t* outputJ = json_object();
		json_object_set_new(outputJ, "id", json_integer(i));
		json_object_set_new(outputJ, "channels", json_integer(module->outputs[i].getChannels()));
		json_object_set_new(outputJ, "connected", json_boolean(module->outputs[i].isConnected()));

		if (i < module->outputInfos.size() && module->outputInfos[i]) {
			json_object_set_new(outputJ, "name", json_string(module->outputInfos[i]->name.c_str()));
			json_object_set_new(outputJ, "description", json_string(module->outputInfos[i]->description.c_str()));
		}

		json_array_append_new(outputsJ, outputJ);
	}
	json_object_set_new(moduleJ, "outputs", outputsJ);

	// Lights
	json_t* lightsJ = json_array();
	for (size_t i = 0; i < module->lights.size(); i++) {
		json_t* lightJ = json_object();
		json_object_set_new(lightJ, "id", json_integer(i));
		json_object_set_new(lightJ, "value", json_real(module->lights[i].getBrightness()));

		if (i < module->lightInfos.size() && module->lightInfos[i]) {
			json_object_set_new(lightJ, "name", json_string(module->lightInfos[i]->name.c_str()));
		}

		json_array_append_new(lightsJ, lightJ);
	}
	json_object_set_new(moduleJ, "lights", lightsJ);

	sendJsonResponse(client, moduleJ);
}


static void handlePostModule(socket_t client, const std::string& body) {
	if (!APP || !APP->scene || !APP->scene->rack) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	json_error_t error;
	json_t* requestJ = json_loads(body.c_str(), 0, &error);
	if (!requestJ) {
		sendErrorResponse(client, 400, string::f("Invalid JSON: %s", error.text));
		return;
	}

	json_t* pluginSlugJ = json_object_get(requestJ, "pluginSlug");
	json_t* modelSlugJ = json_object_get(requestJ, "modelSlug");

	if (!pluginSlugJ || !modelSlugJ) {
		json_decref(requestJ);
		sendErrorResponse(client, 400, "Missing pluginSlug or modelSlug");
		return;
	}

	std::string pluginSlug = json_string_value(pluginSlugJ);
	std::string modelSlug = json_string_value(modelSlugJ);

	plugin::Model* model = plugin::getModel(pluginSlug, modelSlug);
	if (!model) {
		json_decref(requestJ);
		sendErrorResponse(client, 404, "Model not found");
		return;
	}

	// Create module
	engine::Module* module = model->createModule();
	if (!module) {
		json_decref(requestJ);
		sendErrorResponse(client, 500, "Failed to create module");
		return;
	}

	// Create widget
	app::ModuleWidget* moduleWidget = model->createModuleWidget(module);
	if (!moduleWidget) {
		delete module;
		json_decref(requestJ);
		sendErrorResponse(client, 500, "Failed to create module widget");
		return;
	}

	// Set position
	json_t* posJ = json_object_get(requestJ, "pos");
	if (posJ) {
		json_t* xJ = json_object_get(posJ, "x");
		json_t* yJ = json_object_get(posJ, "y");
		if (xJ && yJ) {
			math::Vec pos(json_number_value(xJ), json_number_value(yJ));
			moduleWidget->box.pos = pos;
		}
	}

	// Add to rack
	APP->scene->rack->addModule(moduleWidget);

	json_decref(requestJ);

	// Return module info
	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "id", json_integer(module->id));
	json_object_set_new(responseJ, "pluginSlug", json_string(pluginSlug.c_str()));
	json_object_set_new(responseJ, "modelSlug", json_string(modelSlug.c_str()));
	sendJsonResponse(client, responseJ, 201);
}


static void handleDeleteModule(socket_t client, int64_t moduleId) {
	if (!APP || !APP->scene || !APP->scene->rack) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	app::ModuleWidget* moduleWidget = APP->scene->rack->getModule(moduleId);
	if (!moduleWidget) {
		sendErrorResponse(client, 404, "Module not found");
		return;
	}

	APP->scene->rack->removeModule(moduleWidget);
	delete moduleWidget;

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "success", json_boolean(true));
	sendJsonResponse(client, responseJ);
}


static void handleGetCables(socket_t client) {
	if (!APP || !APP->scene || !APP->scene->rack) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	json_t* cablesJ = json_array();

	std::vector<app::CableWidget*> cables = APP->scene->rack->getCompleteCables();
	for (app::CableWidget* cw : cables) {
		if (!cw->cable)
			continue;

		json_t* cableJ = json_object();
		json_object_set_new(cableJ, "id", json_integer(cw->cable->id));
		json_object_set_new(cableJ, "outputModuleId", json_integer(cw->cable->outputModule->id));
		json_object_set_new(cableJ, "outputId", json_integer(cw->cable->outputId));
		json_object_set_new(cableJ, "inputModuleId", json_integer(cw->cable->inputModule->id));
		json_object_set_new(cableJ, "inputId", json_integer(cw->cable->inputId));

		json_array_append_new(cablesJ, cableJ);
	}

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "cables", cablesJ);
	sendJsonResponse(client, responseJ);
}


static void handlePostCable(socket_t client, const std::string& body) {
	if (!APP || !APP->scene || !APP->scene->rack || !APP->engine) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	json_error_t error;
	json_t* requestJ = json_loads(body.c_str(), 0, &error);
	if (!requestJ) {
		sendErrorResponse(client, 400, string::f("Invalid JSON: %s", error.text));
		return;
	}

	json_t* outputModuleIdJ = json_object_get(requestJ, "outputModuleId");
	json_t* outputIdJ = json_object_get(requestJ, "outputId");
	json_t* inputModuleIdJ = json_object_get(requestJ, "inputModuleId");
	json_t* inputIdJ = json_object_get(requestJ, "inputId");

	if (!outputModuleIdJ || !outputIdJ || !inputModuleIdJ || !inputIdJ) {
		json_decref(requestJ);
		sendErrorResponse(client, 400, "Missing required fields");
		return;
	}

	int64_t outputModuleId = json_integer_value(outputModuleIdJ);
	int outputId = (int)json_integer_value(outputIdJ);
	int64_t inputModuleId = json_integer_value(inputModuleIdJ);
	int inputId = (int)json_integer_value(inputIdJ);

	json_decref(requestJ);

	// Get modules
	app::ModuleWidget* outputModuleWidget = APP->scene->rack->getModule(outputModuleId);
	app::ModuleWidget* inputModuleWidget = APP->scene->rack->getModule(inputModuleId);

	if (!outputModuleWidget || !inputModuleWidget) {
		sendErrorResponse(client, 404, "Module not found");
		return;
	}

	// Get ports
	app::PortWidget* outputPort = NULL;
	app::PortWidget* inputPort = NULL;

	for (widget::Widget* child : outputModuleWidget->children) {
		app::PortWidget* port = dynamic_cast<app::PortWidget*>(child);
		if (port && port->type == engine::Port::OUTPUT && port->portId == outputId) {
			outputPort = port;
			break;
		}
	}

	for (widget::Widget* child : inputModuleWidget->children) {
		app::PortWidget* port = dynamic_cast<app::PortWidget*>(child);
		if (port && port->type == engine::Port::INPUT && port->portId == inputId) {
			inputPort = port;
			break;
		}
	}

	if (!outputPort || !inputPort) {
		sendErrorResponse(client, 404, "Port not found");
		return;
	}

	// Create cable
	engine::Cable* cable = new engine::Cable;
	cable->inputModule = inputModuleWidget->module;
	cable->inputId = inputId;
	cable->outputModule = outputModuleWidget->module;
	cable->outputId = outputId;
	APP->engine->addCable(cable);

	// Create cable widget
	app::CableWidget* cw = new app::CableWidget;
	cw->setCable(cable);
	cw->outputPort = outputPort;
	cw->inputPort = inputPort;
	APP->scene->rack->addCable(cw);

	// Return cable info
	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "id", json_integer(cable->id));
	json_object_set_new(responseJ, "outputModuleId", json_integer(outputModuleId));
	json_object_set_new(responseJ, "outputId", json_integer(outputId));
	json_object_set_new(responseJ, "inputModuleId", json_integer(inputModuleId));
	json_object_set_new(responseJ, "inputId", json_integer(inputId));
	sendJsonResponse(client, responseJ, 201);
}


static void handleDeleteCable(socket_t client, int64_t cableId) {
	if (!APP || !APP->scene || !APP->scene->rack) {
		sendErrorResponse(client, 500, "Rack not initialized");
		return;
	}

	app::CableWidget* cw = APP->scene->rack->getCable(cableId);
	if (!cw) {
		sendErrorResponse(client, 404, "Cable not found");
		return;
	}

	APP->scene->rack->removeCable(cw);
	delete cw;

	json_t* responseJ = json_object();
	json_object_set_new(responseJ, "success", json_boolean(true));
	sendJsonResponse(client, responseJ);
}


static void handleRequest(socket_t client, const HttpRequest& request) {
	// OPTIONS request for CORS
	if (request.method == "OPTIONS") {
		HttpResponse response;
		response.headers["Access-Control-Allow-Methods"] = "GET, POST, DELETE, OPTIONS";
		response.headers["Access-Control-Allow-Headers"] = "Content-Type";
		sendResponse(client, response);
		return;
	}

	// Route requests
	if (request.method == "GET" && request.path == "/api/plugins") {
		handleGetPlugins(client);
	}
	else if (request.method == "GET" && request.path == "/api/models") {
		handleGetModels(client);
	}
	else if (request.method == "GET" && request.path == "/api/modules") {
		handleGetModules(client);
	}
	else if (request.method == "GET" && request.path.rfind("/api/modules/", 0) == 0) {
		std::string idStr = request.path.substr(13);
		int64_t moduleId = std::stoll(idStr);
		handleGetModuleDetails(client, moduleId);
	}
	else if (request.method == "POST" && request.path == "/api/modules") {
		handlePostModule(client, request.body);
	}
	else if (request.method == "DELETE" && request.path.rfind("/api/modules/", 0) == 0) {
		std::string idStr = request.path.substr(13);
		int64_t moduleId = std::stoll(idStr);
		handleDeleteModule(client, moduleId);
	}
	else if (request.method == "GET" && request.path == "/api/cables") {
		handleGetCables(client);
	}
	else if (request.method == "POST" && request.path == "/api/cables") {
		handlePostCable(client, request.body);
	}
	else if (request.method == "DELETE" && request.path.rfind("/api/cables/", 0) == 0) {
		std::string idStr = request.path.substr(12);
		int64_t cableId = std::stoll(idStr);
		handleDeleteCable(client, cableId);
	}
	else {
		sendErrorResponse(client, 404, "Endpoint not found");
	}
}


static void handleClient(socket_t client) {
	char buffer[8192];
	int bytesReceived = recv(client, buffer, sizeof(buffer) - 1, 0);

	if (bytesReceived > 0) {
		buffer[bytesReceived] = '\0';
		HttpRequest request = parseRequest(std::string(buffer));
		handleRequest(client, request);
	}

	CLOSE_SOCKET(client);
}


static void serverLoop() {
	while (running) {
		fd_set readSet;
		FD_ZERO(&readSet);
		FD_SET(serverSocket, &readSet);

		struct timeval timeout;
		timeout.tv_sec = 0;
		timeout.tv_usec = 100000; // 100ms

		int selectResult = select((int)serverSocket + 1, &readSet, NULL, NULL, &timeout);

		if (selectResult > 0 && FD_ISSET(serverSocket, &readSet)) {
			struct sockaddr_in clientAddr;
			socklen_t clientAddrLen = sizeof(clientAddr);
			socket_t client = accept(serverSocket, (struct sockaddr*)&clientAddr, &clientAddrLen);

			if (client != INVALID_SOCKET_VALUE) {
				// Handle client in a new thread
				std::thread clientThread(handleClient, client);
				clientThread.detach();
			}
		}
	}
}


void init(int port) {
	if (running) {
		WARN("HTTP API server already running");
		return;
	}

	serverPort = port;

#ifdef ARCH_WIN
	WSADATA wsaData;
	if (WSAStartup(MAKEWORD(2, 2), &wsaData) != 0) {
		WARN("Failed to initialize Winsock");
		return;
	}
#endif

	serverSocket = socket(AF_INET, SOCK_STREAM, 0);
	if (serverSocket == INVALID_SOCKET_VALUE) {
		WARN("Failed to create socket");
		return;
	}

	// Set socket options
	int opt = 1;
#ifdef ARCH_WIN
	setsockopt(serverSocket, SOL_SOCKET, SO_REUSEADDR, (char*)&opt, sizeof(opt));
#else
	setsockopt(serverSocket, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
#endif

	struct sockaddr_in serverAddr;
	serverAddr.sin_family = AF_INET;
	serverAddr.sin_addr.s_addr = INADDR_ANY;
	serverAddr.sin_port = htons(port);

	if (bind(serverSocket, (struct sockaddr*)&serverAddr, sizeof(serverAddr)) == SOCKET_ERROR_VALUE) {
		WARN("Failed to bind socket to port %d", port);
		CLOSE_SOCKET(serverSocket);
		return;
	}

	if (listen(serverSocket, 10) == SOCKET_ERROR_VALUE) {
		WARN("Failed to listen on socket");
		CLOSE_SOCKET(serverSocket);
		return;
	}

	running = true;
	serverThread = std::thread(serverLoop);

	INFO("HTTP API server started on port %d", port);
}


void destroy() {
	if (!running)
		return;

	running = false;

	if (serverThread.joinable())
		serverThread.join();

	if (serverSocket != INVALID_SOCKET_VALUE) {
		CLOSE_SOCKET(serverSocket);
		serverSocket = INVALID_SOCKET_VALUE;
	}

#ifdef ARCH_WIN
	WSACleanup();
#endif

	INFO("HTTP API server stopped");
}


bool isRunning() {
	return running;
}


int getPort() {
	return serverPort;
}


} // namespace httpapi
} // namespace rack
