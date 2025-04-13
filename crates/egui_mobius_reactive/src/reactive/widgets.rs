// src/tree.rs

use std::sync::Arc;
use egui::Ui; 

/// Base trait for rendering polymorphic reactive widgets.
/// This version is dyn-compatible.
pub trait MobiusWidget: Send + Sync + 'static {    
    
    fn render_widget(&self, ui: &mut egui::Ui)  {    
    }

    fn render_event(&self, triggered: bool, ui: &mut egui::Ui) {
        let _ = (triggered, ui);
    }
    
    fn widget_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Extension trait for attaching optional reactive behavior
pub trait MobiusWidgetReactive: MobiusWidget + Default {
    fn with_dynamic(&mut self, _state: Arc<dyn std::any::Any>) {}

}

pub trait MobiusWidgetSlot : MobiusWidget + Default {
    fn with_slot<T: Send + Sync + 'static>(&mut self, _slot: egui_mobius::slot::Slot<T>) {}
        // Default implementation does nothing
      
}

pub trait MobiusWidgetSignal : MobiusWidget + Default {

    fn with_signal<T: Send + Sync + 'static>(&mut self, _signal: egui_mobius::signals::Signal<T>) {}
        // Default implementation does nothing
    
}