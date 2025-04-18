use egui_mobius_derive::MobiusWidgetReactive;
use egui_mobius_reactive::*;
//use egui_mobius::prelude::*; // ← Make sure this exports MobiusWidget and MobiusWidgetReactive

#[derive(Default, MobiusWidgetReactive)]
pub struct TestWidget {
    pub name  : Dynamic<String>,
    pub count : Dynamic<i32>,
}

fn main() {
    let mut widget = TestWidget::default();

    widget.set_dynamic("log", "hello world".to_string()); // via SetDynamicFieldString
    widget.set_dynamic("count", 42);                      // via SetDynamicFieldi32
    
}

