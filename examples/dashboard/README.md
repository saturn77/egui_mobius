# Dashboard with egui_mobius

## Overview

This Rust-based application demonstrates an interactive dashboard UI built with `egui_mobius`. It features asynchronous backend event processing, real-time logging, and efficient state management using Rust's concurrency primitives.

## Features

- Increment and reset a numeric counter via UI buttons.
- Real-time, thread-safe logging of UI and backend events.
- Filterable logs panel (by UI or backend).
- Scrollable and selectable logs panel with text copying.
- Persistent log storage (`ui_session_log.txt`).

## Architecture

- **UI (`egui`)**: Handles user interactions and sends events.
- **Event and Response Messaging**: Asynchronous communication between frontend and backend using `Signal<Event>` and `Slot<Response>`.
- **Backend Thread**: Processes events, manages application state, and provides logs.
- **State Management**: `AppState` wrapped safely in `egui_mobius::types::Value<T>` (thread-safe, mutex-protected, and reference-counted).

The system implements a reactive UI that communicates with a background thread using signals and slots to handle application state updates.

---

## UML Diagram – UI Layer and Messaging Interface

```mermaid
classDiagram
    direction LR

    %% Messaging Types
    class Event {
        <<enum>>
        + IncrementCounter
        + ResetCounter
        + Custom(String)
    }

    class Response {
        <<enum>>
        + CounterUpdated(usize)
        + Message(String)
    }

    %% Signal/Slot
    class Signal~T~ {
        + send(value: T)
    }

    class Slot~T~ {
        + start(handler: Fn(T))
    }

    %% UI
    class UiApp {
        - Value<AppState> state
        - Signal<Event> event_signal
        + new(event_signal, response_slot)
        + log(message)
        + update(ctx, frame)
    }

    class AppState {
        + DashboardState dashboard
        + Signal<Event> event_signal
        + Vec<LogEntry> logs
        + Vec<String> log_filters
        + new(event_signal)
        + log(source, message)
        + handle_response(response)
    }

    class DashboardState {
        + usize counter
        + handle_response(response)
    }

    %% Logging
    class LogEntry {
        + DateTime timestamp
        + String source
        + String message
        + formatted(): String
    }

    %% Relationships
    UiApp --> AppState
    UiApp --> Signal~Event~
    UiApp --> Slot~Response~
    AppState --> DashboardState
    AppState --> Signal~Event~
    AppState --> LogEntry
    LogEntry --> DateTime
    Signal~Event~ --> Event
    Slot~Response~ --> Response
```

---

## UML Diagram – Backend Thread and Event Processing

```mermaid
classDiagram
    direction LR

    %% Backend Component
    class Backend {
        + run_backend(event_slot, response_signal)
        + process(event): Response
    }

    %% Messaging (shared)
    class Event {
        <<enum>>
        + IncrementCounter
        + ResetCounter
        + Custom(String)
    }

    class Response {
        <<enum>>
        + CounterUpdated(usize)
        + Message(String)
    }

    %% Signal/Slot
    class Signal~Response~ {
        + send(value: Response)
    }

    class Slot~Event~ {
        + start(handler: Fn(Event))
    }

    %% Relationships
    Backend --> Slot~Event~
    Backend --> Signal~Response~
    Slot~Event~ --> Event
    Signal~Response~ --> Response
    Backend --> Event
    Backend --> Response
```

---

## Flowchart of Design

```mermaid
flowchart TD

    %% Nodes — UI Side
    A("fa:fa-mouse-pointer User Click")
    B("fa:fa-signal Signal<Event>")
    C("fa:fa-code UiApp")
    D("fa:fa-database AppState")
    E("fa:fa-terminal Slot<Response>")
    F("fa:fa-eye Log Panel")

    %% Nodes — Backend
    G("fa:fa-code Backend Thread")
    H("fa:fa-inbox Slot<Event>")
    I("fa:fa-microchip process(event)")
    J("fa:fa-signal Signal<Response>")

    %% Types
    K("fa:fa-cube Event")
    L("fa:fa-cube Response")

    %% Connections
    A --> B
    B --> H
    H --> G
    G --> I
    I --> J
    J --> E
    E --> D
    D --> F

    %% Type refs
    K -.-> I
    L -.-> D

    %% Styles
    style C fill:#2962FF,stroke:#2962FF,color:#fff
    style D fill:#304FFE,stroke:#304FFE,color:#fff
    style G fill:#00BFA5,stroke:#00BFA5,color:#fff
    style I fill:#00BFA5,stroke:#00BFA5,color:#fff
    style F fill:#AA00FF,stroke:#AA00FF,color:#fff
    style E fill:#FF6D00,stroke:#FF6D00,color:#fff
```

---

## Features Recap

- Increment and reset counter with immediate backend response.
- Log panel with filter by source (`ui` or `backend`).
- Scrollable, selectable logs supporting copy-paste.

## Running the Application

```bash
cargo run -p dashboard
```

## Dependencies

- `eframe`
- `egui_mobius`
- `chrono`
- `lazy_static`
- `log`
- `env_logger`

## Log Persistence

Logs are automatically persisted to `ui_session_log.txt`.