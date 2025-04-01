//! Extended Reactive Math Helpers for `egui_mobius_reactive`

use std::sync::Arc;
use std::ops::{Add, Sub, Mul, Div, Not};

use crate::{Derived, Dynamic};

// Math ops for i32 and f64, including mixed-type reactive arithmetic
macro_rules! impl_math_ops {
    ($t:ty) => {
        impl Add for Dynamic<$t> {
            type Output = Derived<$t>;
            fn add(self, rhs: Self) -> Self::Output {
                let a = Arc::new(self);
                let b = Arc::new(rhs);
                Derived::new(&[a.clone(), b.clone()], move || *a.lock() + *b.lock())
            }
        }

        impl Sub for Dynamic<$t> {
            type Output = Derived<$t>;
            fn sub(self, rhs: Self) -> Self::Output {
                let a = Arc::new(self);
                let b = Arc::new(rhs);
                Derived::new(&[a.clone(), b.clone()], move || *a.lock() - *b.lock())
            }
        }

        impl Mul for Dynamic<$t> {
            type Output = Derived<$t>;
            fn mul(self, rhs: Self) -> Self::Output {
                let a = Arc::new(self);
                let b = Arc::new(rhs);
                Derived::new(&[a.clone(), b.clone()], move || *a.lock() * *b.lock())
            }
        }

        impl Div for Dynamic<$t> {
            type Output = Derived<$t>;
            fn div(self, rhs: Self) -> Self::Output {
                let a = Arc::new(self);
                let b = Arc::new(rhs);
                Derived::new(&[a.clone(), b.clone()], move || *a.lock() / *b.lock())
            }
        }
    };
}


impl_math_ops!(i32);

// Mixed-type reactive math support for Dynamic + Derived and vice versa
impl Add<Derived<i32>> for Dynamic<i32> {
    type Output = Derived<i32>;

    fn add(self, rhs: Derived<i32>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() + b.get())
    }
}

impl Add<Dynamic<i32>> for Derived<i32> {
    type Output = Derived<i32>;

    fn add(self, rhs: Dynamic<i32>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || a.get() + *b.lock())
    }
}

// f64: Dynamic + Derived
impl Add<Derived<f64>> for Dynamic<f64> {
    type Output = Derived<f64>;

    fn add(self, rhs: Derived<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() + b.get())
    }
}

impl Add<Dynamic<f64>> for Derived<f64> {
    type Output = Derived<f64>;

    fn add(self, rhs: Dynamic<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || a.get() + *b.lock())
    }
}

// f64: Dynamic - Derived
impl Sub<Derived<f64>> for Dynamic<f64> {
    type Output = Derived<f64>;

    fn sub(self, rhs: Derived<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() - b.get())
    }
}

impl Sub<Dynamic<f64>> for Derived<f64> {
    type Output = Derived<f64>;

    fn sub(self, rhs: Dynamic<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || a.get() - *b.lock())
    }
}

// f64: Dynamic * Derived
impl Mul<Derived<f64>> for Dynamic<f64> {
    type Output = Derived<f64>;

    fn mul(self, rhs: Derived<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() * b.get())
    }
}

impl Mul<Dynamic<f64>> for Derived<f64> {
    type Output = Derived<f64>;

    fn mul(self, rhs: Dynamic<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || a.get() * *b.lock())
    }
}

// f64: Dynamic / Derived
impl Div<Derived<f64>> for Dynamic<f64> {
    type Output = Derived<f64>;

    fn div(self, rhs: Derived<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || {
            let denom = b.get();
            if denom == 0.0 {
                eprintln!("⚠️ Division by zero in f64 Derived");
                0.0
            } else {
                *a.lock() / denom
            }
        })
    }
}

impl Div<Dynamic<f64>> for Derived<f64> {
    type Output = Derived<f64>;

    fn div(self, rhs: Dynamic<f64>) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || a.get() / *b.lock())
    }
}


// Boolean negation
impl Not for Dynamic<bool> {
    type Output = Derived<bool>;
    fn not(self) -> Self::Output {
        let a = Arc::new(self);
        Derived::new(&[a.clone()], move || !*a.lock())
    }
}

