use std::sync::Arc;
use std::collections::HashMap;
use std::cell::Cell;
use orbtk::cell::CloneCell;

//structure to store properties 
pub struct Property{
    name: CloneCell<String>,
    value: CloneCell<String>,
}

impl Property {
    pub fn new(name: &str, value: &str) -> Arc<Self> {
        Arc::new(Property {
        name: CloneCell::new(name.to_owned()),
        value: CloneCell::new(value.to_owned()),
        })
    }

    pub fn name<S: Into<String>>(&self, text: S) -> &Self {
        self.name.set(text.into());
        self
    }

    pub fn value<S: Into<String>>(&self, value: S) -> &Self {
        self.value.set(value.into());
        self
    }
}

// Trait to use either str or i32 
pub trait MyTrait: ::std::string::ToString {}
impl MyTrait for i32 {}
impl MyTrait for str {}
impl MyTrait for &'static str {}
impl MyTrait for String {}

//structure to store tools with properties
#[derive(Clone)]
pub struct Tools {
    tools: HashMap<&'static str,Vec<Arc<Property>>>,
    selected: Cell<&'static str>,
}

impl Tools {
    pub fn new() -> Self {
       Tools {
            tools: HashMap::new(),
            selected: Cell::new(""),
        }
    }

    pub fn insert(&mut self, key : &'static str, properties: Vec<Arc<Property>>) {
        self.tools.insert(key,properties);
    }

        ///get tool property as i32 value
    pub fn get(&self, tool_name: &str  , property: &str) -> Option<i32> {
        let properties = &self.tools[tool_name];
        for a in properties {
            if &a.name.get() == property {
                return Some(a.value.get().parse::<i32>().unwrap());
            }
        } 
        None
    }
    
    /// get tool property as string
    pub fn get_str(&self, tool_name: &str  , property: &str) -> Option<String> {
        let properties = &self.tools[tool_name];
        for a in properties {
            if &a.name.get() == property {
                return Some(a.value.get());
            }
        } 
        None
    }

    ///set tool property as i32 value or str 
    pub fn set <T: MyTrait> (&self, tool_name: &str, property: &str, value: T){
        let properties = &self.tools[tool_name];
        for a in properties {
            if &a.name.get() == property {
                a.value.set(value.to_string());
            }
        } 
    }

    ///get current active tool
    pub fn current(&self) -> String {
        self.get_str("tool","Current").unwrap()
        //self.selected.get().to_string()
        
    }

    ///select active tool
    pub fn select(&self, tool_name: &'static str) {
        self.set("tool","Current",tool_name);
        //self.selected.set(tool_name); //#FIXME does not work
    }

}

