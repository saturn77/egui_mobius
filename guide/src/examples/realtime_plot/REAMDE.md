# egui_mobius - Real-Time Plotting Example

## 🛠️ Running the Example
Ensure you have **Rust and Cargo installed**, then run:
```sh
cargo run -p realtime_plot
```

## 🚀 Why This Example Matters
This example serves as a reference implementation for real-time data visualization in `egui_mobius`.

- **Best practices for using signals and slots, producer-consumer models.**
- **Proper separation of frontend and backend logic.**
- **Maintainable, scalable code.**
- **Optimized data handling using FIFO history buffers.**
- **Scalable design for other embedded applications.**

## 📖 Overview

This example shows how to use egui_mobius to provide the framework for an egui application that uses `egui_plot` crate to perform real time plotting. Usually the real
time data would come from a serial-usb or ethernet connected device, but in this example 
there is a background producer thread to create the data. 

This example is easily extended to developing a range of applications, for example, a 
typical serial monitor that would be communicating with a micro-controller or fpga. 

## ⚡ Architecture
This example follows a structured **frontend-backend model** using **signals and slots** for inter-thread communication.

### 🎛️ Frontend (`App`)
- Handles UI rendering and user input (sliders for temperature control).
- Sends updated temperature values to the backend via `Signal<Event>`.
- Receives processed data updates from the backend via `Slot<Event>`.
- Plots real-time temperature data using `egui_plot` with a legend.

### 🔄 Backend (`producer_thread` & `consumer_thread`)
- Producer Thread generates temperature data every second.
- Consumer Thread processes incoming updates and manages the **FIFO buffer**.
- Maintains historical temperature data** (last 300 values).


## 📂 Code Structure
```
examples/real_time_temperature/
│── main.rs          # Main entry point defining frontend and backend logic
│── README.md        # This file
│── Cargo.toml       # Dependencies and build configurations
```



## 🌟 Key Components

### 🔢 Event System (`Event`)
Defines **temperature update messages**, ensuring structured communication.

```rust
#[derive(Debug, Clone)]
enum Event {
    DataUpdated { inlet: f64, exhaust: f64, ambient: f64 },
}
```

---

### 🛠️ Fabric Context (`Fabric`)
Stores **shared application data**, acting as a structured data layer for UI updates.

```rust
struct Fabric {
    inlet_temp      : Value<f64>,
    exhaust_temp    : Value<f64>,
    ambient_temp    : Value<f64>,
    inlet_history   : Value<Vec<f64>>,
    exhaust_history : Value<Vec<f64>>,
    ambient_history : Value<Vec<f64>>,
    y_bounds        : Value<(f64, f64)>,
}
```

---

### 🛠️ Frontend - `App`
Manages UI interactions and plotting.

```rust
struct App {
    fabric_data  : Fabric,
    ui_signal    : Signal<Event>,
    ui_slot      : Slot<Event>,
}
```

---

### 🛠️ Backend - `producer_thread`
Generates temperature data every second.

```rust
fn producer_thread(signal: Signal<Event>, fabric_data: &Fabric) {
    thread::spawn(move || {
        loop {
            let inlet_val = *fabric_data.inlet_temp.lock().unwrap();
            let exhaust_val = *fabric_data.exhaust_temp.lock().unwrap();
            let ambient_val = *fabric_data.ambient_temp.lock().unwrap();

            if signal.send(Event::DataUpdated { inlet: inlet_val, exhaust: exhaust_val, ambient: ambient_val }).is_err() {
                eprintln!("Failed to send data update from producer.");
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}
```

---

### 🛠️ Backend - `consumer_thread`
Processes received temperature updates and manages history.

```rust
fn consumer_thread(mut slot: Slot<Event>, fabric_data: &Fabric) {
    slot.start(move |event| {
        let Event::DataUpdated { inlet: new_inlet, exhaust: new_exhaust, ambient: new_ambient } = event;

        *fabric_data.inlet_temp.lock().unwrap() = new_inlet;
        *fabric_data.exhaust_temp.lock().unwrap() = new_exhaust;
        *fabric_data.ambient_temp.lock().unwrap() = new_ambient;

        append_and_maintain_fifo!(fabric_data.inlet_history.lock().unwrap(), new_inlet, 300);
        append_and_maintain_fifo!(fabric_data.exhaust_history.lock().unwrap(), new_exhaust, 300);
        append_and_maintain_fifo!(fabric_data.ambient_history.lock().unwrap(), new_ambient, 300);
    });
}
```

---

### 📊 Real-Time Plotting
The UI dynamically plots **three temperature lines** (inlet, exhaust, ambient) with **color coding and a legend**.

```rust
Plot::new("temp_plot")
    .view_aspect(2.0)
    .legend(Legend::default())
    .show(ui, |plot_ui| {
        let inlet_points: PlotPoints = fabric_data.inlet_history.lock().unwrap()
            .iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();
        let exhaust_points: PlotPoints = fabric_data.exhaust_history.lock().unwrap()
            .iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();
        let ambient_points: PlotPoints = fabric_data.ambient_history.lock().unwrap()
            .iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();

        plot_ui.line(Line::new(inlet_points).name("Inlet Temp (°C)").color(egui::Color32::RED));
        plot_ui.line(Line::new(exhaust_points).name("Exhaust Temp (°C)").color(egui::Color32::BLUE));
        plot_ui.line(Line::new(ambient_points).name("Ambient Temp (°C)").color(egui::Color32::GREEN));
    });
```

---

## 🛠️ Signals & Slots Workflow
| **Step** | **Action** |
|----------|-----------|
| 1️⃣ User interacts with UI | **Updates inlet, exhaust, or ambient temperature via sliders.** |
| 2️⃣ UI sends event to backend | **`Signal<Event>::send(Event::DataUpdated { ... })`** |
| 3️⃣ Backend processes event | **Updates stored values and manages history.** |
| 4️⃣ UI receives processed data | **Plots updated temperature trends.** |


## 📚 License
This example is part of the `egui_mobius` project and is available under the **MIT License**.


### 🚀 Get Started & Contribute
Contributions are welcome! If you find an issue or have an idea for improvement, feel free to submit a PR.


