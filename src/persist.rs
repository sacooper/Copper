use api::Database;
use std::collections::VecDeque;

pub enum PersistError {}

//pub static PERSIST : fn(&mut Database) -> Result<(), PersistError> = persistDB;

//fn persistDB(db: &mut Database) -> Result<(), PersistError> {
    //Ok(())    
//}

pub struct Persistor {
    foo: i32
}

pub fn test<F>(f : F) where F : FnOnce(){
    let mut q = VecDeque::new();
    q.push_back(f);
    let g = q.pop_front().unwrap();
    g();
}
