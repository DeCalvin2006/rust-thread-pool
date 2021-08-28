use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Type `Job`.
///
/// # Notes
///
/// Just a wrapper. Who likes to write the full type name so many times,uh?
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Message
///
/// # Usage
///
/// `NewJob` has a member `Job`, to run it in the `Worker` thread.
///
/// `Terminate` makes the thread stop.
enum Message {
    NewJob(Job),
    Terminate,
}

/// Worker
///
/// # Notes
///
/// `id` is just a number in our library, not the real pid.
struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker.
    ///
    /// # Parameters
    ///
    /// - `id`: The given id in `ThreadPool::new()`.
    /// - `receiver`:The receiver side constructed in `ThreadPool::new()`.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            match job {
                Message::NewJob(job) => {
                    println!("Worker #{}: get job, running.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker #{}:get singal terminate, terminating.", id);
                    break;
                }
            }
        });

        return Worker {
            handle: Some(handle),
            id,
        };
    }
}

/// Thread Pool.
///
/// # Members
///
/// - `workers`: Contains the `Worker` objects.
/// - `sender`: Send `Message` objects to `Worker` threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Creates a thread pool.
    ///
    /// # Panics
    ///
    /// This function will panic when:
    ///
    /// - `size` euqals to `0`
    /// - `size` is too big that we can't create threads anymore
    pub fn new(size: usize) -> ThreadPool {
        // assert if size is OK
        assert!(size > 0);

        // init workers
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        // Copy the receiver,As Atomic RC and Mutex.
        let receiver = Arc::new(Mutex::new(receiver));

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)))
        }

        return ThreadPool { workers, sender };
    }

    /// Run the given function or closure in the pool.
    ///
    /// # Steps
    ///
    /// 1. Construct A `Box<F>` object.
    /// 2. Send it to the `Worker`s.
    pub fn run<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Drop the ThreadPool.
    ///
    /// # Steps
    ///
    /// 1. Send `Message::Terminate` to all `Worker`s in `workers`
    /// 2. For each `Worker`, use `handle.join().unwrap()` to terminate the thread.
    fn drop(&mut self) {
        for _ in &mut self.workers.iter() {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker #{}", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

#[cfg(test)]

mod test {

    use crate::*;

    #[test]
    #[should_panic]
    fn thread_pool_test_new_1() {
        ThreadPool::new(1000000);
    }
}
