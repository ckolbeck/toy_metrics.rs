#[cfg(test)]
mod metrics {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::collections::HashMap;
    use std::thread::ThreadId;
    use std::ops::Deref;


    pub struct Counter {
        buckets: Arc<Mutex<HashMap<ThreadId, Arc<AtomicI64>>>>,
        initialized: bool,
        local: Arc<AtomicI64>,
    }

    impl Counter {
        pub fn new() -> Counter {
            Counter {
                buckets: Arc::new(Mutex::new(HashMap::new())),
                initialized: false,
                local: Arc::new(AtomicI64::new(0)),
            }
        }

        pub fn clone(&mut self) -> Counter {
            Counter {
                buckets: self.buckets.clone(),
                initialized: false,
                local: Arc::new(AtomicI64::new(0)),
            }
        }

        pub fn increment(&mut self, amount: i64) {
            if !self.initialized {
                let thread_id = thread::current().id();
                let mut buckets = self.buckets.lock().unwrap();
                match buckets.get(&thread_id) {
                    None => buckets.insert(thread_id, self.local.clone()),
                    Some(existing) => {
                        self.local = existing.clone();
                        None
                    }
                };

                self.initialized = true;
            }

            self.local.fetch_add(amount, Ordering::Relaxed);
        }

        pub fn get(&self) -> i64 {
            let mut count = 0;
            let map = &*self.buckets.lock().unwrap();

            for (_, bucket) in map {
                count += bucket.load(Ordering::Relaxed);
            }

            count
        }
    }
}
