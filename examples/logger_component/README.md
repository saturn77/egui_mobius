# Logger Component Example

This example demonstrates the use of the Event Logger component from the egui_mobius_components crate.

## Features

- Interactive log display with colored message types
- Various log message types: Info, Warning, Debug, Error
- Different sender types for log messages
- Real-time logging from background threads
- Clear log functionality

## Running the Example

```bash
cd examples/logger_component
cargo run
```

## Implementation Details

The example shows how to:

1. Initialize the logger with signal/slot architecture
2. Run the logger backend for processing events
3. Send different types of log messages
4. Display the logger in the UI
5. Handle logging from background threads