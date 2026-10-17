use crate::file_tree::FileTree;

/// Every node name in lowercase, stored once when the index is built so a query never
/// lowercases names again. Indexed by arena position, so a pass over the nodes also reads the
/// names front to back.
#[derive(Debug, Clone)]
pub(crate) struct LowercaseNameTable {
    text: Vec<u8>,
    /// `offsets[position]..offsets[position + 1]` is the byte range of the node's name.
    offsets: Vec<u32>,
}

impl LowercaseNameTable {
    pub(crate) fn of_tree(tree: &FileTree) -> Self {
        let node_count = tree.nodes.len();
        let mut text = Vec::with_capacity(tree.names.name_text_bytes());
        let mut offsets = Vec::with_capacity(node_count + 1);
        offsets.push(0);

        for node in &tree.nodes {
            let name = tree.names.name(node.name);
            if name.is_ascii() {
                text.extend(name.bytes().map(|byte| byte.to_ascii_lowercase()));
            } else {
                text.extend_from_slice(name.to_lowercase().as_bytes());
            }
            offsets.push(u32::try_from(text.len()).unwrap_or(u32::MAX));
        }

        text.shrink_to_fit();
        Self { text, offsets }
    }

    pub(crate) fn name(&self, position: usize) -> &[u8] {
        let (Some(&start), Some(&end)) =
            (self.offsets.get(position), self.offsets.get(position + 1))
        else {
            return &[];
        };
        self.text.get(start as usize..end as usize).unwrap_or(&[])
    }

    pub(crate) fn memory_bytes(&self) -> usize {
        self.text.capacity() + self.offsets.capacity() * size_of::<u32>()
    }
}
