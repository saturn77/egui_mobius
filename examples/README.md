# Examples

This directory contains self-contained example crates that demonstrate how to use the `mobius_egui` library. These examples are individual crates due to each example being 
multiple files, illustrating the separation of front end and back end code. 

## Running the Examples

Each example is a separate crate and can be run using the `cargo run` command with the `-p` flag to specify the package name.

### Example: simple_monitor

To run the `simple_monitor` example, use the following command:

```sh
cargo run -p simple_monitor
```

The simple_monitor demonstrates how to: 

- Fundamentally use `mobius_egui` with a simple monitor application.
- Send commands to the backend using buttons.
- Process commands in the backend and send results back to the frontend.
- Display results in a terminal window.
- Clear the terminal window using the 'Clear Terminal' button.
- Display an 'About' message using the 'About' button.

There are macros that handle the sending of signals, this is somewhat
similar to Qt's famous signals/slots mechanism. 