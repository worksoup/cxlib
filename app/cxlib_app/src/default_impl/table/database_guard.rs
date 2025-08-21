//! 也许能防止一个块内试图同时获取读事务与写事务。
use std::{ops::Deref, sync::Arc};

use cxlib_error_utils::CxlibResultUtils;
use redb::{Database, ReadTransaction, ReadableDatabase, WriteTransaction};

use crate::StoreError;
#[derive(Clone)]
pub struct DatabaseGuard {
    db: Arc<Database>,
}
impl DatabaseGuard {
    pub fn new(db: Database) -> Self {
        Self { db: Arc::new(db) }
    }
    pub fn from(db: &Arc<Database>) -> Self {
        Self { db: Arc::clone(db) }
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
        let r_cxt = self.db.begin_read().map_err(|e| m(E::from(e)))?;
        let t = f(&r_cxt)?;
        Ok(t)
    }
    #[inline]
    pub fn read_once<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(&self, f: F) -> Result<T, E>
    where
        E: From<redb::TransactionError>,
    {
        Self::read_once_map_err(self, f, |e| e)
    }
    pub fn read_map_err<T, N, E, F: FnOnce(&ReadTransaction) -> Result<T, N>, M: FnOnce(E) -> N>(
        &self,
        f: F,
        m: M,
    ) -> Result<ReadAccessGuard<T>, N>
    where
        E: From<redb::TransactionError>,
    {
        let r_cxt = self.db.begin_read().map_err(|e| m(E::from(e)))?;
        let t = f(&r_cxt)?;
        Ok(ReadAccessGuard { r_cxt, t })
    }
    #[inline]
    pub fn read<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        &self,
        f: F,
    ) -> Result<ReadAccessGuard<T>, E>
    where
        E: From<redb::TransactionError>,
    {
        Self::read_map_err(self, f, |e| e)
    }
    pub fn write_once_map_err<
        T,
        N,
        E,
        F: FnOnce(&WriteTransaction) -> Result<T, N>,
        M: FnOnce(E) -> N,
    >(
        &mut self,
        f: F,
        m: M,
    ) -> Result<T, N>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        match self.db.begin_write().map_err(E::from) {
            Ok(w_cxt) => {
                let t = f(&w_cxt)?;
                w_cxt.commit().map_err(|e| m(E::from(e)))?;
                Ok(t)
            }
            Err(e) => Err(m(e)),
        }
    }
    #[inline]
    pub fn write_once<T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
        &mut self,
        f: F,
    ) -> Result<T, E>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        Self::write_once_map_err(self, f, |e| e)
    }
    pub fn write_map_err<T, N, E, F: FnOnce(&WriteTransaction) -> Result<T, N>, M: FnOnce(E) -> N>(
        &mut self,
        f: F,
        m: M,
    ) -> Result<WriteAccessGuard<T>, N>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        match self.db.begin_write().map_err(E::from) {
            Ok(w_cxt) => {
                let t = f(&w_cxt)?;
                Ok(WriteAccessGuard { w_cxt, t })
            }
            Err(e) => Err(m(e)),
        }
    }
    #[inline]
    pub fn write<T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
        &mut self,
        f: F,
    ) -> Result<WriteAccessGuard<T>, E>
    where
        E: From<redb::TransactionError> + From<redb::CommitError>,
    {
        Self::write_map_err(self, f, |e| e)
    }
}
// impl Deref for DatabaseGuard {
//     type Target = Database;

//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         &self.db
//     }
// }
pub struct ReadAccessGuard<T> {
    r_cxt: ReadTransaction,
    t: T,
}
impl<T> ReadAccessGuard<T> {
    #[inline]
    pub fn unwrap_inner(self) -> T {
        self.t
    }
}
impl<T> ReadAccessGuard<T> {
    #[inline]
    pub fn into_inner(self) -> (T, ReadAccessGuard<()>) {
        let Self { r_cxt, t } = self;
        (t, ReadAccessGuard { r_cxt, t: () })
    }
}
impl ReadAccessGuard<()> {
    #[inline]
    pub fn read_once<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<T, E> {
        let Self { r_cxt, .. } = self;
        let t = f(&r_cxt)?;
        Ok(t)
    }
    #[inline]
    pub fn read<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<ReadAccessGuard<T>, E> {
        let Self { r_cxt, .. } = self;
        let t = f(&r_cxt)?;
        Ok(ReadAccessGuard { r_cxt, t })
    }
}
impl<T> Deref for ReadAccessGuard<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.t
    }
}
impl<T> AsRef<ReadTransaction> for ReadAccessGuard<T> {
    #[inline]
    fn as_ref(&self) -> &ReadTransaction {
        &self.r_cxt
    }
}
#[must_use = "如不处理则会造成数据未提交。"]
pub struct WriteAccessGuard<T> {
    w_cxt: WriteTransaction,
    t: T,
}
impl<T> WriteAccessGuard<T> {
    #[inline]
    pub fn commit(self) -> Result<T, StoreError> {
        let Self { w_cxt, t, .. } = self;
        w_cxt.commit()?;
        Ok(t)
    }
    #[inline]
    pub fn unwrap_inner(self) -> T {
        let Self { w_cxt, t, .. } = self;
        w_cxt.commit().log_unwrap();
        t
    }
    #[inline]
    pub fn get_inner(self) -> T {
        let Self { w_cxt, t, .. } = self;
        w_cxt.commit().log_ignore();
        t
    }
}
impl<T> WriteAccessGuard<T> {
    #[inline]
    pub fn into_inner(self) -> (T, WriteAccessGuard<()>) {
        let Self { w_cxt, t } = self;
        (t, WriteAccessGuard { w_cxt, t: () })
    }
}
impl WriteAccessGuard<()> {
    #[inline]
    pub fn write_once<T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<T, E>
    where
        E: From<redb::CommitError>,
    {
        let Self { w_cxt, .. } = self;
        let t = match f(&w_cxt) {
            Ok(t) => {
                w_cxt.commit().log_ignore();
                t
            }
            Err(e) => {
                w_cxt.commit()?;
                Err(e)?
            }
        };
        Ok(t)
    }
}
impl WriteAccessGuard<()> {
    #[inline]
    pub fn write<T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
        self,
        f: F,
    ) -> Result<WriteAccessGuard<T>, E>
    where
        E: From<redb::CommitError>,
    {
        let Self { w_cxt, .. } = self;
        match f(&w_cxt) {
            Ok(t) => Ok(WriteAccessGuard { w_cxt, t }),
            Err(e) => {
                w_cxt.commit()?;
                Err(e)
            }
        }
    }
}
impl<T> Deref for WriteAccessGuard<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.t
    }
}
impl<T> AsRef<WriteTransaction> for WriteAccessGuard<T> {
    #[inline]
    fn as_ref(&self) -> &WriteTransaction {
        &self.w_cxt
    }
}
