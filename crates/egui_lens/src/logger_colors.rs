use eframe::egui::Color32;
use std::path::PathBuf;
use std::fs;

use std::collections::HashMap;

/// LogColors configures the colors for different log types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LogColors {
    // Standard log levels - LEVEL COLORS
    #[serde(with = "color32_serde")]
    pub info_level: Color32,
    #[serde(with = "color32_serde")]
    pub warning_level: Color32,
    #[serde(with = "color32_serde")]
    pub error_level: Color32,
    #[serde(with = "color32_serde")]
    pub debug_level: Color32,
    
    // Standard log levels - MESSAGE COLORS
    #[serde(with = "color32_serde")]
    pub info_message: Color32,
    #[serde(with = "color32_serde")]
    pub warning_message: Color32,
    #[serde(with = "color32_serde")]
    pub error_message: Color32,
    #[serde(with = "color32_serde")]
    pub debug_message: Color32,
    
    // For backward compatibility - deprecated, will be removed in future versions
    #[serde(with = "color32_serde")]
    pub info: Color32,
    #[serde(with = "color32_serde")]
    pub warning: Color32,
    #[serde(with = "color32_serde")]
    pub error: Color32,
    #[serde(with = "color32_serde")]
    pub debug: Color32,
    
    // Special message types
    #[serde(with = "color32_serde")]
    pub timestamp: Color32,
    #[serde(with = "color32_serde")]
    pub system: Color32,
    #[serde(with = "color32_serde")]
    pub user_action: Color32,
    #[serde(with = "color32_serde")]
    pub config: Color32,
    #[serde(with = "color32_serde")]
    pub status: Color32,
    #[serde(with = "color32_serde")]
    pub progress: Color32,
    #[serde(with = "color32_serde")]
    pub success: Color32,
    #[serde(with = "color32_serde")]
    pub default: Color32,
    
    // Flexible custom colors - map from identifier string to color
    #[serde(default)]
    pub custom_colors: HashMap<String, Color32Wrapper>,
}

/// Wrapper for Color32 to support serde with the HashMap
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Color32Wrapper {
    #[serde(with = "color32_serde")]
    pub level_color: Color32,
    
    #[serde(with = "color32_serde")]
    pub message_color: Color32,
}

impl Default for Color32Wrapper {
    fn default() -> Self {
        Self {
            level_color: Color32::from_rgb(200, 200, 200), // Default to light gray for level
            message_color: Color32::from_rgb(255, 255, 255), // Default to white for message
        }
    }
}

impl Default for LogColors {
    fn default() -> Self {
        let mut custom_colors = HashMap::new();
        
        // Add some default custom colors for backward compatibility
        custom_colors.insert("custom1".to_string(), Color32Wrapper { 
            level_color: Color32::from_rgb(255, 200, 200),  // Light red for level
            message_color: Color32::from_rgb(255, 220, 220)   // Lighter red for message
        });
        custom_colors.insert("custom2".to_string(), Color32Wrapper { 
            level_color: Color32::from_rgb(200, 255, 200),  // Light green for level
            message_color: Color32::from_rgb(220, 255, 220)   // Lighter green for message
        });
        custom_colors.insert("custom3".to_string(), Color32Wrapper { 
            level_color: Color32::from_rgb(200, 200, 255),  // Light blue for level
            message_color: Color32::from_rgb(220, 220, 255)   // Lighter blue for message
        });
        
        // Define the standard colors
        let info_level = Color32::from_rgb(150, 255, 150);       // Green
        let warning_level = Color32::from_rgb(255, 255, 100);    // Yellow
        let error_level = Color32::from_rgb(255, 100, 100);      // Red
        let debug_level = Color32::from_rgb(150, 150, 255);      // Blue
        
        // Message colors - slightly lighter versions of the level colors
        let info_message = Color32::from_rgb(180, 255, 180);      // Lighter green
        let warning_message = Color32::from_rgb(255, 255, 140);   // Lighter yellow
        let error_message = Color32::from_rgb(255, 140, 140);     // Lighter red
        let debug_message = Color32::from_rgb(180, 180, 255);     // Lighter blue
        
        Self {
            // Level colors
            info_level,
            warning_level,
            error_level,
            debug_level,
            
            // Message colors
            info_message,
            warning_message,
            error_message,
            debug_message,
            
            // Legacy fields (same as level colors for backward compatibility)
            info: info_level,
            warning: warning_level,
            error: error_level,
            debug: debug_level,
            
            // Special message types
            timestamp: Color32::from_rgb(180, 180, 180), // Gray
            system: Color32::from_rgb(100, 200, 255),    // Light blue
            user_action: Color32::from_rgb(255, 180, 100), // Orange
            config: Color32::from_rgb(200, 150, 255),    // Purple
            status: Color32::from_rgb(200, 200, 200),    // Light gray
            progress: Color32::from_rgb(100, 255, 200),  // Cyan
            success: Color32::from_rgb(100, 255, 100),   // Bright green
            default: Color32::from_rgb(255, 255, 255),   // White
            
            // Custom colors via HashMap
            custom_colors,
        }
    }
}

