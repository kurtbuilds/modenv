// use std::path::PathBuf;
// use std::str::FromStr;
//
// pub fn modify_name_for_envfile(name: &str, modifier: Modifier) -> String {
//     match modifier {
//         Modifier::None => name.to_string(),
//         TerraformEnvironment => format!("TF_VAR_{}", name)
//     }
// }
//
//
// pub enum EnvironmentFileTargetKind {
//     EnvFile,
//     ExampleEnvFile,
//     Terraform,
// }
//
// pub struct EnvironmentFileTarget {
//     pub(crate) kind: EnvironmentFileTargetKind,
//     pub(crate) path: String,
// }
//
// #[derive(Clone, Copy)]
// pub enum Modifier {
//     None,
//     TerraformEnvironment,
// }
//
//
// pub fn detect_targets() -> Vec<EnvironmentFileTarget> {
//     let mut result = Vec::new();
//     for path in vec![
//         ".env",
//         ".env.production",
//         ".env.development",
//     ] {
//         if PathBuf::from_str(path).unwrap().exists() {
//             result.push(EnvironmentFileTarget {
//                 kind: EnvironmentFileTargetKind::EnvFile,
//                 path: path.to_string(),
//             })
//         }
//     }
//     for path in vec![
//         ".env.example",
//     ] {
//         if PathBuf::from_str(path).unwrap().exists() {
//             result.push(EnvironmentFileTarget {
//                 kind: EnvironmentFileTargetKind::ExampleEnvFile,
//                 path: path.to_string(),
//             })
//         }
//     }
//     if PathBuf::from_str("terraform/env.tf").unwrap().exists() {
//         result.push(EnvironmentFileTarget {
//             kind: EnvironmentFileTargetKind::Terraform,
//             path: "terraform/env.tf".to_string(),
//         })
//     }
//     return result;
// }
//
//
// pub fn detect_modifier() -> Modifier {
//     if PathBuf::from_str("terraform").unwrap().exists() {
//         Modifier::TerraformEnvironment
//     } else {
//         Modifier::None
//     }
// }