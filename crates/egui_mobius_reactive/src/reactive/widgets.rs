//! ReactiveWidgets – retained-style reactive Widgets for immediate-mode UI
use std::ops::RangeInclusive;
use crate::reactive::reactive_state::ReactiveWidgetRef;
use egui::Ui; 
use crate::reactive::dynamic::Dynamic; 


pub struct ReactiveSlider<'a, T> {
    value: &'a Dynamic<T>,
    range: RangeInclusive<f64>,
    display_value: bool,
    logarithmic: bool,
    text: Option<String>,
    // more configuration options...
}

impl<'a, T: Send + Sync + Clone + Into<f64> + From<f64> + std::fmt::Display + 'static> ReactiveSlider<'a, T> {
    pub fn new(value: &'a Dynamic<T>) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            display_value: false,
            logarithmic: false,
            text: None,
        }
    }
    
    pub fn with_range(mut self, range: RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }
    
    pub fn with_display_value(mut self, display: bool) -> Self {
        self.display_value = display;
        self
    }
    
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    
    pub fn with_logarithmic(mut self, logarithmic: bool) -> Self {
        self.logarithmic = logarithmic;
        self
    }
    
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        // Create the widget reference (avoids double Arc)
        let mut widget_ref = ReactiveWidgetRef::from_dynamic(self.value);
        
        // Refresh cached value if needed
        if widget_ref.cached_value.is_none() {
            if let Some(arc) = widget_ref.weak_ref.upgrade() {
                let guard = arc.lock().unwrap();
                widget_ref.cached_value = Some((*guard).clone());
            }
        }
        
        // If we don't have a cached value and couldn't get one, show a placeholder
        if widget_ref.cached_value.is_none() {
            // Return the Response directly, not a field from it
            return ui.label("Value no longer available");
        }
        
        // Build a slider with our configuration
        let mut slider_value = widget_ref.cached_value.as_ref().unwrap().clone().into();
        let mut slider = egui::Slider::new(&mut slider_value, self.range);
        
        if let Some(text) = &self.text {
            slider = slider.text(text);
        }
        
        slider = slider.show_value(self.display_value);
        
        if self.logarithmic {
            slider = slider.logarithmic(true);
        }
        
        // Add the slider and handle response
        let response = ui.add(slider);
        
        if response.changed() {
            // Update our cached value 
            if let Some(value) = &mut widget_ref.cached_value {
                *value = T::from(slider_value);
                widget_ref.modified = true;
                
                // Update the source
                if let Some(arc) = widget_ref.weak_ref.upgrade() {
                    let mut guard = arc.lock().unwrap();
                    *guard = value.clone();
                }
            }
        }
        
        response
    }
}

