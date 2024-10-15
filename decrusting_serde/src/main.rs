use std::borrow::Cow;

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serialize_docker_yml::Service;

pub mod error_handling;
pub mod serialize_docker_yml;
// #[derive(Serialize,Deserialize)]
// #[serde(crate = "foobar")]
// struct  Foo {
//     a:u64,
//     b:String
// }
// #[derive(Serialize,Deserialize)]
// struct Foo<'a> {
//     a:u64,
//     b:Cow<'a, str>
// }

// #[derive(Serialize,Deserialize)]
// enum Foo {
//     Bar { u :u64},
//     Baz { s: String}
// }
// impl Serialize for Foo {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: serde::Serializer,
//     {
//         let mut s = serializer.serialize_struct("Foo", 2)?;
//         s.serialize_field("a", &self.a)?;
//         s.serialize_field("b", &self.b)?;
//         s.end();
//     }
// }
// serde_yaml is deprecated btw
fn main() {
    // let s = String::new();
    // let x :Foo = serde_json::from_str(&s).unwrap();
    // drop(x);
    let build_string = "
        build:  ./dir
    ";

    let service:Service = serde_yaml::from_str(build_string).unwrap();
    
     println!("{:?}", service);

    let build_struct = "
        build:
          context: ./dir
          dockerfile: Dockerfile-alternate
          args:
            buildno: '1'
    ";
    let service: Service = serde_yaml::from_str(build_struct).unwrap();

    // context="./dir"
    // dockerfile=Some("Dockerfile-alternate")
    // args={"buildno": "1"}
    println!("{:?}", service);
}
