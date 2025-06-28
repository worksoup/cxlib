use cxlib_error_utils::CxlibResultUtils;
use log::warn;
use redb::Database;
use std::sync::atomic::AtomicBool;

use crate::StoreError;

#[derive(Debug)]
pub struct TableBase<'a> {
    db: &'a Database,
    writing: AtomicBool,
}
impl<'a> TableBase<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            db,
            writing: AtomicBool::new(false),
        }
    }
    pub fn as_inner(&self) -> &'a Database {
        self.db
    }
}
/// 数据库事务，内部分为读事务和写事务。
pub enum Transaction {
    Read(redb::ReadTransaction),
    Write(Option<redb::WriteTransaction>),
}
impl Transaction {
    // =========== BOTH ===========
    /// 显式销毁。参见 [`Self::drop`] 实现。
    /// drop 过程中，将会默认提交所有操作，如果出现错误将会在日志中输出。
    pub fn close(self) {}

    // =========== READ ===========
    /// 列出所有表。
    ///
    /// 出现错误将返回 [`redb::StorageError`].
    pub fn list_tables<'s>(&self) -> Result<Vec<redb::UntypedTableHandle>, StoreError> {
        Ok(match self {
            Transaction::Read(r) => r.list_tables()?.collect(),
            Transaction::Write(Some(w)) => w.list_tables()?.collect(),
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        })
    }
    /// 列出所有多值表。
    ///
    /// 出现错误将返回 [`redb::StorageError`].
    pub fn list_multimap_tables<'s>(
        &self,
    ) -> Result<Vec<redb::UntypedMultimapTableHandle>, StoreError> {
        Ok(match self {
            Transaction::Read(r) => r.list_multimap_tables()?.collect(),
            Transaction::Write(Some(w)) => w.list_multimap_tables()?.collect(),
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        })
    }
    /// 打开指定表。
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；
    /// - 也可能会返回 [`redb::StorageError`].
    ///
    /// 对于可写表：
    ///   - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    ///   - 如果表不存在，将新建表。
    ///
    /// 对于只读表：
    ///   - 由于读写事务不能同时存在，所以不会出现存在已经打开的可写表。可同时打开多个只读表；
    ///   - 如果表不存在，将会返回 [`redb::TableError::TableDoesNotExist`].
    pub fn open_table<'txn, K: redb::Key + 'static, V: redb::Value + 'static>(
        &'txn self,
        definition: redb::TableDefinition<K, V>,
    ) -> Result<Table<'txn, K, V>, StoreError> {
        Ok(match self {
            Transaction::Read(r) => Table::ReadOnly(r.open_table(definition)?),
            Transaction::Write(Some(w)) => Table::WriteRead(w.open_table(definition)?),
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        })
    }
    /// 打开指定多值表。
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；
    /// - 也可能会返回 [`redb::StorageError`].
    ///
    /// 对于可写表：
    ///   - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    ///   - 如果表不存在，将新建表。
    ///
    /// 对于只读表：
    ///   - 由于读写事务不能同时存在，所以不会出现存在已经打开的可写表。可同时打开多个只读表；
    ///   - 如果表不存在，将会返回 [`redb::TableError::TableDoesNotExist`].
    pub fn open_multimap_table<'txn, K: redb::Key + 'static, V: redb::Key + 'static>(
        &'txn self,
        definition: redb::MultimapTableDefinition<K, V>,
    ) -> Result<MultimapTable<'txn, K, V>, StoreError> {
        Ok(match self {
            Transaction::Read(r) => MultimapTable::ReadOnly(r.open_multimap_table(definition)?),
            Transaction::Write(Some(w)) => {
                MultimapTable::WriteRead(w.open_multimap_table(definition)?)
            }
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        })
    }

    // =========== WRITE ===========
    /// 中止写事务，事务中所有写入都将回滚。
    /// 读事务不受影响。
    pub fn abort(mut self) -> Result<(), StoreError> {
        match &mut self {
            Transaction::Read(_) => warn!("`ReadTransaction` 无法中止事务，将默认关闭。"),
            Transaction::Write(w @ Some(_)) => {
                let w = w.take().unwrap();
                w.abort()?
            }
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        }
        Ok(())
    }
    fn write<Args, R, E, W>(&self, args: Args, write: W) -> Result<R, StoreError>
    where
        StoreError: From<E>,
        W: FnOnce(&redb::WriteTransaction, Args) -> Result<R, E>,
    {
        match self {
            Transaction::Read(_) => Err(StoreError::WriteOnReadTransaction),
            Transaction::Write(Some(w)) => Ok(write(w, args)?),
            _ => unreachable!("`WriteTransaction` 为空，请检查是否存在 bug!"),
        }
    }
    /// 删除指定表，如果表存在将返回 `true`.
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；
    /// - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    /// - 也可能会返回 [`redb::StorageError`].
    pub fn delete_table(&self, definition: impl redb::TableHandle) -> Result<bool, StoreError> {
        self.write(definition, redb::WriteTransaction::delete_table)
    }
    /// 删除指定的多值表，如果表存在将返回 `true`.
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；
    /// - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    /// - 也可能会返回 [`redb::StorageError`].
    pub fn delete_multimap_table(
        &self,
        definition: impl redb::MultimapTableHandle,
    ) -> Result<bool, StoreError> {
        self.write(definition, redb::WriteTransaction::delete_multimap_table)
    }
    /// 重命名指定表。
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；
    /// - 如果操作对象不存在，将返回 [`redb::TableError::TableDoesNotExist`]；
    /// - 如果新名称已被占用，将返回 [`redb::TableError::TableExists`]；
    /// - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    /// - 也可能会返回 [`redb::StorageError`].
    pub fn rename_table(
        &self,
        definition: impl redb::TableHandle,
        new_name: impl redb::TableHandle,
    ) -> Result<(), StoreError> {
        self.write((definition, new_name), |w, (definition, new_name)| {
            w.rename_table(definition, new_name)
        })
    }
    /// 重命名指定的多值表。
    ///
    /// # Errors
    /// - 表名存在，但类型与定义不符，将返回 [`redb::TableError::TableTypeMismatch`],
    ///   [`redb::TableError::TableIsMultimap`],
    ///   [`redb::TableError::TableIsNotMultimap`] 等错误；+
    /// - 如果操作对象不存在，将返回 [`redb::TableError::TableDoesNotExist`]；
    /// - 如果新名称已被占用，将返回 [`redb::TableError::TableExists`]；
    /// - 如果已经存在打开的表，那么将会返回 [`redb::TableError::TableAlreadyOpen`];
    /// - 也可能会返回 [`redb::StorageError`].
    pub fn rename_multimap_table(
        &self,
        definition: impl redb::MultimapTableHandle,
        new_name: impl redb::MultimapTableHandle,
    ) -> Result<(), StoreError> {
        self.write((definition, new_name), |w, (definition, new_name)| {
            w.rename_multimap_table(definition, new_name)
        })
    }
    // pub fn persistent_savepoint(&self) -> Result<u64, StoreError>;
    // pub fn get_persistent_savepoint(&self, id: u64) -> Result<SavePoint, StoreError>;
    // pub fn delete_persistent_savepoint(&self, id: u64) -> Result<bool, StoreError> {
    //     self.write(id, |w, id| {
    //         w.delete_persistent_savepoint(id)
    //     })
    // }
    // pub fn list_persistent_savepoint(&self) -> Result<impl Iterator<Item = u64>, StoreError>;
    // pub fn ephemeral_savepoint(&self) -> Result<SavePoint, StoreError>;
    // pub fn restore_savepoint(&mut self, savepoint: &Savepoint, ) -> Result<(), StoreError>;
}

impl Drop for Transaction {
    fn drop(&mut self) {
        match self {
            Transaction::Read(_) => (),
            Transaction::Write(w @ Some(_)) => {
                let w = w.take().unwrap();
                _ = w.commit().log_unwrap_or_default();
            }
            _ => warn!("`WriteTransaction` 为空，所有写入已回滚。"),
        }
    }
}

pub enum Table<'txn, K: redb::Key + 'static, V: redb::Value + 'static> {
    ReadOnly(redb::ReadOnlyTable<K, V>),
    WriteRead(redb::Table<'txn, K, V>),
}
impl<'txn, K: redb::Key, V: redb::Value> Table<'txn, K, V> {}

pub enum MultimapTable<'txn, K: redb::Key + 'static, V: redb::Key + 'static> {
    ReadOnly(redb::ReadOnlyMultimapTable<K, V>),
    WriteRead(redb::MultimapTable<'txn, K, V>),
}
impl<'txn, K: redb::Key, V: redb::Key> MultimapTable<'txn, K, V> {}
