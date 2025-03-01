# Simple Monitor 2 Example 

This example is similar to simple_monitor example, but exposes the features of the 
AsCommand procedural macro that can be applied to the command enum. 

One can run this example by 

```bash
cargo run -p simple_monitor_2
```
from anywhere in the crate. 


## AsCommand Macro

AsCommand is a procedural macro that derives implementations for the UiCommand enum. It is not a trait itself, but it generates code that includes methods such as as_any and generate_command_buttons for the UiCommand enum.

Here is a summary of what the AsCommand procedural macro does:

1. Implements the as_any method: This method allows downcasting to a trait object.
2. Implements the generate_command_buttons method: This method generates buttons in the egui UI and sends commands to the command sender.


## Example 

The code below shows how simple_monitor_2 is different from simple_monitor : 


For simple_monitor_2 : 
```rust 
#[derive(AsCommand, Clone)]
pub enum UiCommand {
    FirstTask,
    SecondTask,
    ClearTerminal,
    About,
    CascadeFirstSecond(Vec<UiCommand>),
}
```

Where as in simple_monitor : 
```rust 
#[derive(AsCommand, Clone)]
pub enum UiCommand {
    FirstTask,
    SecondTask,
    ClearTerminal,
    About,
    CascadeFirstSecond(Vec<UiCommand>),
}
```

## Use of Derived Traits

Now using the generate_command derived trait: 

For simple_monitor_2 : 

```rust
ui.horizontal(|ui| {
    let cascade_commands = vec![Command::FirstTask, Command::SecondTask];
    let cascade_first_second = {
        let commands = cascade_commands.clone();
        Command::CascadeFirstSecond(commands)
    };
    UiCommand::generate_buttons(ui, &self.command_sender, vec![
        ("First Task", Command::FirstTask),
        ("Second Task", Command::SecondTask),
        ("Clear Terminal", Command::ClearTerminal),
        ("About", Command::About), 
        ("Cascade First Second", cascade_first_second.clone()),
    ]);
});
```

Where as for simple monitor : 
```rust
ui.horizontal(|ui| {
    let cascade_commands = vec![Command::FirstTask, Command::SecondTask];
    let cascade_first_second = {
        let commands = cascade_commands.clone();
        Command::CascadeFirstSecond(commands)
    };
    Command::generate_buttons(ui, &self.command_sender, vec![
        ("First Task", Command::FirstTask),
        ("Second Task", Command::SecondTask),
        ("Clear Terminal", Command::ClearTerminal),
        ("About", Command::About), 
        ("Cascade First Second", cascade_first_second.clone()),
    ]);
});
```