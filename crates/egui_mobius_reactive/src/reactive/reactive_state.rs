use std::sync::{Arc, Mutex, Weak};

/// A reactive reference used by widgets.
/// Wraps a `Weak<Mutex<T>>` to avoid `Arc` duplication.
///
/// Use `get_live()` to safely clone the current state.
/// Use `get_cached()` only when widget performance matters.
pub struct ReactiveWidgetRef<T> {
    pub weak_ref: Weak<Mutex<T>>,
    pub cached_value: Option<T>,
    pub modified: bool,
}

impl<T: Clone> Clone for ReactiveWidgetRef<T> {
    fn clone(&self) -> Self {
        Self {
            weak_ref: self.weak_ref.clone(),
            cached_value: self.cached_value.clone(),
            modified: self.modified,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> ReactiveWidgetRef<T> {
    /// Create a widget ref from a `Dynamic<T>`
    pub fn from_dynamic(dynamic: &crate::reactive::dynamic::Dynamic<T>) -> Self {
        Self {
            weak_ref: Arc::downgrade(&dynamic.inner),
            cached_value: None,
            modified: false,
        }
    }

    /// Return a fresh cloned value by locking the underlying data
    pub fn get_live(&self) -> Option<T> {
        if let Some(arc) = self.weak_ref.upgrade() {
            match arc.lock() {
                Ok(guard) => Some(guard.clone()),
                Err(poisoned) => Some(poisoned.into_inner().clone()),
            }
        } else {
            None
        }
    }

    /// Mutate the underlying state directly via closure (safe, avoids stale cache)
    pub fn with_live_mut<F: FnOnce(&mut T)>(&self, f: F) -> bool {
        if let Some(arc) = self.weak_ref.upgrade() {
            if let Ok(mut guard) = arc.lock() {
                f(&mut guard);
                return true;
            }
        }
        false
    }

    /// Cached value accessor (for performance-sensitive use like sliders)
    pub fn get_cached(&mut self) -> Option<T> {
        if self.cached_value.is_none() {
            if let Some(arc) = self.weak_ref.upgrade() {
                if let Ok(guard) = arc.lock() {
                    self.cached_value = Some(guard.clone());
                }
            }
        }
        self.cached_value.clone()
    }

    /// Refresh the cached value manually
    pub fn refresh_cache(&mut self) {
        self.cached_value = self.get_live();
    }

    /// Set value and update cache
    pub fn set(&mut self, new_value: T) {
        if let Some(arc) = self.weak_ref.upgrade() {
            if let Ok(mut guard) = arc.lock() {
                *guard = new_value.clone();
                self.cached_value = Some(new_value);
                self.modified = true;
            }
        }
    }
}
