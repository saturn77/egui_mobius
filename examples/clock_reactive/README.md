# Clock Reactive Example

This example demonstrates the use of the reactive core in `egui_mobius` to create a simple reactive clock application. It showcases how to use reactive signals, derived values, and bindings to dynamically update the UI in response to changes in state.

---

## Comparing clock_reactive to clock_async 

**clock_async:**

- Focus: More feature-rich, explicit async event handling.

 - Pros: Greater control, better for complex async tasks, clearer separation of concerns.

 - Cons: More code, more boilerplate, higher learning curve.

**clock_reactive:**

 - Focus: Simplified, reactive programming model.

 - Pros: Less code, automatic state updates, easier maintenance.

 - Cons: Less explicit flow, potentially harder to debug.


## Features

- **Reactive Signals**: Uses `Value` and `Derived` to manage and react to state changes.
- **Dynamic UI Updates**: Automatically updates the UI when signals or derived values change.
- **Debug Panel**: Displays the current state of all registered signals for debugging purposes.

---

## How It Works

The `clock_reactive` example uses the following components:

1. **Signals**:
   - `Value<T>`: Represents a reactive value that can be updated and observed.
   - `Derived<T>`: Represents a computed value that automatically updates when its dependencies change.

2. **Reactive List**:
   - Demonstrates how to use `ReactiveList` to manage a dynamic list of items and react to changes.

3. **UI Bindings**:
   - Binds signals and derived values to UI elements, ensuring the UI updates automatically when the state changes.

4. **Debug Panel**:
   - Lists all registered signals and their current values for debugging purposes.

---

## Example Workflow

### Counter Section
1. **Increment Counter**:
   - Clicking the button increments the counter value.
   - Derived values (`doubled`, `quad`, `fifth`, etc.) update automatically based on the counter.

2. **Display Derived Values**:
   - The UI displays the current counter value and its derived values (e.g., doubled, quad, fifth).

### Reactive List Section
1. **Add/Remove Items**:
   - Add the current counter value to the list.
   - Remove the last item or clear the entire list.

2. **Display List Items**:
   - The UI displays all items in the list and their sum.

### Debug Panel
- Displays all registered signals and their current values, including:
  - Reactive values (`Value<T>`)
  - Derived values (`Derived<T>`)
  - Reactive lists (`ReactiveList<T>`)

---

## Running the Example

To run the `clock_reactive` example, use the following command:

```bash
cargo run --example clock_reactive