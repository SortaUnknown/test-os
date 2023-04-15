pub mod executor;
pub mod keyboard;

use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use alloc::boxed::Box;

pub struct Task
{
    id: TaskID,
    future: Pin<Box<dyn Future<Output = ()>>>
}

impl Task
{
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task
    {
        Task{id: TaskID::new(), future: Box::pin(future)}
    }

    pub fn poll (&mut self, context: &mut Context) -> Poll<()>
    {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskID(u64);

impl TaskID
{
    fn new() -> Self
    {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskID(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}