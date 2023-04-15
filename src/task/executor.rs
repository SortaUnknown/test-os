use super::Task;
use super::TaskID;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::Waker;
use core::task::Context;
use core::task::Poll;
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts;

struct TaskWaker
{
    task_id: TaskID,
    task_queue: Arc<ArrayQueue<TaskID>>
}

impl TaskWaker
{
    fn new(task_id: TaskID, task_queue: Arc<ArrayQueue<TaskID>>) -> Waker
    {
        Waker::from(Arc::new(TaskWaker{task_id, task_queue}))
    }

    fn wake_task(&self)
    {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker
{
    fn wake(self: Arc<Self>)
    {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>)
    {
        self.wake_task();
    }
}

pub struct Executor
{
    tasks: BTreeMap<TaskID, Task>,
    task_queue: Arc<ArrayQueue<TaskID>>,
    waker_cache: BTreeMap<TaskID, Waker>
}

impl Executor
{
    pub fn new() -> Self
    {
        Executor{tasks: BTreeMap::new(), task_queue: Arc::new(ArrayQueue::new(100)), waker_cache: BTreeMap::new()}
    }

    pub fn spawn(&mut self, task: Task)
    {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some()
        {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> !
    {
        loop
        {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self)
    {
        while let Some(task_id) = self.task_queue.pop()
        {
            let task = match self.tasks.get_mut(&task_id)
            {
                Some(task) => task,
                None => continue //task no longer exists
            };

            let waker = self.waker_cache.entry(task_id).or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context)
            {
                Poll::Ready(()) =>
                {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self)
    {
        interrupts::disable();
        if self.task_queue.is_empty() {interrupts::enable_and_hlt();}
        else {interrupts::enable();}
    }
}