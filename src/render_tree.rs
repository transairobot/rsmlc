use crate::base::Length;
use crate::dim3::Dim3;
use crate::error::{Result, RsmlError};
use crate::package::Package;
use crate::style::{self, FlexDirection, SpaceSize, Style};
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
    pub fn new(dom_element: &Element, package: &'a Package) -> Result<Self> {
        let root = Self::build_node_recursive(dom_element)?;
        Ok(Self { root, package })
    }

    pub fn calculate(&self) -> Result<()> {
        // Find the body node and start position calculation from there
        if let Some(body_node) = self.find_body_node(&self.root) {
            {
                let len = SizeValue::Length(Length::from_m(100.0));
                body_node.borrow_mut().computed_style.size = SpaceSize {
                    x: len.clone(),
                    y: len.clone(),
                    z: len.clone(),
                }
            }
            self.calculate_size_by_parent_recursive(&body_node)?;
            self.calculate_size_by_child_recursive(&body_node)?;
            self.calculate_size_by_parent_recursive(&body_node)?;
            self.calculate_pos_recursive(&body_node)?;
            self.print_computed();

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

    fn build_node_recursive(dom_element: &Element) -> Result<Rc<RefCell<RenderNode>>> {
        let node_type = determine_node_type(&dom_element.name);
        let mut render_node = RenderNode::new(dom_element.name.clone(), node_type);

        if let Some(id) = dom_element.get_attribute("id") {
            render_node.set_id(id.clone());
        }

        if !dom_element.text.trim().is_empty() {
            render_node.set_text_content(dom_element.text.trim().to_string());
        }
        let space_style = "display:flex;flex-direction:z-reverse";
        let item_style = "display:flex";
        let body_style = "display:flex;flex-direction:z-reverse;size:10m 10m 10m";

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
    fn cal_flex_child_size(node_ref: &RenderNode) -> SpaceSize {
        let mut child_total_size = node_ref.specified_style.size.create_self_by_auto_to_zero();

        println!(
            "id={:?} raw_size={} create_size={}",
            node_ref.id, node_ref.specified_style.size, child_total_size
        );
        let children = &node_ref.children;
        let flex_direction = node_ref.specified_style.flex_direction.clone();

        for child in children {
            let child_ref = child.borrow();
            let child_size = &child_ref.computed_style.size;
            match flex_direction {
                FlexDirection::X | FlexDirection::ReverseX => {
                    child_total_size.x.add(&child_size.x);
                    child_total_size.y.max(&child_size.y);
                    child_total_size.z.max(&child_size.z);
                }
                FlexDirection::Y | FlexDirection::ReverseY => {
                    child_total_size.x.max(&child_size.x);
                    child_total_size.y.add(&child_size.y);
                    child_total_size.z.max(&child_size.z);
                }
                FlexDirection::Z | FlexDirection::ReverseZ => {
                    child_total_size.x.max(&child_size.x);
                    child_total_size.y.max(&child_size.y);
                    child_total_size.z.add(&child_size.z);
                }
            }
        }

        return child_total_size;
    }

    /// Helper function to calculate a single dimension size based on parent size
    fn calculate_dimension_size(
        size_value: &SizeValue,
        parent_size_value: &SizeValue,
    ) -> SizeValue {
        match size_value {
            SizeValue::Length(length) => SizeValue::Length(*length),
            SizeValue::Percentage(percentage) => {
                if let SizeValue::Length(parent_len) = parent_size_value {
                    SizeValue::Length(*parent_len * percentage.value() / 100)
                } else {
                    SizeValue::Auto
                }
            }
            SizeValue::Auto => SizeValue::Auto,
        }
    }

    fn calculate_size_by_parent_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<()> {
        let mut node_ref = node.borrow_mut();
        let parent = node_ref.parent().ok_or(RsmlError::RenderTree {
            message: "Parent node not found".to_string(),
        })?;
        let parent_ref = parent.borrow();

        let parent_size = &parent_ref.computed_style.size;
        let size = node_ref.specified_style.size.clone();

        // Calculate sizes for each dimension using the helper function
        let new_x = Self::calculate_dimension_size(&size.x, &parent_size.x);
        let new_y = Self::calculate_dimension_size(&size.y, &parent_size.y);
        let new_z = Self::calculate_dimension_size(&size.z, &parent_size.z);

        node_ref
            .computed_style
            .size
            .assign_priority(SpaceSize::new(new_x, new_y, new_z));
        println!("id={:?} size={}", node_ref.id, node_ref.computed_style.size);
        match parent_ref.specified_style.display {
            // parent是flex，就是用flex-basis计算size
            style::Display::Flex => {
                let basis_size = node_ref
                    .specified_style
                    .flex_basis
                    .to_space_size(&parent_ref.specified_style.flex_direction);
                let new_x = Self::calculate_dimension_size(&basis_size.x, &size.x);
                let new_y = Self::calculate_dimension_size(&basis_size.y, &size.y);
                let new_z = Self::calculate_dimension_size(&basis_size.z, &size.z);
                node_ref
                    .computed_style
                    .size
                    .assign_priority(SpaceSize::new(new_x, new_y, new_z));
            }
            style::Display::Cube => {}
        }
        let children = node_ref.children.clone();
        drop(node_ref);
        for child in children {
            self.calculate_size_by_parent_recursive(&child)?;
        }
        Ok(())
    }

    fn calculate_size_by_child_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<()> {
        for child in &node.borrow().children {
            self.calculate_size_by_child_recursive(child)?;
        }

        let mut node_ref = node.borrow_mut();

        match &node_ref.node_type {
            RenderNodeType::Item => {
                let name = &node_ref.text_content;
                node_ref.computed_style.size =
                    SpaceSize::from_dim3_length(self.package.get_space_size(name)?);
            }
            RenderNodeType::Space => match node_ref.specified_style.display {
                style::Display::Flex => {
                    if !node_ref.computed_style.size.all_length() {
                        let child_total_size = Self::cal_flex_child_size(&node_ref);
                        println!("tag={:?} child_size={}", node_ref.id, child_total_size);
                        node_ref
                            .computed_style
                            .size
                            .assign_priority(child_total_size);
                    }
                }
                style::Display::Cube => {
                    if node_ref.computed_style.size.has_auto() {
                        return Err(RsmlError::CubeSizeError);
                    }
                }
            },
        }

        Ok(())
    }

    fn calculate_pos_recursive(&self, node: &Rc<RefCell<RenderNode>>) -> Result<()> {
        let mut node_ref = node.borrow_mut();

        // 不会有auto，全是Length
        match node_ref.specified_style.display {
            style::Display::Flex => {
                // 计算Flex布局中子元素的位置
                self.calculate_flex_child_positions(&mut node_ref)?;
            }
            style::Display::Cube => todo!(),
        }

        let children = node_ref.children.clone();
        drop(node_ref);
        for child in children {
            self.calculate_pos_recursive(&child)?;
        }

        Ok(())
    }

    /// 计算Flex布局中子元素的位置
    fn calculate_flex_child_positions(&self, node_ref: &mut RenderNode) -> Result<()> {
        let flex_direction = &node_ref.specified_style.flex_direction;
        let justify_content = &node_ref.specified_style.justify_content;
        let align_items = &node_ref.specified_style.align_items;
        let node_size = &node_ref.computed_style.size;

        // 获取节点的尺寸（转换为Length）
        let node_length = node_size.get_length().ok_or(RsmlError::RenderTree {
            message: format!("Unable to calculate the size of node({:?})", node_ref.id),
        })?;

        // 计算子元素的总尺寸
        let mut total_child_size =
            Dim3::new(Length::from_mm(0), Length::from_mm(0), Length::from_mm(0));
        let mut child_lengths = Vec::new();

        // 收集子元素的尺寸信息
        for child in &node_ref.children {
            let child_ref = child.borrow();
            let child_size =
                &child_ref
                    .computed_style
                    .size
                    .get_length()
                    .ok_or(RsmlError::RenderTree {
                        message: format!(
                            "Unable to calculate the size of node({:?})",
                            child_ref.id
                        ),
                    })?;
            total_child_size.x += child_size.x;
            total_child_size.y += child_size.y;
            total_child_size.z += child_size.z;
            child_lengths.push(child_size.clone());
        }

        // 根据flex_direction、justify_content和align_items计算子元素的位置
        let mut child_positions = Vec::new();

        match flex_direction {
            FlexDirection::X | FlexDirection::ReverseX => {
                // 主轴是X轴，交叉轴是Y和Z
                let free_space = node_length.x.mm() as f64 - total_child_size.x.mm() as f64;
                let mut positions = self.calculate_positions_on_axis(
                    free_space,
                    &child_lengths
                        .iter()
                        .map(|dim| dim.x.mm() as f64)
                        .collect::<Vec<_>>(),
                    justify_content,
                );

                // 如果是ReverseX，需要反转位置
                if matches!(flex_direction, FlexDirection::ReverseX) {
                    positions.reverse();
                }

                // 计算每个子元素的完整位置
                for (i, child_x_pos) in positions.iter().enumerate() {
                    let mut pos = Dim3::new(
                        Length::from_mm(*child_x_pos as u32),
                        Length::from_mm(0),
                        Length::from_mm(0),
                    );

                    // 根据align-items计算Y和Z轴位置
                    if i < child_lengths.len() {
                        let child_size = child_lengths[i];
                        
                        // 计算Y轴位置（第一个交叉轴）
                        pos.y = match align_items.cross1 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.y.mm() - child_size.y.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.y.mm() - child_size.y.mm()) / 2) as u32),
                        };
                        
                        // 计算Z轴位置（第二个交叉轴）
                        pos.z = match align_items.cross2 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.z.mm() - child_size.z.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.z.mm() - child_size.z.mm()) / 2) as u32),
                        };
                    }

                    child_positions.push(pos);
                }
            }
            FlexDirection::Y | FlexDirection::ReverseY => {
                // 主轴是Y轴，交叉轴是X和Z
                let free_space = node_length.y.mm() as f64 - total_child_size.y.mm() as f64;
                let mut positions = self.calculate_positions_on_axis(
                    free_space,
                    &child_lengths
                        .iter()
                        .map(|dim| dim.y.mm() as f64)
                        .collect::<Vec<_>>(),
                    justify_content,
                );

                // 如果是ReverseY，需要反转位置
                if matches!(flex_direction, FlexDirection::ReverseY) {
                    positions.reverse();
                }

                // 计算每个子元素的完整位置
                for (i, child_y_pos) in positions.iter().enumerate() {
                    let mut pos = Dim3::new(
                        Length::from_m(0.0),
                        Length::from_mm(*child_y_pos as u32),
                        Length::from_m(0.0),
                    );

                    // 根据align-items计算X和Z轴位置
                    if i < child_lengths.len() {
                        let child_size = child_lengths[i];
                        
                        // 计算X轴位置（第一个交叉轴）
                        pos.x = match align_items.cross1 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.x.mm() - child_size.x.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.x.mm() - child_size.x.mm()) / 2) as u32),
                        };
                        
                        // 计算Z轴位置（第二个交叉轴）
                        pos.z = match align_items.cross2 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.z.mm() - child_size.z.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.z.mm() - child_size.z.mm()) / 2) as u32),
                        };
                    }

                    child_positions.push(pos);
                }
            }
            FlexDirection::Z | FlexDirection::ReverseZ => {
                // 主轴是Z轴，交叉轴是X和Y
                let free_space = node_length.z.mm() as f64 - total_child_size.z.mm() as f64;
                let mut positions = self.calculate_positions_on_axis(
                    free_space,
                    &child_lengths
                        .iter()
                        .map(|dim| dim.z.mm() as f64)
                        .collect::<Vec<_>>(),
                    justify_content,
                );

                // 如果是ReverseZ，需要反转位置
                if matches!(flex_direction, FlexDirection::ReverseZ) {
                    positions.reverse();
                }

                // 计算每个子元素的完整位置
                for (i, child_z_pos) in positions.iter().enumerate() {
                    let mut pos = Dim3::new(
                        Length::from_m(0.0),
                        Length::from_m(0.0),
                        Length::from_mm(*child_z_pos as u32),
                    );

                    // 根据align-items计算X和Y轴位置
                    if i < child_lengths.len() {
                        let child_size = child_lengths[i];
                        
                        // 计算X轴位置（第一个交叉轴）
                        pos.x = match align_items.cross1 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.x.mm() - child_size.x.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.x.mm() - child_size.x.mm()) / 2) as u32),
                        };
                        
                        // 计算Y轴位置（第二个交叉轴）
                        pos.y = match align_items.cross2 {
                            style::AlignItem::FlexStart => Length::from_m(0.0),
                            style::AlignItem::FlexEnd => Length::from_mm((node_length.y.mm() - child_size.y.mm()) as u32),
                            style::AlignItem::Center => Length::from_mm(((node_length.y.mm() - child_size.y.mm()) / 2) as u32),
                        };
                    }

                    child_positions.push(pos);
                }
            }
        }

        // 将计算出的位置存储到node_ref中
        node_ref.attr.flex_child_space = child_positions
            .iter()
            .map(|pos| AbsoluteSpace { pos: *pos })
            .collect();

        // 更新子元素的位置
        for (i, child) in node_ref.children.iter().enumerate() {
            if i < child_positions.len() {
                child.borrow_mut().attr.absolute_pos = child_positions[i];
            }
        }

        Ok(())
    }

    /// 根据可用空间和子元素尺寸计算在主轴上的位置
    fn calculate_positions_on_axis(
        &self,
        free_space: f64,
        child_sizes: &[f64],
        justify_content: &style::JustifyContent,
    ) -> Vec<f64> {
        let mut positions = Vec::new();

        match justify_content {
            style::JustifyContent::FlexStart => {
                // 从起始位置开始排列
                let mut pos = 0.0;
                for &size in child_sizes {
                    positions.push(pos);
                    pos += size;
                }
            }
            style::JustifyContent::FlexEnd => {
                // 从结束位置开始排列
                let mut pos = free_space;
                for &size in child_sizes {
                    positions.push(pos);
                    pos += size;
                }
            }
            style::JustifyContent::Center => {
                // 居中排列
                let mut pos = free_space / 2.0;
                for &size in child_sizes {
                    positions.push(pos);
                    pos += size;
                }
            }
            style::JustifyContent::SpaceBetween => {
                // 两端对齐，项目间的间隔都相等
                if child_sizes.len() > 1 {
                    let spacing = free_space / (child_sizes.len() - 1) as f64;
                    let mut pos = 0.0;
                    for &size in child_sizes {
                        positions.push(pos);
                        pos += size + spacing;
                    }
                } else {
                    // 只有一个元素时，居中显示
                    positions.push(free_space / 2.0);
                }
            }
            style::JustifyContent::SpaceAround => {
                // 每个项目两侧的间隔相等
                let spacing = free_space / child_sizes.len() as f64;
                let mut pos = spacing / 2.0;
                for &size in child_sizes {
                    positions.push(pos);
                    pos += size + spacing;
                }
            }
            style::JustifyContent::SpaceEvenly => {
                // 每个项目周围分配相等的空间
                let spacing = free_space / (child_sizes.len() + 1) as f64;
                let mut pos = spacing;
                for &size in child_sizes {
                    positions.push(pos);
                    pos += size + spacing;
                }
            }
        }

        positions
    }

    pub fn print_computed(&self) {
        let body = self.find_body_node(&self.root).unwrap();
        print_render_tree_computed(&body, 0);
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

    println!("{}Computed Size={}", indent, node_ref.computed_style.size);

    // 打印位置信息
    println!("{}Computed Position={}", indent, node_ref.attr.absolute_pos);
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
