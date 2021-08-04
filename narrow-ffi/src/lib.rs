use std::{ffi::CStr, fmt, ptr};

#[allow(non_camel_case_types)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use bindings::*;

impl ArrowSchema {
    pub fn null_mut() -> Self {
        Self {
            format: ptr::null_mut(),
            name: ptr::null_mut(),
            metadata: ptr::null_mut(),
            flags: 0,
            n_children: 0,
            children: ptr::null_mut(),
            dictionary: ptr::null_mut(),
            release: None,
            private_data: ptr::null_mut(),
        }
    }

    pub fn child(&self, index: usize) -> &Self {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, n_children: usize) -> ! {
            panic!(
                "child index (is {}) should be < n_children (is {})",
                index, n_children
            );
        }

        let n_children = self.n_children();
        if index >= n_children {
            assert_failed(index, n_children);
        }

        // Safety
        // - Bound checked above, based on n_children
        unsafe { self.children().get_unchecked(index) }
    }

    pub fn children(&self) -> &[&Self] {
        // Safety
        // -
        unsafe { std::slice::from_raw_parts(self.children as *const &_, self.n_children()) }
    }

    pub fn n_children(&self) -> usize {
        self.n_children as usize
    }

    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.name).to_str().unwrap() }
    }

    pub fn format(&self) -> &str {
        unsafe { CStr::from_ptr(self.format).to_str().unwrap() }
    }

    pub fn nullable(&self) -> bool {
        self.flags & (ARROW_FLAG_NULLABLE as i64) != 0
    }

    pub fn dictionary_ordered(&self) -> bool {
        self.flags & (ARROW_FLAG_DICTIONARY_ORDERED as i64) != 0
    }

    pub fn map_keys_sorted(&self) -> bool {
        self.flags & (ARROW_FLAG_MAP_KEYS_SORTED as i64) != 0
    }
}

impl Drop for ArrowSchema {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            unsafe { release(self) }
        }
    }
}

impl fmt::Display for ArrowSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.format(), self.nullable())
    }
}
