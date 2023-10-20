use ::std::thread;
use ::std::time::{Duration, Instant};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct ExpiringBool {
    last_set: Arc<Mutex<Instant>>,
    value: Arc<AtomicBool>,
    duration: Duration,
    ignores_timer: bool,
}

impl ExpiringBool {
    pub fn new(v: bool, duration: Duration) -> Self {
        ExpiringBool {
            value: Arc::new(AtomicBool::new(v)),
            last_set: Arc::new(Mutex::new(Instant::now())),
            ignores_timer: false,
            duration,
        }
    }

    pub fn set_false(&self) {
        self.value.store(false, Ordering::Relaxed);
    }

    pub fn is_true(&self) -> bool {
        self.ignores_timer || self.value.load(Ordering::Relaxed)
    }

    pub fn toggle(&mut self) -> bool {
        if self.value.load(Ordering::Relaxed) {
            self.value.store(false, Ordering::Relaxed);
            false
        } else {
            self.ignores_timer ^= true;
            self.ignores_timer
        }
    }

    pub fn set(&mut self) {
        if self.ignores_timer {
            return;
        }

        let last_set = self.last_set.lock().unwrap().clone();
        let now = Instant::now();
        let elapsed = now.duration_since(last_set);

        if elapsed > self.duration || !self.value.load(Ordering::Relaxed) {
            self.value.store(true, Ordering::Relaxed);
            *self.last_set.lock().unwrap() = now;

            // Spawn a new thread to reset the boolean after the specified timeout
            let value_clone = Arc::clone(&self.value);
            let last_set_clone = Arc::clone(&self.last_set);
            let duration_clone = self.duration.clone();

            thread::spawn(move || loop {
                let last_set = last_set_clone.lock().unwrap().clone();
                let now = Instant::now();
                let elapsed = now.duration_since(last_set);

                if elapsed > duration_clone {
                    value_clone.store(false, Ordering::Relaxed);
                    break;
                } else {
                    thread::sleep(Duration::from_millis(50));
                }
            });
        } else {
            *self.last_set.lock().unwrap() = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
