use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use crate::println;
use crate::print;
use core::pin::Pin;
use core::task::Poll;
use core::task::Context;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;
use pc_keyboard::DecodedKey;
use pc_keyboard::HandleControl;
use pc_keyboard::Keyboard;
use pc_keyboard::ScancodeSet1;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8)
{
    if let Ok(queue) = SCANCODE_QUEUE.try_get()
    {
        if let Err(_) = queue.push(scancode) {println!("WARNING: scancode queue full; dropping keyboard input");}
        else {WAKER.wake();}
    }
    else {println!("WARNING: scancode queue uninitialized");}
}

pub async fn print_keypresses()
{
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(pc_keyboard::layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await
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
}

impl Stream for ScancodeStream
{
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>>
    {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        //fast path
        if let Ok(scancode) = queue.pop() {return Poll::Ready(Some(scancode));}

        WAKER.register(cx.waker());
        match queue.pop()
        {
            Ok(scancode) =>
            {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending
        }
    }
}