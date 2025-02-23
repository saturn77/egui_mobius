#[macro_export]
macro_rules! mobius_send_command {
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