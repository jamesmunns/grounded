//! Helpers for dealing with statics that are (potentially) uninitialized at the
//! start of a program.

use core::{cell::UnsafeCell, mem::MaybeUninit};

/// ## GroundedCell
///
/// [GroundedCell] is a type that contains a single `T`. The contained T is wrapped
/// with:
///
/// * An [UnsafeCell] - as synchronization *must* be provided by the wrapping user
/// * A [MaybeUninit] - as the contents will not be initialized at program start.
///
/// This type is intended to be used as a building block for other types, such as
/// runtime initialized constants, data within uninitialized memory/linker sections,
/// or similar.
///
/// This type may be used to provide inner mutability, when accessed through the
/// [GroundedCell::get()] interface.
///
/// [GroundedCell] is also `#[repr(transparent)], as are `UnsafeCell` and `MaybeUninit`,
/// which means that it will have the same layout and alignment as `T`.
#[repr(transparent)]
pub struct GroundedCell<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Sync> Sync for GroundedCell<T> {}

impl<T> GroundedCell<T> {
    /// Create an uninitialized `GroundedCell`.
    ///
    /// ```rust
    /// use grounded::uninit::GroundedCell;
    ///
    /// static EXAMPLE: GroundedCell<u32> = GroundedCell::uninit();
    /// ```
    pub const fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Obtain a mutable pointer to the contained T.
    ///
    /// No claims are made on the validity of the T (it may be invalid or uninitialized),
    /// and the caller is required to guarantee synchronization of access, e.g. guaranteeing
    /// that access is shared XOR mutable for the duration of any references created from this
    /// pointer.
    ///
    /// ```rust
    /// use grounded::uninit::GroundedCell;
    /// static EXAMPLE: GroundedCell<u32> = GroundedCell::uninit();
    ///
    /// let ptr: *mut u32 = EXAMPLE.get();
    /// assert_ne!(core::ptr::null_mut(), ptr);
    /// ```
    pub fn get(&'static self) -> *mut T {
        let mu_ptr: *mut MaybeUninit<T> = self.inner.get();
        let t_ptr: *mut T = mu_ptr.cast::<T>();
        t_ptr
    }
}

/// ## GroundedArrayCell
///
/// [GroundedArrayCell] is a type that contains a contiguous array of `[T; N]`.
/// The contained [T; N] is wrapped with:
///
/// * An [UnsafeCell] - as synchronization *must* be provided by the wrapping user
/// * A [MaybeUninit] - as the contents will not be initialized at program start.
///
/// This type is intended to be used as a building block for other types, such as
/// runtime initialized constants, data within uninitialized memory/linker sections,
/// or similar.
///
/// This type may be used to provide inner mutability, when accessed through the
/// [GroundedArrayCell::get_ptr_len()] interface.
///
/// [GroundedArrayCell] is also `#[repr(transparent)], as are `UnsafeCell` and `MaybeUninit`,
/// which means that it will have the same layout and alignment as `[T; N]`.
#[repr(transparent)]
pub struct GroundedArrayCell<T, const N: usize> {
    inner: UnsafeCell<MaybeUninit<[T; N]>>,
}

unsafe impl<T: Sync, const N: usize> Sync for GroundedArrayCell<T, N> {}

impl<T, const N: usize> GroundedArrayCell<T, N> {
    /// Create an uninitialized `GroundedArrayCell`.
    ///
    /// ```rust
    /// use grounded::uninit::GroundedArrayCell;
    ///
    /// static EXAMPLE: GroundedArrayCell<u8, 128> = GroundedArrayCell::uninit();
    /// ```
    pub const fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Obtain a mutable starting pointer and length to the contained [T; N].
    ///
    /// No claims are made on the validity of the [T; N] (they may be partially or wholly
    /// invalid or uninitialized), and the caller is required to guarantee synchronization of
    /// access, e.g. guaranteeing that access is shared XOR mutable for the duration of any
    /// references (including slices) created from this pointer.
    ///
    /// ```rust
    /// use grounded::uninit::GroundedArrayCell;
    /// static EXAMPLE: GroundedArrayCell<u8, 128> = GroundedArrayCell::uninit();
    ///
    /// let (ptr, len): (*mut u8, usize) = EXAMPLE.get_ptr_len();
    /// assert_ne!(core::ptr::null_mut(), ptr);
    /// assert_eq!(len, 128);
    /// ```
    pub fn get_ptr_len(&'static self) -> (*mut T, usize) {
        let mu_ptr: *mut MaybeUninit<[T; N]> = self.inner.get();
        let arr_ptr: *mut [T; N] = mu_ptr.cast::<[T; N]>();
        let t_ptr: *mut T = arr_ptr.cast::<T>();
        (t_ptr, N)
    }
}
