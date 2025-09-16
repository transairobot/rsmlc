mod structs;
mod layout;
mod geom;
pub mod auto;

fn main() {
    let x = quick_xml::de::from_str::<structs::Root>(
        r#"
        <root>
            <head>
                <layout path="layout.xml">Layout content</layout>
            </head>
            <body>
                <space id="room">
                    <space id="table"></space>
                </space>
            </body>
        </root>
        "#,
    );
    println!("{:#?}", x);
}
