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
- *Programmatically* generate command buttons 
- Process commands in the backend and send results back to the frontend.
- *Recursively* use the process_commands() function
- Display results in a terminal window.
- Clear the terminal window using the 'Clear Terminal' button.
- Display an 'About' message using the 'About' button.
- Send cascaded messages, using the AsAny derive on an Command enum


### Programmatic Generation of Commands 
There is a **egui_mobius** macro that will generate command buttons for a 

```rust
(button_name, command_name)
```

tuple as shown below. This code is in the ui_app.rs: 

```rust
GENERATE_COMMAND_BUTTONS!(ui, self.command_sender, [
    // (button_name, command_name)
    ("First Task", Command::FirstTask), 
    ("Second Task", Command::SecondTask),
    ("Clear Terminal", Command::ClearTerminal),
    ("About", Command::About), 
    ("Cascade First Second", cascade_first_second.clone()),
]);
```
The Commands are in the main.rs file as : 
```rust
#[derive(AsAny, Clone)]
pub enum Command {
    FirstTask,
    SecondTask,
    ClearTerminal,
    About,
    CascadeFirstSecond(Vec<Command>),
}
```

What is flexible in the Command enum is the ability to *chain* commands
and then have the GENERATE_COMMAND_BUTTONS macro hanle the chained 
command appropriately. 
```rust
ui.horizontal(|ui| {
    let cascade_commands = vec![Command::FirstTask, Command::SecondTask];
    let cascade_first_second = {
        let commands = cascade_commands.clone();
        Command::CascadeFirstSecond(commands)
    };
    GENERATE_COMMAND_BUTTONS!(ui, self.command_sender, [
        ("First Task", Command::FirstTask),
        ("Second Task", Command::SecondTask),
        ("Clear Terminal", Command::ClearTerminal),
        ("About", Command::About), 
        ("Cascade First Second", cascade_first_second.clone()),
    ]);
});
```

There are macros that handle the sending of signals, this is somewhat
similar to Qt's famous signals/slots mechanism. 

### Example: signals_slots

To run the `signals_slots` example, use the following command:

```sh
cargo run -p signals_slots
```

The `signals_slots` example demonstrates how to:

- Implement a signal-slot mechanism using the `mobius_egui` library.
- Define and use `Signal` structs to send commands asynchronously.
- Define and use `Slot` structs to receive and handle commands.
- Showcase the interaction between frontend and backend using signals and slots.
- Illustrate how to handle multiple commands and process them in a separate thread.
- Provide a clear and concise way to manage asynchronous communication in a Rust application, similar to Qt's signal-slot mechanism.

This example helps you understand how to set up and use the signal-slot pattern in your `mobius_egui` applications, enabling efficient and organized communication between different parts of your application.

