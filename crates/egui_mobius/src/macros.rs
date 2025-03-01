

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

#[macro_export]
macro_rules! clear_logger {
    ($logger_text:expr) => {
        {
            $logger_text.lock().unwrap().clear();
        }
    };
}


