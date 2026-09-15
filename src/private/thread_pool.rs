// A small thread pool for running independent tasks in parallel.

use std::{
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

/// An owned, type-erased task that can be moved to a worker thread.
///
/// # Design Decision
///
/// Each closure may have a different size and capture different values, while
/// a channel must carry one concrete type. Boxing preserves type and memory
/// safety while giving every task the same representation. A function pointer
/// would only support functions and closures that capture no values.
type Task = Box<dyn FnOnce() + Send + 'static>;

/// Tracks tasks that are queued or running and wakes callers waiting for idle.
struct Activity {
    pending: Mutex<usize>,
    idle: Condvar,
}

/// Runs submitted tasks on a fixed set of persistent worker threads.
///
/// # Design Decision
///
/// The pool uses an unbounded standard-library channel. Task submission does
/// not wait for queue capacity or task completion, and scheduling or
/// backpressure policies remain the caller's responsibility. Workers are
/// created with the pool and stay alive until it is dropped so repeated work
/// does not repeatedly create operating-system threads.
///
/// The pool does not catch task panics. A panicking Rust task therefore stops
/// its worker thread according to Rust's normal panic behavior.
///
/// `sender` is `Some` throughout the pool's normal lifetime. During
/// [`Drop::drop`], it is taken and dropped before joining the workers. Dropping
/// the final sender disconnects the channel and lets each worker leave
/// `recv`; keeping it until after `join` would leave the workers waiting and
/// deadlock the drop operation.
///
/// The standard-library receiver is shared behind a mutex because it is not
/// clonable. Workers release that mutex before executing a task, so only task
/// receipt is serialized. The standard-library MPMC channel is still unstable;
/// adding a third-party MPMC dependency is unnecessary unless receiving tasks
/// becomes a measured bottleneck.
///
/// # Example
///
/// ```
/// use std::sync::{
///     Arc,
///     atomic::{AtomicUsize, Ordering},
/// };
/// # mod thread_pool {
/// #     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/private/thread_pool.rs"));
/// # }
/// # use thread_pool::ThreadPool;
///
/// let pool = ThreadPool::with_threads(2);
/// let completed = Arc::new(AtomicUsize::new(0));
///
/// for _ in 0..4 {
///     let completed = Arc::clone(&completed);
///     pool.execute(move || {
///         completed.fetch_add(1, Ordering::Relaxed);
///     });
/// }
///
/// pool.wait_until_idle();
/// assert!(pool.is_idle());
/// assert_eq!(completed.load(Ordering::Relaxed), 4);
/// ```
pub(crate) struct ThreadPool {
    sender: Option<Sender<Task>>,
    workers: Vec<JoinHandle<()>>,
    activity: Arc<Activity>,
}

impl ThreadPool {
    /// Creates a pool using the number of threads available to the process.
    ///
    /// If the available parallelism cannot be determined, one worker is used.
    ///
    /// # Panics
    ///
    /// Panics if an operating-system thread cannot be created.
    pub(crate) fn new() -> Self {
        Self::with_threads(thread::available_parallelism().map_or(1, usize::from))
    }

    /// Creates a pool with exactly `thread_count` persistent worker threads.
    ///
    /// # Panics
    ///
    /// Panics if `thread_count` is zero or an operating-system thread cannot
    /// be created.
    ///
    /// # Design Decision
    ///
    /// A pool without workers could accept tasks that can never finish, so a
    /// zero thread count violates the pool's invariant.
    pub(crate) fn with_threads(thread_count: usize) -> Self {
        assert!(
            thread_count > 0,
            "a thread pool requires at least one thread"
        );

        let (sender, receiver) = mpsc::channel::<Task>();
        let receiver = Arc::new(Mutex::new(receiver));
        let activity = Arc::new(Activity {
            pending: Mutex::new(0),
            idle: Condvar::new(),
        });
        let workers = (0..thread_count)
            .map(|_| {
                let receiver = Arc::clone(&receiver);
                thread::spawn(move || Self::work(receiver))
            })
            .collect();

        Self {
            sender: Some(sender),
            workers,
            activity,
        }
    }

    /// Submits a task for asynchronous execution.
    ///
    /// The task does not return a result through the pool. Use
    /// [`Self::is_idle`] or [`Self::wait_until_idle`] to observe completion.
    ///
    /// # Panics
    ///
    /// Panics if every worker thread has stopped and the task cannot be sent.
    /// For example, this happens if a one-worker pool's task panics, stopping
    /// its only receiver, and another task is then submitted.
    pub(crate) fn execute(&self, task: impl FnOnce() + Send + 'static) {
        *self
            .activity
            .pending
            .lock()
            .expect("thread pool activity lock poisoned") += 1;

        // Keeping the completion guard inside the queued closure also marks a
        // task as finished if every receiver stops and the channel discards it.
        let finished = TaskFinished(Arc::clone(&self.activity));
        let task = Box::new(move || {
            task();
            drop(finished);
        });

        if self
            .sender
            .as_ref()
            .expect("thread pool sender missing")
            .send(task)
            .is_err()
        {
            panic!("all thread pool workers stopped");
        }
    }

    /// Returns whether no submitted tasks are queued or running.
    ///
    /// This is a snapshot. Another caller may submit work immediately after
    /// this method returns.
    pub(crate) fn is_idle(&self) -> bool {
        *self
            .activity
            .pending
            .lock()
            .expect("thread pool activity lock poisoned")
            == 0
    }

    /// Blocks until no submitted tasks are queued or running.
    ///
    /// Tasks submitted while waiting are included until the pool reaches an
    /// idle state.
    ///
    /// This method must not be called by a task running on this pool. That task
    /// remains pending until it returns, so it would wait for itself forever.
    pub(crate) fn wait_until_idle(&self) {
        let pending = self
            .activity
            .pending
            .lock()
            .expect("thread pool activity lock poisoned");
        // `wait_while` atomically unlocks the counter while sleeping so workers
        // can update it. After every wake-up it reacquires the lock and checks
        // the predicate again, which also handles spurious wake-ups.
        let _pending = self
            .activity
            .idle
            .wait_while(pending, |pending| *pending > 0)
            .expect("thread pool activity lock poisoned");
    }

    /// Receives and runs tasks until every sender has been dropped.
    fn work(receiver: Arc<Mutex<Receiver<Task>>>) {
        loop {
            let task = receiver
                .lock()
                .expect("thread pool receiver lock poisoned")
                .recv();
            let Ok(task) = task else {
                // Dropping the final sender permanently disconnects the
                // channel. Further receives would also fail, so this worker
                // exits instead of continuing into a busy loop.
                break;
            };
            task();
        }
    }
}

impl Drop for ThreadPool {
    /// Stops every worker after accepted tasks and joins their threads.
    ///
    /// Disconnecting the channel does not discard buffered tasks: live workers
    /// drain them before `recv` fails. Joining then blocks the dropping thread
    /// until those workers finish. Because dropping requires exclusive
    /// ownership, no caller can submit more tasks while this happens.
    /// If every worker has already panicked, the disconnected receiver instead
    /// discards any tasks that could no longer be run.
    ///
    /// The pool must not be dropped by one of its own tasks, such as through
    /// the final [`Arc`] owning it, because a worker cannot join itself.
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Marks a task as finished when its execution scope ends.
///
/// This guard maintains completion accounting when a task returns, unwinds, or
/// is discarded. It does not catch or otherwise change panic behavior.
struct TaskFinished(Arc<Activity>);

impl Drop for TaskFinished {
    /// Updates pool activity when a task returns, unwinds, or is discarded.
    fn drop(&mut self) {
        let mut pending = self
            .0
            .pending
            .lock()
            .expect("thread pool activity lock poisoned");
        *pending -= 1;
        if *pending == 0 {
            self.0.idle.notify_all();
        }
    }
}
