use crate::auto::{Auto, Length};
use crate::dim3::Dim3;
use crate::package::Package;
use crate::style::{self, ComputedStyle, Style};
use crate::xml_parser::Element;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 渲染节点类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum RenderNodeType {
    Space,
    Object,
    Group,
    Unknown,
}

#[derive(Debug, Default, Clone, Copy)]
struct AbsoluteSpace {
    pub pos: Dim3<Length>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeAttr {
    content_size: Dim3<Length>,
    absolute_pos: Dim3<Length>,
    flex_child_space: Vec<AbsoluteSpace>,
}

impl NodeAttr {
    /// 获取content_size
    pub fn content_size(&self) -> Dim3<Length> {
        self.content_size
    }

    /// 设置content_size
    pub fn set_content_size(&mut self, size: Dim3<Length>) {
        self.content_size = size;
    }
}

/// 渲染节点结构体
#[derive(Debug)]
pub struct RenderNode {
    /// 节点类型
    pub node_type: RenderNodeType,

    /// 节点ID（如果有的话）
    pub id: Option<String>,
    /// 节点名称/标签名
    pub tag_name: String,

    /// 节点文本内容
    pub text_content: String,

    /// 节点上指定的样式
    pub specified_style: Style,

    /// 计算后的样式
    pub computed_style: ComputedStyle,

    pub attr: NodeAttr,

    /// 父节点（弱引用，避免循环引用）
    pub parent: Weak<RefCell<RenderNode>>,

    /// 子节点
    pub children: Vec<Rc<RefCell<RenderNode>>>,
}

impl RenderNode {
    /// 创建新的渲染节点
    pub fn new(tag_name: String, node_type: RenderNodeType) -> Self {
        RenderNode {
            node_type,
            id: None,
            tag_name,
            text_content: String::new(),
            specified_style: Style::new(),
            computed_style: ComputedStyle::default(),
            attr: NodeAttr::default(),
            parent: Weak::new(),
            children: Vec::new(),
        }
    }

    /// 设置节点ID
    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    /// 设置文本内容
    pub fn set_text_content(&mut self, text: String) {
        self.text_content = text;
    }

    /// 设置指定的样式
    pub fn set_specified_style(&mut self, style: Style) {
        self.specified_style = style;
    }

    /// 添加子节点
    pub fn append_child(node: &Rc<RefCell<RenderNode>>, child: Rc<RefCell<RenderNode>>) {
        // 设置子节点的父节点引用
        child.borrow_mut().parent = Rc::downgrade(node);
        node.borrow_mut().children.push(child);
    }

    /// 获取父节点
    pub fn parent(&self) -> Option<Rc<RefCell<RenderNode>>> {
        self.parent.upgrade()
    }
}

pub struct RenderTree<'a> {
    pub root: Rc<RefCell<RenderNode>>,
    package: &'a Package,
}

impl<'a> RenderTree<'a> {
    pub fn new(dom_element: &Element, package: &'a Package) -> Result<Self, anyhow::Error> {
        let root = Self::build_node_recursive(dom_element)?;
        Ok(Self { root, package })
    }

    pub fn calculate(&self) -> Result<(), anyhow::Error> {
        self.calculate_size_recursive(&self.root)?;
        self.calculate_pos_recursive(&self.root)?;
        Ok(())
    }

    fn build_node_recursive(dom_element: &Element) -> Result<Rc<RefCell<RenderNode>>, anyhow::Error> {
        let node_type = determine_node_type(&dom_element.name);
        let mut render_node = RenderNode::new(dom_element.name.clone(), node_type);

        if let Some(id) = dom_element.get_attribute("id") {
            render_node.set_id(id.clone());
        }

        if !dom_element.text.trim().is_empty() {
            render_node.set_text_content(dom_element.text.trim().to_string());
        }

        if let Some(style_str) = dom_element.get_attribute("style") {
            match Style::from_style_string(style_str) {
                Ok(style) => render_node.set_specified_style(style),
                Err(e) => eprintln!(
                    "Warning: Failed to parse style for element '{}': {}",
                    dom_element.name, e
                ),
            }
        }

        let rc_node = Rc::new(RefCell::new(render_node));

        for child_element in &dom_element.children {
            let child_render_node = Self::build_node_recursive(child_element)?;
            RenderNode::append_child(&rc_node, child_render_node);
        }

        Ok(rc_node)
    }

