#[macro_export]
macro_rules! Signal {
    ($sender:expr, $command:expr) => {
        {
            let sender = $sender.clone();
            std::thread::spawn(move || {
                if let Err(e) = sender.send($command) {
                    eprintln!("\n***** Failed to send command: {:?}", e);
                }
            });
        }
    };
    ($sender:expr, $commands:expr, multiple) => {
        {
            let sender = $sender.clone();
            std::thread::spawn(move || {
                for command in $commands {
                    if let Err(e) = sender.send(command) {
                        eprintln!("\n***** Failed to send command: {:?}", e);
                    }
                }
            });
        }
    };
}

/// Generate command buttons
/// Emphasize a consistent pattern for creating command buttons with 
/// their associated commands. This macro is a utility function
/// that becomes integral to the egui_mobius library.
///
/// # Arguments
/// * `$ui` - The Egui UI object
/// * `$command_sender` - The command sender object
/// * `[$($label:expr, $command:expr),*]` - A list of tuples containing the button label and the command to be sent
///

#[macro_export]
macro_rules! GENERATE_COMMAND_BUTTONS {
    ($ui:expr, $command_sender:expr, [$(($label:expr, $command:expr)),* $(,)?]) => {
        $(
            if $ui.button($label).clicked() {
                println!("{} button clicked.", $label);
                let command = $command.clone();
                if let Some(commands) = command.as_any().downcast_ref::<Vec<Command>>() {
                    let commands_clone = commands.clone();
                    Signal!($command_sender, commands_clone, multiple);
                } else {
                    Signal!($command_sender, command);
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! clear_logger {
    ($logger_text:expr) => {
        {
            $logger_text.lock().unwrap().clear();
        }
    };
}