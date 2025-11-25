//! Module state management and naming

use std::collections::HashMap;

use crate::api::{AvailableModel, ModuleDetails, PatchModule, Port};

/// A module in our local state with friendly name
#[derive(Debug, Clone)]
pub struct LocalModule {
    /// Unique ID from VCV Rack
    pub id: i64,
    /// Human-friendly name (e.g., "VCO-1-1")
    pub friendly_name: String,
    /// Plugin slug
    pub plugin_slug: String,
    /// Model slug
    pub model_slug: String,
    /// Model display name
    pub model_name: String,
    /// Full name including brand
    pub full_name: String,
    /// Input ports (populated from details)
    pub inputs: Vec<Port>,
    /// Output ports (populated from details)
    pub outputs: Vec<Port>,
    /// X position
    pub pos_x: f64,
    /// Y position
    pub pos_y: f64,
}

impl LocalModule {
    /// Get a summary for display in the modules list
    pub fn summary(&self) -> String {
        format!(
            "{:<12} {:<24} [in:{} out:{}]",
            self.friendly_name,
            self.full_name,
            self.inputs.len(),
            self.outputs.len()
        )
    }

    /// Find an input port by name (case-insensitive partial match)
    pub fn find_input(&self, name: &str) -> Option<&Port> {
        let name_lower = name.to_lowercase();
        // Try exact match first
        if let Some(port) = self.inputs.iter().find(|p| p.name.to_lowercase() == name_lower) {
            return Some(port);
        }
        // Try prefix match
        self.inputs
            .iter()
            .find(|p| p.name.to_lowercase().starts_with(&name_lower))
    }

    /// Find an output port by name (case-insensitive partial match)
    pub fn find_output(&self, name: &str) -> Option<&Port> {
        let name_lower = name.to_lowercase();
        // Try exact match first
        if let Some(port) = self.outputs.iter().find(|p| p.name.to_lowercase() == name_lower) {
            return Some(port);
        }
        // Try prefix match
        self.outputs
            .iter()
            .find(|p| p.name.to_lowercase().starts_with(&name_lower))
    }
}

/// Manages module state and naming
#[derive(Debug, Default)]
pub struct ModuleManager {
    /// Map from friendly name to module ID
    name_to_id: HashMap<String, i64>,
    /// Map from module ID to friendly name
    id_to_name: HashMap<i64, String>,
    /// Map from module ID to full module info
    modules: HashMap<i64, LocalModule>,
    /// Instance counters for generating unique names
    name_counters: HashMap<String, u32>,
    /// Cached available models
    available_models: Vec<AvailableModel>,
}

impl ModuleManager {
    /// Create a new module manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Update available models cache
    pub fn set_available_models(&mut self, models: Vec<AvailableModel>) {
        self.available_models = models;
    }

    /// Get available models
    pub fn available_models(&self) -> &[AvailableModel] {
        &self.available_models
    }

    /// Find an available model by search string (fuzzy)
    pub fn find_available_model(&self, search: &str) -> Option<&AvailableModel> {
        let search_lower = search.to_lowercase();

        // Try exact full name match
        if let Some(m) = self
            .available_models
            .iter()
            .find(|m| m.full_name.to_lowercase() == search_lower)
        {
            return Some(m);
        }

        // Try exact slug match
        if let Some(m) = self
            .available_models
            .iter()
            .find(|m| m.slug.to_lowercase() == search_lower)
        {
            return Some(m);
        }

        // Try prefix match on full name
        if let Some(m) = self
            .available_models
            .iter()
            .find(|m| m.full_name.to_lowercase().starts_with(&search_lower))
        {
            return Some(m);
        }

        // Try contains match
        self.available_models
            .iter()
            .find(|m| m.full_name.to_lowercase().contains(&search_lower))
    }

