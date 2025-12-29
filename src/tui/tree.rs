use prefer::ConfigValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub key: String,
    pub value: NodeValue,
    pub expanded: bool,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub enum NodeValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<TreeNode>),
    Object(Vec<TreeNode>),
}

impl TreeNode {
    pub fn from_config_value(key: String, value: &ConfigValue, depth: usize) -> Self {
        let node_value = match value {
            ConfigValue::Null => NodeValue::Null,
            ConfigValue::Bool(b) => NodeValue::Bool(*b),
            ConfigValue::Integer(n) => NodeValue::Number(n.to_string()),
            ConfigValue::Float(f) => NodeValue::Number(f.to_string()),
            ConfigValue::String(s) => NodeValue::String(s.clone()),
            ConfigValue::Array(arr) => {
                let children: Vec<TreeNode> = arr
                    .iter()
                    .enumerate()
                    .map(|(i, v)| TreeNode::from_config_value(format!("[{}]", i), v, depth + 1))
                    .collect();
                NodeValue::Array(children)
            }
            ConfigValue::Object(obj) => {
                let mut children: Vec<TreeNode> = obj
                    .iter()
                    .map(|(k, v)| TreeNode::from_config_value(k.clone(), v, depth + 1))
                    .collect();
                children.sort_by(|a, b| a.key.cmp(&b.key));
                NodeValue::Object(children)
            }
        };

        Self {
            key,
            value: node_value,
            expanded: depth < 2,
            depth,
        }
    }

    pub fn to_config_value(&self) -> ConfigValue {
        match &self.value {
            NodeValue::Null => ConfigValue::Null,
            NodeValue::Bool(b) => ConfigValue::Bool(*b),
            NodeValue::Number(n) => {
                if let Ok(i) = n.parse::<i64>() {
                    ConfigValue::Integer(i)
                } else if let Ok(f) = n.parse::<f64>() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::String(n.clone())
                }
            }
            NodeValue::String(s) => ConfigValue::String(s.clone()),
            NodeValue::Array(children) => {
                ConfigValue::Array(children.iter().map(|c| c.to_config_value()).collect())
            }
            NodeValue::Object(children) => {
                let map: HashMap<String, ConfigValue> = children
                    .iter()
                    .map(|c| (c.key.clone(), c.to_config_value()))
                    .collect();
                ConfigValue::Object(map)
            }
        }
    }

    pub fn is_expandable(&self) -> bool {
        matches!(self.value, NodeValue::Array(_) | NodeValue::Object(_))
    }

    pub fn is_editable(&self) -> bool {
        matches!(
            self.value,
            NodeValue::Null | NodeValue::Bool(_) | NodeValue::Number(_) | NodeValue::String(_)
        )
    }

    pub fn editable_value(&self) -> Option<String> {
        match &self.value {
            NodeValue::Null => Some("null".to_string()),
            NodeValue::Bool(b) => Some(b.to_string()),
            NodeValue::Number(n) => Some(n.clone()),
            NodeValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set_value_from_string(&mut self, s: &str) {
        self.value = if s == "null" {
            NodeValue::Null
        } else if s == "true" {
            NodeValue::Bool(true)
        } else if s == "false" {
            NodeValue::Bool(false)
        } else if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
            NodeValue::Number(s.to_string())
        } else {
            NodeValue::String(s.to_string())
        };
    }

    pub fn children(&self) -> Option<&Vec<TreeNode>> {
        match &self.value {
            NodeValue::Array(children) | NodeValue::Object(children) => Some(children),
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode>> {
        match &mut self.value {
            NodeValue::Array(children) | NodeValue::Object(children) => Some(children),
            _ => None,
        }
    }

    pub fn remove_child(&mut self, index: usize) -> Option<TreeNode> {
        let is_array = matches!(self.value, NodeValue::Array(_));
        match &mut self.value {
            NodeValue::Array(children) | NodeValue::Object(children) => {
                if index < children.len() {
                    let removed = children.remove(index);
                    if is_array {
                        for (i, child) in children.iter_mut().enumerate().skip(index) {
                            child.key = format!("[{}]", i);
                        }
                    }
                    Some(removed)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn add_child(&mut self, key: String, value: NodeValue) -> bool {
        match &mut self.value {
            NodeValue::Object(children) => {
                let depth = self.depth + 1;
                children.push(TreeNode {
                    key,
                    value,
                    expanded: false,
                    depth,
                });
                children.sort_by(|a, b| a.key.cmp(&b.key));
                true
            }
            NodeValue::Array(children) => {
                let depth = self.depth + 1;
                let index = children.len();
                children.push(TreeNode {
                    key: format!("[{}]", index),
                    value,
                    expanded: false,
                    depth,
                });
                true
            }
            _ => false,
        }
    }

    pub fn type_indicator(&self) -> &'static str {
        match &self.value {
            NodeValue::Null => "null",
            NodeValue::Bool(_) => "bool",
            NodeValue::Number(_) => "num",
            NodeValue::String(_) => "str",
            NodeValue::Array(children) => {
                if children.is_empty() {
                    "[]"
                } else {
                    "[…]"
                }
            }
            NodeValue::Object(children) => {
                if children.is_empty() {
                    "{}"
                } else {
                    "{…}"
                }
            }
        }
    }

    pub fn value_preview(&self) -> String {
        match &self.value {
            NodeValue::Null => "null".to_string(),
            NodeValue::Bool(b) => b.to_string(),
            NodeValue::Number(n) => n.clone(),
            NodeValue::String(s) => {
                if s.len() > 40 {
                    format!("\"{}…\"", &s[..37])
                } else {
                    format!("\"{}\"", s)
                }
            }
            NodeValue::Array(children) => format!("[{} items]", children.len()),
            NodeValue::Object(children) => format!("{{{} keys}}", children.len()),
        }
    }
}

#[derive(Debug)]
pub struct FlattenedTree {
    pub nodes: Vec<FlatNode>,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub key: String,
    pub depth: usize,
    pub expanded: bool,
    pub expandable: bool,
    pub editable: bool,
    pub type_indicator: &'static str,
    pub value_preview: String,
    pub path: Vec<usize>,
}

impl FlattenedTree {
    pub fn from_root(root: &TreeNode) -> Self {
        let mut nodes = Vec::new();
        Self::flatten_node(root, &mut nodes, vec![]);
        Self { nodes }
    }

    fn flatten_node(node: &TreeNode, nodes: &mut Vec<FlatNode>, path: Vec<usize>) {
        nodes.push(FlatNode {
            key: node.key.clone(),
            depth: node.depth,
            expanded: node.expanded,
            expandable: node.is_expandable(),
            editable: node.is_editable(),
            type_indicator: node.type_indicator(),
            value_preview: node.value_preview(),
            path: path.clone(),
        });

        if node.expanded {
            if let Some(children) = node.children() {
                for (i, child) in children.iter().enumerate() {
                    let mut child_path = path.clone();
                    child_path.push(i);
                    Self::flatten_node(child, nodes, child_path);
                }
            }
        }
    }
}
