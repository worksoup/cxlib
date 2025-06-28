use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard},
};

pub trait GetKeyStr {
    fn key_str(&self) -> &str;
}
pub trait Builder<T> {
    fn build(&self) -> T;
}

impl<A, B, T: Fn() -> B> Builder<A> for T
where
    A: From<B>,
{
    #[inline]
    fn build(&self) -> A {
        self().into()
    }
}
type Builders<T> = HashMap<String, Box<dyn Builder<T> + Send + Sync>>;

pub struct GlobalMultimap<T> {
    builders: Arc<RwLock<Builders<T>>>,
    values: Arc<RwLock<HashMap<String, T>>>,
}

impl<T: 'static> GlobalMultimap<T> {
    pub fn register_builder<B: Builder<T> + Send + Sync + 'static>(&self, solver_builder: B)
    where
        T: GetKeyStr,
    {
        let value = solver_builder.build();
        let key = value.key_str();
        let boxed: Box<dyn Builder<T> + Send + Sync> = Box::new(solver_builder);
        self.builders
            .write()
            .unwrap()
            .insert(key.to_string(), boxed);

        self.values.write().unwrap().insert(key.to_string(), value);
    }
    pub fn build(&self, key: &str) -> Option<T> {
        self.builders.read().unwrap().get(key).map(|b| b.build())
    }
}
impl<T> Default for GlobalMultimap<T> {
    #[inline]
    fn default() -> Self {
        GlobalMultimap {
            builders: Arc::new(RwLock::new(HashMap::new())),
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
impl<T> Clone for GlobalMultimap<T> {
    fn clone(&self) -> Self {
        Self {
            builders: Arc::clone(&self.builders),
            values: Arc::clone(&self.values),
        }
    }
}

pub struct GlobalMultimapItem<'gm, 'str, T> {
    key: &'str str,
    global_multimap: &'gm GlobalMultimap<T>,
}
impl<'gm, 's, T> GlobalMultimapItem<'gm, 's, T> {
    #[inline]
    pub fn new(
        global_multimap: &'gm GlobalMultimap<T>,
        key: &'s str,
    ) -> Option<GlobalMultimapItem<'gm, 's, T>> {
        if global_multimap.values.read().unwrap().contains_key(key) {
            Some(Self {
                key,
                global_multimap,
            })
        } else if let Some(builder) = global_multimap.builders.read().unwrap().get(key) {
            let solver = builder.build();
            global_multimap
                .values
                .write()
                .unwrap()
                .insert(key.to_string(), solver);
            Some(Self {
                key,
                global_multimap,
            })
        } else {
            None
        }
    }
}
impl<T> GlobalMultimapItem<'_, '_, T> {
    #[inline]
    pub fn get(&self) -> T {
        unsafe {
            self.global_multimap
                .builders
                .read()
                .unwrap()
                .get(self.key)
                .unwrap_unchecked()
                .build()
        }
    }
    #[inline]
    pub fn get_ref(&self) -> GlobalMultimapItemReadGuard<'_, '_, T> {
        GlobalMultimapItemReadGuard {
            rw_lock_read_guard: self.global_multimap.values.read().unwrap(),
            key: self.key,
        }
    }
}

impl<'s, T> GetKeyStr for GlobalMultimapItem<'_, 's, T> {
    #[inline]
    fn key_str(&self) -> &'s str {
        self.key
    }
}
pub struct GlobalMultimapItemReadGuard<'a, 's, T> {
    rw_lock_read_guard: RwLockReadGuard<'a, HashMap<String, T>>,
    key: &'s str,
}
impl<T> GlobalMultimapItemReadGuard<'_, '_, T> {
    pub fn get(&self) -> &T {
        self.rw_lock_read_guard.get(self.key).unwrap()
    }
}
impl<T> Deref for GlobalMultimapItemReadGuard<'_, '_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
