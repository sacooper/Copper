use rustc_serialize::json::{self, Json, ToJson};
use rand::{thread_rng, Rng};
use mmap::{MemoryMap, MapOption};
//use std::hash::{Hash, Hasher};
//use std::ptr;
use std::fs;
use std::io::{Write, SeekFrom, Seek};
use std::io::Error as IOError;
use std::os::unix::prelude::AsRawFd;
use std::collections::{HashMap, VecDeque, BTreeMap};
//use std::borrow::Borrow;
//use std::str::FromStr;
use std::mem;
use std::sync::{Arc, RwLock};
use entry::*;
use persist::Persist;
use util::JsonValue;
use index::{Index, IndexEntry};

pub type Field = String;

pub struct Database {
    pub file: Arc<RwLock<fs::File>>,
    pub temp: Arc<RwLock<fs::File>>,
    pub log: Arc<RwLock<Box<VecDeque<Entry>>>>,
    pub indexes: HashMap<Field,Index>,
    pub persist: Persist
}

pub enum DBError{
    EncodeError(json::EncoderError),
    WriteError(IOError)
}

impl Database {
    pub fn new(name: &str) -> Database { 
        info!(target: "Database", "Creating database, filename: {}", name);
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(name)
            .unwrap();

        let mut temp = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(".copper.tmp.db")
            .unwrap();

        //let size= 1024*1024*32;
        //temp.seek(SeekFrom::Start(size)).unwrap();
        //temp.write_all(&[0]).unwrap();
        
        //temp.seek(SeekFrom::Start(0)).unwrap();

        let file = Arc::new(RwLock::new(f));
        let temp = Arc::new(RwLock::new(temp));
        let log = Arc::new(RwLock::new(Box::new(VecDeque::new())));
        let persist = Persist::new();
        
        let mut db = Database{
            file: file,
            temp: temp, 
            log: log,
            indexes: HashMap::new(),
            persist: persist
        };
       
        db.load_database();
        db.persist.persist(&db.file, &db.temp, &db.log);
        db
    }

    pub fn insert(&mut self, data: json::Json) -> Result<(), DBError>{
        let mut log = self.log.write().unwrap();
        let js = data.clone();
        let mut obj = if js.is_object() { js } else {
            let x : BTreeMap<String, Json> = BTreeMap::new();
            x.to_json()
        };

        if let Json::Object(ref mut map) = obj {
            map.insert("_id".to_string(), Json::String(Database::gen_random_id()));
        } else {
            unimplemented!();
        }
        
        log.push_back(Entry::Insert(obj));
        Ok(())
        //let enc = json::encode(&obj);

        //if enc.is_err() { return Err(DBError::EncodeError(enc.err().unwrap())) }

        //let s = enc.unwrap().replace("\n", "") + "\n";
        //let res = self.file.lock().unwrap().write(&s.into_bytes());
        
        ////self.updateIndexes();

        //res.map_err(|e|{ DBError::WriteError(e) }).map(|_|{()})
    }

    //pub fn find(&mut self, data: json::Json) -> Result<Vec<json::Json>, DBError>{
        
    //}

    fn gen_random_id() -> String {
        thread_rng().gen_ascii_chars().filter(|c| c.is_alphanumeric()).take(15).collect::<String>()
    }

    fn load_database(&mut self){
        let id = "_id".to_string();
        let mut file = self.file.write().unwrap();
        self.indexes.insert(id.clone(), BTreeMap::new());
        // Generate _id index

        let mmap_opts = &[
            // Then make the mapping *public* so it is written back to the file
            //MapOption::MapNonStandardFlags(libc::consts::os::posix88::MAP_SHARED),
            MapOption::MapReadable,
            //MapOption::MapWritable,
            MapOption::MapFd(file.as_raw_fd()) 
        ];
        
        
        let len = file.metadata().unwrap().len() as usize;
        if len == 0 { return }
        let mmap = MemoryMap::new(len, mmap_opts).unwrap();
        let raw = unsafe{String::from_raw_parts(mmap.data(), len, mmap.len())};
        let data = raw.split('\n').collect::<Vec<&str>>();
        for (i, x) in (0..data.len()-1).map(|i|{(i, data[i])}){
            // TODO: More general indexing op
            //let x = data[i];
            let json_result : Result<_,_> = Json::from_str(x);
            //TODO: check json_result
            let parsed = JsonValue::new(&json_result.unwrap());

            let tree = self.indexes.get_mut(&id).unwrap();

            if tree.contains_key(&parsed) {
                //tree.get_mut(&parsed).unwrap().push(i);
            } else {
                let mut v = Vec::new();
                v.push(i);
                //tree.insert(parsed, v);
            }
        }
        file.seek(SeekFrom::Start(len as u64));
    }
}
