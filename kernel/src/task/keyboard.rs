use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use crate::{print, println};
//use futures::stream::Stream;
//use futures::stream::StreamExt;
//use futures::task::AtomicWaker;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
//static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8)
{
    if let Ok(queue) = SCANCODE_QUEUE.try_get()
    {
        if let Err(_) = queue.push(scancode) {println!("WARNING: scancode queue full; dropping keyboard input");}
        //else {WAKER.wake();}
    }
    else {println!("WARNING: scancode queue uninitialized");}
}

pub fn print_keypresses()
{
    SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100)).expect("ScancodeStream::new should only be called once");
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), pc_keyboard::layouts::Uk105Key, HandleControl::Ignore);

    while let Some(scancode) = SCANCODE_QUEUE.get().expect("hrr").pop()
    {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode)
        {
            if let Some(key) = keyboard.process_keyevent(key_event)
            {
                match key
                {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key)
                }
            }
        }
    }
}

pub struct ScancodeStream
{
    _private: ()
}

impl ScancodeStream
{
    pub fn new() -> Self
    {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100)).expect("ScancodeStream::new should only be called once");
        ScancodeStream{_private: ()}
    }

    /*pub fn poll_next(self: Pin<&mut self>, cx: &mut Context) -> Poll<Option<u8>>
    {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        //fast path
        if let Some(scancode) = queue.pop() {return Poll::Ready(Some(scancode));}

        WAKER.register(cx.waker());
        match queue.pop()
        {
            Some(scancode) =>
            {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending
        }
    }*/
}

/*impl Stream for ScancodeStream
{
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>>
    {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        //fast path
        if let Some(scancode) = queue.pop() {return Poll::Ready(Some(scancode));}

        WAKER.register(cx.waker());
        match queue.pop()
        {
            Some(scancode) =>
            {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending
        }
    }
}*/