pub trait WithRef<T> {
    fn with_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R;
}
