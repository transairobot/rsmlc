use crate::auto::{Auto, Length};
use crate::style::{ComputedStyle, Style};
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

#[derive(Debug)]
pub struct NodeAttr {
    content_size: Option<(Length, Length, Length)>,
}

impl NodeAttr {
    /// 获取content_size
    pub fn content_size(&self) -> Option<(Length, Length, Length)> {
        self.content_size
    }

    /// 设置content_size
    pub fn set_content_size(&mut self, size: (Length, Length, Length)) {
        self.content_size = Some(size);
    }
}

impl Default for NodeAttr {
    fn default() -> Self {
        Self {
            content_size: Default::default(),
        }
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

/// 从DOM元素构建渲染树
pub fn build_render_tree(dom_element: &Element) -> Result<Rc<RefCell<RenderNode>>, anyhow::Error> {
    // 创建根渲染节点
    let node_type = determine_node_type(&dom_element.name);
    let mut render_node = RenderNode::new(dom_element.name.clone(), node_type);

    // 设置ID（如果有的话）
    if let Some(id) = dom_element.get_attribute("id") {
        render_node.set_id(id.clone());
    }

    // 设置文本内容
    if !dom_element.text.trim().is_empty() {
        render_node.set_text_content(dom_element.text.trim().to_string());
    }

    // 解析并设置样式（如果有的话）
    if let Some(style_str) = dom_element.get_attribute("style") {
        match Style::from_style_string(style_str) {
            Ok(style) => render_node.set_specified_style(style),
            Err(e) => eprintln!(
                "Warning: Failed to parse style for element '{}': {}",
                dom_element.name, e
            ),
        }
    }

    // 创建Rc包装的节点
    let rc_node = Rc::new(RefCell::new(render_node));

    // 递归处理子元素
    for child_element in &dom_element.children {
        let child_render_node = build_render_tree(child_element)?;
        RenderNode::append_child(&rc_node, child_render_node);
    }

    Ok(rc_node)
}

pub fn precalculate(
    render_tree: &Rc<RefCell<RenderNode>>,
    package: &crate::package::Package,
) -> Result<(), anyhow::Error> {
    precalculate_recursive(render_tree, package)
}

fn precalculate_recursive(
    node: &Rc<RefCell<RenderNode>>,
    package: &crate::package::Package,
) -> Result<(), anyhow::Error> {
    // 先递归处理子节点
    for child in &node.borrow().children {
        precalculate_recursive(child, package)?;
    }

    // 然后计算当前节点的content_size
    let mut node_ref = node.borrow_mut();

    // 如果是Object或Group节点，计算content_size
    match &node_ref.node_type {
        RenderNodeType::Object => {
            // 对于Object节点，text_content就是object_name
            let object_name = &node_ref.text_content;
            if !object_name.is_empty() {
                if let Some(object) = package.get_object(object_name) {
                    match object.space_size() {
                        Ok((x, y, z)) => {
                            node_ref.attr.set_content_size((x, y, z));
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
            // 对于Group节点，text_content就是group_name
            let group_name = &node_ref.text_content;
            if !group_name.is_empty() {
                if let Some(group) = package.get_group(group_name) {
                    match group.space_size(package) {
                        Ok((x, y, z)) => {
                            node_ref.attr.set_content_size((x, y, z));
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
            // 对于Space节点，content_size是所有子节点content_size的累加
            let mut total_x = Length::from_mm(0);
            let mut total_y = Length::from_mm(0);
            let mut total_z = Length::from_mm(0);
            let children = node_ref.children.clone();
            drop(node_ref); // 释放借用

            // 累加所有子节点的content_size
            for child in &children {
                let child_ref = child.borrow();
                if let Some((child_x, child_y, child_z)) = child_ref.attr.content_size() {
                    total_x = total_x + child_x;
                    total_y = total_y + child_y;
                    total_z = total_z + child_z;
                }
            }

            // 重新获取借用以设置content_size
            node_ref = node.borrow_mut();
            if total_x > Length::from_mm(0)
                || total_y > Length::from_mm(0)
                || total_z > Length::from_mm(0)
            {
                node_ref.attr.set_content_size((total_x, total_y, total_z));
            }
        }
        _ => {}
    }

    Ok(())
}

/// 计算渲染树的布局
pub fn calculate_layout(render_tree: &Rc<RefCell<RenderNode>>) -> Result<(), anyhow::Error> {
    // 首先计算根节点的布局
    calculate_node_layout(
        render_tree,
        (
            Length::from_m(100),
            Length::from_m(100),
            Length::from_m(100),
        ),
    )?;
    Ok(())
}

/// 计算单个节点的布局
fn calculate_node_layout(
    node: &Rc<RefCell<RenderNode>>,
    parent_size: (Length, Length, Length),
) -> Result<(), anyhow::Error> {
    let mut node_ref = node.borrow_mut();

    // 计算节点的尺寸
    let computed_size = calculate_size(
        &node_ref.specified_style.size.0,
        &node_ref.specified_style.size.1,
        &node_ref.specified_style.size.2,
        parent_size,
    );

    // 计算节点的位置
    let computed_pos = calculate_position(
        &node_ref.specified_style.position,
        computed_size,
        parent_size,
    );

    // 更新计算后的样式
    node_ref.computed_style = ComputedStyle {
        size: computed_size,
        pos: computed_pos,
    };

    // 如果是flex容器，需要特殊处理子元素
    if matches!(
        node_ref.specified_style.display,
        crate::style::Display::Stack
    ) {
        // 对于flex容器，需要先计算所有子元素的尺寸，然后根据flex规则重新计算位置
        drop(node_ref); // 释放借用，以便递归调用

        // 先递归计算所有子元素的尺寸
        let mut child_sizes = Vec::new();
        for child in &node.borrow().children {
            let child_size = {
                let child_ref = child.borrow();
                calculate_size(
                    &child_ref.specified_style.size.0,
                    &child_ref.specified_style.size.1,
                    &child_ref.specified_style.size.2,
                    (computed_size.0, computed_size.1, computed_size.2),
                )
            };
            child_sizes.push(child_size);
        }

        // 然后重新计算子元素的位置（使用flex布局算法）
        calculate_flex_layout(node, computed_size, &child_sizes)?;
    } else {
        // 对于非flex容器，直接递归计算子元素
        let computed_size = (computed_size.0, computed_size.1, computed_size.2);
        drop(node_ref); // 释放借用，以便递归调用

        for child in &node.borrow().children {
            calculate_node_layout(child, computed_size)?;
        }
    }

    Ok(())
}

/// 计算尺寸
fn calculate_size(
    size_x: &Auto<Length>,
    size_y: &Auto<Length>,
    size_z: &Auto<Length>,
    parent_size: (Length, Length, Length),
) -> (Length, Length, Length) {
    let x = match size_x {
        Auto::Value(len) => *len,
        Auto::Auto => {
            // 对于auto，我们给一个默认值，实际应该根据内容或布局算法确定
            parent_size.0
        }
    };

    let y = match size_y {
        Auto::Value(len) => *len,
        Auto::Auto => {
            // 对于auto，我们给一个默认值，实际应该根据内容或布局算法确定
            parent_size.1
        }
    };

    let z = match size_z {
        Auto::Value(len) => *len,
        Auto::Auto => {
            // 对于auto，我们给一个默认值，实际应该根据内容或布局算法确定
            parent_size.1
        }
    };

    (x, y, z)
}

/// 计算位置
fn calculate_position(
    position: &Option<crate::style::Position>,
    size: (Length, Length, Length),
    parent_size: (Length, Length, Length),
) -> (Length, Length, Length) {
    if let Some(pos) = position {
        let x = match pos.x {
            crate::style::AxisPos::Min => Length::from_mm(0),
            crate::style::AxisPos::Max => {
                if parent_size.0 > size.0 {
                    parent_size.0 - size.0
                } else {
                    Length::from_mm(0)
                }
            }
            crate::style::AxisPos::Random => {
                // 简单实现随机位置
                Length::from_mm(0)
            }
            crate::style::AxisPos::Length(len) => len,
        };

        let y = match pos.y {
            crate::style::AxisPos::Min => Length::from_mm(0),
            crate::style::AxisPos::Max => {
                if parent_size.1 > size.1 {
                    parent_size.1 - size.1
                } else {
                    Length::from_mm(0)
                }
            }
            crate::style::AxisPos::Random => {
                // 简单实现随机位置
                Length::from_mm(0)
            }
            crate::style::AxisPos::Length(len) => len,
        };

        let z = match pos.z {
            crate::style::AxisPos::Min => Length::from_mm(0),
            crate::style::AxisPos::Max => {
                if parent_size.2 > size.2 {
                    parent_size.2 - size.2
                } else {
                    Length::from_mm(0)
                }
            }
            crate::style::AxisPos::Random => {
                // 简单实现随机位置
                Length::from_mm(0)
            }
            crate::style::AxisPos::Length(len) => len,
        };

        (x, y, z)
    } else {
        // 默认位置为(0, 0, 0)
        (Length::from_mm(0), Length::from_mm(0), Length::from_mm(0))
    }
}

/// 计算flex布局
fn calculate_flex_layout(
    node: &Rc<RefCell<RenderNode>>,
    container_size: (Length, Length, Length),
    child_sizes: &[(Length, Length, Length)],
) -> Result<(), anyhow::Error> {
    let node_ref = node.borrow();

    // 获取flex方向
    let flex_direction = node_ref
        .specified_style
        .stack_direction
        .clone()
        .unwrap_or(crate::style::StackDirection::Z);

    // 获取justify-content
    let justify_content = node_ref
        .specified_style
        .justify_content
        .clone()
        .unwrap_or(crate::style::JustifyContent::FlexStart);

    // 根据flex方向和justify-content计算子元素的位置
    let container_size = (container_size.0, container_size.1, container_size.2);

    // 释放借用以便修改子元素
    drop(node_ref);

    // 计算子元素位置
    // match flex_direction {
    //     crate::style::FlexDirection::X => {
    //         calculate_flex_layout_x(node, container_size, child_sizes, &justify_content)?;
    //     }
    // crate::style::FlexDirection::Y => {
    //     calculate_flex_layout_y(node, container_size, child_sizes, &justify_content)?;
    // }
    // crate::style::FlexDirection::Z => {
    //     calculate_flex_layout_z(node, container_size, child_sizes, &justify_content)?;
    // }
    // }

    Ok(())
}

/// 计算X方向的flex布局
fn calculate_flex_layout_x(
    node: &Rc<RefCell<RenderNode>>,
    container_size: (Length, Length, Length),
    child_sizes: &[(Length, Length, Length)],
    justify_content: &crate::style::JustifyContent,
) -> Result<(), anyhow::Error> {
    let node_ref = node.borrow();
    let children_count = node_ref.children.len();

    if children_count == 0 {
        return Ok(());
    }

    // 计算子元素总宽度
    let total_width: Length = child_sizes.iter().map(|s| s.0).sum();

    // 根据justify-content确定起始位置
    let start_x = match justify_content {
        crate::style::JustifyContent::FlexStart => Length::from_m(0),
        crate::style::JustifyContent::FlexEnd => {
            if container_size.0 > total_width {
                container_size.0 - total_width
            } else {
                Length::from_m(0)
            }
        }
        crate::style::JustifyContent::Center => {
            if container_size.0 > total_width {
                (container_size.0 - total_width) / 2
            } else {
                Length::from_m(0)
            }
        }
        // 其他情况暂时使用FlexStart
        _ => Length::from_m(0),
    };

    // 释放借用以便递归调用
    drop(node_ref);

    // 计算并设置每个子元素的位置
    let mut current_x = start_x;
    for (i, child) in node.borrow().children.iter().enumerate() {
        let child_size = child_sizes[i];

        // 更新子元素的计算样式
        {
            let mut child_ref = child.borrow_mut();
            child_ref.computed_style.pos.0 = current_x;
            // Y和Z位置保持不变或根据其他规则计算
        }

        // 递归计算子元素的布局
        calculate_node_layout(child, container_size)?;

        // 更新当前X位置
        current_x = current_x + child_size.0;
    }

    Ok(())
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

    // 打印节点信息
    print!("{}{}", indent, node_ref.tag_name);

    // 打印ID（如果有的话）
    if let Some(id) = &node_ref.id {
        print!(" #{}", id);
    }

    // 打印节点类型
    match &node_ref.node_type {
        RenderNodeType::Space => print!(" [Space]"),
        RenderNodeType::Object => print!(" [Object]"),
        RenderNodeType::Group => print!(" [Group]"),
        RenderNodeType::Unknown => print!(" [Unknown]"),
    }

    // 打印文本内容（如果有的话）
    if !node_ref.text_content.is_empty() {
        print!(" {}", node_ref.text_content);
    }

    println!();

    // 打印计算后的样式信息
    print_computed_style_info(&node_ref, depth + 1);

    // 递归打印子节点
    for child in &node_ref.children {
        print_render_tree_computed(child, depth + 1);
    }
}

/// 打印样式信息
fn print_style_info(style: &Style, depth: usize) {
    let indent = "  ".repeat(depth);

    // 打印尺寸信息
    print!("{}Size: ", indent);
    match style.size_x() {
        Auto::Auto => print!("x=auto "),
        Auto::Value(len) => print!("x={} ", len),
    }
    match style.size_y() {
        Auto::Auto => print!("y=auto "),
        Auto::Value(len) => print!("y={} ", len),
    }
    match style.size_z() {
        Auto::Auto => print!("z=auto"),
        Auto::Value(len) => print!("z={}", len),
    }
    println!();

    // 打印显示类型
    println!("{}Display: {:?}", indent, style.display);

    // 打印justify-content（如果有的话）
    if let Some(justify_content) = &style.justify_content {
        println!("{}Justify Content: {:?}", indent, justify_content);
    }

    // 打印stack-direction（如果有的话）
    if let Some(stack_direction) = &style.stack_direction {
        println!("{}Flex Direction: {:?}", indent, stack_direction);
    }

    // 打印位置信息（如果有的话）
    if let Some(position) = &style.position {
        println!(
            "{}Position: x={:?} y={:?} z={:?}",
            indent, position.x, position.y, position.z
        );
    }
}

/// 打印计算后的样式信息
fn print_computed_style_info(node_ref: &RenderNode, depth: usize) {
    let indent = "  ".repeat(depth);

    println!(
        "{}Computed Size: x={} y={} z={}",
        indent,
        node_ref.computed_style.size.0,
        node_ref.computed_style.size.1,
        node_ref.computed_style.size.2
    );

    println!(
        "{}Computed Position: x={} y={} z={}",
        indent,
        node_ref.computed_style.pos.0,
        node_ref.computed_style.pos.1,
        node_ref.computed_style.pos.2
    );

    // 打印content_size（如果有的话）
    if let Some(content_size) = node_ref.attr.content_size() {
        println!("{}Content Size: {:?}", indent, content_size);
    }
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
        // 创建一个简单的DOM元素
        let mut element = DomElement::new("space".to_string());
        element
            .attributes
            .insert("id".to_string(), "main".to_string());
        element.attributes.insert(
            "style".to_string(),
            "display:flex;size:10m 10m 10m".to_string(),
        );

        // 构建渲染树
        let render_tree = build_render_tree(&element).unwrap();
        let node = render_tree.borrow();

        assert_eq!(node.tag_name, "space");
        assert_eq!(node.id, Some("main".to_string()));
        assert_eq!(node.node_type, RenderNodeType::Space);
    }

    #[test]
    fn test_node_attr_content_size() {
        let mut attr = NodeAttr::default();
        assert_eq!(attr.content_size(), None);

        let content_size = (
            Length::from_cm(10),
            Length::from_cm(10),
            Length::from_cm(10),
        );
        attr.set_content_size(content_size);
        assert_eq!(attr.content_size(), Some(content_size));
    }
}
