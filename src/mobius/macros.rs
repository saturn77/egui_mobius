#[macro_export]
macro_rules! mobius_send_command {
    ($sender:expr, $command:expr) => {
        {
            let sender = $sender.clone();
            task::spawn(async move {
                if let Err(e) = sender.send($command).await {
                    eprintln!("Failed to send {:?} command: {:?}", $command, e);
                }
            });
        }
    };
}