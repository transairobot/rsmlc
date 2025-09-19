use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;
use crate::error::{RsmlError, Result};

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub text: String,
    pub children: Vec<Element>,
}

impl Element {
    pub fn new(name: String) -> Self {
        Element {
            name,
            attributes: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
        }
    }
    
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }
    
    pub fn find_child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }
    
    pub fn find_children(&self, name: &str) -> Vec<&Element> {
        self.children.iter().filter(|child| child.name == name).collect()
    }
}

pub fn parse_xml_file(file_path: &str) -> Result<Element> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);
    
    let mut buf = Vec::new();
    let mut stack: Vec<Element> = Vec::new();
    
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut element = Element::new(name);
                
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(&attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    element.attributes.insert(key, value);
                }
                
                stack.push(element);
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut element = Element::new(name);
                
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8_lossy(&attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    element.attributes.insert(key, value);
                }
                
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    // This is the root element
                    return Ok(element);
                }
            }
            Ok(Event::End(_)) => {
                if stack.len() > 1 {
                    let element = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(element);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(element) = stack.last_mut() {
                    element.text.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
            }
            Ok(Event::CData(e)) => {
                if let Some(element) = stack.last_mut() {
                    element.text.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(RsmlError::XmlParse(e)),
            _ => (),
        }
        buf.clear();
    }
    
    if stack.is_empty() {
        Err(RsmlError::InvalidStructure { 
            message: "No root element found".to_string() 
        })
    } else {
        Ok(stack.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_xml() {
        let xml = r#"<root><child id="1">Text</child></root>"#;
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        
        let mut buf = Vec::new();
        let mut stack: Vec<Element> = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut element = Element::new(name);
                    
                    for attr in e.attributes() {
                        let attr = attr.unwrap();
                        let key = String::from_utf8_lossy(&attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        element.attributes.insert(key, value);
                    }
                    
                    stack.push(element);
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut element = Element::new(name);
                    
                    for attr in e.attributes() {
                        let attr = attr.unwrap();
                        let key = String::from_utf8_lossy(&attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        element.attributes.insert(key, value);
                    }
                    
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(element);
                    }
                }
                Ok(Event::End(_)) => {
                    if stack.len() > 1 {
                        let element = stack.pop().unwrap();
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(element);
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(element) = stack.last_mut() {
                        element.text.push_str(&String::from_utf8_lossy(e.as_ref()));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
        
        assert_eq!(stack.len(), 1);
        let root = &stack[0];
        assert_eq!(root.name, "root");
        assert_eq!(root.children.len(), 1);
        
        let child = &root.children[0];
        assert_eq!(child.name, "child");
        assert_eq!(child.get_attribute("id"), Some(&"1".to_string()));
        assert_eq!(child.text.trim(), "Text");
    }
}