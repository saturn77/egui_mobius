# Simple Monitor Example

This example send commands to the backend, and the commands are also echoed to a logger terminal within the application. 


## Features

This application demonstrates how to: 

- Use `mobius_egui` with a simple monitor application.
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