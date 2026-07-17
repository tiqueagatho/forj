#![allow(unsafe_code)]
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
    Idle = 4,
}

#[derive(Debug, Clone)]
pub struct RealtimeTask {
    pub name: &'static str,
    pub priority: Priority,
    pub deadline_ms: u64,
    pub period_ms: u64,
    pub jitter_ms: u64,
}

impl RealtimeTask {
    pub fn new(name: &'static str, priority: Priority, deadline_ms: u64, period_ms: u64) -> Self {
        Self {
            name,
            priority,
            deadline_ms,
            period_ms,
            jitter_ms: 0,
        }
    }

    pub fn deadline_missed(&self, start: tokio::time::Instant) -> bool {
        start.elapsed() > Duration::from_millis(self.deadline_ms)
    }
}

/// Set thread scheduling priority (Linux only, requires CAP_SYS_NICE).
/// Uses safe libc bindings. Returns Ok(()) on non-Linux (no-op).
pub fn set_thread_priority(priority: Priority) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let (policy, prio) = match priority {
            Priority::Critical => (libc::SCHED_FIFO, 90),
            Priority::High => (libc::SCHED_RR, 60),
            _ => (libc::SCHED_OTHER, 0),
        };
        unsafe {
            let mut param: libc::sched_param = std::mem::zeroed();
            param.sched_priority = prio;
            let ret = libc::sched_setscheduler(0, policy, &param);
            if ret != 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
    }
    let _ = priority;
    Ok(())
}

pub struct DeadlineGuard {
    deadline: tokio::time::Instant,
    task_name: &'static str,
}

impl DeadlineGuard {
    pub fn new(name: &'static str, timeout_ms: u64) -> Self {
        Self {
            deadline: tokio::time::Instant::now() + Duration::from_millis(timeout_ms),
            task_name: name,
        }
    }

    pub fn exceeded(&self) -> bool {
        tokio::time::Instant::now() > self.deadline
    }

    pub async fn run<F, T>(&self, future: F) -> Result<T, DeadlineMissed>
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::time::timeout(
            self.deadline
                .saturating_duration_since(tokio::time::Instant::now()),
            future,
        )
        .await
        {
            Ok(result) => Ok(result),
            Err(_) => Err(DeadlineMissed {
                task: self.task_name,
                deadline: self.deadline,
            }),
        }
    }
}

#[derive(Debug)]
pub struct DeadlineMissed {
    pub task: &'static str,
    pub deadline: tokio::time::Instant,
}

impl std::fmt::Display for DeadlineMissed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "deadline missed for task '{}'", self.task)
    }
}
