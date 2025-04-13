use egui_mobius_reactive::*; 

pub struct LoggerWidget {
    pub logger_text: Dynamic<String>,
}
impl Default for LoggerWidget {
    fn default() -> Self {
        Self {
            logger_text: Dynamic::new("".into()),
        }
    }
}

impl MobiusWidget for LoggerWidget {
    fn render_widget(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Logger").color(egui::Color32::GREEN).heading());
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Log: {}", self.logger_text.get())).color(egui::Color32::GREEN));
        });
    }

    fn widget_name(&self) -> &str {
        "LoggerWidget"
    }
}

impl MobiusWidgetReactive for LoggerWidget {
    fn with_dynamic(&mut self, state: Arc<dyn std::any::Any>) {
        if let Some(d) = state.downcast_ref::<Dynamic<String>>() {
            self.logger_text = d.clone();
        }
    }
}