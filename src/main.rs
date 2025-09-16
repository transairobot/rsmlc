pub mod auto;
mod style;
mod xml_parser;
mod render_tree;

use anyhow::Result;
use xml_parser::{parse_xml_file, Element};
use render_tree::{build_render_tree, print_render_tree};

fn main() -> Result<()> {
    // 解析RSML XML文件
    let root_element = parse_xml_file("rsml_example.xml")?;
    
    // 打印DOM树结构
    println!("DOM Tree:");
    print_element(&root_element, 0);
    
    // 验证解析结果
    validate_rsml_structure(&root_element)?;
    
    // 构建渲染树
    println!("\n正在构建渲染树...");
    let render_tree = build_render_tree(&root_element)?;
    
    // 计算布局
    println!("\n正在计算布局...");
    render_tree::calculate_layout(&render_tree)?;
    
    // 打印渲染树
    println!("\n渲染树:");
    print_render_tree(&render_tree, 0);
    
    println!("\nXML文件解析、渲染树构建和布局计算成功完成！");
    
    Ok(())
}

fn print_element(element: &Element, depth: usize) {
    let indent = "  ".repeat(depth);
    
    // 打印元素名称
    print!("{}{}", indent, element.name);
    
    // 打印属性
    if !element.attributes.is_empty() {
        print!(" [");
        let mut first = true;
        for (key, value) in &element.attributes {
            if !first {
                print!(", ");
            }
            print!("{}=\"{}\"", key, value);
            first = false;
        }
        print!("]");
    }
    
    // 打印文本内容（如果有的话）
    if !element.text.trim().is_empty() {
        println!(" \"{}\"", element.text.trim());
    } else {
        println!();
    }
    
    // 递归打印子元素
    for child in &element.children {
        print_element(child, depth + 1);
    }
}

fn validate_rsml_structure(element: &Element) -> Result<()> {
    // 验证根元素是rsml
    if element.name != "rsml" {
        return Err(anyhow::anyhow!("根元素应该是'rsml'，但实际是'{}'", element.name));
    }
    
    // 验证必须有head和body子元素
    let has_head = element.find_child("head").is_some();
    let has_body = element.find_child("body").is_some();
    
    if !has_head {
        return Err(anyhow::anyhow!("缺少必需的'head'元素"));
    }
    
    if !has_body {
        return Err(anyhow::anyhow!("缺少必需的'body'元素"));
    }
    
    println!("\n结构验证通过:");
    println!("- 根元素: {}", element.name);
    println!("- 包含head元素: {}", has_head);
    println!("- 包含body元素: {}", has_body);
    
    // 验证body中的主要元素
    if let Some(body) = element.find_child("body") {
        if let Some(main_room) = body.find_child("space") {
            println!("- 主房间ID: {:?}", main_room.get_attribute("id"));
            
            // 验证主房间的子元素
            println!("- 主房间子元素数量: {}", main_room.children.len());
        }
    }
    
    Ok(())
}
