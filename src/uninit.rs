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

    /// Initialize each element from the provided value, if `T: Copy`.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that no other access is made to the data contained within this
    /// cell for the duration of this function
    #[inline]
    pub unsafe fn initialize_all_copied(&'static self, val: T)
    where
        T: Copy,
    {
        let (mut ptr, len) = self.get_ptr_len();
        let end = ptr.add(len);
        while ptr != end {
            ptr.write(val);
            ptr = ptr.add(1);
        }
    }

    /// Initialize each item, using a provided closure on a per-element basis
    ///
    /// ## Safety
    ///
    /// The caller must ensure that no other access is made to the data contained within this
    /// cell for the duration of this function
    #[inline]
    pub unsafe fn initialize_all_with<F: FnMut() -> T>(&'static self, mut f: F)
    {
        let (mut ptr, len) = self.get_ptr_len();
        let end = ptr.add(len);
        while ptr != end {
            ptr.write(f());
            ptr = ptr.add(1);
        }
    }

    /// Obtain a mutable starting pointer to the contained [T; N].
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
    /// let ptr: *mut u8 = EXAMPLE.as_mut_ptr();
    /// assert_ne!(core::ptr::null_mut(), ptr);
    /// ```
    #[inline]
    pub fn as_mut_ptr(&'static self) -> *mut T {
        let mu_ptr: *mut MaybeUninit<[T; N]> = self.inner.get();
        let arr_ptr: *mut [T; N] = mu_ptr.cast::<[T; N]>();
        let t_ptr: *mut T = arr_ptr.cast::<T>();
        t_ptr
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
    ///
    /// ## NOTE
    ///
    /// This method is suggested to only be used for actions such as initializing the entire
    /// range. If you are building a data structure that provides partial access safely, such
    /// as a channel, bip-buffer, or similar, consider using one of the following methods, which
    /// can help avoid cases where strict provenance is invalidated by creation of an aliasing
    /// slice:
    ///
    /// * For getting a single element:
    ///     * [Self::get_element_unchecked()]
    ///     * [Self::get_element_mut_unchecked()]
    /// * For getting a subslice:
    ///     * [Self::get_subslice_unchecked()]
    ///     * [Self::get_subslice_mut_unchecked()]
    #[inline]
    pub fn get_ptr_len(&'static self) -> (*mut T, usize) {
        (self.as_mut_ptr(), N)
    }

    /// Obtain a reference to a single element, which can be thought of as `&data[offset]`.
    ///
    /// The reference is created **without** creating the entire slice this cell represents.
    /// This is important, if a mutable reference of a disjoint region is currently live.
    ///
    /// ## Safety
    ///
    /// The caller **must** ensure all of the following:
    ///
    /// * The selected element has been initialized with a valid value prior to calling
    ///   this function
    /// * No `&mut` slices or references may overlap the produced reference for the duration the reference is live
    /// * No modifications (even via pointers) are made to to the element pointed to
    ///   while the reference is live
    /// * `offset` is < N
    #[inline]
    pub unsafe fn get_element_unchecked(&'static self, offset: usize) -> &'static T {
        &*self.as_mut_ptr().add(offset)
    }

    /// Obtain a mutable reference to a single element, which can be thought of as `&mut data[offset]`.
    ///
    /// The reference is created **without** creating the entire slice this cell represents.
    /// This is important, if a mutable reference of a disjoint region is currently live.
    ///
    /// ## Safety
    ///
    /// The caller **must** ensure all of the following:
    ///
    /// * The selected element has been initialized with a valid value prior to calling
    ///   this function
    /// * No slices or references of any kind may overlap the produced reference for the duration
    ///   the reference is live
    /// * No modifications (even via pointers) are made to to the element pointed to
    ///   while the reference is live
    /// * `offset` is < N
    #[allow(clippy::mut_from_ref)]
    #[inline]
    pub unsafe fn get_element_mut_unchecked(&'static self, offset: usize) -> &'static mut T {
        &mut *self.as_mut_ptr().add(offset)
    }

    /// Obtain a subslice starting at `offset`, of length `len`, which
    /// can be thought of as `&data[offset..][..len]`.
    ///
    /// The subslice is created **without** creating the entire slice this cell represents.
    /// This is important, if a mutable reference of a disjoint region is currently live.
    ///
    /// ## Safety
    ///
    /// The caller **must** ensure all of the following:
    ///
    /// * All elements in this region have been initialized with a valid value prior to calling
    ///   this function
    /// * No `&mut` slices may overlap the produced slice for the duration the slice is live
    /// * No modifications (even via pointers) are made to data within the range of this slice
    ///   while the slice is live
    /// * `offset` and `offset + len` are <= N
    #[inline]
    pub unsafe fn get_subslice_unchecked(&'static self, offset: usize, len: usize) -> &'static [T] {
        core::slice::from_raw_parts(self.as_mut_ptr().add(offset), len)
    }

    /// Obtain a mutable subslice starting at `offset`, of length `len`, which
    /// can be thought of as `&mut data[offset..][..len]`.
    ///
    /// The subslice is created **without** creating the entire slice this cell represents.
    /// This is important, if ANY reference of a disjoint region is currently live.
    ///
    /// ## Safety
    ///
    /// The caller **must** ensure all of the following:
    ///
    /// * All elements in this region have been initialized with a valid value prior to calling
    ///   this function
    /// * No ``&` or &mut` slices may overlap the produced slice for the duration the slice is live
    /// * No modifications (even via pointers) are made to data within the range of this slice
    ///   while the slice is live
    /// * `offset` and `offset + len` are <= N
    #[allow(clippy::mut_from_ref)]
    #[inline]
    pub unsafe fn get_subslice_mut_unchecked(&'static self, offset: usize, len: usize) -> &'static mut [T] {
        core::slice::from_raw_parts_mut(self.as_mut_ptr().add(offset), len)
    }
}
