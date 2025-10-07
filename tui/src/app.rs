use anyhow::Result;
use outliner_core::{
    models::{Note, OutlineNode},
    storage::{Connection, Database, NoteRepository, NodeRepository, TagRepository, LinkRepository},
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
    pub is_editing: bool,
    pub edit_buffer: String,
    // Phase 4 - Pages management
    pub notes: Vec<Note>,
    pub sidebar_pages_selected_index: usize,
    pub page_switcher_open: bool,
    pub page_filter: String,
    pub page_switcher_selection_index: usize,
    // Phase 5 - Search & Tags & Backlinks
    pub search_open: bool,
    pub search_query: String,
    pub search_results: Vec<OutlineNode>,
    pub tag_filter: Option<String>,
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
            is_editing: false,
            edit_buffer: String::new(),
            notes: Vec::new(),
            sidebar_pages_selected_index: 0,
            page_switcher_open: false,
            page_filter: String::new(),
            page_switcher_selection_index: 0,
            search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            tag_filter: None,
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
            let _root1_id = nodes[0].id.clone();
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
        self.refresh_notes_list()?;
        if let Some(note) = self.notes.first() {
            let id = note.id.clone();
            self.load_note(&id)?;
            self.sidebar_pages_selected_index = 0;
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

    /// Build a list of visible paths (indices into the tree). Each path represents a visible node.
    fn build_visible_paths(&self) -> Vec<Vec<usize>> {
        fn walk(node: &TreeNode, path: &mut Vec<usize>, acc: &mut Vec<Vec<usize>>) {
            acc.push(path.clone());
            if node.is_expanded {
                for (i, child) in node.children.iter().enumerate() {
                    path.push(i);
                    walk(child, path, acc);
                    path.pop();
                }
            }
        }

        let mut paths = Vec::new();
        for (i, node) in self.outline_tree.iter().enumerate() {
            let mut path = vec![i];
            walk(node, &mut path, &mut paths);
        }
        paths
    }

    /// Get mutable reference to a tree node by its path
    fn get_node_mut_by_path(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() { return None; }
        let mut current: *mut TreeNode = match self.outline_tree.get_mut(path[0]) {
            Some(n) => n,
            None => return None,
        } as *mut TreeNode;

        // Safety: We only use one mutable borrow chain at a time
        for idx in &path[1..] {
            unsafe {
                let cur_ref: &mut TreeNode = &mut *current;
                current = match cur_ref.children.get_mut(*idx) {
                    Some(n) => n,
                    None => return None,
                } as *mut TreeNode;
            }
        }

        unsafe { Some(&mut *current) }
    }

    /// Get the selected node's ID (if any)
    pub fn get_selected_node_id(&self) -> Option<String> {
        let visible = self.get_visible_nodes();
        visible.get(self.cursor_position).map(|t| t.node.id.clone())
    }

    /// Toggle expansion state of the selected node
    pub fn toggle_selected_expand_collapse(&mut self, expand: Option<bool>) {
        let paths = self.build_visible_paths();
        if let Some(path) = paths.get(self.cursor_position) {
            if let Some(node) = self.get_node_mut_by_path(path) {
                if !node.children.is_empty() {
                    match expand {
                        Some(true) => node.is_expanded = true,
                        Some(false) => node.is_expanded = false,
                        None => node.is_expanded = !node.is_expanded,
                    }
                }
            }
        }
    }

    /// Move cursor up (saturating at 0)
    pub fn move_cursor_up(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position < self.scroll_offset {
                self.scroll_offset = self.cursor_position;
            }
        }
    }

    /// Move cursor down (saturating at last visible)
    pub fn move_cursor_down(&mut self) {
        let last = self.get_visible_nodes().len().saturating_sub(1);
        if self.cursor_position < last {
            self.cursor_position += 1;
        }
    }

    /// Start editing the selected node
    pub fn start_editing(&mut self) {
        if self.is_editing { return; }
        if let Some(id) = self.get_selected_node_id() {
            if let Ok(node) = NodeRepository::get_by_id(&self.db_connection, &id) {
                self.edit_buffer = node.content.clone();
                self.is_editing = true;
            }
        }
    }

    /// Cancel edit mode without saving
    pub fn cancel_edit(&mut self) {
        self.is_editing = false;
        self.edit_buffer.clear();
    }

    /// Commit edit buffer to the database and refresh
    pub fn commit_edit(&mut self) -> Result<()> {
        if !self.is_editing { return Ok(()); }
        let selected_id = match self.get_selected_node_id() { Some(id) => id, None => return Ok(()) };
        let mut node = NodeRepository::get_by_id(&self.db_connection, &selected_id)?;
        node.content = self.edit_buffer.clone();
        node.touch();
        NodeRepository::update(&self.db_connection, &node)?;
        // Phase 5: update tags and links after content change
        self.update_tags_and_links_for_node(&node)?;
        self.is_editing = false;
        self.edit_buffer.clear();
        self.refresh_current_note_preserve_selection(Some(&selected_id))?;
        Ok(())
    }

    /// Phase 5: Parse tags and wiki links, persist associations
    fn update_tags_and_links_for_node(&mut self, node: &OutlineNode) -> Result<()> {
        // Parse tags like #tag-name
        let re_tags = regex::Regex::new(r"(?P<tag>#([A-Za-z0-9_-]+))").unwrap();
        let mut tags: Vec<String> = re_tags
            .captures_iter(&node.content)
            .filter_map(|c| c.get(2).map(|m| m.as_str().to_string()))
            .collect();
        tags.sort();
        tags.dedup();
        TagRepository::set_tags_for_node(&self.db_connection, &node.id, &tags)?;

        // Refresh links: delete old ones for this node, then create from [[Title]]
        LinkRepository::delete_by_source_node(&self.db_connection, &node.id)?;
        let re_links = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for cap in re_links.captures_iter(&node.content) {
            let title = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            if title.is_empty() { continue; }
            if let Ok(target) = NoteRepository::get_by_title_exact(&self.db_connection, title) {
                let source_note_id = match &self.current_note { Some(n) => n.id.clone(), None => continue };
                let link = outliner_core::models::Link::new_wiki_link(
                    source_note_id,
                    Some(node.id.clone()),
                    target.id,
                    Some(title.to_string()),
                );
                let _ = LinkRepository::create(&self.db_connection, &link)?;
            }
        }
        Ok(())
    }

    /// Create a new sibling node below the current selection
    pub fn create_sibling_below(&mut self) -> Result<()> {
        let note_id = match &self.current_note { Some(n) => n.id.clone(), None => return Ok(()) };
        let selected_paths = self.build_visible_paths();
        if let Some(path) = selected_paths.get(self.cursor_position) {
            // Determine parent of selected
            let parent_id_opt = if path.len() == 1 {
                None
            } else {
                // Get parent path
                let parent_path = &path[..path.len()-1];
                let parent_node = self.get_node_by_path_readonly(parent_path);
                parent_node.map(|n| n.node.id.clone())
            };

            // Next position among siblings
            let next_pos = NodeRepository::get_next_child_position(
                &self.db_connection,
                parent_id_opt.as_deref(),
                &note_id,
            )?;

            let new_node = OutlineNode::new(note_id.clone(), parent_id_opt.clone(), "".to_string(), next_pos);
            NodeRepository::create(&self.db_connection, &new_node)?;
            let new_id = new_node.id.clone();
            self.refresh_current_note_preserve_selection(Some(&new_id))?;
        }
        Ok(())
    }

    /// Delete the selected node
    pub fn delete_selected(&mut self) -> Result<()> {
        if let Some(id) = self.get_selected_node_id() {
            NodeRepository::delete(&self.db_connection, &id)?;
            // Move cursor up if needed
            if self.cursor_position > 0 { self.cursor_position -= 1; }
            self.refresh_current_note_preserve_selection(None)?;
        }
        Ok(())
    }

    /// Indent the selected node (make it a child of previous visible sibling)
    pub fn indent_selected(&mut self) -> Result<()> {
        let paths = self.build_visible_paths();
        if let Some(path) = paths.get(self.cursor_position) {
            if path.is_empty() { return Ok(()); }
            // Need previous visible node that is a sibling or ancestor sibling
            if path.last() == Some(&0) { return Ok(()); } // first child cannot indent relative to prev sibling
            // Previous sibling within same parent
            let parent_path = path[..path.len()-1].to_vec();
            let idx_in_parent = *path.last().unwrap();
            if idx_in_parent == 0 { return Ok(()); }
            let prev_sibling_path = {
                let mut p = parent_path.clone();
                p.push(idx_in_parent - 1);
                p
            };
            let prev_id = match self.get_node_by_path_readonly(&prev_sibling_path) { Some(n) => n.node.id.clone(), None => return Ok(()) };
            // Move selected under previous sibling at end
            let selected_id = self.get_node_by_path_readonly(path).map(|n| n.node.id.clone()).unwrap();
            let note_id = self.current_note.as_ref().map(|n| n.id.clone()).unwrap_or_default();
            let next_pos = NodeRepository::get_next_child_position(&self.db_connection, Some(&prev_id), &note_id)?;
            NodeRepository::update_parent_and_position(&self.db_connection, &selected_id, Some(&prev_id), next_pos)?;
            self.refresh_current_note_preserve_selection(Some(&selected_id))?;
        }
        Ok(())
    }

    /// Outdent the selected node (move it to parent's parent, after parent)
    pub fn outdent_selected(&mut self) -> Result<()> {
        let paths = self.build_visible_paths();
        if let Some(path) = paths.get(self.cursor_position) {
            if path.len() < 2 { return Ok(()); }
            // Parent path and grandparent path
            let _parent_path = &path[..path.len()-1];
            let grandparent_path = &path[..path.len()-2];
            let grandparent_id_opt = if grandparent_path.is_empty() { None } else { self.get_node_by_path_readonly(grandparent_path).map(|n| n.node.id.clone()) };
            let selected_id = self.get_node_by_path_readonly(path).map(|n| n.node.id.clone()).unwrap();
            let note_id = self.current_note.as_ref().map(|n| n.id.clone()).unwrap_or_default();
            // New position is after the parent among its siblings
            let new_pos = if let Some(grand_id) = &grandparent_id_opt {
                let next = NodeRepository::get_next_child_position(&self.db_connection, Some(grand_id), &note_id)?;
                next
            } else {
                NodeRepository::get_next_child_position(&self.db_connection, None, &note_id)?
            };
            NodeRepository::update_parent_and_position(&self.db_connection, &selected_id, grandparent_id_opt.as_deref(), new_pos)?;
            self.refresh_current_note_preserve_selection(Some(&selected_id))?;
        }
        Ok(())
    }

    /// Move selected node up among siblings
    pub fn move_selected_up(&mut self) -> Result<()> {
        let paths = self.build_visible_paths();
        if let Some(path) = paths.get(self.cursor_position) {
            if path.is_empty() { return Ok(()); }
            let idx_in_parent = *path.last().unwrap();
            if idx_in_parent == 0 { return Ok(()); }
            let parent_path = &path[..path.len()-1];
            let current_id = self.get_node_by_path_readonly(path).map(|n| n.node.id.clone()).unwrap();
            let prev_path = {
                let mut p = parent_path.to_vec();
                p.push(idx_in_parent - 1);
                p
            };
            let prev_id = self.get_node_by_path_readonly(&prev_path).map(|n| n.node.id.clone()).unwrap();
            NodeRepository::swap_positions(&self.db_connection, &current_id, &prev_id)?;
            self.refresh_current_note_preserve_selection(Some(&current_id))?;
        }
        Ok(())
    }

    /// Move selected node down among siblings
    pub fn move_selected_down(&mut self) -> Result<()> {
        let paths = self.build_visible_paths();
        if let Some(path) = paths.get(self.cursor_position) {
            if path.is_empty() { return Ok(()); }
            let parent_path = &path[..path.len()-1];
            let idx_in_parent = *path.last().unwrap();
            let siblings_count = self.get_children_count_by_path(parent_path);
            if idx_in_parent + 1 >= siblings_count { return Ok(()); }
            let current_id = self.get_node_by_path_readonly(path).map(|n| n.node.id.clone()).unwrap();
            let next_path = {
                let mut p = parent_path.to_vec();
                p.push(idx_in_parent + 1);
                p
            };
            let next_id = self.get_node_by_path_readonly(&next_path).map(|n| n.node.id.clone()).unwrap();
            NodeRepository::swap_positions(&self.db_connection, &current_id, &next_id)?;
            self.refresh_current_note_preserve_selection(Some(&current_id))?;
        }
        Ok(())
    }

    fn get_children_count_by_path(&self, parent_path: &[usize]) -> usize {
        if parent_path.is_empty() { return self.outline_tree.len(); }
        self.get_node_by_path_readonly(parent_path).map(|n| n.children.len()).unwrap_or(0)
    }

    fn get_node_by_path_readonly(&self, path: &[usize]) -> Option<&TreeNode> {
        if path.is_empty() { return None; }
        let mut node = self.outline_tree.get(path[0])?;
        for idx in &path[1..] {
            node = node.children.get(*idx)?;
        }
        Some(node)
    }

    /// Reload current note's tree from DB and try to preserve selection by node id
    pub fn refresh_current_note_preserve_selection(&mut self, prefer_id: Option<&str>) -> Result<()> {
        if let Some(note) = &self.current_note {
            let nodes = NodeRepository::get_by_note_id(&self.db_connection, &note.id)?;
            self.outline_tree = TreeNode::build_tree(nodes);

            // Determine preferred target id as owned String to avoid lifetime issues
            let preferred: Option<String> = match prefer_id {
                Some(id) => Some(id.to_string()),
                None => self.get_selected_node_id(),
            };

            if let Some(target_id) = preferred {
                // Find index of target_id in new visible listing
                let visible = self.get_visible_nodes();
                if let Some(new_idx) = visible.iter().position(|t| t.node.id == target_id) {
                    self.cursor_position = new_idx;
                } else {
                    self.cursor_position = 0;
                }
            } else {
                self.cursor_position = 0;
            }
        }
        Ok(())
    }

    /// Handle tick events
    pub fn tick(&mut self) {
        // Future: periodic updates, autosave, etc.
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    // =========================
    // Phase 4: Pages management
    // =========================

    /// Refresh the cached list of notes for pages UI
    pub fn refresh_notes_list(&mut self) -> Result<()> {
        self.notes = NoteRepository::get_all(&self.db_connection)?;
        // Apply tag filter if present (Phase 5)
        if let Some(tag_name) = &self.tag_filter {
            let note_ids = TagRepository::get_note_ids_for_tag_name(&self.db_connection, tag_name)?;
            self.notes.retain(|n| note_ids.iter().any(|id| *id == n.id));
        }
        // Keep sidebar selection aligned with current note if possible
        if let Some(current) = &self.current_note {
            if let Some(idx) = self.notes.iter().position(|n| n.id == current.id) {
                self.sidebar_pages_selected_index = idx;
            }
        }
        Ok(())
    }

    // =========================
    // Phase 5: Search
    // =========================
    pub fn open_search(&mut self) {
        self.search_open = true;
        self.search_query.clear();
        self.search_results.clear();
    }

    pub fn close_search(&mut self) {
        self.search_open = false;
        self.search_query.clear();
        self.search_results.clear();
    }

    pub fn update_search_query(&mut self, ch: char) {
        self.search_query.push(ch);
        self.run_search();
    }

    pub fn backspace_search_query(&mut self) {
        self.search_query.pop();
        self.run_search();
    }

    pub fn run_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            return;
        }
        if let Ok(results) = NodeRepository::search(&self.db_connection, &self.search_query) {
            self.search_results = results;
        }
    }

    // =========================
    // Phase 5: Tags filter
    // =========================
    pub fn clear_tag_filter(&mut self) -> Result<()> {
        self.tag_filter = None;
        self.refresh_notes_list()
    }

    pub fn set_tag_filter(&mut self, tag_name: String) -> Result<()> {
        self.tag_filter = Some(tag_name);
        self.refresh_notes_list()
    }

    /// Select a page by index from `notes`
    pub fn select_page_by_index(&mut self, index: usize) -> Result<()> {
        if index < self.notes.len() {
            let id = self.notes[index].id.clone();
            self.sidebar_pages_selected_index = index;
            self.load_note(&id)?;
        }
        Ok(())
    }

    /// Create a new page with a generated title and switch to it
    pub fn create_new_page(&mut self) -> Result<()> {
        // Generate a unique title like "Untitled" or "Untitled (n)"
        let base = "Untitled".to_string();
        let mut title = base.clone();
        let mut suffix = 1;
        let existing_titles: std::collections::HashSet<String> = self
            .notes
            .iter()
            .map(|n| n.title.to_lowercase())
            .collect();
        while existing_titles.contains(&title.to_lowercase()) {
            title = format!("{} ({})", base, suffix);
            suffix += 1;
        }

        let note = Note::new(title);
        NoteRepository::create(&self.db_connection, &note)?;
        self.refresh_notes_list()?;
        if let Some(idx) = self.notes.iter().position(|n| n.id == note.id) {
            self.select_page_by_index(idx)?;
        }
        Ok(())
    }

    /// Delete the current page; if none remain, create a new default
    pub fn delete_current_page(&mut self) -> Result<()> {
        let current_id = match &self.current_note { Some(n) => n.id.clone(), None => return Ok(()) };
        NoteRepository::delete(&self.db_connection, &current_id)?;
        self.refresh_notes_list()?;
        if self.notes.is_empty() {
            // Ensure at least one page exists
            let note = Note::new("Welcome".to_string());
            NoteRepository::create(&self.db_connection, &note)?;
            self.refresh_notes_list()?;
        }
        // Load first note or keep index if valid
        let idx = self.sidebar_pages_selected_index.min(self.notes.len().saturating_sub(1));
        if !self.notes.is_empty() {
            self.select_page_by_index(idx)?;
        }
        Ok(())
    }

    /// Navigate sidebar page selection up
    pub fn sidebar_select_up(&mut self) {
        if self.sidebar_pages_selected_index > 0 {
            self.sidebar_pages_selected_index -= 1;
        }
    }

    /// Navigate sidebar page selection down
    pub fn sidebar_select_down(&mut self) {
        let last = self.notes.len().saturating_sub(1);
        if self.sidebar_pages_selected_index < last {
            self.sidebar_pages_selected_index += 1;
        }
    }

    /// Activate the sidebar-selected page
    pub fn sidebar_activate_selected(&mut self) -> Result<()> {
        self.select_page_by_index(self.sidebar_pages_selected_index)
    }

    /// Open the page switcher overlay
    pub fn open_page_switcher(&mut self) -> Result<()> {
        self.page_switcher_open = true;
        self.page_filter.clear();
        self.page_switcher_selection_index = 0;
        // Ensure notes list is up to date
        self.refresh_notes_list()?;
        Ok(())
    }

    /// Close the page switcher overlay
    pub fn close_page_switcher(&mut self) {
        self.page_switcher_open = false;
        self.page_filter.clear();
        self.page_switcher_selection_index = 0;
    }

    /// Get filtered notes based on the current page filter (substring, case-insensitive)
    pub fn get_filtered_notes(&self) -> Vec<&Note> {
        if self.page_filter.is_empty() {
            return self.notes.iter().collect();
        }
        let needle = self.page_filter.to_lowercase();
        self
            .notes
            .iter()
            .filter(|n| n.title.to_lowercase().contains(&needle))
            .collect()
    }

    /// Move selection in page switcher up
    pub fn page_switcher_up(&mut self) {
        if self.page_switcher_selection_index > 0 {
            self.page_switcher_selection_index -= 1;
        }
    }

    /// Move selection in page switcher down
    pub fn page_switcher_down(&mut self) {
        let last = self.get_filtered_notes().len().saturating_sub(1);
        if self.page_switcher_selection_index < last {
            self.page_switcher_selection_index += 1;
        }
    }

    /// Apply the current selection in page switcher
    pub fn page_switcher_activate(&mut self) -> Result<()> {
        let filtered = self.get_filtered_notes();
        if let Some(note) = filtered.get(self.page_switcher_selection_index) {
            // Take copies before mutable borrows
            let selected_id = note.id.clone();
            let sidebar_idx = self.notes.iter().position(|n| n.id == selected_id);
            if let Some(idx) = sidebar_idx { self.sidebar_pages_selected_index = idx; }
            self.load_note(&selected_id)?;
        }
        self.close_page_switcher();
        Ok(())
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

