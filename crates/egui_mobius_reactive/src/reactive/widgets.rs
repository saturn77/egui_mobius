//! MobiusWidget traits for creating customizable, modular widgets
//!
//! This module defines the `MobiusWidget` trait and its extensions for creating
//! customizable, modular widgets in a variety of styles : 
//! // - `MobiusWidget`
//! // - `MobiusWidgetReactive`
//! // - `MobiusWidgetSlot`
//! // - `MobiusWidgetSignal`
//!
//! The `MobiusWidget` trait is the base trait for all widgets, providing a
//! default implementation for rendering the widget and handling events.
//! 
//! The `MobiusWidgetReactive` trait extends the base trait to allow for
//! attaching optional reactive behavior.
//! 
//! The `MobiusWidgetSlot` and `MobiusWidgetSignal` traits extend the base trait
//! to allow for attaching slots and signals, respectively.
//!
//! The `MobiusWidget` trait is designed to be used with the `egui` library,
//! providing a simple and flexible way to create and manage widgets.

use std::sync::Arc;


/// Base trait for rendering polymorphic reactive widgets.
/// This version is dyn-compatible.
pub trait MobiusWidget: Send + Sync + 'static {    
    
    fn render_widget(&self, _ui: &mut egui::Ui)  {    
    }

    fn render_event(&self, triggered: bool, ui: &mut egui::Ui) {
        let _ = (triggered, ui);
    }
    
    fn widget_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// **MobiusWidgetReactive** Extension trait for attaching Dynamic<T> to widgets
/// 
/// This trait is used to attach optional reactive behavior to widgets,
/// allowing for this widget to receive data from other widgets or systems.
/// This is useful for creating widgets that can be updated dynamically
/// based on changes in the application state, facilitating modular and
/// customizable designs.
///
/// The `with_dynamic` method allows for attaching a dynamic state to the widget,
/// enabling it to react to changes in the state. The state is passed as an
/// `Arc<dyn std::any::Any>`, allowing for dynamic typing and flexibility.
/// The default implementation does nothing, allowing for widgets to be created
/// without any reactive behavior if desired.
///
pub trait MobiusWidgetReactive: MobiusWidget + Default {
    fn with_dynamic(&mut self, _state: Arc<dyn std::any::Any>) {}

}

/// **MobiusWidgetSlot** Extension trait for attaching slots to widgets
///
/// This trait is used to attach slots to widgets, allowing for
/// this widget to receive data from other widgets or systems.
pub trait MobiusWidgetSlot : MobiusWidget + Default {
    fn with_slot<T: Send + Sync + 'static>(&mut self, _slot: egui_mobius::slot::Slot<T>) {}
        // Default implementation does nothing
}

/// **MobiusWidgetSignal** Extension trait for attaching signals to widgets
/// 
/// This trait is used to attach signals to widgets, allowing for
/// sending data from this widget to other widgets or systems.
pub trait MobiusWidgetSignal : MobiusWidget + Default {

    fn with_signal<T: Send + Sync + 'static>(&mut self, _signal: egui_mobius::signals::Signal<T>) {}
        // Default implementation does nothing
    
}