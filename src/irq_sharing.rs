//! Types for handling data that needs to be moved into an interrupt handler or
//! thread.
//!
//! Written by Ferrous Systems.
//!
//! They are designed to work with the
//! [`cortex-m`](https://crates.io/crates/cortex-m) `#[interrupt]` macro.
//!
//! ```rust, ignore
//! use grounded::irq_sharing::{Global, Local};
//! static GLOBAL_UART0: Global<Uart0> = Global::empty();
//!
//! #[entry]
//! fn main() -> {
//!     let p = hal::init();
//!     GLOBAL_UART0.load(p.uart);
//!     p.enable_uart_interrupt();
//!     loop {
//!         // Do your main thread stuff
//!     }
//! }
//!
//! #[interrupt]
//! fn UART0 {
//!     // This is re-written to be safe by the #[interrupt] attribute
//!     static mut UART0: Local<Uart0> = Local::empty();
//!     
//!     let uart0 = UART0.get_or_init(&GLOBAL_UART0);
//!     
//!     // can use uart0 here safely, knowing no other code has access
//!     // to this object.
//! }
//! ```

/// The global type for sharing things with an interrupt handler
pub struct Global<T> {
    inner: critical_section::Mutex<core::cell::RefCell<Option<T>>>,
}

impl<T> Global<T> {
    /// Create a new, empty, object
    pub const fn empty() -> Global<T> {
        Global {
            inner: critical_section::Mutex::new(core::cell::RefCell::new(None)),
        }
    }

    /// Load a value into the global
    ///
    /// Returns the old value, if any
    pub fn load(&self, value: T) -> Option<T> {
        critical_section::with(|cs| self.inner.borrow(cs).replace(Some(value)))
    }
}

/// The local type for sharing things with an interrupt handler
pub struct Interrupt<T> {
    inner: Option<T>,
}

impl<T> Interrupt<T> {
    /// Create a new, empty, object
    pub const fn empty() -> Interrupt<T> {
        Interrupt { inner: None }
    }

    /// Grab a mutable reference to the contents.
    ///
    /// If the value is empty, the contents are taken from a mutex-locked global
    /// variable. That global must have been initialised before calling this
    /// function. If not, this function panics.
    pub fn get_or_init_with(&mut self, global: &Global<T>) -> &mut T {
        let result = self.inner.get_or_insert_with(|| {
            critical_section::with(|cs| global.inner.borrow(cs).replace(None).unwrap())
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_not_sync() {
        // Is Send but !Sync
        struct Racy {
            inner: core::cell::Cell<u8>,
        }

        // Initialisation
        static GLOBAL_TEST: Global<Racy> = Global::empty();
        let racy = Racy {
            inner: core::cell::Cell::new(0),
        };
        GLOBAL_TEST.load(racy);
        // Usage - we don't have the static mut re-write here so use a stack
        // variable
        let mut local: Interrupt<Racy> = Interrupt::empty();
        let local_ref = local.get_or_init_with(&GLOBAL_TEST);
        assert_eq!(local_ref.inner.get(), 0);
    }
}

// End of file
