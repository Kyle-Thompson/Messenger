use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};

#[derive(Clone)]
pub struct MpmcQueue<T> {
    data: Arc<(Mutex<VecDeque<T>>, Condvar)>,
}

impl<T> MpmcQueue<T> {

    pub fn new() -> MpmcQueue<T> {
        MpmcQueue {
            data: Arc::new( (Mutex::new(VecDeque::new()), Condvar::new()) )
        }
    }

    pub fn pop(&self) -> T {
        let &(ref queue, ref cvar) = &*self.data;
        loop {
            let mut queue = queue.lock().unwrap();
            while queue.is_empty() { queue = cvar.wait(queue).unwrap(); }
            
            if let Some(element) = queue.pop_front() {
                return element;
            }
        }
    }

    pub fn push(&self, element: T) {
        let &(ref queue, ref cvar) = &*self.data;
        { queue.lock().unwrap().push_back(element); }
        cvar.notify_one();
    }
}
