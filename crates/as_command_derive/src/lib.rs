extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// This macro generates a trait implementation for a command enum,
/// while also providing a as_any method for downcasting to a trait object.
/// The trait object can be used to send commands to the backend.
///
/// # Example
/// ```
/// use as_command_derive::AsCommand;
///
/// #[derive(AsCommand, Clone)]
/// pub enum Command {
///   FirstTask,
///   SecondTask,
///   ClearTerminal,
///   About,
///   CascadeFirstSecond(Vec<Command>),
/// }
/// ```
/// The above code will generate the following implementation:
/// ```
/// impl Command {
///    pub fn as_any(&self) -> &dyn std::any::Any {
///       self
///   }
///
///  pub fn generate_buttons(ui: &mut egui::Ui, command_sender: &std::sync::mpsc::Sender<Command>, commands: Vec<(&str, Command)>) {
///    for (label, command) in commands {
///      if ui.button(label).clicked() {
///       println!("{} button clicked.", label);
///       let command = command.clone();
///         if let Some(commands) = command.as_any().downcast_ref::<Vec<Command>>() {
///           let commands_clone = commands.clone();
///           Signal!(command_sender, commands_clone, multiple);
///         } else {
///            Signal!(command_sender, command);
///          }
///       }
///    }
/// }
/// ```
/// The generate_buttons method can be used to generate buttons for the commands in the enum.
/// The method takes a reference to the egui::Ui object, a reference to the command sender, 
/// and a vector of tuples containing the label and the command.
/// An example of using the generate_buttons method is shown below:
/// ```
/// let cascade_commands = vec![Command::FirstTask, Command::SecondTask];
/// let cascade_first_second = {
///   let commands = cascade_commands.clone();
///   Command::CascadeFirstSecond(commands)
/// };
/// Command::generate_buttons(ui, &self.command_sender, vec![
///   ("First Task", Command::FirstTask),
///   ("Second Task", Command::SecondTask),
///   ("Clear Terminal", Command::ClearTerminal),
///   ("About", Command::About),
///   ("Cascade First Second", cascade_first_second.clone()),
/// ]);
/// ```
/// When a button is clicked, the command is sent to the backend using the Signal! macro.
/// If the command is a vector of commands, the commands are sent one by one.
///
/// # Arguments
/// * `input` - The input token stream
///
/// # Returns
/// A token stream containing the generated code
///
/// # Panics
/// This macro will panic if the input is not a valid enum
///
#[proc_macro_derive(AsCommand)]
pub fn as_command_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl #name {
            pub fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            pub fn generate_buttons(ui: &mut egui::Ui, command_sender: &std::sync::mpsc::Sender<#name>, commands: Vec<(&str, #name)>) {
                for (label, command) in commands {
                    if ui.button(label).clicked() {
                        println!("{} button clicked.", label);
                        let command = command.clone();
                        if let Some(commands) = command.as_any().downcast_ref::<Vec<#name>>() {
                            let commands_clone = commands.clone();
                            Signal!(command_sender, commands_clone, multiple);
                        } else {
                            Signal!(command_sender, command);
                        }
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}