    /// Search available models with fuzzy matching
    pub fn search_available_models(&self, query: &str, max_results: usize) -> Vec<&AvailableModel> {
        if query.is_empty() {
            return self.available_models.iter().take(max_results).collect();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&AvailableModel, i32)> = self
            .available_models
            .iter()
            .filter_map(|m| {
                let full_name_lower = m.full_name.to_lowercase();
                let name_lower = m.name.to_lowercase();
                let plugin_lower = m.plugin_name.to_lowercase();

                // Calculate a simple relevance score
                let score = if full_name_lower == query_lower {
                    100 // Exact match
                } else if full_name_lower.starts_with(&query_lower) {
                    90 // Prefix match
                } else if name_lower.starts_with(&query_lower) {
                    80 // Model name prefix
                } else if full_name_lower.contains(&query_lower) {
                    70 // Contains
                } else if name_lower.contains(&query_lower) {
                    60 // Model name contains
                } else if plugin_lower.contains(&query_lower) {
                    50 // Plugin name contains
                } else {
                    // Check if all characters appear in order (fuzzy)
                    let mut chars = query_lower.chars().peekable();
                    for c in full_name_lower.chars() {
                        if chars.peek() == Some(&c) {
                            chars.next();
                        }
                    }
                    if chars.peek().is_none() {
                        40 // Fuzzy match
                    } else {
                        0
                    }
                };

                if score > 0 {
                    Some((m, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results
            .into_iter()
            .take(max_results)
            .map(|(m, _)| m)
            .collect()
    }

    /// Generate a friendly name for a new module
    fn generate_friendly_name(&mut self, model_name: &str) -> String {
        let counter = self.name_counters.entry(model_name.to_string()).or_insert(0);
        *counter += 1;
        format!("{}-{}", model_name, counter)
    }

    /// Sync modules from server response
    pub fn sync_modules(&mut self, patch_modules: Vec<PatchModule>) {
        // Find modules that were removed
        let current_ids: std::collections::HashSet<i64> =
            patch_modules.iter().map(|m| m.id).collect();
        let removed_ids: Vec<i64> = self
            .modules
            .keys()
            .filter(|id| !current_ids.contains(id))
            .copied()
            .collect();

        // Remove deleted modules
        for id in removed_ids {
            if let Some(name) = self.id_to_name.remove(&id) {
                self.name_to_id.remove(&name);
            }
            self.modules.remove(&id);
        }

        // Add new modules
        for pm in patch_modules {
            if !self.modules.contains_key(&pm.id) {
                let friendly_name = self.generate_friendly_name(&pm.model_name);
                self.name_to_id.insert(friendly_name.clone(), pm.id);
                self.id_to_name.insert(pm.id, friendly_name.clone());

                // Find full name from available models
                let full_name = self
                    .available_models
                    .iter()
                    .find(|m| m.plugin_slug == pm.plugin_slug && m.slug == pm.model_slug)
                    .map(|m| m.full_name.clone())
                    .unwrap_or_else(|| format!("{} {}", pm.plugin_slug, pm.model_name));

                let module = LocalModule {
                    id: pm.id,
                    friendly_name,
                    plugin_slug: pm.plugin_slug,
                    model_slug: pm.model_slug,
                    model_name: pm.model_name,
                    full_name,
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    pos_x: pm.pos.x,
                    pos_y: pm.pos.y,
                };
                self.modules.insert(pm.id, module);
            }
        }
    }

    /// Update a module's port information from details
    pub fn update_module_details(&mut self, details: ModuleDetails) {
        if let Some(module) = self.modules.get_mut(&details.id) {
            module.inputs = details.inputs;
            module.outputs = details.outputs;
        }
    }

    /// Add a newly created module
    pub fn add_module(&mut self, id: i64, plugin_slug: &str, model_slug: &str) {
        // Find the model info
        let model = self
            .available_models
            .iter()
            .find(|m| m.plugin_slug == plugin_slug && m.slug == model_slug);

        let model_name = model.map(|m| m.name.clone()).unwrap_or_else(|| model_slug.to_string());
        let full_name = model
            .map(|m| m.full_name.clone())
            .unwrap_or_else(|| format!("{} {}", plugin_slug, model_slug));

        let friendly_name = self.generate_friendly_name(&model_name);
        self.name_to_id.insert(friendly_name.clone(), id);
        self.id_to_name.insert(id, friendly_name.clone());

        let module = LocalModule {
            id,
            friendly_name,
            plugin_slug: plugin_slug.to_string(),
            model_slug: model_slug.to_string(),
            model_name,
            full_name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            pos_x: 0.0,
            pos_y: 0.0,
        };
        self.modules.insert(id, module);
    }

    /// Remove a module by ID
    pub fn remove_module(&mut self, id: i64) {
        if let Some(name) = self.id_to_name.remove(&id) {
            self.name_to_id.remove(&name);
        }
        self.modules.remove(&id);
    }

    /// Get a module by friendly name
    pub fn get_by_name(&self, name: &str) -> Option<&LocalModule> {
        let name_lower = name.to_lowercase();
        // Try exact match first
        if let Some(id) = self.name_to_id.get(name) {
            return self.modules.get(id);
        }
        // Try case-insensitive match
        for (key, id) in &self.name_to_id {
            if key.to_lowercase() == name_lower {
                return self.modules.get(id);
            }
        }
        // Try prefix match
        for (key, id) in &self.name_to_id {
            if key.to_lowercase().starts_with(&name_lower) {
                return self.modules.get(id);
            }
        }
        None
    }

    /// Get a module by ID
    pub fn get_by_id(&self, id: i64) -> Option<&LocalModule> {
        self.modules.get(&id)
    }

    /// Get all modules sorted by position
    pub fn all_modules(&self) -> Vec<&LocalModule> {
        let mut modules: Vec<&LocalModule> = self.modules.values().collect();
        modules.sort_by(|a, b| {
            a.pos_x
                .partial_cmp(&b.pos_x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        modules
    }

    /// Get module count
    pub fn count(&self) -> usize {
        self.modules.len()
    }

    /// Get the next X position for a new module
    pub fn next_x_position(&self, spacing: f64) -> f64 {
        self.modules
            .values()
            .map(|m| m.pos_x)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|x| x + spacing)
            .unwrap_or(0.0)
    }

    /// Search modules by partial name match
    pub fn search_modules(&self, query: &str) -> Vec<&LocalModule> {
        if query.is_empty() {
            return self.all_modules();
        }

        let query_lower = query.to_lowercase();
        self.modules
            .values()
            .filter(|m| {
                m.friendly_name.to_lowercase().contains(&query_lower)
                    || m.model_name.to_lowercase().contains(&query_lower)
                    || m.full_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Clear all state (for refresh)
    pub fn clear(&mut self) {
        self.name_to_id.clear();
        self.id_to_name.clear();
        self.modules.clear();
        self.name_counters.clear();
    }
}
