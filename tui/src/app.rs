use anyhow::Result;
use outliner_core::{
    models::{Note, OutlineNode},
    storage::{Database, NoteRepository, NodeRepository, Connection},
};

/// Represents a node in the outline tree with its children
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub node: OutlineNode,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub depth: usize,
}

impl TreeNode {
    pub fn new(node: OutlineNode, depth: usize) -> Self {
        Self {
            node,
            children: Vec::new(),
            is_expanded: true,
            depth,
        }
    }

    /// Build a tree structure from a flat list of nodes
    pub fn build_tree(nodes: Vec<OutlineNode>) -> Vec<TreeNode> {
        let mut root_nodes = Vec::new();
        let mut node_map: std::collections::HashMap<String, Vec<OutlineNode>> = std::collections::HashMap::new();

        // Group nodes by parent
        for node in nodes {
            if let Some(parent_id) = &node.parent_node_id {
                node_map.entry(parent_id.clone()).or_default().push(node);
            } else {
                root_nodes.push(node);
            }
        }

        // Recursively build tree
        fn build_subtree(
            node: OutlineNode,
            node_map: &std::collections::HashMap<String, Vec<OutlineNode>>,
            depth: usize,
        ) -> TreeNode {
            let mut tree_node = TreeNode::new(node.clone(), depth);
            
            if let Some(children) = node_map.get(&node.id) {
                tree_node.children = children
                    .iter()
                    .cloned()
                    .map(|child| build_subtree(child, node_map, depth + 1))
                    .collect();
            }
            
            tree_node
        }

        root_nodes
            .into_iter()
            .map(|node| build_subtree(node, &node_map, 0))
            .collect()
    }

    /// Flatten the tree for display, respecting expanded/collapsed state
    pub fn flatten(&self) -> Vec<&TreeNode> {
        let mut result = vec![self];
        
        if self.is_expanded {
            for child in &self.children {
                result.extend(child.flatten());
            }
        }
        
        result
    }
}

/// Application state
pub struct App {
    pub should_quit: bool,
    pub current_note: Option<Note>,
    pub outline_tree: Vec<TreeNode>,
    pub cursor_position: usize,
    pub scroll_offset: usize,
    pub db_connection: Connection,
}

impl App {
    /// Create a new App instance
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::new(db_path);
        let conn = db.get_or_create()?;
        
        Ok(Self {
            should_quit: false,
            current_note: None,
            outline_tree: Vec::new(),
            cursor_position: 0,
            scroll_offset: 0,
            db_connection: conn,
        })
    }

    /// Initialize with sample data if database is empty
    pub fn initialize_sample_data(&mut self) -> Result<()> {
        let note_count = NoteRepository::count(&self.db_connection)?;
        
        if note_count == 0 {
            // Create a sample note
            let note = Note::new("Welcome to Outliner".to_string());
            NoteRepository::create(&self.db_connection, &note)?;
            
            // Create sample outline
            let nodes = vec![
                OutlineNode::new(note.id.clone(), None, "🎉 Welcome! This is a simple outliner application.".to_string(), 0),
                OutlineNode::new(note.id.clone(), None, "Features".to_string(), 1),
                OutlineNode::new_task(note.id.clone(), None, "Infinite nesting support".to_string(), 2, None, None),
                OutlineNode::new_task(note.id.clone(), None, "Task management with checkboxes".to_string(), 3, None, None),
                OutlineNode::new(note.id.clone(), None, "Getting Started".to_string(), 4),
                OutlineNode::new(note.id.clone(), None, "Navigation".to_string(), 5),
            ];
            
            // Create nodes and build hierarchy
            let root1_id = nodes[0].id.clone();
            let root2_id = nodes[1].id.clone();
            let root5_id = nodes[4].id.clone();
            let root6_id = nodes[5].id.clone();
            
            for node in &nodes[0..6] {
                NodeRepository::create(&self.db_connection, node)?;
            }
            
            // Add children to "Features"
            let feature_child1 = OutlineNode::new(
                note.id.clone(),
                Some(root2_id.clone()),
                "SQLite database backend".to_string(),
                0,
            );
            let feature_child2 = OutlineNode::new(
                note.id.clone(),
                Some(root2_id.clone()),
                "Full-text search".to_string(),
                1,
            );
            NodeRepository::create(&self.db_connection, &feature_child1)?;
            NodeRepository::create(&self.db_connection, &feature_child2)?;
            
            // Add children to "Getting Started"
            let getting_started_child = OutlineNode::new(
                note.id.clone(),
                Some(root5_id.clone()),
                "Press 'q' to quit the application".to_string(),
                0,
            );
            NodeRepository::create(&self.db_connection, &getting_started_child)?;
            
            // Add children to "Navigation"
            let nav_children = vec![
                OutlineNode::new(note.id.clone(), Some(root6_id.clone()), "↑/↓ - Navigate up and down (Phase 3)".to_string(), 0),
                OutlineNode::new(note.id.clone(), Some(root6_id.clone()), "←/→ - Collapse/Expand nodes (Phase 3)".to_string(), 1),
                OutlineNode::new(note.id.clone(), Some(root6_id.clone()), "Enter - Edit node (Phase 3)".to_string(), 2),
            ];
            
            for child in nav_children {
                NodeRepository::create(&self.db_connection, &child)?;
            }
        }
        
        Ok(())
    }

    /// Load a note and its outline
    pub fn load_note(&mut self, note_id: &str) -> Result<()> {
        let note = NoteRepository::get_by_id(&self.db_connection, note_id)?;
        let nodes = NodeRepository::get_by_note_id(&self.db_connection, note_id)?;
        
        self.current_note = Some(note);
        self.outline_tree = TreeNode::build_tree(nodes);
        self.cursor_position = 0;
        self.scroll_offset = 0;
        
        Ok(())
    }

    /// Load the first available note
    pub fn load_first_note(&mut self) -> Result<()> {
        let notes = NoteRepository::get_all(&self.db_connection)?;
        
        if let Some(note) = notes.first() {
            self.load_note(&note.id)?;
        }
        
        Ok(())
    }

    /// Get all visible nodes (flattened tree)
    pub fn get_visible_nodes(&self) -> Vec<&TreeNode> {
        self.outline_tree
            .iter()
            .flat_map(|node| node.flatten())
            .collect()
    }

    /// Handle tick events
    pub fn tick(&mut self) {
        // Future: periodic updates, autosave, etc.
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tree_node_build() {
        let nodes = vec![
            OutlineNode::new("note1".to_string(), None, "Root".to_string(), 0),
            OutlineNode::new("note1".to_string(), Some("parent".to_string()), "Child".to_string(), 1),
        ];

        // Can't fully test without proper parent IDs, but structure is valid
        let tree = TreeNode::build_tree(nodes);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_app_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let app = App::new(db_path.to_str().unwrap()).unwrap();
        assert!(!app.should_quit);
        assert!(app.current_note.is_none());
    }

    #[test]
    fn test_initialize_and_load() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let mut app = App::new(db_path.to_str().unwrap()).unwrap();
        app.initialize_sample_data().unwrap();
        app.load_first_note().unwrap();
        
        assert!(app.current_note.is_some());
        assert!(!app.outline_tree.is_empty());
    }
}

