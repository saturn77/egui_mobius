//! Core types and traits for building **egui_mobius_reactive** applications. 
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Subscribers
/// 
/// A vector of boxed Fn() function trait objects wrapped in an Arc<Mutex>.
/// This becomes this vector of **callbacks** that are 
/// triggered when a `Dynamic<T>` or `Derived<T>` changes, or any other
/// reactive type that implements the `ReactiveValue` trait.
/// 
/// Subscribers are very similar to Slots in the Signal-Slot pattern, in that
/// both are used to invoke a callback when a value is change (in the case of Subscribers)
/// or when a signal has been received (in the case of Slots).
///
pub type Subscribers = Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>;

/// Trait implemented by all reactive types (`Dynamic`, `Derived`, `ReactiveList`) 
/// that can be observed for changes.
///
/// This trait provides a common interface for subscribing to changes in reactive types
/// and includes a method for downcasting to the concrete type when needed.
///
/// # Polymorphism and Downcasting
///
/// The `ReactiveValue` trait is designed to be implemented for multiple concrete types,
/// such as `Dynamic<T>`, `Derived<T>`, and `ReactiveList<T>`. These types can be stored
/// as trait objects (e.g., `Box<dyn ReactiveValue>`) to enable polymorphism. However,
/// when you need to access type-specific functionality or data, you can use the `as_any`
/// method to recover the concrete type.
///
/// The `as_any` method returns a reference to the object as `dyn Any`, which allows for
/// type-safe downcasting to the original concrete type.
///
/// # Example
///
/// ```rust
/// use std::any::Any;
/// use egui_mobius_reactive::{ReactiveValue, ReactiveList};
///
/// let list: ReactiveList<i32> = ReactiveList::new();
/// let reactive_value: &dyn ReactiveValue = &list; // Store as a trait object
///
/// // Downcast to the concrete type
/// if let Some(concrete_list) = reactive_value.as_any().downcast_ref::<ReactiveList<i32>>() {
///     println!("Successfully downcasted! Items: {:?}", concrete_list.get_all());
/// } else {
///     println!("Downcast failed!");
/// }
/// ```
pub trait ReactiveValue: Send + Sync {
    /// Subscribes a callback to be triggered when the value changes.
    ///
    /// # Arguments
    /// * `callback` - A boxed function to be called when the value changes.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::{ReactiveValue, ReactiveList};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// list.subscribe(Box::new(|| println!("Value changed!")));
    /// ```
    fn subscribe(&self, callback: Box<dyn Fn() + Send + Sync>);

    /// Returns a reference to the object as `dyn Any`.
    ///
    /// This method enables downcasting from a `ReactiveValue` trait object to its
    /// concrete type. This is useful when you need to access type-specific functionality
    /// or data that is not part of the `ReactiveValue` trait.
    ///
    /// # Polymorphism
    ///
    /// The `ReactiveValue` trait allows for polymorphic behavior by enabling different
    /// reactive types to be treated uniformly. However, when you need to recover the
    /// original concrete type, you can use `as_any` in combination with `downcast_ref`
    /// or `downcast_mut` from the `Any` trait.
    ///
    /// # Example
    ///
    /// ```rust
    /// use egui_mobius_reactive::{ReactiveValue, ReactiveList};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// let any_ref = list.as_any();
    ///
    /// // Downcast to the concrete type
    /// if let Some(concrete_list) = any_ref.downcast_ref::<ReactiveList<i32>>() {
    ///     println!("Successfully downcasted! Items: {:?}", concrete_list.get_all());
    /// } else {
    ///     println!("Downcast failed!");
    /// }
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
/// use egui_mobius_reactive::ReactiveList;
/// let list: ReactiveList<i32> = ReactiveList::new();
/// list.push(42);
/// list.on_change(|| println!("List changed!"));
/// ```
pub struct ReactiveList<T> {
    items       : Arc<Mutex<Vec<T>>>,
    subscribers : Subscribers,
}

impl<T: Clone + Send + Sync + 'static> ReactiveList<T> {
    /// Creates a new empty reactive list.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::ReactiveList;
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
    /// use egui_mobius_reactive::{ReactiveList, ReactiveValue};
    /// let list: ReactiveList<i32> = ReactiveList::new();
    /// list.subscribe(Box::new(|| println!("List changed!")));
    /// ```
    fn subscribe(&self, f: Box<dyn Fn() + Send + Sync>) {
        // Directly pass the function `f` instead of wrapping it in a closure
        self.on_change(f);
    }

    /// Returns a reference to self as `dyn Any`.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::{ReactiveList, ReactiveValue};
    /// let list = ReactiveList::<i32>::new();
    /// let any_ref = list.as_any();
    /// ```
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Removed redundant implementation of Default for ReactiveList<T>

impl<T: Clone + Send + Sync + 'static> Default for ReactiveList<T> {
    /// Creates a default instance of `ReactiveList`.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::ReactiveList;
    /// let list: ReactiveList<i32> = ReactiveList::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}
