//! 也许能防止一个块内试图同时获取读事务与写事务。
use std::{
    mem::MaybeUninit,
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard, atomic::AtomicUsize},
};

use redb::{Database, ReadTransaction, WriteTransaction};
pub struct DatabaseGuard {
    db: Arc<Database>,
    w_cxt: Arc<RwLock<Option<WriteTransaction>>>,
    waiting_for_w_cxt: Arc<AtomicUsize>,
}
impl DatabaseGuard {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            db: Arc::clone(db),
            w_cxt: Default::default(),
            waiting_for_w_cxt: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn read_once_map_err<
        T,
        N,
        E,
        F: FnOnce(&ReadTransaction) -> Result<T, N>,
        M: FnOnce(E) -> N,
    >(
        &self,
        f: F,
        m: M,
    ) -> Result<T, N>
    where
        E: From<redb::TransactionError>,
    {
        let read_lock = self.w_cxt.read().unwrap();
        let r_cxt = self.begin_read().map_err(|e| m(E::from(e)))?;
        let t = f(&r_cxt)?;
        drop(read_lock);
        Ok(t)
    }
    #[inline]
    pub fn read_once<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(&self, f: F) -> Result<T, E>
    where
        E: From<redb::TransactionError>,
    {
        Self::read_once_map_err(self, f, |e| e)
    }
    pub fn read_map_err<
        's: 'g,
        'g,
        T,
        N,
        E,
        F: FnOnce(&ReadTransaction) -> Result<T, N>,
        M: FnOnce(E) -> N,
    >(
        &'s self,
        f: F,
        m: M,
    ) -> Result<ReadAccessGuard<'g, T>, N>
    where
        E: From<redb::TransactionError>,
    {
        let read_lock = self.w_cxt.read().unwrap();
        let r_cxt = self.begin_read().map_err(|e| m(E::from(e)))?;
        let t = f(&r_cxt)?;
        Ok(ReadAccessGuard {
            r_cxt,
            t,
            read_lock,
        })
    }
    #[inline]
    pub fn read<'s: 'g, 'g, T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        &'s self,
        f: F,
    ) -> Result<ReadAccessGuard<'g, T>, E>
    where
        E: From<redb::TransactionError>,
    {
        Self::read_map_err(self, f, |e| e)
    }
    pub fn write_map_err<T, N, E, F: FnOnce(&WriteTransaction) -> Result<T, N>, M: FnOnce(E) -> N>(
        &mut self,
        f: F,
        m: M,
    ) -> Result<T, N>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        let e = 'e: {
            self.waiting_for_w_cxt
                .fetch_add(1, std::sync::atomic::Ordering::Release);
            let mut w_cxt_mutex_guard: std::sync::RwLockWriteGuard<'_, Option<WriteTransaction>> =
                self.w_cxt.write().unwrap();
            self.waiting_for_w_cxt
                .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            let mut w_cxt_owned = MaybeUninit::uninit();
            let w_cxt = if let Some(w_cxt) = w_cxt_mutex_guard.as_ref() {
                w_cxt
            } else {
                let w_cxt = match self.begin_write().map_err(E::from) {
                    Ok(w_cxt) => w_cxt,
                    Err(e) => {
                        break 'e e;
                    }
                };
                w_cxt_owned.write(w_cxt);
                unsafe { &w_cxt_owned.assume_init_read() }
            };
            let t = f(w_cxt)?;
            let waiting_for_w_cxt = self
                .waiting_for_w_cxt
                .load(std::sync::atomic::Ordering::Acquire);
            if waiting_for_w_cxt == 0 {
                let w_cxt = if let Some(w_cxt) = w_cxt_mutex_guard.take() {
                    w_cxt
                } else {
                    unsafe { w_cxt_owned.assume_init() }
                };
                if let Err(e) = w_cxt.commit().map_err(E::from) {
                    break 'e e;
                }
            } else if w_cxt_mutex_guard.as_ref().is_none() {
                let w_cxt = unsafe { w_cxt_owned.assume_init() };
                w_cxt_mutex_guard.replace(w_cxt);
            }
            drop(w_cxt_mutex_guard);
            return Ok(t);
        };
        Err(m(e))
    }
    #[inline]
    pub fn write<T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
        &mut self,
        f: F,
    ) -> Result<T, E>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        Self::write_map_err(self, f, |e| e)
    }
}
impl Deref for DatabaseGuard {
    type Target = Database;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
pub struct ReadAccessGuard<'g, T> {
    r_cxt: ReadTransaction,
    t: T,
    read_lock: RwLockReadGuard<'g, Option<WriteTransaction>>,
}
impl<T> ReadAccessGuard<'_, T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.t
    }
}
impl<'g, R> ReadAccessGuard<'g, R> {
    #[inline]
    pub fn read_once<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<(R, T), E> {
        let Self {
            r_cxt,
            t: r,
            read_lock,
        } = self;
        let t = f(&r_cxt)?;
        drop(read_lock);
        Ok((r, t))
    }
    #[inline]
    pub fn read<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<(R, ReadAccessGuard<'g, T>), E> {
        let Self {
            r_cxt,
            t: r,
            read_lock,
        } = self;
        let t = f(&r_cxt)?;
        Ok((
            r,
            ReadAccessGuard {
                r_cxt,
                t,
                read_lock,
            },
        ))
    }
}
impl<T> Deref for ReadAccessGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.t
    }
}
impl<T> AsRef<ReadTransaction> for ReadAccessGuard<'_, T> {
    #[inline]
    fn as_ref(&self) -> &ReadTransaction {
        &self.r_cxt
    }
}
