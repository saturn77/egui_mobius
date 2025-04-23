//! ReactiveWidgetState – retained-style local state for immediate-mode UI

use std::sync::{Arc, Mutex, Weak};
use std::ops::RangeInclusive;
use crate::reactive::dynamic::Dynamic; 

/// A lightweight reference to a reactive value optimized for use in widgets
pub struct ReactiveWidgetRef<T> {
    // Weak reference to avoid double Arc wrapping
    pub weak_ref: Weak<Mutex<T>>,
    // Option to cache the last known value to reduce mutex locks
    pub cached_value: Option<T>,
    // Track if this widget has modified the value
    pub modified: bool,
}

impl<T: Clone + 'static + Send + Sync> ReactiveWidgetRef<T> {
    pub fn from_dynamic(dynamic: &Dynamic<T>) -> Self {
        Self {
            weak_ref: Arc::downgrade(&dynamic.inner),
            cached_value: None,
            modified: false,
        }
    }
    
    // pub fn from_derived(derived: &Derived<T>) -> Self {
    //     Self {
    //         weak_ref: Arc::downgrade(&derived.get()), // Assuming `get_inner` is the correct method to access the inner Arc<Mutex<T>>
    //         cached_value: None,
    //         modified: false,
    //     }
    // }
    
    // Helper to use in widgets, returns true if value was changed
    pub fn ui_slider(&mut self, ui: &mut egui::Ui, range: RangeInclusive<f64>) -> bool 
    where T: Into<f64> + From<f64> + std::fmt::Display
    {
        // Refresh cached value if needed
        if self.cached_value.is_none() {
            if let Some(arc) = self.weak_ref.upgrade() {
                let guard = arc.lock().unwrap();
                self.cached_value = Some((*guard).clone());
            }
        }
        
        if let Some(ref mut value) = self.cached_value {
            let mut float_val: f64 = (*value).clone().into();
            if ui.add(egui::Slider::new(&mut float_val, range)).changed() {
                *value = T::from(float_val);
                self.modified = true;
                
                // Also update the source if possible
                if let Some(arc) = self.weak_ref.upgrade() {
                    let mut guard = arc.lock().unwrap();
                    *guard = value.clone();
                }
                return true;
            }
        } else {
            ui.label("Value no longer available");
        }
        false
    }
}