// String concat
impl Add for Dynamic<String> {
    type Output = Derived<String>;
    fn add(self, rhs: Self) -> Self::Output {
        let a = Arc::new(self);
        let b = Arc::new(rhs);
        Derived::new(&[a.clone(), b.clone()], move || format!("{}{}", *a.lock(), *b.lock()))
    }
}

// ReactiveMath for i32
pub trait ReactiveMath {
    fn doubled(&self) -> Derived<i32>;
    fn negated(&self) -> Derived<i32>;
    fn powi(&self, exp: u32) -> Derived<i32>;
    fn abs(&self) -> Derived<i32>;
    fn min(&self, other: &Dynamic<i32>) -> Derived<i32>;
    fn max(&self, other: &Dynamic<i32>) -> Derived<i32>;
    fn rem(&self, other: &Dynamic<i32>) -> Derived<i32>;
}

impl ReactiveMath for Dynamic<i32> {
    fn doubled(&self) -> Derived<i32> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || *a.lock() * 2)
    }

    fn negated(&self) -> Derived<i32> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || -*a.lock())
    }

    fn powi(&self, exp: u32) -> Derived<i32> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || a.lock().pow(exp))
    }

    fn abs(&self) -> Derived<i32> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || a.lock().abs())
    }

    fn min(&self, other: &Dynamic<i32>) -> Derived<i32> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || a.lock().min(*b.lock()))
    }

    fn max(&self, other: &Dynamic<i32>) -> Derived<i32> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || a.lock().max(*b.lock()))
    }

    fn rem(&self, other: &Dynamic<i32>) -> Derived<i32> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() % *b.lock())
    }
}

// ReactiveMathF64 for f64
pub trait ReactiveMathF64 {
    fn powf(&self, exp: f64) -> Derived<f64>;
    fn abs(&self) -> Derived<f64>;
    fn min(&self, other: &Dynamic<f64>) -> Derived<f64>;
    fn max(&self, other: &Dynamic<f64>) -> Derived<f64>;
    fn rem(&self, other: &Dynamic<f64>) -> Derived<f64>;
}

impl ReactiveMathF64 for Dynamic<f64> {
    fn powf(&self, exp: f64) -> Derived<f64> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || a.lock().powf(exp))
    }

    fn abs(&self) -> Derived<f64> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || a.lock().abs())
    }

    fn min(&self, other: &Dynamic<f64>) -> Derived<f64> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || a.lock().min(*b.lock()))
    }

    fn max(&self, other: &Dynamic<f64>) -> Derived<f64> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || a.lock().max(*b.lock()))
    }

    fn rem(&self, other: &Dynamic<f64>) -> Derived<f64> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || *a.lock() % *b.lock())
    }
}

// ReactiveList Sum Extension

pub trait ReactiveListSum<T: Clone + Send + Sync + 'static> {
    fn sum(&self) -> Derived<T>;
}

impl ReactiveListSum<i32> for crate::ReactiveList<i32> {
    fn sum(&self) -> Derived<i32> {
        let list = Arc::new(self.clone());
        Derived::new(&[list.clone()], move || list.get_all().iter().copied().sum())
    }
}

impl ReactiveListSum<f64> for crate::ReactiveList<f64> {
    fn sum(&self) -> Derived<f64> {
        let list = Arc::new(self.clone());
        Derived::new(&[list.clone()], move || list.get_all().iter().copied().sum())
    }
}

// Logic and String helpers
pub trait ReactiveLogic {
    fn not(&self) -> Derived<bool>;
}

impl ReactiveLogic for Dynamic<bool> {
    fn not(&self) -> Derived<bool> {
        let a = Arc::new(self.clone());
        Derived::new(&[a.clone()], move || !*a.lock())
    }
}

pub trait ReactiveString {
    fn append(&self, other: &Dynamic<String>) -> Derived<String>;
}

