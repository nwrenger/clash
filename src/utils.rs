use std::collections::HashSet;

pub fn all_unique(nums: &[usize]) -> bool {
    let mut seen = HashSet::new();
    nums.iter().all(|&x| seen.insert(x))
}
