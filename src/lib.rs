mod metrics {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::collections::HashMap;
    use std::thread::ThreadId;


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

#[cfg(test)]
mod tests {
    use crate::metrics::Counter;
    use std::thread;

    #[test]
    fn basic_op() {
        let mut counter = Counter::new();
        assert_eq!(0, counter.get());

        counter.increment(1);
        assert_eq!(1, counter.get());
    }

    #[test]
    fn cloned_same_thread() {
        let mut counter = Counter::new();
        assert_eq!(0, counter.get());

        counter.increment(1);

        let mut counter_2 = counter.clone();
        counter_2.increment(1);

        let mut counter_3 = counter.clone();
        counter_3.increment(1);

        assert_eq!(3, counter.get());
        assert_eq!(3, counter_2.get());
        assert_eq!(3, counter_3.get());
    }

    #[test]
    fn multi_thread() {
        let mut counter = Counter::new();
        assert_eq!(0, counter.get());

        let num_threads = 10;
        let incrs_per_thread = 100;
        let mut threads = vec![];

        for _ in 0..num_threads {
            let mut thread_counter = counter.clone();
            let handle = thread::spawn(move || {
                for _ in 0..incrs_per_thread {
                    thread_counter.increment(1);
                }
            });

            threads.push(handle);
        }

        for handle in threads {
            handle.join();
        }

        assert_eq!(num_threads * incrs_per_thread, counter.get())
    }
}
