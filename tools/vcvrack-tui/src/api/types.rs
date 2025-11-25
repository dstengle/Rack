//! API response and request types matching VCV Rack HTTP API

use serde::{Deserialize, Serialize};

/// Position in the rack
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Size of a module
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Size {
    pub x: f64,
    pub y: f64,
}

/// Plugin information from /api/plugins
#[derive(Debug, Clone, Deserialize)]
pub struct Plugin {
    pub slug: String,
    pub name: String,
    pub brand: String,
    pub version: String,
    pub author: String,
    pub models: Vec<PluginModel>,
}

/// Model within a plugin
#[derive(Debug, Clone, Deserialize)]
pub struct PluginModel {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// Response from /api/plugins
#[derive(Debug, Clone, Deserialize)]
pub struct PluginsResponse {
    pub plugins: Vec<Plugin>,
}

/// Available module model from /api/models
#[derive(Debug, Clone, Deserialize)]
pub struct AvailableModel {
    #[serde(rename = "pluginSlug")]
    pub plugin_slug: String,
    #[serde(rename = "pluginName")]
    pub plugin_name: String,
    pub slug: String,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(default)]
    pub description: String,
}

/// Response from /api/models
#[derive(Debug, Clone, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<AvailableModel>,
}

/// Module in the patch from /api/modules
#[derive(Debug, Clone, Deserialize)]
pub struct PatchModule {
    pub id: i64,
    #[serde(rename = "pluginSlug")]
    pub plugin_slug: String,
    #[serde(rename = "modelSlug")]
    pub model_slug: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(default)]
    pub pos: Position,
    #[serde(default)]
    pub size: Size,
}

/// Response from /api/modules
#[derive(Debug, Clone, Deserialize)]
pub struct ModulesResponse {
    pub modules: Vec<PatchModule>,
}

/// Parameter information from module details
#[derive(Debug, Clone, Deserialize)]
pub struct Param {
    pub id: i64,
    pub value: f64,
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub unit: String,
    #[serde(rename = "minValue", default)]
    pub min_value: Option<f64>,
    #[serde(rename = "maxValue", default)]
    pub max_value: Option<f64>,
    #[serde(rename = "defaultValue", default)]
    pub default_value: Option<f64>,
    #[serde(rename = "displayValue", default)]
    pub display_value: String,
}

/// Port (input or output) information
#[derive(Debug, Clone, Deserialize)]
pub struct Port {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub channels: i32,
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub description: String,
}

/// Light information from module details
#[derive(Debug, Clone, Deserialize)]
pub struct Light {
    pub id: i64,
    pub value: f64,
    #[serde(default)]
    pub name: String,
}

/// Detailed module information from /api/modules/:id
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleDetails {
    pub id: i64,
    #[serde(rename = "pluginSlug")]
    pub plugin_slug: String,
    #[serde(rename = "modelSlug")]
    pub model_slug: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub inputs: Vec<Port>,
    #[serde(default)]
    pub outputs: Vec<Port>,
    #[serde(default)]
    pub lights: Vec<Light>,
}

/// Cable in the patch from /api/cables
#[derive(Debug, Clone, Deserialize)]
pub struct Cable {
    pub id: i64,
    #[serde(rename = "outputModuleId")]
    pub output_module_id: i64,
    #[serde(rename = "outputId")]
    pub output_id: i64,
    #[serde(rename = "inputModuleId")]
    pub input_module_id: i64,
    #[serde(rename = "inputId")]
    pub input_id: i64,
}

/// Response from /api/cables
#[derive(Debug, Clone, Deserialize)]
pub struct CablesResponse {
    pub cables: Vec<Cable>,
}

/// Request body for POST /api/modules
#[derive(Debug, Clone, Serialize)]
pub struct CreateModuleRequest {
    #[serde(rename = "pluginSlug")]
    pub plugin_slug: String,
    #[serde(rename = "modelSlug")]
    pub model_slug: String,
    pub pos: Position,
}

/// Response from POST /api/modules
#[derive(Debug, Clone, Deserialize)]
pub struct CreateModuleResponse {
    pub id: i64,
    #[serde(rename = "pluginSlug")]
    pub plugin_slug: String,
    #[serde(rename = "modelSlug")]
    pub model_slug: String,
}

/// Request body for POST /api/cables
#[derive(Debug, Clone, Serialize)]
pub struct CreateCableRequest {
    #[serde(rename = "outputModuleId")]
    pub output_module_id: i64,
    #[serde(rename = "outputId")]
    pub output_id: i64,
    #[serde(rename = "inputModuleId")]
    pub input_module_id: i64,
    #[serde(rename = "inputId")]
    pub input_id: i64,
}

/// Response from POST /api/cables
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCableResponse {
    pub id: i64,
    #[serde(rename = "outputModuleId")]
    pub output_module_id: i64,
    #[serde(rename = "outputId")]
    pub output_id: i64,
    #[serde(rename = "inputModuleId")]
    pub input_module_id: i64,
    #[serde(rename = "inputId")]
    pub input_id: i64,
}

/// Generic success response
#[derive(Debug, Clone, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/// Error response from the API
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
