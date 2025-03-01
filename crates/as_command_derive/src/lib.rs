extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AsCommand)]
pub fn as_any_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl #name {
            pub fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            pub fn generate_command_buttons(ui: &mut egui::Ui, command_sender: &std::sync::mpsc::Sender<#name>, commands: Vec<(&str, #name)>) {
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


