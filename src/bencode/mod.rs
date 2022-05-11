pub mod decode;
pub mod encode;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Item {
    Integer(usize),
    // question: why use Vec<u8>? not String?
    String(Vec<u8>),
    List(Vec<Item>),
    Dict(BTreeMap<Vec<u8>, Item>),
}

impl Item {
    pub fn get_integer(&self) -> usize {
        let value = match self {
            Item::Integer(i) => i,
            _ => panic!("expected integer"),
        };
        *value
    }

    pub fn get_string(&self) -> Vec<u8> {
        let value = match self {
            Item::String(s) => s,
            _ => panic!("expected string"),
        };
        value.clone()
    }

    pub fn get_list(&self) -> Vec<Item> {
        let value = match self {
            Item::List(l) => l,
            _ => panic!("expected list"),
        };
        value.clone()
    }

    pub fn get_dict(&self) -> BTreeMap<Vec<u8>, Item> {
        let value = match self {
            Item::Dict(d) => d,
            _ => panic!("expected dict"),
        };
        value.clone()
    }
}
