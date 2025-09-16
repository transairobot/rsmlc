use anyhow::Result;
use rsmlc::package::Package;

fn main() -> Result<()> {
    // 解析package.toml文件
    let package = Package::from_file("package.toml")?;
    
    println!("=== Package Configuration ===");
    
    // 显示依赖项信息
    println!("\nDependencies:");
    for (name, dependency) in package.dependencies() {
        println!("  {}: size-limit = {}", name, dependency.size_limit());
    }
    
    // 显示对象信息
    println!("\nObjects:");
    for (name, object) in package.objects() {
        println!("  {}: geom-type = {}, size = {}", 
                 name, object.geom_type(), object.size());
    }
    
    // 显示组信息
    println!("\nGroups:");
    for group in package.groups() {
        println!("  {}: items = {:?}", group.name(), group.items());
    }
    
    // 演示查找功能
    println!("\n=== Lookup Examples ===");
    
    if let Some(bottle) = package.get_dependency("bottle") {
        println!("Found dependency 'bottle' with size limit: {}", bottle.size_limit());
    }
    
    if let Some(table) = package.get_object("table_plane") {
        println!("Found object 'table_plane' with geom-type: {} and size: {}", 
                 table.geom_type(), table.size());
    }
    
    if let Some(bottles_group) = package.get_group("bottles") {
        println!("Found group 'bottles' with items: {:?}", bottles_group.items());
    }
    
    // 演示存在性检查
    println!("\n=== Existence Checks ===");
    println!("Has dependency 'bottle': {}", package.has_dependency("bottle"));
    println!("Has object 'table_plane': {}", package.has_object("table_plane"));
    println!("Has group 'bottles': {}", package.has_group("bottles"));
    println!("Has dependency 'nonexistent': {}", package.has_dependency("nonexistent"));
    
    Ok(())
}