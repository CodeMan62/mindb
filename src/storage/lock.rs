use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

/// Lock types for database file access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    /// No lock held
    None,
    /// Shared lock - allows multiple readers
    Shared,
    /// Exclusive lock - only one writer
    Exclusive,
}

/// File lock manager for concurrent database access
///
/// Provides both shared (read) and exclusive (write) locks
/// to allow safe concurrent access to the database file.
pub struct FileLock {
    /// The lock file handle
    lock_file: Option<File>,
    /// Path to the lock file
    lock_path: PathBuf,
    /// Current lock type
    current_lock: LockType,
}

impl FileLock {
    /// Create a new FileLock for the given database path
    ///
    /// Creates a separate .lock file next to the database file
    pub fn new<P: AsRef<Path>>(db_path: P) -> io::Result<Self> {
        let mut lock_path = PathBuf::from(db_path.as_ref());
        lock_path.set_extension("lock");

        Ok(Self {
            lock_file: None,
            lock_path,
            current_lock: LockType::None,
        })
    }

    /// Acquire a shared (read) lock
    ///
    /// Multiple processes can hold shared locks simultaneously.
    /// Blocks until the lock can be acquired.
    pub fn lock_shared(&mut self) -> io::Result<()> {
        if self.current_lock == LockType::Shared {
            return Ok(());
        }

        // Must release exclusive lock before acquiring shared
        if self.current_lock == LockType::Exclusive {
            self.unlock()?;
        }

        let file = self.open_lock_file()?;
        FileExt::lock_shared(&file)?;
        self.lock_file = Some(file);
        self.current_lock = LockType::Shared;
        Ok(())
    }

    /// Try to acquire a shared lock without blocking
    ///
    /// Returns Ok(true) if lock acquired, Ok(false) if would block
    pub fn try_lock_shared(&mut self) -> io::Result<bool> {
        if self.current_lock == LockType::Shared {
            return Ok(true);
        }

        if self.current_lock == LockType::Exclusive {
            self.unlock()?;
        }

        let file = self.open_lock_file()?;
        match FileExt::try_lock_shared(&file) {
            Ok(()) => {
                self.lock_file = Some(file);
                self.current_lock = LockType::Shared;
                Ok(true)
            }
            Err(ref e)
                if e.raw_os_error() == Some(libc::EWOULDBLOCK)
                    || e.raw_os_error() == Some(libc::EAGAIN) =>
            {
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Acquire an exclusive (write) lock
    ///
    /// Only one process can hold an exclusive lock.
    /// Blocks until the lock can be acquired.
    pub fn lock_exclusive(&mut self) -> io::Result<()> {
        if self.current_lock == LockType::Exclusive {
            return Ok(());
        }

        // Must release shared lock before acquiring exclusive
        if self.current_lock == LockType::Shared {
            self.unlock()?;
        }

        let file = self.open_lock_file()?;
        FileExt::lock_exclusive(&file)?;
        self.lock_file = Some(file);
        self.current_lock = LockType::Exclusive;
        Ok(())
    }

    /// Try to acquire an exclusive lock without blocking
    ///
    /// Returns Ok(true) if lock acquired, Ok(false) if would block
    pub fn try_lock_exclusive(&mut self) -> io::Result<bool> {
        if self.current_lock == LockType::Exclusive {
            return Ok(true);
        }

        if self.current_lock == LockType::Shared {
            self.unlock()?;
        }

        let file = self.open_lock_file()?;
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => {
                self.lock_file = Some(file);
                self.current_lock = LockType::Exclusive;
                Ok(true)
            }
            Err(ref e)
                if e.raw_os_error() == Some(libc::EWOULDBLOCK)
                    || e.raw_os_error() == Some(libc::EAGAIN) =>
            {
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Release the current lock
    pub fn unlock(&mut self) -> io::Result<()> {
        if let Some(ref file) = self.lock_file {
            FileExt::unlock(file)?;
        }
        self.lock_file = None;
        self.current_lock = LockType::None;
        Ok(())
    }

    /// Get the current lock type
    pub fn lock_type(&self) -> LockType {
        self.current_lock
    }

    /// Check if any lock is held
    pub fn is_locked(&self) -> bool {
        self.current_lock != LockType::None
    }

    /// Upgrade from shared to exclusive lock
    ///
    /// Note: This releases the shared lock first, so there's a window
    /// where no lock is held. Use with caution in concurrent scenarios.
    pub fn upgrade_to_exclusive(&mut self) -> io::Result<()> {
        if self.current_lock == LockType::Exclusive {
            return Ok(());
        }

        // Release shared lock and acquire exclusive
        self.unlock()?;
        self.lock_exclusive()
    }

    /// Downgrade from exclusive to shared lock
    pub fn downgrade_to_shared(&mut self) -> io::Result<()> {
        if self.current_lock == LockType::Shared {
            return Ok(());
        }

        // Release exclusive lock and acquire shared
        self.unlock()?;
        self.lock_shared()
    }

    fn open_lock_file(&self) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_path)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_lock_exclusive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        let mut lock = FileLock::new(&db_path).unwrap();
        assert_eq!(lock.lock_type(), LockType::None);

        lock.lock_exclusive().unwrap();
        assert_eq!(lock.lock_type(), LockType::Exclusive);

        lock.unlock().unwrap();
        assert_eq!(lock.lock_type(), LockType::None);
    }

    #[test]
    fn test_lock_shared() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        let mut lock = FileLock::new(&db_path).unwrap();
        lock.lock_shared().unwrap();
        assert_eq!(lock.lock_type(), LockType::Shared);

        lock.unlock().unwrap();
        assert_eq!(lock.lock_type(), LockType::None);
    }

    #[test]
    fn test_multiple_shared_locks() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        let mut lock1 = FileLock::new(&db_path).unwrap();
        let mut lock2 = FileLock::new(&db_path).unwrap();

        lock1.lock_shared().unwrap();
        // Second shared lock should succeed
        let acquired = lock2.try_lock_shared().unwrap();
        assert!(acquired);

        lock1.unlock().unwrap();
        lock2.unlock().unwrap();
    }

    #[test]
    fn test_exclusive_blocks_shared() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        let mut lock1 = FileLock::new(&db_path).unwrap();
        let mut lock2 = FileLock::new(&db_path).unwrap();

        lock1.lock_exclusive().unwrap();
        // Try to acquire shared lock should fail (would block)
        let acquired = lock2.try_lock_shared().unwrap();
        assert!(!acquired);

        lock1.unlock().unwrap();
        // Now it should work
        let acquired = lock2.try_lock_shared().unwrap();
        assert!(acquired);
    }

    #[test]
    fn test_upgrade_downgrade() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        let mut lock = FileLock::new(&db_path).unwrap();

        lock.lock_shared().unwrap();
        assert_eq!(lock.lock_type(), LockType::Shared);

        lock.upgrade_to_exclusive().unwrap();
        assert_eq!(lock.lock_type(), LockType::Exclusive);

        lock.downgrade_to_shared().unwrap();
        assert_eq!(lock.lock_type(), LockType::Shared);
    }

    #[test]
    fn test_drop_releases_lock() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test").unwrap();

        {
            let mut lock = FileLock::new(&db_path).unwrap();
            lock.lock_exclusive().unwrap();
            // lock dropped here
        }

        // Should be able to acquire lock now
        let mut lock2 = FileLock::new(&db_path).unwrap();
        let acquired = lock2.try_lock_exclusive().unwrap();
        assert!(acquired);
    }
}
