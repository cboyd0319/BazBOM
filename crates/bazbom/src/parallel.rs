//! Parallel processing for dependency analysis
//!
//! Provides concurrent scanning of multiple targets to improve performance
//! on multi-core systems and large monorepos.

use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = auto-detect)
    pub num_threads: usize,
    /// Maximum number of tasks to queue
    pub queue_size: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            queue_size: 100,
        }
    }
}

impl ParallelConfig {
    /// Create with custom thread count
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            num_threads,
            ..Default::default()
        }
    }

    /// Get effective thread count (auto-detect if 0)
    pub fn effective_threads(&self) -> usize {
        if self.num_threads == 0 {
            num_cpus::get()
        } else {
            self.num_threads
        }
    }
}

/// Parallel processor for batch operations
pub struct ParallelProcessor<T, R> {
    config: ParallelConfig,
    _phantom_t: std::marker::PhantomData<T>,
    _phantom_r: std::marker::PhantomData<R>,
}

impl<T, R> ParallelProcessor<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// Create a new parallel processor
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            config,
            _phantom_t: std::marker::PhantomData,
            _phantom_r: std::marker::PhantomData,
        }
    }

    /// Process items in parallel
    ///
    /// Distributes work across multiple threads and collects results.
    /// Errors from individual tasks are propagated but don't stop other tasks.
    pub fn process<F>(&self, items: Vec<T>, f: F) -> Vec<Result<R>>
    where
        F: Fn(T) -> Result<R> + Send + Sync + 'static,
    {
        let num_threads = self.config.effective_threads();

        if items.is_empty() {
            return vec![];
        }

        // For small workloads, process serially
        if items.len() < num_threads || num_threads == 1 {
            return items.into_iter().map(f).collect();
        }

        // Distribute work across threads
        let items = Arc::new(Mutex::new(items.into_iter()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let f = Arc::new(f);

        let mut handles = vec![];

        for _ in 0..num_threads {
            let items = Arc::clone(&items);
            let results = Arc::clone(&results);
            let f = Arc::clone(&f);

            let handle = thread::spawn(move || {
                loop {
                    // Get next item
                    let item = {
                        let mut items = items.lock().unwrap();
                        items.next()
                    };

                    match item {
                        Some(item) => {
                            let result = f(item);
                            let mut results = results.lock().unwrap();
                            results.push(result);
                        }
                        None => break, // No more work
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Extract results
        let results = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner().unwrap(),
            Err(_) => panic!("Results Arc has multiple references"),
        };

        results
    }

    /// Process items in parallel with progress tracking
    pub fn process_with_progress<F, P>(
        &self,
        items: Vec<T>,
        f: F,
        mut progress_fn: P,
    ) -> Vec<Result<R>>
    where
        F: Fn(T) -> Result<R> + Send + Sync + 'static,
        P: FnMut(usize, usize) + Send + 'static,
    {
        let total = items.len();
        let num_threads = self.config.effective_threads();

        if items.is_empty() {
            return vec![];
        }

        // For small workloads, process serially with progress
        if items.len() < num_threads || num_threads == 1 {
            return items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let result = f(item);
                    progress_fn(i + 1, total);
                    result
                })
                .collect();
        }

        // Parallel processing with progress
        let items = Arc::new(Mutex::new(items.into_iter()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let completed = Arc::new(Mutex::new(0usize));
        let f = Arc::new(f);

        let mut handles = vec![];

        for _ in 0..num_threads {
            let items = Arc::clone(&items);
            let results = Arc::clone(&results);
            let completed = Arc::clone(&completed);
            let f = Arc::clone(&f);

            let handle = thread::spawn(move || {
                loop {
                    let item = {
                        let mut items = items.lock().unwrap();
                        items.next()
                    };

                    match item {
                        Some(item) => {
                            let result = f(item);
                            {
                                let mut results = results.lock().unwrap();
                                results.push(result);
                            }
                            {
                                let mut completed = completed.lock().unwrap();
                                *completed += 1;
                            }
                        }
                        None => break,
                    }
                }
            });

            handles.push(handle);
        }

        // Progress monitoring thread
        let progress_handle = {
            let completed = Arc::clone(&completed);
            thread::spawn(move || {
                loop {
                    let count = *completed.lock().unwrap();
                    progress_fn(count, total);

                    if count >= total {
                        break;
                    }

                    thread::sleep(std::time::Duration::from_millis(100));
                }
            })
        };

        // Wait for workers
        for handle in handles {
            handle.join().expect("Worker thread panicked");
        }

        // Wait for progress thread
        progress_handle.join().expect("Progress thread panicked");

        // Extract results
        let results = match Arc::try_unwrap(results) {
            Ok(mutex) => mutex.into_inner().unwrap(),
            Err(_) => panic!("Results Arc has multiple references"),
        };

        results
    }
}

/// Get recommended thread count for parallel processing
pub fn recommended_threads() -> usize {
    // Use number of logical CPUs, capped at 16 to avoid excessive overhead
    num_cpus::get().min(16)
}

/// Estimate if parallel processing is worthwhile
pub fn should_parallelize(item_count: usize, thread_count: usize) -> bool {
    // Only parallelize if we have enough items per thread
    let items_per_thread = item_count / thread_count;
    items_per_thread >= 2 && thread_count > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.num_threads > 0);
        assert_eq!(config.queue_size, 100);
    }

    #[test]
    fn test_parallel_config_with_threads() {
        let config = ParallelConfig::with_threads(4);
        assert_eq!(config.effective_threads(), 4);
    }

    #[test]
    fn test_parallel_config_auto_detect() {
        let config = ParallelConfig::with_threads(0);
        let threads = config.effective_threads();
        assert!(threads > 0);
    }

    #[test]
    fn test_parallel_processor_empty() {
        let processor = ParallelProcessor::<i32, i32>::new(ParallelConfig::default());
        let results = processor.process(vec![], |x| Ok(x * 2));
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parallel_processor_basic() {
        let config = ParallelConfig::with_threads(2);
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let results = processor.process(items, |x| Ok(x * 2));

        assert_eq!(results.len(), 5);
        for result in &results {
            assert!(result.is_ok());
        }

        // Collect values (order may vary due to parallelism)
        let mut values: Vec<i32> = results.into_iter().map(|r| r.unwrap()).collect();
        values.sort();
        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_processor_with_errors() {
        let config = ParallelConfig::with_threads(2);
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let results = processor.process(items, |x| {
            if x == 3 {
                Err(anyhow::anyhow!("Error on 3"))
            } else {
                Ok(x * 2)
            }
        });

        assert_eq!(results.len(), 5);

        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();

        assert_eq!(successes, 4);
        assert_eq!(failures, 1);
    }

    #[test]
    fn test_parallel_processor_with_progress() {
        let config = ParallelConfig::with_threads(2);
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let progress_updates = Arc::new(Mutex::new(Vec::new()));
        let progress_clone = Arc::clone(&progress_updates);

        let results = processor.process_with_progress(
            items,
            |x| {
                thread::sleep(std::time::Duration::from_millis(10));
                Ok(x * 2)
            },
            move |completed, total| {
                progress_clone.lock().unwrap().push((completed, total));
            },
        );

        assert_eq!(results.len(), 5);

        let updates = progress_updates.lock().unwrap();
        assert!(!updates.is_empty());

        // Should have at least one progress update
        let (_completed, total) = updates[0];
        assert_eq!(total, 5);
    }

    #[test]
    fn test_recommended_threads() {
        let threads = recommended_threads();
        assert!(threads > 0);
        assert!(threads <= 16);
    }

    #[test]
    fn test_should_parallelize() {
        // Not enough items per thread
        assert!(!should_parallelize(5, 4));

        // Enough items
        assert!(should_parallelize(10, 4));

        // Single thread
        assert!(!should_parallelize(10, 1));

        // Many items
        assert!(should_parallelize(100, 8));
    }

    #[test]
    fn test_parallel_processor_serial_fallback() {
        let config = ParallelConfig::with_threads(1);
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3];
        let results = processor.process(items, |x| Ok(x * 2));

        assert_eq!(results.len(), 3);
        // Serial execution preserves order
        assert_eq!(results[0].as_ref().unwrap(), &2);
        assert_eq!(results[1].as_ref().unwrap(), &4);
        assert_eq!(results[2].as_ref().unwrap(), &6);
    }
}
