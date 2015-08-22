use mmap::{MemoryMap, MapOption};
use std::os::unix::io::AsRawFd;
use std::io::Write;
//use api::Database;
use entry::Entry;
use std::ops::DerefMut;
use std::collections::VecDeque;
use std::fs::File;
use std::thread;
use std::sync::{Arc, RwLock, Mutex, atomic};
use std::mem;
use rustc_serialize::json;

static N : usize = 50;

pub enum PersistError {}

//pub static PERSIST : fn(&mut Database) -> Result<(), PersistError> = persistDB;

//fn persistDB(db: &mut Database) -> Result<(), PersistError> {
//Ok(())    
//}

pub fn test<F>(f : F) where F : FnOnce(){
    let mut q = VecDeque::new();
    q.push_back(f);
    let g = q.pop_front().unwrap();
    g();
}

pub struct Persist {
    running   : Arc<atomic::AtomicBool>,
    lock      : Arc<Mutex<()>>
}

impl Drop for Persist {
    fn drop(&mut self){
        self.running.store(false, atomic::Ordering::Relaxed);
        self.lock.lock();
    }
}

impl Persist {
    pub fn new() -> Persist {
        Persist{
            running: Arc::new(atomic::AtomicBool::new(true)),
            lock: Arc::new(Mutex::new(()))
        }
    }

    pub fn persist(&self, file_lock: &Arc<RwLock<File>>, temp_lock : &Arc<RwLock<File>>, log_lock: &Arc<RwLock<Box<VecDeque<Entry>>>>){
        let file_lock = file_lock.clone();
        let temp_lock = temp_lock.clone();
        let log_lock = log_lock.clone();
        let running = self.running.clone();
        let lock = self.lock.clone();

        thread::spawn(move ||{
            maintainPersistance(running, lock, file_lock, temp_lock, log_lock);
        });
    }

}
//pub fn persist(){
//let file_lock = db.file.clone();
//let temp_lock = db.temp.clone();
//let log_lock = db.log.clone();

//thread::spawn(move ||{
//maintainPersistance(file_lock, temp_lock, log_lock);
//});
//}

type ARW<T> = Arc<RwLock<T>>;

fn maintainPersistance(running: Arc<atomic::AtomicBool>, mut lock: Arc<Mutex<()>>, 
                       mut file_lock: ARW<File>, mut temp_lock: ARW<File>, mut log_lock: ARW<Box<VecDeque<Entry>>>){
    let guard = lock.lock();
    let mut count : usize = 0;
    while running.load(atomic::Ordering::Relaxed) {
        count += 1;
        let mut lck = log_lock.write().unwrap();
        let mut log = mem::replace(lck.deref_mut(), Box::new(VecDeque::new()));
        drop(lck);

        copy_log(&mut temp_lock, &mut log);

        if count == N {
            count = 0;
            copy_temp(&mut file_lock, &mut temp_lock);
        }
        thread::sleep_ms(10);
    }


    let mut lck = log_lock.write().unwrap();
    let mut log = mem::replace(lck.deref_mut(), Box::new(VecDeque::new()));
    drop(lck);
    copy_log(&mut temp_lock, &mut log);
    temp_lock.write().unwrap().sync_all();
    copy_temp(&mut file_lock, &mut temp_lock);
    file_lock.write().unwrap().sync_all();
    drop(guard);
}




fn copy_log(temp_lock: &mut Arc<RwLock<File>>, log: &mut Box<VecDeque<Entry>>){
    let mut temp = temp_lock.write().unwrap();
    for entry in log.iter() {
        temp.write(&(json::encode(entry).unwrap().into_bytes()));
        temp.write("\n".as_bytes());
    }
}

fn copy_temp(file_lock: &mut Arc<RwLock<File>>, temp_lock: &mut Arc<RwLock<File>>){
    let temp = temp_lock.write().unwrap();
    let len = temp.metadata().unwrap().len() as usize;

    if len == 0 { return }

    let mmap_opts = &[
        MapOption::MapReadable,
        MapOption::MapFd(temp.as_raw_fd())
    ];

    let mmap = MemoryMap::new(len, mmap_opts).unwrap();
    let data : *mut u8 = mmap.data();
    let raw = unsafe{ 
        Vec::from_raw_parts(data, len, mmap.len())
    };
    file_lock.write().unwrap().write(&raw);
}
