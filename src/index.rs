use rustc_serialize::json::{self, Json, ToJson};
use util::JsonValue;
use std::collections::BTreeMap;

pub enum IndexEntry {
    Uniq(json::Json),
    Multiple(Vec<json::Json>)
}

//pub trait Idx {
    //fn insert(&mut self, value : json::Json);
//}
pub type Index = BTreeMap<JsonValue, IndexEntry>;

//impl Idx for Index {
    //fn insertJson(&mut self, value: json::Json){
        //let mut data = self.get_mut(&JsonValue::new(&value));
        
    //}
//}
