//! The `citizen_panel!` macro — panel-struct boilerplate in one line.

/// Define a citizen panel struct with the standard boilerplate.
///
/// Expands to a struct holding `citizen_id` + `citizen_state` (plus any
/// extra fields you declare), a `new(citizen_state)` constructor, and the
/// [`Citizen`](crate::Citizen) trait implementation. The lifecycle hooks
/// keep their trait defaults — implement `Citizen` by hand instead when a
/// panel needs custom `on_activate` / `on_deactivate` / `on_click` logic.
///
/// Extra fields are declared as `name: Type = default` and become `pub`
/// fields initialized to the given default in `new()`.
///
/// # Example
///
/// ```rust
/// use egui_citizen::{citizen_panel, Citizen, Registrar, CitizenId};
///
/// citizen_panel!(PlotPanel, "plot",
///     samples: Vec<f32> = Vec::new(),
///     autoscale: bool = true,
/// );
///
/// let mut registrar = Registrar::new();
/// let state = registrar.register(CitizenId::new("plot"));
/// let panel = PlotPanel::new(state);
///
/// assert_eq!(panel.id().0, "plot");
/// assert!(panel.autoscale);
/// ```
#[macro_export]
macro_rules! citizen_panel {
    ($name:ident, $id:expr $(, $field:ident : $ty:ty = $default:expr)* $(,)?) => {
        pub struct $name {
            citizen_id: $crate::CitizenId,
            citizen_state: $crate::CitizenState,
            $( pub $field: $ty, )*
        }

        impl $name {
            pub fn new(citizen_state: $crate::CitizenState) -> Self {
                Self {
                    citizen_id: $crate::CitizenId::new($id),
                    citizen_state,
                    $( $field: $default, )*
                }
            }
        }

        impl $crate::Citizen for $name {
            fn id(&self) -> &$crate::CitizenId { &self.citizen_id }
            fn citizen_state(&self) -> &$crate::CitizenState { &self.citizen_state }
            fn citizen_state_mut(&mut self) -> &mut $crate::CitizenState { &mut self.citizen_state }
        }
    };
}
