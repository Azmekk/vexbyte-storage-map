use super::node::NodeIndex;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TreeQueryError {
    #[error("node {} does not exist in this tree", node.get())]
    NodeNotFound { node: NodeIndex },
    #[error("node {} was removed from the tree", node.get())]
    NodeRemoved { node: NodeIndex },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TreeRemovalError {
    #[error("node {} does not exist in this tree", node.get())]
    NodeNotFound { node: NodeIndex },
    #[error("the root of the tree cannot be removed")]
    CannotRemoveRoot,
    #[error("node {} was created by the scanner and does not exist on disk", node.get())]
    CannotRemoveSyntheticNode { node: NodeIndex },
}