impl ReactiveString for Dynamic<String> {
    fn append(&self, other: &Dynamic<String>) -> Derived<String> {
        let a = Arc::new(self.clone());
        let b = Arc::new(other.clone());
        Derived::new(&[a.clone(), b.clone()], move || format!("{}{}", *a.lock(), *b.lock()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_math_extensions() {
        let a = Dynamic::new(5);
        let b = Dynamic::new(3);

        assert_eq!(a.doubled().get(), 10);
        assert_eq!(a.negated().get(), -5);
        assert_eq!(a.powi(3).get(), 125);
        assert_eq!(a.abs().get(), 5);
        assert_eq!(a.min(&b).get(), 3);
        assert_eq!(a.max(&b).get(), 5);
        assert_eq!(a.rem(&b).get(), 2);
    }

    #[test]
    fn test_f64_math_extensions() {
        let x = Dynamic::new(-2.5);
        let y = Dynamic::new(3.0);

        assert_eq!(x.powf(2.0).get(), 6.25);
        assert_eq!(x.abs().get(), 2.5);
        assert_eq!(x.min(&y).get(), -2.5);
        assert_eq!(x.max(&y).get(), 3.0);
        assert_eq!(y.rem(&x).get(), 0.5);
    }

    #[test]
    fn test_boolean_not() {
        let flag = Dynamic::new(true);
        assert_eq!((!flag).get(), false);
    }

    #[test]
    fn test_string_add() {
        let a = Dynamic::new("Hello, ".to_string());
        let b = Dynamic::new("world!".to_string());
        let result = a + b;
        assert_eq!(result.get(), "Hello, world!");
    }

    #[test]
    fn test_string_append() {
        let a = Dynamic::new("foo".to_string());
        let b = Dynamic::new("bar".to_string());
        let result = a.append(&b);
        assert_eq!(result.get(), "foobar");
    }
    #[test]
    fn test_mixed_type_i32_math() {
        let dyn_val = Dynamic::new(7);
        let derived_val = dyn_val.doubled();

        let sum = dyn_val.clone() + derived_val.clone();
        let alt_sum = derived_val + dyn_val.clone();

        assert_eq!(sum.get(), 21);
        assert_eq!(alt_sum.get(), 21);
    }

    #[test]
    fn test_mixed_type_f64_math() {
        let a = Dynamic::new(2.0);
        let _b = Dynamic::new(4.0);

        let d = a.clone().powf(2.0); // 4.0

        let sum = a.clone() + d.clone();     // 2.0 + 4.0 = 6.0
        let diff = d.clone() - a.clone();    // 4.0 - 2.0 = 2.0
        let prod = a.clone() * d.clone();    // 2.0 * 4.0 = 8.0
        let quot = d.clone() / a.clone();    // 4.0 / 2.0 = 2.0

        assert_eq!(sum.get(), 6.0);
        assert_eq!(diff.get(), 2.0);
        assert_eq!(prod.get(), 8.0);
        assert_eq!(quot.get(), 2.0);
    }
    #[test]
    fn test_mixed_type_i32_min_max_rem() {
        let a = Dynamic::new(10);
        let b = Dynamic::new(-a.get());

        let min = a.clone().min(&b.clone());
        let max = a.clone().max(&b.clone());
        let rem = a.clone().rem(&b.clone());

        assert_eq!(min.get(), -10);
        assert_eq!(max.get(), 10);
        assert_eq!(rem.get(), 0); // 10 % -10 = 0
    }

    #[test]
    fn test_mixed_type_f64_min_max_rem() {
        let x = Dynamic::new(9.0);
        let y = Dynamic::new(-x.get());

        let min = x.clone().min(&y.clone());
        let max = x.clone().max(&y.clone());
        let rem = x.clone().rem(&y.clone());

        assert_eq!(min.get(), -9.0);
        assert_eq!(max.get(), 9.0);
        assert_eq!(rem.get(), 0.0);
    }



    #[test]
    fn test_reactive_logic_trait() {
        let val = Dynamic::new(false);
        let toggled = val.not();
        assert_eq!(toggled.get(), true);
    }
}
