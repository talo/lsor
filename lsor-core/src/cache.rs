use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::RwLock,
};

pub trait Cache {
    fn get(&self, key: &str) -> Option<String>;
    fn insert(&self, key: String, value: String);
}

#[derive(Clone, Debug)]
pub struct FifoCache {
    cache: Rc<RwLock<HashMap<String, String>>>,
    cache_fifo: Rc<RwLock<VecDeque<String>>>,
    cache_limit_n: usize,
}

impl FifoCache {
    pub fn new(cache_limit_n: usize) -> Self {
        Self {
            cache: Rc::new(RwLock::new(HashMap::new())),
            cache_fifo: Rc::new(RwLock::new(VecDeque::new())),
            cache_limit_n,
        }
    }
}

impl Cache for FifoCache {
    fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        let mut cache = self.cache.write().unwrap();
        let mut cache_fifo = self.cache_fifo.write().unwrap();

        if cache_fifo.len() >= self.cache_limit_n {
            if let Some(key) = cache_fifo.pop_front() {
                cache.remove(&key);
            }
        }

        cache.insert(key.clone(), value);

        cache_fifo.retain(|k| k != &key);
        cache_fifo.push_back(key);
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::{col, from, gt, table, Driver, PushPrql as _};

    use super::*;

    #[test]
    fn test_fifo_cache() {
        let cache = FifoCache::new(10);

        let mut driver = Driver::with_cache(Box::new(cache.clone()));
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .push_to_driver(&mut driver);
        }
        let begin = Instant::now();
        assert_eq!(driver.sql(), "SELECT * FROM users WHERE age > $1");
        let end = Instant::now();
        let duration = end.duration_since(begin);
        println!("duration: {:?}", duration);

        let mut driver = Driver::with_cache(Box::new(cache.clone()));
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .push_to_driver(&mut driver);
        }
        let begin = Instant::now();
        assert_eq!(driver.sql(), "SELECT * FROM users WHERE age > $1");
        let end = Instant::now();
        let cached_duration = end.duration_since(begin);

        assert!(cached_duration < duration);
        println!("cached_duration: {:?}", cached_duration);
    }
}
