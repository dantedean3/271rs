use std::fmt;
use std::collections::HashMap;

/// A single course in the DAG.
#[derive(Debug)]
pub struct Node {
    pub code: String,
    pub prereqs: Vec<String>,
}

impl Node {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            prereqs: Vec::new(),
        }
    }

    pub fn add_prereq(&mut self, prereq: &str) {
        if !self.prereqs.iter().any(|p| p == prereq) {
            self.prereqs.push(prereq.to_string());
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prereqs.is_empty() {
            write!(f, "{} has no prerequisites", self.code)
        } else {
            write!(f, "{} requires {}", self.code, self.prereqs.join(","))
        }
    }
}

//
// === DAG ANALYSIS FUNCTIONS BELOW ===
//

pub type Graph = HashMap<String, Node>;

/// Compute the depth of a node:
/// the longest path backwards through prerequisites.
pub fn depth(start: &str, graph: &Graph) -> usize {
    fn dfs(course: &str,
           graph: &Graph,
           memo: &mut HashMap<String, usize>) -> usize
    {
        if let Some(&cached) = memo.get(course) {
            return cached;
        }

        if let Some(node) = graph.get(course) {
            if node.prereqs.is_empty() {
                memo.insert(course.to_string(), 0);
                return 0;
            }

            let max_depth = node
                .prereqs
                .iter()
                .map(|p| dfs(p, graph, memo))
                .max()
                .unwrap_or(0);

            memo.insert(course.to_string(), max_depth + 1);
            max_depth + 1
        } else {
            0
        }
    }

    let mut memo = HashMap::new();
    dfs(start, graph, &mut memo)
}

/// Compute the height of a node:
/// the longest path forward into dependent courses.
pub fn height(start: &str, graph: &Graph) -> usize {
    // Build a reverse adjacency list: prereq → dependents
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();

    for (course, node) in graph.iter() {
        for prereq in &node.prereqs {
            reverse
                .entry(prereq.clone())
                .or_default()
                .push(course.clone());
        }
    }

    fn dfs_forward(course: &str,
                   rev: &HashMap<String, Vec<String>>,
                   memo: &mut HashMap<String, usize>) -> usize
    {
        if let Some(&cached) = memo.get(course) {
            return cached;
        }

        match rev.get(course) {
            None => {
                memo.insert(course.to_string(), 0);
                0
            }
            Some(children) => {
                let max_child = children
                    .iter()
                    .map(|c| dfs_forward(c, rev, memo))
                    .max()
                    .unwrap_or(0);

                memo.insert(course.to_string(), max_child + 1);
                max_child + 1
            }
        }
    }

    let mut memo = HashMap::new();
    dfs_forward(start, &reverse, &mut memo)
}
