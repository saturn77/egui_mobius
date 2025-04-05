//! masonry with egui_mobius integration
//!
//! Demonstrates how to integrate egui_mobius_reactive with another
//! GUI framework other than egui/eframe.
//!
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

// masonry imports
use masonry::app::{AppDriver, DriverCtx};
use masonry::core::{Action, StyleProperty, WidgetId};
use masonry::dpi::LogicalSize;
use masonry::parley::style::FontWeight;
use masonry::widgets::{Button, Flex, Label, RootWidget};
use winit::window::Window;

// egui_mobius_reactive imports using prelude import
use egui_mobius_reactive::*;
use std::sync::Arc;

const VERTICAL_WIDGET_SPACING: f64 = 40.0;

pub struct AppState {
    click_count : Dynamic<i32>,
    doubled     : Derived<i32>,
    sum         : Derived<i32>,
    message     : Dynamic<String>,
}

impl AppState {
    pub fn new() -> Self {
        let click_count = Dynamic::new(0);
        let doubled = click_count.doubled();
        let sum = click_count.clone() + doubled.clone();
        let message = Dynamic::new("Click the button!".to_string());

        let registry = SignalRegistry::default();
        registry.register_named_signal("click_count", Arc::new(click_count.clone()));
        registry.register_named_signal("doubled", Arc::new(doubled.clone()));
        registry.register_named_signal("sum", Arc::new(sum.clone()));
        registry.register_named_signal("message", Arc::new(message.clone()));

        Self {
            click_count,
            doubled,
            sum, 
            message,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

struct Driver {
    state: Arc<AppState>,
}

impl AppDriver for Driver {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: Action) {
        match action {
            Action::ButtonPressed(_) => {
                // First update the count
                let count = self.state.click_count.get() + 1;
                self.state.click_count.set(count);
                
                // Small delay to let the reactive system update
                std::thread::sleep(std::time::Duration::from_micros(200));
                
                // Now get the doubled value after it's had time to update
                let doubled = self.state.doubled.get();
                println!("Count: {}, Doubled: {}", count, doubled);
                self.state.message.set(format!("Count: {} × 2 = {}", count, doubled));
                
                // Make a "separate" sum message to be displayed 
                let sum_string = format!("Derived Value `sum` = {}", self.state.sum.get());
                
                // Force UI update
                ctx.render_root().edit_root_widget(|mut root| {
                    let mut root = root.downcast::<RootWidget<Flex>>();
                    let mut flex = RootWidget::child_mut(&mut root);
                    // Clear existing children and rebuild
                    Flex::clear(&mut flex);
                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);
                    Flex::add_child(&mut flex, Label::new(self.state.message.get())
                        .with_style(StyleProperty::FontSize(24.0))
                        .with_style(StyleProperty::FontWeight(FontWeight::BOLD)));
                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);

                    Flex::add_child(&mut flex, Label::new(sum_string)
                    .with_style(StyleProperty::FontSize(24.0))
                    .with_style(StyleProperty::FontWeight(FontWeight::BOLD)));
                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);
                    
                    Flex::add_child(&mut flex, Button::new("Click me!"));
                });
            }
            action => {
                eprintln!("Unexpected action {action:?}");
            }
        }
    }
}

fn main() {
    let state = Arc::new(AppState::new());

    let label = Label::new(state.message.get())
        .with_style(StyleProperty::FontSize(32.0))
        .with_style(StyleProperty::FontWeight(FontWeight::BOLD));
    let button = Button::new("Click me!");

    // Arrange the two widgets vertically, with some padding
    let main_widget = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(button);

    let window_size = LogicalSize::new(400.0, 400.0);
    let window_attributes = Window::default_attributes()
        .with_title("Reactive Counter")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    masonry::app::run(
        masonry::app::EventLoop::with_user_event(),
        window_attributes,
        RootWidget::new(main_widget),
        Driver {
            state: state.clone(),
        },
    )
    .unwrap();
}