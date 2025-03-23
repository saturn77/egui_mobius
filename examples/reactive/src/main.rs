use std::sync::Arc;
use eframe::NativeOptions;
use egui_mobius_reactive::{Value, Derived, SignalRegistry, ReactiveList, ReactiveValue};
use egui_mobius::factory;
use egui_mobius::Signal;

// Reactive Core Example
// =====================
// This example demonstrates the use of the reactive core in `egui_mobius`.
// It shows how to create reactive signals and bind them to UI elements.
macro_rules! register_derived {
    ($registry:expr, $name:expr, $dep:expr, $power:expr) => {{
        let derived = Derived::new(&[Arc::new($dep.clone()) as Arc<dyn ReactiveValue>], move || {
            let val: i32 = $dep.get();
            val.pow($power as u32)
        });
        $registry.register_named_signal($name, Arc::new(derived.clone()));
        derived
    }};
}

macro_rules! register_list {
    ($registry:expr, $name:expr, $list:expr, $effect:block) => {{
        let list_arc = Arc::new($list.clone());
        $registry.effect(&[list_arc.clone()], {
            let _list = list_arc.clone();
            move || $effect
        });
        $registry.register_named_signal($name, list_arc.clone());
        list_arc
    }};
}

macro_rules! register_effect {
    ($registry:expr, $name:expr, $signal:expr, $effect:block) => {{
        let signal_arc = Arc::new($signal.clone());
        $registry.effect(&[signal_arc.clone()], {
            let _signal = signal_arc.clone();
            move || $effect
        });
        $registry.register_named_signal($name, signal_arc.clone());
        signal_arc
    }};
}

fn register_signal<T: ReactiveValue + 'static>(
    registry: &SignalRegistry,
    name: &str,
    signal: Arc<T>,
) {
    registry.register_named_signal(name, signal);
}

#[derive(Clone, Debug)]
pub enum Event {
    IncrementClicked,
    CountChanged(i32),
    LabelChanged(String),
}

pub struct AppState {
    pub registry : SignalRegistry,
    count        : Value<i32>,
    label        : Value<String>,
    doubled      : Derived<i32>,
    quad         : Derived<i32>,
    fifth        : Derived<i32>,
    sum_derived  : Derived<i32>,
    list_sum     : Derived<i32>,
    list         : ReactiveList<i32>,
    signal       : Signal<Event>,
}

impl AppState {
    pub fn new(registry: SignalRegistry, signal: Signal<Event>) -> Self {
        let count = Value::new(0);
        let count_clone = count.clone();
        let count_clone2 = count.clone();
        let count_clone3 = count.clone();
        let count_clone4 = count.clone();
        register_signal(&registry, "count", Arc::new(count.clone()));

        let doubled = register_derived!(registry, "doubled", count_clone.clone(), 2);
        let doubled_clone = doubled.clone();
        let quad = register_derived!(registry, "quad", count_clone2.clone(), 4);
        let fifth = register_derived!(registry, "fifth", count_clone3.clone(), 5);

        let sum_derived = Derived::new(&[
            Arc::new(count_clone4.clone()) as Arc<dyn ReactiveValue>,
            Arc::new(doubled_clone.clone()) as Arc<dyn ReactiveValue>,
        ], move || {
            &count_clone4.get() + &doubled_clone.get()
        });

        let _sum_derived_arc = register_effect!(registry, "sum_derived", sum_derived, {
            println!("💥 sum_derived changed");
        });

        let list = ReactiveList::new();
        let list_clone = list.clone();
        let list_clone2 = list.clone();
        list.push(42);
        list.push(7);
        list.push(13);

        let list_arc = register_list!(registry, "list", list_clone2, {
            println!("📋 list changed: {:?}", list_clone2.get_all());
        });

        let list_sum = Derived::new(&[list_arc.clone() as Arc<dyn ReactiveValue>], move || {
            list_clone.get_all().iter().sum()
        });
        register_signal(&registry, "list_sum", Arc::new(list_sum.clone()));

        let label = Value::new("Click to increment".to_string());
        register_signal(&registry, "label", Arc::new(label.clone()));

        Self {
            registry,
            count,
            label,
            doubled,
            quad,
            fifth,
            sum_derived,
            list_sum,
            list,
            signal,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive UI with egui_mobius");
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let count = *self.count.lock();
                ui.label(format!("Count: {}", count));
                ui.add_space(20.0);
                ui.label(format!("Doubled: {}", self.doubled.get()));
                ui.add_space(20.0);
                ui.label(format!("Quad: {}", self.quad.get()));
                ui.add_space(20.0);
                ui.label(format!("Fifth: {}", self.fifth.get()));
                ui.add_space(20.0);
                ui.label(format!("Sum: {}", self.sum_derived.get()));
            });

            ui.add_space(20.0);
            ui.label("Reactive List:");
            for item in self.list.get_all() {
                ui.label(format!("- {}", item));
            }
            ui.label(format!("List Sum: {}", self.list_sum.get()));

            ui.add_space(10.0);
            if ui.button("Add Item to List").clicked() {
                let new_item = *self.count.lock();
                self.list.push(new_item);
            }

            if ui.button("Remove Last Item").clicked() {
                let all = self.list.get_all();
                if !all.is_empty() {
                    self.list.remove(all.len() - 1);
                }
            }

            if ui.button("Clear List").clicked() {
                self.list.clear();
            }

            ui.add_space(20.0);
            if ui.button(self.label.lock().as_str()).clicked() {
                let new_count = *self.count.lock() + 1;
                self.count.set(new_count);
                if let Err(e) = self.signal.send(Event::IncrementClicked) {
                    eprintln!("Failed to send increment event: {}", e);
                }
            }
        });

        // Debug Panel for Reactive Graph
        egui::Window::new("🔍 Reactive Graph Debug").show(ctx, |ui| {
            ui.label("⚙️ Registered Signals:");
            for (name, signal) in self.registry.list_signals() {
                let any = signal.as_any();
                if let Some(val) = any.downcast_ref::<Value<i32>>() {
                    ui.label(format!("- {}: {}", name, val.get()));
                } else if let Some(val) = any.downcast_ref::<Derived<i32>>() {
                    ui.label(format!("- {} (derived): {}", name, val.get()));
                } else if let Some(val) = any.downcast_ref::<ReactiveList<i32>>() {
                    ui.label(format!("- {} (list): {:?}", name, val.get_all()));
                } else if let Some(val) = any.downcast_ref::<Value<String>>() {
                    ui.label(format!("- {}: \"{}\"", name, val.get()));
                } else {
                    ui.label(format!("- {} (?)", name));
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let (event_signal, _event_slot) = factory::create_signal_slot::<Event>();

    eframe::run_native(
        "egui_mobius Reactive Example",
        NativeOptions::default(),
        Box::new(move |cc| {
            let _ctx = cc.egui_ctx.clone();
            let registry = SignalRegistry::new();
            let app_state = AppState::new(registry, event_signal.clone());
            Ok(Box::new(app_state))
        }),
    )
}
