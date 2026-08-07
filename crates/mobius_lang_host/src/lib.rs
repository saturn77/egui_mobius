//! # mobius_lang_host
//!
//! The egui-facing plumbing that turns a wired mobius_lang application into
//! rendered, dispatched panels — and the **plugin** boundary that lets a
//! `.mobius` file instantiate *real* `egui_mobius` citizen crates.
//!
//! A citizen named in source (`PlotPanel`, `LensLogger`, …) is resolved to a
//! [`Plugin`]: a constructor that receives the DSL-wired `Dynamic<T>` handles
//! and returns a live citizen wrapping a real ecosystem crate. This crate
//! owns nothing about any particular citizen — it defines the socket; the
//! host binary supplies the plugins.
//!
//! See `PLUGINS.md` for a step-by-step guide to writing one.
//!
//! ## The two kinds of citizen in a `.mobius` app
//!
//! - **Source citizens** are declared from widget primitives in the file and
//!   rendered generically by [`render_source`] — no Rust per citizen.
//! - **Plugin citizens** wrap a compiled crate. Each is a [`Plugin`]
//!   registered in a [`Plugins`] table; [`Plugins::build`] constructs one
//!   from a wired instance.

use std::any::Any;
use std::collections::HashMap;

use egui_citizen::Citizen;
use mobius_lang::execute::{BindingSet, WiredInstance};
use mobius_lang::ir::{IrWidget, IrWidgetTarget};

/// Turns an event path written in source (`"BenchCmd::Apply"`) into the
/// boxed compiled event a signal carries. The host owns the event types, so
/// the host supplies this; a `button ->` with no matching event is inert.
pub type EventResolver<'a> = dyn Fn(&str) -> Option<Box<dyn Any + Send>> + 'a;

/// Host callbacks for user interactions in source-rendered widgets.
pub struct Interactions<'a> {
    /// Constructs the compiled event a `button ->` fires.
    pub events: &'a EventResolver<'a>,
    /// Called when a slider/drag is *released* (mouse up), with the widget's
    /// label and its new value — for "committed value" logging. `None` = no-op.
    pub on_release: Option<&'a dyn Fn(&str, f64)>,
}

impl Interactions<'_> {
    /// Interactions that only resolve events (no release notifications).
    pub fn events_only<'a>(events: &'a EventResolver<'a>) -> Interactions<'a> {
        Interactions {
            events,
            on_release: None,
        }
    }
}

/// A live citizen the host can render: lifecycle ([`Citizen`]) plus a
/// per-frame draw. Rendering is intentionally *not* on the `Citizen` trait
/// (that stays object-safe and lifecycle-only), so this subtrait carries the
/// one method a dock host needs.
pub trait CitizenView: Citizen {
    /// Draw this citizen for one frame.
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Constructs a citizen from its instance name and DSL-wired bindings. This
/// closure is the plugin: it downcasts the shared values it needs out of the
/// [`BindingSet`] and hands them to a real citizen crate's constructor.
pub type Plugin = Box<dyn Fn(&str, &BindingSet) -> Box<dyn CitizenView>>;

/// A table of citizen plugins, keyed by the type name used in source.
#[derive(Default)]
pub struct Plugins {
    builders: HashMap<String, Plugin>,
}

impl Plugins {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin for a citizen type name (e.g. `"LensLogger"`). The
    /// builder receives the instance name and its wired bindings.
    pub fn register(
        mut self,
        type_name: &str,
        builder: impl Fn(&str, &BindingSet) -> Box<dyn CitizenView> + 'static,
    ) -> Self {
        self.builders
            .insert(type_name.to_owned(), Box::new(builder));
        self
    }

    /// Every registered citizen type name — feed these to
    /// [`mobius_lang::Registry`] so the language validates instantiations
    /// against exactly the plugins the host provides.
    pub fn names(&self) -> Vec<String> {
        self.builders.keys().cloned().collect()
    }

