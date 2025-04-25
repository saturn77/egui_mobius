//! Banner Module
//!
//! This module provides a banner component that can be displayed at startup.

/// Simple banner with version and description
#[derive(Clone)]
pub struct Banner {
    pub message: String,
}

impl Banner {
    /// Creates a new banner with an empty message
    pub fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
    
    /// Formats the banner with version and build information
    pub fn format(&mut self) {
        self.message = format!(
            "**** Welcome to Arrakis Serial Runtime, Version {}.{}.{}\n**** Today is {} {}:{}:{}", 
            0, 1, 0,
            "04-25-2025", 16, 41, 3
        );
    }
}

impl Default for Banner {
    fn default() -> Self {
        let mut banner = Self::new();
        banner.format();
        banner
    }
}