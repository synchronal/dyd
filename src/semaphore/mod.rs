use std::fmt;
use std::sync::{Arc, Condvar, Mutex};

/// A counting semaphore for limiting concurrent operations.
pub struct Semaphore {
  count: Mutex<usize>,
  condvar: Condvar,
}

impl fmt::Debug for Semaphore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let count = self.count.lock().map(|c| *c).unwrap_or(0);
    f.debug_struct("Semaphore")
      .field("available_permits", &count)
      .finish()
  }
}

/// RAII guard that releases a permit when dropped.
pub struct SemaphorePermit {
  semaphore: Arc<Semaphore>,
}

impl Semaphore {
  pub fn new(permits: usize) -> Self {
    Self {
      count: Mutex::new(permits),
      condvar: Condvar::new(),
    }
  }

  /// Acquires a permit, blocking until one is available.
  /// Returns a guard that releases the permit when dropped.
  pub fn acquire(self: &Arc<Self>) -> SemaphorePermit {
    let mut count = self.count.lock().unwrap();
    while *count == 0 {
      count = self.condvar.wait(count).unwrap();
    }
    *count -= 1;
    SemaphorePermit {
      semaphore: Arc::clone(self),
    }
  }

  fn release(&self) {
    let mut count = self.count.lock().unwrap();
    *count += 1;
    self.condvar.notify_one();
  }
}

impl Drop for SemaphorePermit {
  fn drop(&mut self) {
    self.semaphore.release();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::thread;
  use std::time::Duration;

  #[test]
  fn test_semaphore_limits_concurrency() {
    let semaphore = Arc::new(Semaphore::new(2));
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..10)
      .map(|_| {
        let sem = Arc::clone(&semaphore);
        let active = Arc::clone(&active);
        let max_active = Arc::clone(&max_active);

        thread::spawn(move || {
          let _permit = sem.acquire();

          let current = active.fetch_add(1, Ordering::SeqCst) + 1;
          max_active.fetch_max(current, Ordering::SeqCst);

          thread::sleep(Duration::from_millis(10));

          active.fetch_sub(1, Ordering::SeqCst);
        })
      })
      .collect();

    for handle in handles {
      handle.join().unwrap();
    }

    assert!(max_active.load(Ordering::SeqCst) > 0);
    assert!(max_active.load(Ordering::SeqCst) <= 2);
  }

  #[test]
  fn test_permit_released_on_drop() {
    let semaphore = Arc::new(Semaphore::new(1));

    {
      let _permit = semaphore.acquire();
      assert_eq!(*semaphore.count.lock().unwrap(), 0);
    }

    assert_eq!(*semaphore.count.lock().unwrap(), 1);
  }
}
