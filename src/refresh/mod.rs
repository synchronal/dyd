use crate::app::Event;
use std::sync::mpsc;

/// Counts a refresh's repo updates the event loop hasn't seen finish, so a
/// refresh ends on the same timeline as key presses.
#[derive(Debug, Default)]
pub struct Refresh {
  running: usize,
}

/// RAII guard that sends [`Event::RepoUpdateDone`] when dropped, including by a panic.
#[derive(Debug)]
pub struct RefreshTask {
  sender: mpsc::Sender<Event>,
}

impl Refresh {
  pub fn is_running(&self) -> bool {
    self.running > 0
  }

  pub fn track(&mut self, sender: mpsc::Sender<Event>) -> RefreshTask {
    self.running += 1;
    RefreshTask { sender }
  }

  /// Records one [`Event::RepoUpdateDone`] received by the event loop.
  pub fn finish(&mut self) {
    self.running = self.running.saturating_sub(1);
  }
}

impl Drop for RefreshTask {
  fn drop(&mut self) {
    let _ = self.sender.send(Event::RepoUpdateDone);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_running_until_every_done_event_received() {
    let (sender, receiver) = mpsc::channel();
    let mut refresh = Refresh::default();
    assert!(!refresh.is_running());

    drop(refresh.track(sender.clone()));
    drop(refresh.track(sender));
    assert!(refresh.is_running(), "tasks ended but their events are unprocessed");

    for _ in 0..2 {
      assert!(matches!(receiver.recv().unwrap(), Event::RepoUpdateDone));
      refresh.finish();
    }
    assert!(!refresh.is_running());
  }

  #[test]
  fn test_task_sends_done_when_thread_panics() {
    let (sender, receiver) = mpsc::channel();
    let mut refresh = Refresh::default();
    let task = refresh.track(sender);

    let result = std::thread::spawn(move || {
      let _task = task;
      panic!("git operation failed");
    })
    .join();

    assert!(result.is_err());
    assert!(matches!(receiver.recv().unwrap(), Event::RepoUpdateDone));
  }
}
