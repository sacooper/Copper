use rustc_serialize::json;

pub enum Entry{
    Insert(json::Json),
    Delete(String),
    Update(String, json::Json)
}
