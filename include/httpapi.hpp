#pragma once
#include <common.hpp>
#include <jansson.h>


namespace rack {
/** HTTP API server for remote control of VCV Rack */
namespace httpapi {


/** Starts the HTTP API server on the specified port.
Runs in a separate thread.
*/
void init(int port = 8080);

/** Stops the HTTP API server and cleans up resources.
*/
void destroy();

/** Returns true if the HTTP API server is running.
*/
bool isRunning();

/** Returns the port the HTTP API server is listening on.
*/
int getPort();


} // namespace httpapi
} // namespace rack
