#[macro_export]
macro_rules! mobius_send_command {
    ($sender:expr, $command:expr) => {
        {
            let sender = $sender.clone();
            std::thread::spawn(move || {
                if let Err(e) = sender.try_send($command) {
                    eprintln!("\n***** Failed to send {:?} command: {:?}", $command, e);
                }
            });
        }
    };
}