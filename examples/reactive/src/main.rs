use std::sync::Arc;
use eframe::NativeOptions;
use egui_mobius_reactive::*;
use egui_mobius::factory; 
use egui_mobius::signals::Signal;

#[derive(Clone, Debug)]
pub enum Event {
    IncrementClicked,
    CountChanged(i32),
    LabelChanged(String),
}

pub struct AppState {
    pub registry: SignalRegistry,
    count: Dynamic<i32>,
    label: Dynamic<String>,
    doubled: Derived<i32>,
    quad: Derived<i32>,
    fifth: Derived<i32>,
    sum_derived: Derived<i32>,
    list_sum: Derived<i32>,
    list: ReactiveList<i32>,
    signal: Signal<Event>,
}

impl AppState {
    pub fn new(registry: SignalRegistry, signal: Signal<Event>) -> Self {
        let count = Dynamic::new(0);
        let label = Dynamic::new("Click to increment".to_string());
        registry.register_named_signal("count", Arc::new(count.clone()));
        registry.register_named_signal("label", Arc::new(label.clone()));

        // Use ReactiveMath
        let doubled : Derived<i32> = count.powi(2);
        let quad = count.powi(4);
        let fifth = count.powi(5);
        let sum_derived : Derived<i32> = count.clone() + doubled.clone();

        registry.register_named_signal("doubled", Arc::new(doubled.clone()));
        registry.register_named_signal("quad", Arc::new(quad.clone()));
        registry.register_named_signal("fifth", Arc::new(fifth.clone()));
        registry.register_named_signal("sum_derived", Arc::new(sum_derived.clone()));

        registry.effect(&[Arc::new(sum_derived.clone())], move || {
            println!("💥 sum_derived changed");
        });

        let list = ReactiveList::new();
        list.push(42);
        list.push(7);
        list.push(13);

        let list_arc = Arc::new(list.clone());
        registry.register_named_signal("list", list_arc.clone());

        let list_clone = list.clone();
        registry.effect(&[list_arc.clone()], move || {
            println!("📋 list changed: {:?}", list_clone.get_all());
        });

        let list_sum = list.clone().sum();
        registry.register_named_signal("list_sum", Arc::new(list_sum.clone()));

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
            ui.vertical_centered(|ui| {
                ui.heading("Reactive UI with egui_mobius");
            });
            ui.add_space(20.0);

            egui::CollapsingHeader::new("📊 Counter Values")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(self.label.lock().as_str()).clicked() {
                            let new_count = *self.count.lock() + 1;
                            self.count.set(new_count);
                            if let Err(e) = self.signal.send(Event::IncrementClicked) {
                                eprintln!("Failed to send increment event: {}", e);
                            }
                        }
                    });

                    egui::Grid::new("counter_grid")
                        .striped(true)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Count:");
                            ui.label(format!("{}", self.count.get()));
                            ui.end_row();

                            ui.label("Doubled:");
                            ui.label(format!("{}", self.doubled.get()));
                            ui.end_row();

                            ui.label("Quad:");
                            ui.label(format!("{}", self.quad.get()));
                            ui.end_row();

                            ui.label("Fifth:");
                            ui.label(format!("{}", self.fifth.get()));
                            ui.end_row();

                            ui.label("Sum:");
                            ui.label(format!("{}", self.sum_derived.get()));
                            ui.end_row();
                        });
                });

            ui.add_space(10.0);
            ui.separator();

            egui::CollapsingHeader::new("📋 Reactive List")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("➕ Add Item").clicked() {
                            self.list.push(*self.count.lock());
                        }
                        if ui.button("➖ Remove Last").clicked() {
                            let all = self.list.get_all();
                            if !all.is_empty() {
                                self.list.remove(all.len() - 1);
                            }
                        }
                        if ui.button("🗑️ Clear All").clicked() {
                            self.list.clear();
                        }
                    });

                    ui.add_space(8.0);

                    egui::Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            for item in self.list.get_all() {
                                ui.label(format!("• {}", item));
                            }
                            ui.separator();
                            ui.strong(format!("Sum: {}", self.list_sum.get()));
                        });
                });
        });

        egui::Window::new("🔍 Reactive Graph Debug").show(ctx, |ui| {
            ui.label("⚙️ Registered Signals:");
            for (name, signal) in self.registry.list_signals() {
                let any = signal.as_any();
                if let Some(val) = any.downcast_ref::<Dynamic<i32>>() {
                    ui.label(format!("- {}: {}", name, val.get()));
                } else if let Some(val) = any.downcast_ref::<Derived<i32>>() {
                    ui.label(format!("- {} (derived): {}", name, val.get()));
                } else if let Some(val) = any.downcast_ref::<ReactiveList<i32>>() {
                    ui.label(format!("- {} (list): {:?}", name, val.get_all()));
                } else if let Some(val) = any.downcast_ref::<Dynamic<String>>() {
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
