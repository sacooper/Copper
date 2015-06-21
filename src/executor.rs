use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc;

struct Executor {
    recv: mpsc::Receiver<fn()>
}

impl Executor {
    fn init(receiver: mpsc::Receiver<fn()>){
       let mut this = Executor {
           recv: receiver
       };

       let handle = thread::spawn(move||{
            for f in this.recv.iter(){
                f()
            }
       });
       drop(handle);
    }
}
