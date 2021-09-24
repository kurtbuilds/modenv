// use crate::detection::detect_targets;
// use std::fs;
// use crate::audit::CompareResult::{FullMatch, ValueDiffers, KeyMissing, KeyUnexpected};
// use crate::envfile::{Pair, read_envfile};
//
//
// #[derive(PartialEq)]
// enum CompareResult {
//     FullMatch,
//     ValueDiffers,
//     KeyMissing,
//     KeyUnexpected,
// }
//
//
// struct PairComparison {
//     pair: Pair,
//     result: CompareResult
// }
//
// fn compare_pairs(standard: &Vec<Pair>, candidate: &Vec<Pair>) -> Vec<PairComparison> {
//     let mut result = Vec::new();
//     for pair in standard {
//         let candidate_pair= candidate.iter().find(|&candidate_pair| candidate_pair.key == pair.key);
//         match candidate_pair {
//             None => result.push(PairComparison{
//                 pair: pair.clone(),
//                 result: CompareResult::KeyMissing
//             }),
//             Some(candidate_pair) => if candidate_pair.value == pair.value {
//                 result.push(PairComparison{
//                     pair: pair.clone(),
//                     result: CompareResult::FullMatch
//                 })
//             } else {
//                 result.push(PairComparison{
//                     pair: pair.clone(),
//                     result: CompareResult::ValueDiffers,
//                 })
//             }
//         }
//     }
//     for pair in candidate {
//         let standard_pair= standard.iter().find(|&standard_pair| standard_pair.key == pair.key);
//         match standard_pair {
//             None =>  result.push(PairComparison{
//                 pair: pair.clone(),
//                 result: CompareResult::KeyUnexpected,
//             }),
//             Some(_) => {}
//         }
//     }
//     result
// }
//
//
// pub fn audit() {
//     let targets= detect_targets();
//     let first = read_envfile(&targets[0].path);
//     let second = read_envfile(&targets[1].path);
//     let results = compare_pairs(&first, &second);
//     eprintln!("These keys match perfectly:");
//     results.iter()
//         .filter(|cmp| cmp.result == FullMatch)
//         .for_each(|cmp| eprintln!("{}", cmp.pair.key));
//     eprintln!("These keys differ in value:");
//     results.iter()
//         .filter(|cmp| cmp.result == ValueDiffers)
//         .for_each(|cmp| eprintln!("{}", cmp.pair.key));
//     eprintln!("These keys exist in {} but are missing from {}", &targets[0].path, &targets[1].path);
//     results.iter()
//         .filter(|cmp| cmp.result == KeyMissing)
//         .for_each(|cmp| eprintln!("{}", cmp.pair.key));
//     eprintln!("These keys exist in {} but are missing from {}", &targets[1].path, &targets[0].path);
//     results.iter()
//         .filter(|cmp| cmp.result == KeyUnexpected)
//         .for_each(|cmp| eprintln!("{}", cmp.pair.key));
// }