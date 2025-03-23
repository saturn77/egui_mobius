use std::any::Any;
use std::sync::{Arc, Mutex};

/// Trait implemented by all reactive types (`Value`, `Derived`, `ReactiveList`) that can be observed for changes.
pub trait ReactiveValue: Send + Sync {
    /// Subscribes a callback to be triggered when the value changes.
    ///
    /// # Arguments
    /// * `callback` - A boxed function to be called when the value changes.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::{ReactiveValue, ReactiveList};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// list.subscribe(Box::new(|| println!("Value changed!")));
    /// ```
    fn subscribe(&self, callback: Box<dyn Fn() + Send + Sync>);

    /// Returns a reference to the object as `dyn Any`.
    ///
    /// This allows for downcasting to the concrete type.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::{ReactiveValue, ReactiveList};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// let any_ref = list.as_any();
    /// ```
    fn as_any(&self) -> &dyn Any;
}

/// A reactive list that notifies subscribers when items are added, removed, or cleared.
///
/// Each modification to the internal `Vec<T>` triggers all registered callbacks.
/// This is useful for binding list updates to UI refreshes or effects.
///
/// # Example
/// ```rust
/// use egui_mobius_reactive::reactive::ReactiveList;
/// let list: ReactiveList<i32> = ReactiveList::new();
/// list.push(42);
/// list.on_change(|| println!("List changed!"));
/// ```
pub struct ReactiveList<T> {
    items: Arc<Mutex<Vec<T>>>,
    subscribers: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl<T: Clone + Send + Sync + 'static> ReactiveList<T> {
    /// Creates a new empty reactive list.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// ```
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Pushes an item to the end of the list and notifies subscribers.
    ///
    /// # Arguments
    /// * `item` - The item to add to the list.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new(); // Specify the type explicitly
    /// list.push(42);
    /// ```
    pub fn push(&self, item: T) {
        self.items.lock().unwrap().push(item);
        self.notify_subscribers();
    }

    /// Removes the item at the specified index and notifies subscribers.
    ///
    /// # Arguments
    /// * `index` - The index of the item to remove.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new(); // Specify the type explicitly
    /// list.push(42);
    /// list.remove(0);
    /// ```
    pub fn remove(&self, index: usize) {
        self.items.lock().unwrap().remove(index);
        self.notify_subscribers();
    }

    /// Clears all items from the list and notifies subscribers.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new(); // Specify the type explicitly
    /// list.push(42);
    /// list.clear();
    /// ```
    pub fn clear(&self) {
        self.items.lock().unwrap().clear();
        self.notify_subscribers();
    }

    /// Returns a cloned copy of the entire list.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list = ReactiveList::new();
    /// list.push(42);
    /// let items = list.get_all();
    /// assert_eq!(items, vec![42]);
    /// ```
    pub fn get_all(&self) -> Vec<T> {
        self.items.lock().unwrap().clone()
    }

    /// Registers a callback to be called when the list changes.
    ///
    /// # Arguments
    /// * `f` - The callback function to register.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new(); // Specify the type explicitly
    /// list.on_change(|| println!("List changed!"));
    /// list.push(42); // This will trigger the callback
    /// ```
    pub fn on_change(&self, f: impl Fn() + Send + Sync + 'static) {
        self.subscribers.lock().unwrap().push(Box::new(f));
    }

    /// Notifies all registered subscribers.
    ///
    /// This method is called internally whenever the list is modified.
    fn notify_subscribers(&self) {
        for f in self.subscribers.lock().unwrap().iter() {
            f();
        }
    }
}

impl<T> Clone for ReactiveList<T> {
    /// Creates a deep clone of the `ReactiveList`.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// let cloned_list = list.clone();
    /// ```
    fn clone(&self) -> Self {
        Self {
            items: Arc::clone(&self.items),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> ReactiveValue for ReactiveList<T> {
    /// Subscribes to list changes using the `ReactiveValue` trait.
    ///
    /// # Arguments
    /// * `f` - The callback function to register.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::{ReactiveList, ReactiveValue};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// list.subscribe(Box::new(|| println!("List changed!")));
    /// ```
    fn subscribe(&self, f: Box<dyn Fn() + Send + Sync>) {
        self.on_change(move || f());
    }

    /// Returns a reference to self as `dyn Any`.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::reactive::{ReactiveList, ReactiveValue};
    /// let list = ReactiveList::<i32>::new();
    /// let any_ref = list.as_any();
    /// ```
    fn as_any(&self) -> &dyn Any {
        self
    }
}
