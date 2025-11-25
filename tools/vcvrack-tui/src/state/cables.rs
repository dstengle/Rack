//! Cable state management

use std::collections::HashMap;

use crate::api::Cable;
use crate::state::modules::ModuleManager;

/// A cable with resolved names for display
#[derive(Debug, Clone)]
pub struct DisplayCable {
    /// Cable ID from VCV Rack
    pub id: i64,
    /// Source module ID
    pub output_module_id: i64,
    /// Source module friendly name
    pub source_module: String,
    /// Source port name
    pub source_port: String,
    /// Source port ID
    pub output_id: i64,
    /// Target module ID
    pub input_module_id: i64,
    /// Target module friendly name
    pub target_module: String,
    /// Target port name
    pub target_port: String,
    /// Target port ID
    pub input_id: i64,
}

impl DisplayCable {
    /// Format for display in cable list
    pub fn display(&self) -> String {
        format!(
            "{}:{} -> {}:{}",
            self.source_module, self.source_port, self.target_module, self.target_port
        )
    }
}

/// Manages cable state
#[derive(Debug, Default)]
pub struct CableManager {
    /// All cables indexed by ID
    cables: HashMap<i64, DisplayCable>,
}

impl CableManager {
    /// Create a new cable manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Sync cables from server response, resolving names using module manager
    pub fn sync_cables(&mut self, api_cables: Vec<Cable>, modules: &ModuleManager) {
        self.cables.clear();

        for cable in api_cables {
            if let Some(display_cable) = Self::resolve_cable(&cable, modules) {
                self.cables.insert(cable.id, display_cable);
            }
        }
    }

    /// Resolve a cable's module and port names
    fn resolve_cable(cable: &Cable, modules: &ModuleManager) -> Option<DisplayCable> {
        let source_module = modules.get_by_id(cable.output_module_id)?;
        let target_module = modules.get_by_id(cable.input_module_id)?;

        let source_port = source_module
            .outputs
            .iter()
            .find(|p| p.id == cable.output_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Output {}", cable.output_id));

        let target_port = target_module
            .inputs
            .iter()
            .find(|p| p.id == cable.input_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Input {}", cable.input_id));

        Some(DisplayCable {
            id: cable.id,
            output_module_id: cable.output_module_id,
            source_module: source_module.friendly_name.clone(),
            source_port,
            output_id: cable.output_id,
            input_module_id: cable.input_module_id,
            target_module: target_module.friendly_name.clone(),
            target_port,
            input_id: cable.input_id,
        })
    }

    /// Add a newly created cable
    pub fn add_cable(&mut self, cable: &Cable, modules: &ModuleManager) {
        if let Some(display_cable) = Self::resolve_cable(cable, modules) {
            self.cables.insert(cable.id, display_cable);
        }
    }

    /// Remove a cable by ID
    pub fn remove_cable(&mut self, id: i64) {
        self.cables.remove(&id);
    }

    /// Get all cables
    pub fn all_cables(&self) -> Vec<&DisplayCable> {
        let mut cables: Vec<&DisplayCable> = self.cables.values().collect();
        // Sort by source module, then source port
        cables.sort_by(|a, b| {
            (&a.source_module, &a.source_port).cmp(&(&b.source_module, &b.source_port))
        });
        cables
    }

    /// Get cable count
    pub fn count(&self) -> usize {
        self.cables.len()
    }

    /// Find a cable by source and target
    pub fn find_cable(
        &self,
        source_module: &str,
        source_port: &str,
        target_module: &str,
        target_port: &str,
    ) -> Option<&DisplayCable> {
        let source_module_lower = source_module.to_lowercase();
        let source_port_lower = source_port.to_lowercase();
        let target_module_lower = target_module.to_lowercase();
        let target_port_lower = target_port.to_lowercase();

        self.cables.values().find(|c| {
            c.source_module.to_lowercase() == source_module_lower
                && c.source_port.to_lowercase().contains(&source_port_lower)
                && c.target_module.to_lowercase() == target_module_lower
                && c.target_port.to_lowercase().contains(&target_port_lower)
        })
    }

    /// Get cables connected to a specific module
    pub fn cables_for_module(&self, module_name: &str) -> Vec<&DisplayCable> {
        let name_lower = module_name.to_lowercase();
        self.cables
            .values()
            .filter(|c| {
                c.source_module.to_lowercase() == name_lower
                    || c.target_module.to_lowercase() == name_lower
            })
            .collect()
    }

    /// Clear all cables
    pub fn clear(&mut self) {
        self.cables.clear();
    }
}
