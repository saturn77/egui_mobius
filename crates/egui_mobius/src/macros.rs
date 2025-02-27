
/// A macro to send a command to the command sender
/// The command sender is an egui_mobius Enqueue object
/// This can be called with the Signal! syntax
/// 
/// # Arguments
/// * `$sender` - The command sender object
/// * `$command` - The command to be sent
/// 
/// Note that the command can be a single command or a vector of commands
/// 
/// # Example
/// ```
/// use std::sync::mpsc;
/// use egui_mobius::Signal;
/// 
/// #[derive(Clone)]
/// enum Command {
///     FirstTask,
///     SecondTask,
/// }
/// 
/// let (command_sender, command_receiver) = mpsc::channel();
/// Signal!(command_sender, Command::FirstTask);
/// ```
/// For a vector of commands:
/// ```
/// use std::sync::mpsc;
/// use egui_mobius::Signal;
/// 
/// #[derive(Clone)]
/// enum Command {
///     FirstTask,
///     SecondTask,
/// }
/// 
/// let (command_sender, command_receiver) = mpsc::channel();
/// Signal!(command_sender, vec![Command::FirstTask, Command::SecondTask], multiple);
/// ```
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



/// A macro to generate command buttons
/// This macro is used to generate buttons in the egui window
/// The buttons are used to send commands to the command sender
///
/// # Arguments
/// * `$ui` - The egui ui object
/// * `$command_sender` - The command sender object
/// * `$commands` - A list of tuples containing the button label and the command to be sent
///
/// # Example
/// ```
/// use std::sync::mpsc;
/// use egui_mobius::GENERATE_COMMAND_BUTTONS;
///
/// #[derive(Clone)]
/// enum Command {
///     FirstTask,
///     SecondTask,
/// }
///
/// let (command_sender, command_receiver) = mpsc::channel();
/// GENERATE_COMMAND_BUTTONS!(ui, command_sender, [
///     ("First Task", Command::FirstTask),
///     ("Second Task", Command::SecondTask),
/// ]);
/// ```
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


