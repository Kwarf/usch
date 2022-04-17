use std::{
    fs::read_to_string,
    path::Path,
    sync::mpsc::{channel, TryRecvError},
    time::Duration,
};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub struct SourceWatcher {
    _watcher: RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
}

impl SourceWatcher {
    pub fn new(path: &Path) -> SourceWatcher {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

        SourceWatcher {
            _watcher: watcher,
            rx,
        }
    }

    pub fn get_new_content(&self) -> Option<String> {
        match self.rx.try_recv() {
            Ok(notify::DebouncedEvent::Write(path)) => {
                return Some(read_to_string(path).unwrap());
            }
            Ok(_) | Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!(),
        }
    }
}
