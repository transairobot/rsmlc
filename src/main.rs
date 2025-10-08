mod api;
pub mod base;
mod dim3;
mod error;
mod package;
mod render_tree;
mod style;
mod target;
mod xml_parser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use error::RsmlError;
use package::Package;
use render_tree::RenderTree;
use target::MjcfGenerator;
use target::threejs::ThreeJsGenerator;
use xml_parser::{Element, parse_xml_file};

#[derive(Parser)]
#[command(name = "rsmlc")]
#[command(about = "RSML (Robot Scene Markup Language) compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build an RSML file to a target format
    Build {
        #[arg(long, default_value = ".")]
        dir: String,
        /// Target format (currently supports mjcf)
        #[arg(long, default_value = "threejs", help = "Target format (currently supports mjcf, threejs)")]
        target: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { dir, target } => {
            build_command(dir, target)?;
        }
    }

    Ok(())
}

fn build_command(package_dir: &str, target: &str) -> Result<()> {
    let package_file = format!("{}/package.toml", package_dir);
    let main_file = format!("{}/src/main.xml", package_dir);
    let target_dir = format!("{}/target", package_dir);
    // Parse package.toml file
    let package = Package::from_file(&package_file)?;
    // Parse RSML XML file
    let root_element = parse_xml_file(&main_file)?;

    let render_tree = RenderTree::new(&root_element, &package)?;

    // Validate parsing result
    validate_rsml_structure(&root_element)?;
    render_tree.calculate()?;

    // Validate target format
    match target {
        "mjcf" => {
            // Generate MJCF file
            println!("\n正在生成MJCF文件...");
            let mut mjcf_generator = MjcfGenerator::new();
            mjcf_generator.generate_to_directory(&render_tree, &target_dir)?;

            println!("MJCF files generated successfully in {}", target_dir);
        },
        "threejs" => {
            // Generate Three.js JSON file
            println!("\n正在生成Three.js场景文件...");
            let threejs_generator = ThreeJsGenerator::new();
            threejs_generator.generate_to_directory(&render_tree, &target_dir)?;

            println!("Three.js files generated successfully in {}", target_dir);
        },
        _ => {
            eprintln!(
                "Unsupported target format: {}. Currently supported: mjcf, threejs",
                target
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn validate_rsml_structure(element: &Element) -> Result<()> {
    // 验证根元素是rsml
    if element.name != "rsml" {
        return Err(RsmlError::InvalidStructure {
            message: format!("根元素应该是'rsml'，但实际是'{}'", element.name),
        }
        .into());
    }

    // 验证必须有head和body子元素
    let has_head = element.find_child("head").is_some();
    let has_body = element.find_child("body").is_some();

    if !has_head {
        return Err(RsmlError::MissingElement {
            element: "head".to_string(),
        }
        .into());
    }

    if !has_body {
        return Err(RsmlError::MissingElement {
            element: "body".to_string(),
        }
        .into());
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
