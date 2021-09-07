use std::rc::Rc;

use fnv::FnvHashMap;
use wasm_bindgen::JsCast;

#[derive(Clone)]
pub struct InternedStr(Rc<InternedData>);

impl InternedStr {
    pub fn as_js_string(&self) -> &js_sys::JsString {
        &self.0.js_string
    }
}

impl InternedStr {
    pub fn as_str(&self) -> &str {
        &self.0.as_ref().value
    }
}

struct InternedData {
    value: String,
    js_string: js_sys::JsString,
}

impl PartialEq for InternedStr {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for InternedStr {}

impl std::fmt::Display for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InternedStr").field(&self.as_str()).finish()
    }
}

impl std::cmp::PartialOrd for InternedStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.as_ref().value.partial_cmp(&other.0.as_ref().value)
    }
}

impl std::cmp::Ord for InternedStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().value.cmp(&other.0.as_ref().value)
    }
}

impl std::hash::Hash for InternedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

pub struct Interner {
    values: FnvHashMap<String, InternedStr>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            values: FnvHashMap::default(),
        }
    }

    pub fn intern(&mut self, value: String) -> InternedStr {
        if let Some(value) = self.values.get(&value) {
            value.clone()
        } else {
            let s = InternedStr(Rc::new(InternedData {
                value: value.clone(),
                js_string: wasm_bindgen::JsValue::NULL.unchecked_into(),
            }));

            self.values.insert(value, s.clone());
            s
        }
    }
}
