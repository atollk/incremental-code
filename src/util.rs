#[macro_export]
macro_rules! global_variable {
    ( $variable:ident, $variable_type:ty ) => {
        paste::paste! {
            static [<GLOBAL_ $variable>]: std::sync::LazyLock<parking_lot::ReentrantMutex<std::cell::RefCell<$variable_type>>> =
                std::sync::LazyLock::new(|| parking_lot::ReentrantMutex::new(std::cell::RefCell::new(Default::default())));

            #[doc = "Lock the global "]
            #[doc = stringify!($variable)]
            #[doc = "and run `f` with a reference to it.\n"]
            #[doc = "This is the single entry point for reading `"]
            #[doc = stringify!($variable_type)]
            #[doc = "`."]
            pub fn [<with_ $variable>]<T>(f: impl FnOnce(&$variable_type) -> T) -> T {
                use std::ops::Deref;
                let lock = [<GLOBAL_ $variable>].lock();
                f(lock.deref().borrow().deref())
            }

            #[doc = "Lock the global "]
            #[doc = stringify!($variable)]
            #[doc = "and run `f` with a mutable reference to it.\n"]
            #[doc = "This is the single entry point for mutating `"]
            #[doc = stringify!($variable_type)]
            #[doc = "`."]
            pub fn [<with_ $variable _mut>]<T>(f: impl FnOnce(&mut $variable_type) -> T) -> T {
                use std::ops::Deref;
                use std::ops::DerefMut;
                let lock = [<GLOBAL_ $variable>].lock();
                f(lock.deref().borrow_mut().deref_mut())
            }
        }
    };
}
