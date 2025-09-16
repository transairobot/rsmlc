use crate::auto::Length;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Sphere {
    #[serde(rename = "@radius")]
    pub radius: Length,
    #[serde(rename = "@static_friction")]
    pub static_friction: f32,
    #[serde(rename = "@dynamic_friction")]
    pub dynamic_friction: f32,
}

#[derive(Debug, Deserialize)]
pub struct Box {
    #[serde(rename = "@x_len")]
    pub x_len: Length,
    #[serde(rename = "@y_len")]
    pub y_len: Length,
    #[serde(rename = "@z_len")]
    pub z_len: Length,
    #[serde(rename = "@static_friction")]
    pub static_friction: f32,
    #[serde(rename = "@dynamic_friction")]
    pub dynamic_friction: f32,
}

#[derive(Debug, Deserialize)]
pub struct Cylinder {
    #[serde(rename = "@radius")]
    pub radius: Length,
    #[serde(rename = "@len")]
    pub len: Length,
    #[serde(rename = "@static_friction")]
    pub static_friction: f32,
    #[serde(rename = "@dynamic_friction")]
    pub dynamic_friction: f32,
}

#[derive(Debug, Deserialize)]
pub struct ObjectGroup {
    #[serde(rename = "@name")]
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub enum Geom {
    #[serde(rename = "sphere")]
    Sphere(Sphere),
    #[serde(rename = "box")]
    Box(Box),
    #[serde(rename = "cylinder")]
    Cylinder(Cylinder),
    #[serde(rename = "group")]
    ObjectGroup(ObjectGroup),
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn test_geom_deserialization() {
        // Test sphere deserialization
        let sphere_xml = r#"<sphere radius="5cm" static_friction="0.5" dynamic_friction="0.3" />"#;
        let sphere: Geom = from_str(sphere_xml).unwrap();
        match sphere {
            Geom::Sphere(s) => {
                assert_eq!(s.radius.mm(), 50); // 5cm = 50mm
                assert_eq!(s.static_friction, 0.5);
                assert_eq!(s.dynamic_friction, 0.3);
            }
            _ => panic!("Expected Sphere variant"),
        }

        // Test box deserialization
        let box_xml = r#"<box x_len="10cm" y_len="20cm" z_len="30cm" static_friction="0.4" dynamic_friction="0.2" />"#;
        let box_geom: Geom = from_str(box_xml).unwrap();
        match box_geom {
            Geom::Box(b) => {
                assert_eq!(b.x_len.mm(), 100); // 10cm = 100mm
                assert_eq!(b.y_len.mm(), 200); // 20cm = 200mm
                assert_eq!(b.z_len.mm(), 300); // 30cm = 300mm
                assert_eq!(b.static_friction, 0.4);
                assert_eq!(b.dynamic_friction, 0.2);
            }
            _ => panic!("Expected Box variant"),
        }

        // Test cylinder deserialization
        let cylinder_xml =
            r#"<cylinder radius="3cm" len="15cm" static_friction="0.6" dynamic_friction="0.4" />"#;
        let cylinder: Geom = from_str(cylinder_xml).unwrap();
        match cylinder {
            Geom::Cylinder(c) => {
                assert_eq!(c.radius.mm(), 30); // 3cm = 30mm
                assert_eq!(c.len.mm(), 150); // 15cm = 150mm
                assert_eq!(c.static_friction, 0.6);
                assert_eq!(c.dynamic_friction, 0.4);
            }
            _ => panic!("Expected Cylinder variant"),
        }

        // Test group deserialization
        let group_xml = r#"<group group_name="models/car.xml" />"#;
        let group: Geom = from_str(group_xml).unwrap();
        match group {
            Geom::ObjectGroup(g) => {
                assert_eq!(g.name, "models/car.xml");
            }
            _ => panic!("Expected ObjectGroup variant"),
        }
    }

    #[test]
    fn test_object_with_geom_deserialization() {
        /// A parent element that contains a Geom
        #[derive(Debug)]
        pub struct Object {
            pub id: String,
            pub object_type: String,
            pub geom: Geom,
            pub position: Option<[f32; 3]>, // x, y, z coordinates
            pub rotation: Option<[f32; 3]>, // rx, ry, rz rotation angles
        }
        // First test that Geom itself works
        let sphere_xml = r#"<sphere radius="5cm" static_friction="0.5" dynamic_friction="0.3"/>"#;
        let sphere: Geom = from_str(sphere_xml).unwrap();
        match sphere {
            Geom::Sphere(ref s) => {
                assert_eq!(s.radius.mm(), 50); // 5cm = 50mm
                assert_eq!(s.static_friction, 0.5);
                assert_eq!(s.dynamic_friction, 0.3);
            }
            _ => panic!("Expected Sphere variant"),
        }

        // Demonstrate how to use Geom in a parent structure
        // Note: For complex XML deserialization with nested elements,
        // you might need to implement custom deserialization or use a different approach
        match sphere {
            Geom::Sphere(sphere_geom) => {
                let object = Object {
                    id: "ball".to_string(),
                    object_type: "decoration".to_string(),
                    geom: Geom::Sphere(sphere_geom),
                    position: Some([1.0, 2.0, 3.0]),
                    rotation: Some([0.0, 0.0, 0.0]),
                };

                // Verify the object was created correctly
                assert_eq!(object.id, "ball");
                assert_eq!(object.object_type, "decoration");
                match object.geom {
                    Geom::Sphere(ref s) => {
                        assert_eq!(s.radius.mm(), 50);
                        assert_eq!(s.static_friction, 0.5);
                        assert_eq!(s.dynamic_friction, 0.3);
                    }
                    _ => panic!("Expected Sphere variant"),
                }
                assert_eq!(object.position, Some([1.0, 2.0, 3.0]));
                assert_eq!(object.rotation, Some([0.0, 0.0, 0.0]));
            }
            _ => panic!("Expected Sphere variant"),
        }
    }
}
