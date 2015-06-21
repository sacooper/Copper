use rustc_serialize::json::{Json, ToJson};
use rustc_serialize::json;
use rand::{thread_rng, Rng};
use mmap::{MemoryMap, MapOption};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::fs;
use std::io::{Write, SeekFrom, Seek};
use std::io::Error as IOError;
use std::os::unix::prelude::AsRawFd;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::borrow::Borrow;
use std::str::FromStr;
use std::mem;
use entry::*;

type Field = String;
type Index = BTreeMap<JsonValue, Vec<usize>>;

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
enum JsonValue {
    I64(i64),
    F64(i64),
    U64(u64),
    String(String),
    Boolean(bool),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
    Null
}

impl JsonValue {
    pub fn new(j: &Json) -> JsonValue{
        match j {
            &Json::Null => { JsonValue::Null },
            &Json::I64(ref x) => { JsonValue::I64(*x) },
            &Json::U64(ref x) => { JsonValue::U64(*x) },
            &Json::F64(ref x) => { JsonValue::F64(unsafe{mem::transmute(*x)})},
            &Json::String(ref s) => {JsonValue::String(s.clone()) },
            &Json::Boolean(ref b) => { JsonValue::Boolean(*b) },
            &Json::Array(ref a) => { JsonValue::Array(a.iter().map(JsonValue::new).collect::<Vec<JsonValue>>()) }
            &Json::Object(ref t) => { JsonValue::Object(t.iter().map(|(k, v)|{ (k.clone(), JsonValue::new(v))}).collect::<BTreeMap<String,JsonValue>>()) }
        }
    }
}


pub struct Database {
    file: fs::File,
    temp: fs::File,
    log: VecDeque<Entry>,
    indexes: HashMap<Field,Index>
}

pub enum DBError{
    EncodeError(json::EncoderError),
    WriteError(IOError)
}

impl Database {
    pub fn new(name: &str) -> Database { 
        let mut f = fs::OpenOptions::new().read(true)
            .write(true)
            .create(true)
            .open(name)
            .unwrap();

        let mut temp = fs::OpenOptions::new().read(true)
            .write(true)
            .create(true)
            .open(".copper.tmp.db")
            .unwrap();

        let size= 1024*1024*32;
        temp.seek(SeekFrom::Start(size)).unwrap();
        temp.write_all(&[0]).unwrap();
        temp.seek(SeekFrom::Start(0)).unwrap();

        let mut db = Database{
            file: f,
            temp: temp,
            log: VecDeque::new(),
            indexes: HashMap::new()
        };
        db.load_database();
        db
    }

    pub fn insert(&mut self, data: &json::ToJson) -> Result<(), DBError>{
        let js = data.to_json();
        let mut obj = if js.is_object() { js } else {
            let x : BTreeMap<String, Json> = BTreeMap::new();
            x.to_json()
        };

        if let Json::Object(ref mut map) = obj {
            map.insert("_id".to_string(), Json::String(Database::gen_random_id()));
        } else {
            unreachable!()
        }

        let enc = json::encode(&obj);

        if enc.is_err() { return Err(DBError::EncodeError(enc.err().unwrap())) }

        let s = enc.unwrap().replace("\n", "") + "\n";
        let res = self.file.write(&s.into_bytes());
        
        //self.updateIndexes();

        res.map_err(|e|{ DBError::WriteError(e) }).map(|_|{()})
    }

    fn gen_random_id() -> String {
        thread_rng().gen_ascii_chars().filter(|c| c.is_alphanumeric()).take(15).collect::<String>()
    }


    fn load_database(&mut self){
        let id = "_id".to_string();
        self.indexes.insert(id.clone(), BTreeMap::new());
        
        // Generate _id index
        let mmap_opts = &[
            // Then make the mapping *public* so it is written back to the file
            //MapOption::MapNonStandardFlags(libc::consts::os::posix88::MAP_SHARED),
            MapOption::MapReadable,
            MapOption::MapWritable,
            MapOption::MapFd(self.file.as_raw_fd()) 
        ];
        
        
        let len = self.file.metadata().unwrap().len() as usize;
        let mmap = MemoryMap::new(len, mmap_opts).unwrap();
        let raw = unsafe{String::from_raw_parts(mmap.data(), len, len)};
        let data = raw.split('\n');

        for (i, x) in data.enumerate() {
            // TODO: More general indexing op
            let parsed = JsonValue::new(&Json::from_str(x).unwrap());
            let tree = self.indexes.get_mut(&id).unwrap();
            if tree.contains_key(&parsed) {
                tree.get_mut(&parsed).unwrap().push(i);
            } else {
                let mut v = Vec::new();
                v.push(i);
                tree.insert(parsed, v);
            }
        }
        self.file.seek(SeekFrom::Start(len as u64));
    }
}
