/*!
A very simple job scheduler. Runs one job (one-time or recurring) on one apawned thread.

# Usage

```rust
fn main() {
    let _once_handle = scheduling::Scheduler::once(|| println!("ONCE")).start();

    let recurring_handle = scheduling::Scheduler::delayed_recurring(
        std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(1),
        || println!("1 SEC ELAPSED"),
    )
    .start();

    std::thread::sleep(std::time::Duration::from_secs(5));

    recurring_handle.cancel();

    std::thread::sleep(std::time::Duration::from_secs(5));
}
```
*/

use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Handle for a `Scheduler`, running or not
pub struct SchedulerHandle(Arc<AtomicBool>);

impl Drop for SchedulerHandle {
    fn drop(&mut self) {
        self.cancel()
    }
}

impl SchedulerHandle {
    /// Cancel the `Scheduler` (stop any running job)
    pub fn cancel(&self) {
        self.0.store(true, atomic::Ordering::SeqCst);
    }
}

enum JobType {
    Once(Box<dyn FnMut() + Send + 'static>),
    Recurring { f: Box<dyn FnMut() + Send + 'static>, rate: Duration },
}

struct Job {
    type_: JobType,
    time: Instant,
}

/// The `Scheduler` container
#[derive(Default)]
pub struct Scheduler {
    job: Option<Job>,
    cancelled: Arc<AtomicBool>,
}

impl Scheduler {
    /// Get the `SchedulerHandle` for this scheduler. Can be used to setup tasks that cancel themselves on some condition.
    pub fn handle(&self) -> SchedulerHandle {
        SchedulerHandle(self.cancelled.clone())
    }

    /// Create a scheduler to run a one-time job in a background thread
    pub fn once<F>(f: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let job = Job { type_: JobType::Once(Box::new(f)), time: Instant::now() };

        Self { job: Some(job), cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Create a scheduler to run a one-time job with an initial delay
    pub fn delayed_once<F>(delay: Duration, f: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let job = Job { type_: JobType::Once(Box::new(f)), time: Instant::now() + delay };

        Self { job: Some(job), cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Create a scheduler to run a recurring job at a fixed rate
    pub fn recurring<F>(rate: Duration, f: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let job = Job { type_: JobType::Recurring { f: Box::new(f), rate }, time: Instant::now() };

        Self { job: Some(job), cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Create a scheduler to run a recurring job at a fixed rate, with an initial delay
    pub fn delayed_recurring<F>(delay: Duration, rate: Duration, f: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let job = Job {
            type_: JobType::Recurring { f: Box::new(f), rate },
            time: Instant::now() + delay,
        };

        Self { job: Some(job), cancelled: Arc::new(AtomicBool::new(false)) }
    }

    /// Start running the `Scheduler` and return its `handle`
    pub fn start(self) -> SchedulerHandle {
        let handle = self.handle();
        std::thread::spawn(move || self.run());
        handle
    }

    fn run(mut self) {
        while let Some(job) = self.get_job() {
            self.run_job(job);
        }
    }

    fn get_job(&mut self) -> Option<Job> {
        loop {
            if self.cancelled.load(atomic::Ordering::SeqCst) {
                return None;
            }

            let now = Instant::now();

            match self.job.as_ref() {
                None => {
                    break;
                }
                Some(j) if j.time <= now => {
                    break;
                }
                Some(j) => {
                    std::thread::sleep(j.time - now);
                }
            };
        }

        self.job.take()
    }

    fn run_job(&mut self, job: Job) {
        match job.type_ {
            JobType::Once(mut f) => {
                f();
            }
            JobType::Recurring { mut f, rate } => {
                f();
                let new_job = Job { type_: JobType::Recurring { f, rate }, time: job.time + rate };
                self.job = Some(new_job);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