    /// Build the citizen for a wired instance, if a plugin is registered for
    /// its type. Returns `None` for source citizens (rendered by
    /// [`render_source`]) and for unregistered types.
    pub fn build(&self, instance: &WiredInstance) -> Option<Box<dyn CitizenView>> {
        let builder = self.builders.get(&instance.ty)?;
        Some(builder(&instance.name, &instance.bindings))
    }
}

/// Render a source-declared citizen generically from its IR widget tree
/// against its wired bindings. No per-citizen Rust — the data is the panel.
/// `events` constructs the compiled event a `button ->` fires.
pub fn render_source(
    ui: &mut egui::Ui,
    widgets: &[IrWidget],
    bindings: &BindingSet,
    ix: &Interactions,
) {
    for widget in widgets {
        render_widget(ui, widget, bindings, ix);
    }
}

fn render_widget(ui: &mut egui::Ui, widget: &IrWidget, bindings: &BindingSet, ix: &Interactions) {
    match widget {
        IrWidget::Container { kind, children } => match kind.as_str() {
            "row" => {
                ui.horizontal(|ui| render_source(ui, children, bindings, ix));
            }
            "group" => {
                ui.group(|ui| render_source(ui, children, bindings, ix));
            }
            "scroll" => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| render_source(ui, children, bindings, ix));
            }
            _ => {
                ui.vertical(|ui| render_source(ui, children, bindings, ix));
            }
        },
        IrWidget::Primitive {
            kind,
            label,
            range,
            options,
            target,
        } => render_primitive(
            ui,
            kind,
            label.as_deref(),
            *range,
            options,
            target,
            bindings,
            ix,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_primitive(
    ui: &mut egui::Ui,
    kind: &str,
    label: Option<&str>,
    range: Option<(f64, f64)>,
    options: &[String],
    target: &IrWidgetTarget,
    bindings: &BindingSet,
    ix: &Interactions,
) {
    let label = label.unwrap_or("");
    match (kind, target) {
        ("combo", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<String>(value) {
                let mut current = binding.get();
                egui::ComboBox::from_label(label)
                    .selected_text(current.clone())
                    .show_ui(ui, |ui| {
                        for variant in options {
                            ui.selectable_value(&mut current, variant.clone(), variant);
                        }
                    });
                if current != binding.get() {
                    binding.set(current);
                }
            }
        }
        ("radio", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<String>(value) {
                let mut current = binding.get();
                ui.horizontal(|ui| {
                    if !label.is_empty() {
                        ui.label(label);
                    }
                    for variant in options {
                        if ui
                            .radio_value(&mut current, variant.clone(), variant)
                            .changed()
                        {
                            binding.set(current.clone());
                        }
                    }
                });
            }
        }
        ("checkbox", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<bool>(value) {
                let mut current = binding.get();
                if ui.checkbox(&mut current, label).changed() {
                    binding.set(current);
                }
            }
        }
        // A prominent on/off switch (larger than a checkbox).
        ("toggle", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<bool>(value) {
                let mut on = binding.get();
                let (mark, fill) = if on {
                    ("●  ON", egui::Color32::from_rgb(58, 122, 72))
                } else {
                    ("○  OFF", egui::Color32::from_rgb(96, 64, 64))
                };
                let button = egui::Button::new(
                    egui::RichText::new(format!("{label}   {mark}"))
                        .size(15.0)
                        .strong(),
                )
                .fill(fill)
                .min_size(egui::vec2(180.0, 30.0));
                if ui.add(button).clicked() {
                    on = !on;
                    binding.set(on);
                }
            }
        }
        ("slider" | "drag", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<f32>(value) {
                let (lo, hi) = range.unwrap_or((0.0, 1.0));
                let mut current = binding.get();
                let slider = egui::Slider::new(&mut current, lo as f32..=hi as f32).text(label);
                let response = ui.add(slider);
                if response.changed() {
                    binding.set(current);
                }
                // On mouse release (drag end), notify the host of the value.
                if response.drag_stopped()
                    && let Some(on_release) = ix.on_release
                {
                    on_release(label, current as f64);
                }
            }
        }
        ("text", IrWidgetTarget::Write { value }) => {
            if let Some(binding) = bindings.write::<String>(value) {
                let mut current = binding.get();
                ui.horizontal(|ui| {
                    ui.label(label);
                    if ui.text_edit_singleline(&mut current).changed() {
                        binding.set(current);
                    }
                });
            }
        }
        ("button", IrWidgetTarget::Event { signal, event }) => {
            if ui.button(label).clicked()
                && let Some(emit) = bindings.emit(signal)
                && let Some(boxed) = (ix.events)(event)
            {
                emit.send_boxed(boxed);
            }
        }
        ("progress", IrWidgetTarget::Read { value }) => {
            if let Some(watch) = bindings.read::<f32>(value) {
                ui.add(egui::ProgressBar::new(watch.get()).show_percentage());
            }
        }
        ("label" | "value", IrWidgetTarget::Read { value }) => {
            if let Some(watch) = bindings.read::<f32>(value) {
                ui.label(format!("{label} {}", watch.get()));
            } else if let Some(watch) = bindings.read::<String>(value) {
                ui.label(format!("{label} {}", watch.get()));
            }
        }
        ("separator", _) => {
            ui.separator();
        }
        ("spacer", _) => {
            ui.add_space(8.0);
        }
        ("label", IrWidgetTarget::None) => {
            ui.label(label);
        }
        _ => {
            ui.weak(format!("<{kind}?>"));
        }
    }
}