    fn calculate_size_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<(), anyhow::Error> {
        for child in &node.borrow().children {
            self.calculate_size_recursive(child)?;
        }

        let mut node_ref = node.borrow_mut();

        match &node_ref.node_type {
            RenderNodeType::Object => {
                let object_name = &node_ref.text_content;
                if !object_name.is_empty() {
                    if let Some(object) = self.package.get_object(object_name) {
                        match object.space_size() {
                            Ok(space_size) => {
                                node_ref.attr.set_content_size(space_size);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to calculate space size for object '{}': {}",
                                    object_name, e
                                );
                            }
                        }
                    } else {
                        eprintln!(
                            "Warning: Object '{}' not found in package configuration",
                            object_name
                        );
                    }
                }
            }
            RenderNodeType::Group => {
                let group_name = &node_ref.text_content;
                if !group_name.is_empty() {
                    if let Some(group) = self.package.get_group(group_name) {
                        match group.space_size(self.package) {
                            Ok(size) => {
                                node_ref.attr.set_content_size(size);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to calculate space size for group '{}': {}",
                                    group_name, e
                                );
                            }
                        }
                    } else {
                        eprintln!(
                            "Warning: Group '{}' not found in package configuration",
                            group_name
                        );
                    }
                }
            }
            RenderNodeType::Space => {
                let mut total_size = Dim3::default();
                let children = node_ref.children.clone();
                drop(node_ref);

                for child in &children {
                    let child_ref = child.borrow();
                    let child_size = child_ref.attr.content_size();
                    total_size = total_size + child_size;
                }

                let mut node_ref = node.borrow_mut();
                if let Auto::Value(x) = node_ref.specified_style.size.x {
                    if x >= total_size.x {
                        total_size.x = x;
                    }
                }
                if let Auto::Value(y) = node_ref.specified_style.size.y {
                    if y >= total_size.y {
                        total_size.y = y;
                    }
                }
                if let Auto::Value(z) = node_ref.specified_style.size.z {
                    if z >= total_size.z {
                        total_size.z = z;
                    }
                }
                if total_size.x > Length::from_mm(0)
                    || total_size.y > Length::from_mm(0)
                    || total_size.z > Length::from_mm(0)
                {
                    node_ref.attr.set_content_size(total_size);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn calculate_pos_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<(), anyhow::Error> {
        if let Some(parent_rc) = node.borrow().parent() {
            let parent = parent_rc.borrow();
            let mut node_ref = node.borrow_mut();

            match parent.specified_style.display {
                style::Display::Block => {
                    node_ref.attr.absolute_pos = parent.attr.absolute_pos;
                    let max = parent.attr.content_size() - node_ref.attr.content_size;
                    if let Auto::Value(x) = node_ref.specified_style.position_x() {
                        node_ref.attr.absolute_pos.x = x.absolute_pos(Length::from_mm(0), max.x);
                    }
                    if let Auto::Value(y) = node_ref.specified_style.position_y() {
                        node_ref.attr.absolute_pos.y = y.absolute_pos(Length::from_mm(0), max.y);
                    }
                    if let Auto::Value(z) = node_ref.specified_style.position_z() {
                        node_ref.attr.absolute_pos.z = z.absolute_pos(Length::from_mm(0), max.z);
                    }
                }
                style::Display::Stack => {
                    // This is handled by the parent in a stack layout
                }
            }
        }

        {
            let mut node_ref = node.borrow_mut();
            if let style::Display::Stack = node_ref.specified_style.display {
                let mut base = node_ref.attr.content_size.z;
                node_ref.attr.flex_child_space.clear();
                for child in &node_ref.children {
                    let mut child_ref = child.borrow_mut();
                    let child_size = child_ref.attr.content_size;
                    base = base - child_size.z;
                    let child_pos = Dim3::new(
                        node_ref.attr.absolute_pos.x,
                        node_ref.attr.absolute_pos.y,
                        node_ref.attr.absolute_pos.z + base,
                    );
                    child_ref.attr.absolute_pos = child_pos;
                    node_ref.attr.flex_child_space.push(AbsoluteSpace { pos: child_pos });
                }
            }
        }

        for child in &node.borrow().children {
            self.calculate_pos_recursive(child)?;
        }

        Ok(())
    }

    pub fn print_computed(&self) {
        print_render_tree_computed(&self.root, 0);
    }
}

/// 根据标签名确定节点类型
fn determine_node_type(tag_name: &str) -> RenderNodeType {
    match tag_name.to_lowercase().as_str() {
        "space" => RenderNodeType::Space,
        "object" => RenderNodeType::Object,
        "group" => RenderNodeType::Group,
        _ => RenderNodeType::Unknown,
    }
}

/// 打印渲染树
pub fn print_render_tree_computed(node: &Rc<RefCell<RenderNode>>, depth: usize) {
    let node_ref = node.borrow();
    let indent = "  ".repeat(depth);

    print!("{}{}", indent, node_ref.tag_name);

    if let Some(id) = &node_ref.id {
        print!(" #{}", id);
    }

    match &node_ref.node_type {
        RenderNodeType::Space => print!(" [Space]"),
        RenderNodeType::Object => print!(" [Object]"),
        RenderNodeType::Group => print!(" [Group]"),
        RenderNodeType::Unknown => print!(" [Unknown]"),
    }

    if !node_ref.text_content.is_empty() {
        print!(" {}", node_ref.text_content);
    }

    println!();

    print_computed_style_info(&node_ref, depth + 1);

    for child in &node_ref.children {
        print_render_tree_computed(child, depth + 1);
    }
}

/// 打印计算后的样式信息
fn print_computed_style_info(node_ref: &RenderNode, depth: usize) {
    let indent = "  ".repeat(depth);

    println!(
        "{}Computed Size: x={} y={} z={}",
        indent,
        node_ref.computed_style.size.x,
        node_ref.computed_style.size.y,
        node_ref.computed_style.size.z
    );

    println!(
        "{}Computed Position: x={} y={} z={}",
        indent,
        node_ref.attr.absolute_pos.x,
        node_ref.attr.absolute_pos.y,
        node_ref.attr.absolute_pos.z
    );

    println!("{}Content Size: {:?}", indent, node_ref.attr.content_size());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parser::Element as DomElement;
    use std::collections::HashMap;

    #[test]
    fn test_render_node_creation() {
        let node = RenderNode::new("space".to_string(), RenderNodeType::Space);
        assert_eq!(node.tag_name, "space");
        assert_eq!(node.node_type, RenderNodeType::Space);
        assert_eq!(node.id, None);
        assert_eq!(node.text_content, "");
    }

    #[test]
    fn test_render_node_with_id() {
        let mut node = RenderNode::new("object".to_string(), RenderNodeType::Object);
        node.set_id("test-id".to_string());
        assert_eq!(node.id, Some("test-id".to_string()));
    }

    #[test]
    fn test_determine_node_type() {
        assert_eq!(determine_node_type("space"), RenderNodeType::Space);
        assert_eq!(determine_node_type("object"), RenderNodeType::Object);
        assert_eq!(determine_node_type("group"), RenderNodeType::Group);
        assert_eq!(determine_node_type("unknown"), RenderNodeType::Unknown);
    }

    #[test]
    fn test_build_simple_render_tree() {
        let mut element = DomElement::new("space".to_string());
        element
            .attributes
            .insert("id".to_string(), "main".to_string());
        element.attributes.insert(
            "style".to_string(),
            "display:flex;size:10m 10m 10m".to_string(),
        );

        let package = Package::from_file("package.toml").unwrap();
        let render_tree = RenderTree::new(&element, &package).unwrap();
        let node = render_tree.root.borrow();

        assert_eq!(node.tag_name, "space");
        assert_eq!(node.id, Some("main".to_string()));
        assert_eq!(node.node_type, RenderNodeType::Space);
    }

    #[test]
    fn test_node_attr_content_size() {
        let mut attr = NodeAttr::default();

        let content_size = Dim3::new(
            Length::from_cm(10),
            Length::from_cm(10),
            Length::from_cm(10),
        );
        attr.set_content_size(content_size);
        assert_eq!(attr.content_size(), content_size);
    }
}