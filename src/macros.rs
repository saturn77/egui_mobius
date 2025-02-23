#[macro_export]
macro_rules! mobius_send_command {
    ($sender:expr, $commands:expr) => {
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