use std::time::{Duration, Instant};

pub struct TimedTask {
    pub interval: Duration,
    pub last_run: Instant,
    pub action: Box<dyn FnMut() + Send>,
}

impl TimedTask {
    pub fn new<F>(interval_secs: u64, action: F) -> Self
    where
        F: FnMut() + 'static + Send,
    {
        Self {
            interval: Duration::from_secs(interval_secs),
            last_run: Instant::now() - Duration::from_secs(interval_secs),
            action: Box::new(action),
        }
    }

    pub fn should_run(&self) -> bool {
        self.last_run.elapsed() >= self.interval
    }

    pub fn run(&mut self) {
        (self.action)();
        self.last_run = Instant::now();
    }
}

pub struct Scheduler {
    pub tasks: Vec<TimedTask>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: TimedTask) {
        self.tasks.push(task);
    }

    pub fn tick(&mut self) {
        for task in &mut self.tasks {
            if task.should_run() {
                task.run();
            }
        }
    }
}
