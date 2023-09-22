use alloc::collections::VecDeque;
use crate::proc::{Process, ProcessStatus};
use spin::{Mutex, MutexGuard};
use core::sync::atomic::{AtomicU64, Ordering};

pub const RETRY_LIMIT: u64 = 1000;

pub static PROCESS_QUEUE: Mutex<VecDeque<Process>> = Mutex::new(VecDeque::new());
pub static RUNNING_PROCESS: AtomicU64 = AtomicU64::new(0);

pub fn find(proc_iter: alloc::collections::vec_deque::Iter<'_, Process>, pid: u64) -> Option<usize>
{
    let mut c = 0;
    for i in proc_iter
    {
        if i.pid == pid {return Some(c);}
        c += 1;
    }
    None
}

fn push_proc_and_call(mut guard: MutexGuard<'_, VecDeque<Process>>, mut proc: Process)
{
    guard.pop_front();
    proc.retries = 0;
    guard.push_back(proc);
    RUNNING_PROCESS.store(guard.front().unwrap().pid, Ordering::Relaxed);
    guard.front_mut().unwrap().call();
}

pub fn check()
{
    let mut pq = PROCESS_QUEUE.lock();
    let proc = pq.get(find(pq.iter(), RUNNING_PROCESS.load(Ordering::Relaxed)).unwrap()).unwrap();
    let mut proc = proc.clone();
    if let ProcessStatus::Done(_) = proc.status
    {
        push_proc_and_call(pq, proc);
        
    }
    else
    {
        if proc.retries > RETRY_LIMIT
        {
            proc.status = ProcessStatus::Done(false);
            push_proc_and_call(pq, proc);
        }
        else
        {
            pq.pop_back();
            proc.retries += 1;
            pq.push_back(proc);
        }
    }
}

pub fn spawn(proc: Process)
{
    PROCESS_QUEUE.lock().push_back(proc);
}

pub fn remove(pid: u64) -> bool
{
    let mut pq = PROCESS_QUEUE.lock();
    let opt = find(pq.iter(), pid);
    if let Some(i) = opt
    {
        pq.remove(i).unwrap();
        true
    }
    else {false}
}