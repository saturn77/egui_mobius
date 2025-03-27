use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc;
use tokio::sync::Notify;
use futures::FutureExt;

use crate::slot::Slot;
use crate::signals::Signal;

#[derive(Debug, Clone)]
pub enum Processed {
    Loading(String),
    Success(String),
}

pub type DynEventHandler<E> = Arc<dyn Fn(E) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Trait for routing events to the appropriate handler by string key
pub trait EventRoute {
    fn route(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Idle,
    Running,
    ShuttingDown,
    Done,
}

pub struct MobiusHandle<E> {
    sender: crate::signals::Signal<E>,
    shutdown: Arc<Notify>,
}

impl<E> Clone for MobiusHandle<E> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

impl<E: Send + 'static> MobiusHandle<E> {
    pub async fn send(&self, event: E) {
        self.sender.send(event);
        tokio::task::yield_now().await;
    }

    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }
}

pub struct MobiusRuntime<E: Send + Clone + 'static> {
    slot: Slot<E>,
    handlers: HashMap<String, DynEventHandler<E>>,
    processed: mpsc::Sender<Processed>,
    state: RuntimeState,
    shutdown_notify: Arc<Notify>,
}

enum Event {
    Add(i32),
    Sub(i32),
}

impl EventRoute for Event {
    fn route(&self) -> &str {
        match self {
            Event::Add(_) => "add",
            Event::Sub(_) => "sub",
        }
    }
}

impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::Add(x) => Event::Add(*x),
            Event::Sub(x) => Event::Sub(*x),
        }
    }
}

impl<E: EventRoute + Send + Clone + 'static> MobiusRuntime<E> {
    pub fn new() -> (Self, MobiusHandle<E>, mpsc::Receiver<Processed>) {
        let (processed_tx, processed_rx) = mpsc::channel();
        let (signal, slot) = crate::factory::create_signal_slot::<E>();
        let shutdown_notify = Arc::new(Notify::new());

        let runtime = Self {
            slot,
            handlers: HashMap::new(),
            processed: processed_tx,
            state: RuntimeState::Idle,
            shutdown_notify: shutdown_notify.clone(),
        };

        let handle = MobiusHandle {
            sender: signal,
            shutdown: shutdown_notify,
        };
        (runtime, handle, processed_rx)
    }

    pub fn register_handler<Fut>(&mut self, route: &str, handler: impl Fn(E) -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.handlers
            .insert(route.to_string(), Arc::new(move |e| Box::pin(handler(e))));
    }

    pub async fn run(mut self) {
        self.state = RuntimeState::Running;
        let processed_tx = self.processed.clone();
        let handlers = Arc::new(self.handlers);
        let shutdown = self.shutdown_notify.clone();

        let mut slot = self.slot;
        slot.start_async(move |event| {
            let handlers = handlers.clone();
            let processed_tx = processed_tx.clone();
            let shutdown = shutdown.clone();
            async move {
                // Check if shutdown was requested
                if shutdown.notified().now_or_never().is_some() {
                    return;
                }
                let route = event.route().to_string();
                if let Some(handler) = handlers.get(&route) {
                    let route_clone = route.clone();
                    let _ = processed_tx.send(Processed::Loading(route_clone.clone()));
                    handler(event).await;
                    let _ = processed_tx.send(Processed::Success(route_clone));
                }
            }
        });

        // Give the slot time to start processing
        tokio::task::yield_now().await;

        self.shutdown_notify.notified().await;
        self.state = RuntimeState::Done;
        
        // Drop the slot to close the channel
        drop(slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    const TEST_TIMEOUT: Duration = Duration::from_secs(1);
    const PING_ROUTE: &str = "ping";
    const PONG_RESPONSE: &str = "pong";

    #[derive(Clone)]
    enum TestEvent {
        Ping,
        Message(String),
    }

    impl EventRoute for TestEvent {
        fn route(&self) -> &str {
            match self {
                TestEvent::Ping => PING_ROUTE,
                TestEvent::Message(_) => "message",
            }
        }
    }

    #[tokio::test]
    async fn should_successfully_handle_ping_event() {
        let (mut runtime, handle, _processed_rx) = MobiusRuntime::new();

        let pong_flag = Arc::new(tokio::sync::Notify::new());
        let pong_notify = pong_flag.clone();

        runtime.register_handler(PING_ROUTE, move |_event| {
            let pong_notify = pong_notify.clone();
            async move {
                pong_notify.notify_one();
            }
        });

        let rt = tokio::spawn(runtime.run());

        // Give runtime time to start
        tokio::task::yield_now().await;

        // Send ping and wait for response
        handle.send(TestEvent::Ping).await;
        let notified = time::timeout(TEST_TIMEOUT, pong_flag.notified()).await;
        assert!(notified.is_ok(), "Ping handler did not complete in time");

        // Ensure clean shutdown
        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await; // Give time for cleanup
        let _ = rt.abort(); // Force abort the runtime task
    }

    #[tokio::test]
    async fn should_handle_unregistered_message_gracefully() {
        let (runtime, handle, _processed_rx) = MobiusRuntime::new();
        let rt = tokio::spawn(runtime.run());
        handle.send(TestEvent::Message("hi".into())).await;
        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = rt.abort();
    }
}
