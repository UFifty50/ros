use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub struct SimpleExecutor {
    taskQueue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            taskQueue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.taskQueue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.taskQueue.pop_front() {
            let waker = dummyWaker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => self.taskQueue.push_back(task),
            }
        }
    }
}

fn dummyRawWaker() -> RawWaker {
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummyRawWaker()
    }

    let vtable = &RawWakerVTable::new(clone, noop, noop, noop);
    RawWaker::new(0 as *const (), vtable)
}

fn dummyWaker() -> Waker {
    unsafe { Waker::from_raw(dummyRawWaker()) }
}