// Module for serializing and deserializing Color32
/// Helper methods for custom log types colors
impl LogColors {
    /// Get the level color for a custom log type
    pub fn get_custom_color_level(&self, identifier: &str) -> Color32 {
        if let Some(wrapper) = self.custom_colors.get(identifier) {
            wrapper.level_color
        } else {
            // Return default color if the custom type is not found
            self.default
        }
    }
    
    /// Get the message color for a custom log type
    pub fn get_custom_color_message(&self, identifier: &str) -> Color32 {
        if let Some(wrapper) = self.custom_colors.get(identifier) {
            wrapper.message_color
        } else {
            // Return default color if the custom type is not found
            self.default
        }
    }
    
    /// Get a color for a custom log type (legacy support - returns level color)
    pub fn get_custom_color(&self, identifier: &str) -> Color32 {
        self.get_custom_color_level(identifier)
    }
    
    /// Add or update a custom color with the same color for level and message
    pub fn set_custom_color(&mut self, identifier: &str, color: Color32) {
        self.custom_colors.insert(identifier.to_string(), Color32Wrapper { 
            level_color: color,
            message_color: color 
        });
    }
    
    /// Add or update a custom color with different colors for level and message
    pub fn set_custom_colors(&mut self, identifier: &str, level_color: Color32, message_color: Color32) {
        self.custom_colors.insert(identifier.to_string(), Color32Wrapper { 
            level_color,
            message_color 
        });
    }
}

pub mod color32_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use eframe::egui::Color32;

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        rgba.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rgba = <[u8; 4]>::deserialize(deserializer)?;
        Ok(Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]))
    }
}

impl LogColors {
    #[allow(dead_code)]
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("egui_mobius_template");
        let config_path = config_dir.join("log_colors.json");
        
        println!("Loading colors from: {}", config_path.display());
        
        match fs::read_to_string(&config_path) {
            Ok(file_content) => {
                match serde_json::from_str(&file_content) {
                    Ok(colors) => {
                        println!("Successfully loaded colors from file");
                        colors
                    }
                    Err(e) => {
                        eprintln!("Failed to parse colors JSON: {}", e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read colors file: {}", e);
                Self::default()
            }
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        let colors = self.clone();
        std::thread::spawn(move || {
            // Get config directory path
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("egui_mobius_template");
            
            // Create config directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
            
            // Create config file path
            let config_path = config_dir.join("log_colors.json");
            
            // Serialize colors to JSON
            match serde_json::to_string_pretty(&colors) {
                Ok(json) => {
                    // Write JSON to file
                    if let Err(e) = fs::write(&config_path, json) {
                        eprintln!("Failed to write colors to {}: {}", config_path.display(), e);
                    } else {
                        println!("Successfully saved colors to {}", config_path.display());
                    }
                },
                Err(e) => eprintln!("Failed to serialize colors: {}", e),
            }
        });
    }
}