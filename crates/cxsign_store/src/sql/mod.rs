pub trait StorageTrait: Sized {
    fn create<T: StorageTableCommandTrait<Self>>(&self) {
        T::init(self);
    }
    fn is_existed<T: StorageTableCommandTrait<Self>>(&self) -> bool {
        !T::uninit(self)
    }
    fn delete<T: StorageTableCommandTrait<Self>>(&self) {
        T::clear(self);
    }
    fn import<T: StorageTableCommandTrait<Self>>(&self, content: &str) {
        T::import(self, content);
    }
    fn export<T: StorageTableCommandTrait<Self>>(&self) -> String {
        T::export(self)
    }
}
pub trait StorageTableCommandTrait<Storage: StorageTrait> {
    fn init(storage: &Storage);
    fn uninit(storage: &Storage) -> bool {
        let _ = storage;
        true
    }
    fn clear(storage: &Storage) {
        let _ = storage;
    }
    fn import(storage: &Storage, content: &str) {
        let _ = storage;
        let _ = content;
    }
    fn export(storage: &Storage) -> String {
        let _ = storage;
        String::new()
    }
}
