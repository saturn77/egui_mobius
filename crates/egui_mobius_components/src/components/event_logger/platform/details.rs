//! Details Module
//!
//! This module provides system details that can be displayed in the UI.

/// System details collector
#[derive(Clone)]
pub struct Details {
    pub os_name: String,
    pub ip_address: String,
}

impl Details {
    /// Creates a new details collector
    pub fn new() -> Self {
        Self {
            os_name: String::new(),
            ip_address: String::new(),
        }
    }
    
    /// Gets the operating system name
    pub fn get_os(&mut self) {
        // A simple OS detection for example purposes
        #[cfg(target_os = "linux")]
        {
            self.os_name = "Linux".to_string();
        }
        
        #[cfg(target_os = "windows")]
        {
            self.os_name = "Windows".to_string();
        }
        
        #[cfg(target_os = "macos")]
        {
            self.os_name = "macOS".to_string();
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            self.os_name = "Unknown OS".to_string();
        }
    }
    
    /// Gets the IP address
    pub fn get_ip(&mut self) {
        // Simplified example - would use network APIs in a real implementation
        self.ip_address = "127.0.0.1".to_string();
    }
    
    /// Formats system details for display
    pub fn format_os(&self) -> String {
        format!(
            "System Information\n\
            ╔══════════════════════════════════════╗\n\
            ║ OS:        {}                       ║\n\
            ║ IP:        {}                     ║\n\
            ╚══════════════════════════════════════╝",
            self.os_name,
            self.ip_address
        )
    }
}

impl Default for Details {
    fn default() -> Self {
        let mut details = Self::new();
        details.get_os();
        details.get_ip();
        details
    }
}