use super::{Task, TaskId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::Context;
use core::task::{Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    taskQueue: Arc<ArrayQueue<TaskId>>,
    wakerCache: BTreeMap<TaskId, Waker>,
}

struct TaskWaker {
    taskID: TaskId,
    taskQueue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(taskID: TaskId, taskQueue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker { taskID, taskQueue }))
    }

    fn wakeTask(&self) {
        self.taskQueue.push(self.taskID).expect("TaskQueue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wakeTask();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wakeTask();
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            taskQueue: Arc::new(ArrayQueue::new(100)),
            wakerCache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let taskID = task.id;
        if self.tasks.insert(taskID, task).is_some() {
            panic!("Task with same ID already in tasks");
        }
        self.taskQueue.push(taskID).expect("queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.runReadyTasks();
            self.sleepIfIdle();
        }
    }

    fn sleepIfIdle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        interrupts::disable();
        if self.taskQueue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }

    fn runReadyTasks(&mut self) {
        let Self {
            tasks,
            taskQueue,
            wakerCache,
        } = self;

        while let Some(taskID) = taskQueue.pop() {
            let task = match tasks.get_mut(&taskID) {
                Some(task) => task,
                None => continue,
            };
            let waker = wakerCache
                .entry(taskID)
                .or_insert_with(|| TaskWaker::new(taskID, taskQueue.clone()));
            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&taskID);
                    wakerCache.remove(&taskID);
                }
                Poll::Pending => {}
            }
        }
    }
}
