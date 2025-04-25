//! Log Colors Module
//! 
//! **Description**
//! 
//! This file contains the LogColors struct and its implementation.
//! 
//! **Contents**
//! 
//! - LogColors struct
//! - Default implementation
//! - Save and load methods
//! 
use serde::{Deserialize, Serialize};
use egui::Color32;
use std::path::PathBuf;
use std::fs;

use crate::components::event_logger::serialization::color32_serde;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogColors {
    // Style-based colors
    #[serde(with = "color32_serde")]
    pub clock: Color32,
    #[serde(with = "color32_serde")]
    pub slider: Color32,
    #[serde(with = "color32_serde")]
    pub option_a: Color32,
    #[serde(with = "color32_serde")]
    pub option_b: Color32,
    #[serde(with = "color32_serde")]
    pub option_c: Color32,
    #[serde(with = "color32_serde")]
    pub time_format: Color32,
    #[serde(with = "color32_serde")]
    pub custom_event: Color32,
    #[serde(with = "color32_serde")]
    pub run_stop_log: Color32,
    
    // Severity-based colors
    #[serde(with = "color32_serde")]
    pub info_text: Color32,
    #[serde(with = "color32_serde")]
    pub warn_text: Color32,
    #[serde(with = "color32_serde")]
    pub debug_text: Color32,
    #[serde(with = "color32_serde")]
    pub error_text: Color32,
    
    // Whether to prioritize style-based colors over severity-based colors
    #[serde(default = "default_style_priority")]
    pub prioritize_style_colors: bool,
}

fn default_style_priority() -> bool {
    true
}

impl Default for LogColors {
    fn default() -> Self {
        Self {
            // Style-based colors
            clock: Color32::from_rgb(100, 200, 255),  // Light blue
            slider: Color32::from_rgb(255, 180, 100),  // Orange
            option_a: Color32::from_rgb(255, 150, 150),  // Soft red
            option_b: Color32::from_rgb(150, 255, 150),  // Soft green
            option_c: Color32::from_rgb(150, 150, 255),  // Soft blue
            time_format: Color32::from_rgb(180, 180, 180),  // Gray
            custom_event: Color32::from_rgb(255, 255, 100),  // Yellow
            run_stop_log: Color32::from_rgb(100, 200, 255),  // Light blue
            
            // Severity-based colors
            info_text: Color32::WHITE,
            warn_text: Color32::from_rgb(255, 220, 100),  // Yellow
            debug_text: Color32::from_rgb(100, 200, 255),  // Light blue
            error_text: Color32::from_rgb(255, 100, 100),  // Red
            
            // Default to prioritizing style colors
            prioritize_style_colors: true,
        }
    }
}


impl LogColors {
    #[allow(dead_code)]
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("egui_mobius_components");
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
                .join("egui_mobius_components");
            
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