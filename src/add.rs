// use std::{fs, io};
// use crate::{EnvironmentFileTarget, Modifier, EnvironmentFileTargetKind, detection, add};
// use std::io::Read;
//
// pub fn add_to_terraform(name: &str, _value: &str, tf_path: &str) {
//     let mut s = fs::read_to_string(tf_path).unwrap();
//     s.push_str(&format!(r#"
//
// variable "{}" {{
//   type = string
// }}"#, name));
//     fs::write(tf_path, s);
//     eprintln!("Added {0} to {1}", name, tf_path);
// }
//
// pub fn add_to_target(name: &str, value: &str, env_file: &EnvironmentFileTarget, modifier: Modifier) {
//     match env_file.kind {
//         EnvironmentFileTargetKind::EnvFile => {
//             let name = detection::modify_name_for_envfile(name, modifier);
//             add_to_dotenv(&name, value, &env_file.path);
//         }
//         EnvironmentFileTargetKind::ExampleEnvFile => {
//             let name = detection::modify_name_for_envfile(name, modifier);
//             add_to_dotenv(&name, "", &env_file.path);
//         }
//         EnvironmentFileTargetKind::Terraform => {
//             add::add_to_terraform(name, value, &env_file.path);
//         }
//     }
// }
//
// fn add_to_dotenv(name: &str, value: &str, env_path: &str) {
//     let mut s = fs::read_to_string(env_path).unwrap();
//     s.push_str(&format!("\n{}={}", name, value));
//     fs::write(env_path, s);
//     eprintln!("Added {0} to {1}", name, env_path);
// }
//
//
// pub fn add(name: &str, value: Option<&str>) {
//     let mut buffer = String::new();
//     let values: Vec<(&str, &str)> = if name == "-" {
//         io::stdin().read_to_string(&mut buffer).unwrap();
//         buffer.split("\n")
//             .map(|line| {
//                 let kv: Vec<&str> = line.splitn(2, "=").collect();
//                 (kv[0], kv[1])
//             })
//             .collect()
//     } else {
//         vec![(name, value.unwrap_or(""))]
//     };
//     let targets = detection::detect_targets();
//     let modifier = detection::detect_modifier();
//     for (name, value) in &values {
//         for target in &targets {
//             add::add_to_target(name, value, &target, modifier);
//         }
//     }
// }