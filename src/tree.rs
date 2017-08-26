use super::{Symbol};
use std::fmt;

pub type NodeId = usize;


pub struct NodeData {
    pub symbol: Symbol,
}

impl From<Symbol> for NodeData {
    fn from(symbol: Symbol) -> Self {
        NodeData {
            symbol: symbol
        }
    }
}


pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub data: NodeData,
}

impl Node {
    pub fn new<T: Into<NodeData>>(id: NodeId, data: T) -> Node {
        Node {
            id: id,
            parent: None,
            children: vec![],
            data: data.into(),
        }
    }
}


impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data.symbol)
    }
}


pub struct Tree {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            nodes: vec![],
            root: None
        }
    }

    pub fn set_root(&mut self, new_root: NodeId) {
        self.root = Some(new_root);
    }

    pub fn new_node<T: Into<NodeData>>(&mut self, data: T) -> NodeId {
        let id = self.nodes.len();
        let node = Node::new(id, data.into());
        self.nodes.push(node);
        id
    }

    pub fn append(&mut self, parent_id: NodeId, child_id: NodeId) {
        {
            let child = self.nodes.get_mut(child_id).expect("Tree.append(): Unexpected Child Id");
            child.parent = Some(parent_id);
        }

        let parent = self.nodes.get_mut(parent_id).expect("Tree.append(): Unexpected Parent Id");
        parent.children.push(child_id);
    }

    pub fn preorder_walk(&self, x: NodeId, level: usize) {
        if let Some(x) = self.nodes.get(x) {
            let separator = "|-- ";
            let s = format!("{}{}", separator, x);
            let space_n = level * separator.len();

            let mut space = String::new();
            for _ in 0..space_n {
                space.push_str(" ");
            }
            println!("{}{}", space, s);

            for c in &x.children {
                self.preorder_walk(*c, level + 1);
            }
        }
    }

    pub fn print(&self) {
        if self.root == None {
            return
        }

        self.preorder_walk(self.root.unwrap(), 0);
    }
}
