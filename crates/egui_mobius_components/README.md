# egui_mobius_components

A collection of reusable UI components for the [egui_mobius](https://github.com/saturn77/egui_mobius) framework.

## Features

The `egui_mobius_components` crate provides high-level, reusable UI components built on top of the egui_mobius architecture:

- **EventLogger**: A sophisticated terminal-like widget for logging events with:
  - Different severity levels (Info, Warn, Debug, Error)
  - Rich text formatting with customizable colors
  - Thread-safe implementation compatible with egui_mobius signal/slot
  - Support for categorizing logs by sender type
  - Built-in timestamping and filtering
  - Complete with examples demonstrating usage in multi-threaded environments

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
egui_mobius_components = "0.3.0-alpha.31"
```

And then import components via the prelude:

```rust
use egui_mobius_components::prelude::*;
```

## Example: EventLogger

```rust
use eframe::egui;
use egui_mobius_components::prelude::*;

fn main() -> Result<(), eframe::Error> {
    // Initialize logger with signal/slot
    let (logger, event_slot, response_signal) = create_event_logger(
        egui::Context::default(), 
        LogColors::default()
    );
    
    // Add a log entry
    logger.info(
        "Application started".to_string(),
        LogSender::system(),
        LogType::Default
    );
    
    // Show the logger in your UI
    eframe::run_ui(&egui::Context::default(), |ui| {
        logger.show(ui);
    });
}
```

For more detailed examples, check out the [logger_component example](https://github.com/saturn77/egui_mobius/tree/master/examples/logger_component) in the egui_mobius repository.

## License

This project is licensed under the MIT License - see the LICENSE file for details.