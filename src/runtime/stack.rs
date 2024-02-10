use std::mem::MaybeUninit;
use std::ptr::null_mut;

use crate::value::Value;

/// ## Stack layout
///
/// ## Stack slots
///
/// The stack is accessed by stack-slots.
/// Each stack-slot is an `i16`.
///
/// If the slot is `>= 0`, then a local stack slot is meant.
///
/// The slot `0` is the local slot closest to the bottom of the stack.
///
/// If the slot is `< 0`, then a parameter or return slot is meant.
///
/// The slot `-1` is the parameter or return slot closest to the top of the stack.
///
/// All slots are local to the current function call.
#[repr(C)]
pub struct Stack {
    /// Stack top
    pub(super) sp: *mut Value,
    /// Call frame
    pub(super) fp: *mut Value,
    owner: Box<[MaybeUninit<Value>]>,
}

impl Stack {
    pub fn new(size: usize) -> Self {
        unsafe {
            let mut owner = Box::new_uninit_slice(size);
            let sp = owner.as_mut_ptr().add(size) as _;
            Self {
                sp,
                fp: null_mut(),
                owner,
            }
        }
    }

    /// Reserves size for `n` elements
    #[inline]
    pub fn alloc(&mut self, n: usize) {
        unsafe {
            self.sp = self.sp.sub(n);
        }
    }

    /// Loads the value at the given stack slot
    #[inline]
    pub fn load(&mut self, slot: i16) -> Value {
        unsafe {
            if slot < 0 {
                return *self.fp.add(1 + (-slot) as usize);
            }
            *self.fp.sub(1 + slot as usize)
        }
    }

    /// Stores a value at the given stack slot
    #[inline]
    pub fn store(&mut self, slot: i16, value: Value) {
        unsafe {
            if slot < 0 {
                *self.fp.add(1 + (-slot) as usize) = value;
                return;
            }
            *self.fp.sub(1 + slot as usize) = value;
        }
    }

    /// Pushes a new [StackFrame] with the given return address
    #[inline]
    pub fn push_frame(&mut self, ra: *const u8) {
        unsafe {
            let old_fp = self.fp;
            self.alloc(2);
            self.fp = self.sp;
            let fp = self.fp as *mut StackFrame;
            (*fp).fp = old_fp;
            (*fp).ra = ra;
        }
    }

    /// Pops the current [StackFrame] and returns the return address
    #[inline]
    pub fn return_call(&mut self) -> *const u8 {
        unsafe {
            let fp = self.fp as *mut StackFrame;
            self.fp = (*fp).fp;
            (*fp).ra
        }
    }
}

/// ## Stack frame layout
///
/// ```text
/// +=========================+
/// | Return address          |
/// +-------------------------+
/// | Caller frame            |
/// +=========================+ <- fp
/// | Local 0                 |
/// +-------------------------+
/// | Local 1                 |
/// +-------------------------+
/// | ...                     |
/// +-------------------------+
/// | Local N - 1             |
/// +-------------------------+ <- sp
/// ```
#[repr(C)]
pub struct StackFrame {
    /// Caller frame
    fp: *mut Value,
    /// Return address
    ra: *const u8,
}
