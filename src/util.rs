use std::collections::BTreeMap;
use rustc_serialize::json::{self, Json, ToJson};
use std::mem;

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum JsonValue {
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
