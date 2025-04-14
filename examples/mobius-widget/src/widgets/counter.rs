use egui_mobius_reactive::*; 

pub struct CounterWidget {
    pub count : Dynamic<i32>,
    pub log   : Option<Dynamic<String>>,
}

impl Default for CounterWidget {
    fn default() -> Self {
        Self {
            count: Dynamic::new(0),
            log: None,
        }
    }
}

impl MobiusWidget for CounterWidget {
    fn render_widget(&self, ui: &mut egui::Ui) {
        use egui::Color32;

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Reactive Counter").color(Color32::GREEN).heading());
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.count.set(self.count.get() - 1);
                    if let Some(log) = &self.log {
                        log.set(format!("Decremented to {}", self.count.get()));
                    }
                }
                ui.label(egui::RichText::new(format!("Count: {}", self.count.get())).color(Color32::GREEN));
                if ui.button("+").clicked() {
                    self.count.set(self.count.get() + 1);
                    if let Some(log) = &self.log {
                        log.set(format!("Incremented to {}", self.count.get()));
                    }
                }
            });
        });
    }

    fn widget_name(&self) -> &str {
        "CounterWidget"
    }
}

impl MobiusWidgetReactive for CounterWidget {
    fn with_dynamic(&mut self, state: Arc<dyn std::any::Any>) {
        if let Some(d) = state.downcast_ref::<Dynamic<i32>>() {
            self.count = d.clone();
        }
        if let Some(d) = state.downcast_ref::<Dynamic<String>>() {
            self.log = Some(d.clone());
        }
    }
}