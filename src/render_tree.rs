use crate::base::{Auto, Length};
use crate::dim3::Dim3;
use crate::package::Package;
use crate::style::{self, ComputedStyle, FlexDirection, Style};
use crate::xml_parser::Element;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use style::SizeValue;

/// 渲染节点类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum RenderNodeType {
    Space, // 除了group和object的其他tag
    Item,  // Group and Object
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AbsoluteSpace {
    pub pos: Dim3<Length>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeAttr {
    pub absolute_size: Dim3<Length>,
    pub absolute_pos: Dim3<Length>,
    pub flex_child_space: Vec<AbsoluteSpace>,
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
    pub computed_style: Style,

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
            computed_style: Style::default(),
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
        // Find the body node and start position calculation from there
        if let Some(body_node) = self.find_body_node(&self.root) {
            self.calculate_size_by_child_recursive(&body_node)?;
            self.calculate_pos_recursive(&body_node)?;
        }
        Ok(())
    }

    /// Find the body node in the render tree
    fn find_body_node(&self, node: &Rc<RefCell<RenderNode>>) -> Option<Rc<RefCell<RenderNode>>> {
        let node_ref = node.borrow();

        // Check if current node is body
        if node_ref.tag_name == "body" {
            return Some(node.clone());
        }

        // Recursively search in children
        for child in &node_ref.children {
            if let Some(body_node) = self.find_body_node(child) {
                return Some(body_node);
            }
        }

        None
    }

    fn build_node_recursive(
        dom_element: &Element,
    ) -> Result<Rc<RefCell<RenderNode>>, anyhow::Error> {
        let node_type = determine_node_type(&dom_element.name);
        let mut render_node = RenderNode::new(dom_element.name.clone(), node_type);

        if let Some(id) = dom_element.get_attribute("id") {
            render_node.set_id(id.clone());
        }

        if !dom_element.text.trim().is_empty() {
            render_node.set_text_content(dom_element.text.trim().to_string());
        }
        let space_style = "display:flex;flex-direction:z-reverse";
        let item_style = "display:block";
        let body_style = "display:flex;flex-direction:z-reverse;size:100m 100m 100m";

        let mut style = dom_element
            .attributes
            .get("style")
            .cloned()
            .unwrap_or_default();
        if render_node.tag_name == "body" {
            style = body_style.to_owned();
        }
        match render_node.node_type {
            RenderNodeType::Space => style = format!("{};{}", space_style, style),
            RenderNodeType::Item => style = format!("{};{}", item_style, style),
        }

        match Style::from_style_string(&style) {
            Ok(style) => render_node.set_specified_style(style),
            Err(e) => eprintln!(
                "Warning: Failed to parse style for element '{}': {}",
                dom_element.name, e
            ),
        }

        let rc_node = Rc::new(RefCell::new(render_node));

        for child_element in &dom_element.children {
            let child_render_node = Self::build_node_recursive(child_element)?;
            RenderNode::append_child(&rc_node, child_render_node);
        }

        Ok(rc_node)
    }

    fn cal_flex_child_size(node_ref: &RenderNode) -> Dim3<Length> {
        let mut child_total_size: Dim3<Length> = Dim3::default();
        let children = &node_ref.children;
        let flex_direction = node_ref.specified_style.flex_direction.clone();

        for child in children {
            let child_ref = child.borrow();
            let child_size = child_ref.attr.absolute_size;
            match flex_direction {
                FlexDirection::X | FlexDirection::ReverseX => {
                    child_total_size.x += child_size.x;
                    child_total_size.y = child_total_size.y.max(child_size.y);
                    child_total_size.z = child_total_size.z.max(child_size.z);
                }
                FlexDirection::Y | FlexDirection::ReverseY => {
                    child_total_size.x = child_total_size.x.max(child_size.x);
                    child_total_size.y += child_size.y;
                    child_total_size.z = child_total_size.z.max(child_size.z);
                }
                FlexDirection::Z | FlexDirection::ReverseZ => {
                    child_total_size.x = child_total_size.x.max(child_size.x);
                    child_total_size.y = child_total_size.y.max(child_size.y);
                    child_total_size.z += child_size.z;
                }
            }
        }

        // match node_ref.specified_style.size.x {
        //     style::SizeValue::Length(length) => ,
        //     style::SizeValue::Percentage(percentage) => todo!(),
        //     style::SizeValue::Auto => todo!(),
        // }
        // if let Auto::Value(y) = node_ref.specified_style.size.y {
        //     if y >= total_size.y {
        //         total_size.y = y;
        //     }
        // }
        // if let Auto::Value(z) = node_ref.specified_style.size.z {
        //     if z >= total_size.z {
        //         total_size.z = z;
        //     }
        // }
        return child_total_size;
    }

    fn calculate_size_by_parent_recursive(
        &self,
        node: &Rc<RefCell<RenderNode>>,
    ) -> anyhow::Result<()> {
        let mut node_ref = node.borrow_mut();
        let parent = node_ref
            .parent()
            .ok_or(anyhow::anyhow!("Parent node not found"))?;
        let parent_ref = parent.borrow();

        let parent_size = &parent_ref.computed_style.size;
        let size = node_ref.specified_style.size.clone();

        let new_x = match size.x {
            SizeValue::Length(length) => SizeValue::Length(length),
            SizeValue::Percentage(percentage) => {
                if let SizeValue::Length(parent_len) = parent_size.x {
                    SizeValue::Length(parent_len * percentage.value() / 100)
                } else {
                    SizeValue::Auto
                }
            }
            SizeValue::Auto => SizeValue::Auto,
        };

        let new_y = match size.y {
            SizeValue::Length(length) => SizeValue::Length(length),
            SizeValue::Percentage(percentage) => {
                if let SizeValue::Length(parent_len) = parent_size.x {
                    SizeValue::Length(parent_len * percentage.value() / 100)
                } else {
                    SizeValue::Auto
                }
            }
            SizeValue::Auto => SizeValue::Auto,
        };

        let new_z = match size.z {
            SizeValue::Length(length) => SizeValue::Length(length),
            SizeValue::Percentage(percentage) => {
                if let SizeValue::Length(parent_len) = parent_size.z {
                    SizeValue::Length(parent_len * percentage.value() / 100)
                } else {
                    SizeValue::Auto
                }
            }
            SizeValue::Auto => SizeValue::Auto,
        };

        if node_ref.children.len() == 1 {
            node_ref.children[0].borrow_mut().attr.absolute_pos = node_ref.attr.absolute_size;
        }
        let children = node_ref.children.clone();
        drop(node_ref);
        for child in children {
            self.calculate_size_by_parent_recursive(&child);
        }
        Ok(())
    }

    fn calculate_size_by_child_recursive(
        &self,
        node: &Rc<RefCell<RenderNode>>,
    ) -> Result<(), anyhow::Error> {
        for child in &node.borrow().children {
            self.calculate_size_by_child_recursive(child)?;
        }

        let mut node_ref = node.borrow_mut();

        match &node_ref.node_type {
            RenderNodeType::Item => {
                let name = &node_ref.text_content;
                println!("name={}", name);
                node_ref.attr.absolute_size = self.package.get_space_size(name)?;
                println!("get_space_size end");
            }
            RenderNodeType::Space => match node_ref.specified_style.display {
                style::Display::Block => todo!(),
                style::Display::Flex => {
                    node_ref.attr.absolute_size = Self::cal_flex_child_size(&node_ref)
                }
                style::Display::Cube => todo!(),
            },
        }

        Ok(())
    }

    fn calculate_pos_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<(), anyhow::Error> {
        let mut node_ref = node.borrow_mut();

        if let Some(parent_rc) = node_ref.parent() {
            let parent = parent_rc.borrow();

            match parent.specified_style.display {
                style::Display::Block => {
                    node_ref.attr.absolute_pos = parent.attr.absolute_pos;
                    let max = parent.attr.absolute_size - node_ref.attr.absolute_size;
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
                style::Display::Flex => {
                    // This is handled by the parent in a flex layout
                }
                style::Display::Cube => todo!(),
            }
        }

        {
            if let style::Display::Flex = node_ref.specified_style.display {
                let mut base = node_ref.attr.absolute_size.z;
                let mut flex_child_space = vec![];

                for child in &node_ref.children {
                    let mut child_ref = child.borrow_mut();
                    let child_size = child_ref.attr.absolute_size;
                    base = base - child_size.z;
                    let child_pos = Dim3::new(
                        Length::from_cm(0),
                        Length::from_cm(0),
                        node_ref.attr.absolute_pos.z + base,
                    );

                    child_ref.attr.absolute_pos = child_pos;
                    flex_child_space.push(AbsoluteSpace { pos: child_pos });
                }
                node_ref.attr.flex_child_space = flex_child_space;
            }
        }

        let children = node_ref.children.clone();
        drop(node_ref);
        for child in children {
            self.calculate_pos_recursive(&child)?;
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
        "object" | "group" => RenderNodeType::Item,
        _ => RenderNodeType::Space,
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
        RenderNodeType::Item => print!(" [Object]"),
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

    // println!(
    //     "{}Computed Size: x={} y={} z={}",
    //     indent,
    //     node_ref.computed_style.size.x,
    //     node_ref.computed_style.size.y,
    //     node_ref.computed_style.size.z
    // );

    println!(
        "{}Computed Position: x={} y={} z={}",
        indent,
        node_ref.attr.absolute_pos.x,
        node_ref.attr.absolute_pos.y,
        node_ref.attr.absolute_pos.z
    );

    println!("{}Content Size: {:?}", indent, node_ref.attr.absolute_size);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parser::Element as DomElement;

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
        let mut node = RenderNode::new("object".to_string(), RenderNodeType::Item);
        node.set_id("test-id".to_string());
        assert_eq!(node.id, Some("test-id".to_string()));
    }

    #[test]
    fn test_determine_node_type() {
        assert_eq!(determine_node_type("space"), RenderNodeType::Space);
        assert_eq!(determine_node_type("object"), RenderNodeType::Item);
        assert_eq!(determine_node_type("group"), RenderNodeType::Item);
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
        attr.absolute_size = content_size;
        assert_eq!(attr.absolute_size, content_size);
    }
}
