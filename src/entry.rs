use rustc_serialize::json;
use std::ops::Deref;
use rustc_serialize::{Encoder, Encodable};

pub static DELETE_KEY : &'static str = "$$delete";
//pub static UPDATE_KEY : &'static str = "$$delete";

#[derive(Clone, Debug)]
pub enum Entry{
    Insert(json::Json),
    Delete(json::Json),
    Update(json::Json)
}

//impl Deref for Entry {
    //type Target = json::Json;

    //fn deref<'a>(&'a self) -> &'a json::Json {
        //match self {
            //&Entry::Insert(ref x) => { x }
            //&Entry::Delete(ref x) => { x }
            //&Entry::Update(ref x) => { x }
        //}
    //}
//}

impl Encodable for Entry {
    fn encode<S : Encoder>(&self, s : &mut S) -> Result<(), S::Error> {
        match self {
            &Entry::Insert(ref x) => {x.encode(s)}
            &Entry::Delete(ref x) => {x.encode(s)}
            &Entry::Update(ref x) => {x.encode(s)}
        }
    }
}
