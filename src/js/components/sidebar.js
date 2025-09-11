// Sidebar Component
// Phase 1: Navigation, recent pages, and task management

class SidebarComponent {
    constructor(app) {
        this.app = app;
        this.isLeftSidebarCollapsed = false;
        this.isRightSidebarCollapsed = false;
        this.recentPages = [];
        this.currentTasks = [];
        this.activeTaskFilter = 'all';
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadRecentPages();
        this.loadTasks();
    }

    setupEventListeners() {
        // Sidebar toggle buttons
        document.getElementById('sidebar-toggle').addEventListener('click', () => {
            this.toggleLeftSidebar();
        });
        
        document.getElementById('right-sidebar-toggle').addEventListener('click', () => {
            this.toggleRightSidebar();
        });
        
        // Task filters
        document.querySelectorAll('.task-filter').forEach(filter => {
            filter.addEventListener('click', (e) => {
                this.setTaskFilter(e.target.dataset.status);
            });
        });
        
        // Navigation items are handled by the main app
    }

    // ================================
    // Sidebar Toggle
    // ================================

    toggleLeftSidebar() {
        const sidebar = document.getElementById('left-sidebar');
        this.isLeftSidebarCollapsed = !this.isLeftSidebarCollapsed;
        
        if (this.isLeftSidebarCollapsed) {
            sidebar.classList.add('collapsed');
            document.getElementById('sidebar-toggle').textContent = '▶';
        } else {
            sidebar.classList.remove('collapsed');
            document.getElementById('sidebar-toggle').textContent = '◀';
        }
        
        // Trigger window resize event for other components to adapt
        window.dispatchEvent(new Event('resize'));
    }

    toggleRightSidebar() {
        const sidebar = document.getElementById('right-sidebar');
        this.isRightSidebarCollapsed = !this.isRightSidebarCollapsed;
        
        if (this.isRightSidebarCollapsed) {
            sidebar.classList.add('collapsed');
            document.getElementById('right-sidebar-toggle').textContent = '◀';
        } else {
            sidebar.classList.remove('collapsed');
            document.getElementById('right-sidebar-toggle').textContent = '▶';
        }
        
        // Trigger window resize event
        window.dispatchEvent(new Event('resize'));
    }

    // ================================
    // Recent Pages
    // ================================

    async loadRecentPages() {
        try {
            const response = await this.app.invokeCommand('list_pages', {
                filter: null,
                limit: 10
            });
            
            if (response.success) {
                this.recentPages = response.data || [];
                this.renderRecentPages();
            }
        } catch (error) {
            console.error('Failed to load recent pages:', error);
        }
    }

    renderRecentPages() {
        const container = document.getElementById('recent-pages');
        
        if (this.recentPages.length === 0) {
            container.innerHTML = '<div class="no-recent">No recent pages</div>';
            return;
        }
        
        let html = '';
        for (const page of this.recentPages.slice(0, 10)) {
            const isDaily = this.isDatePath(page.path);
            const displayTitle = isDaily ? this.formatDateDisplay(page.path) : page.title;
            const icon = isDaily ? '📅' : '📄';
            
            html += `
                <div class="page-item" 
                     onclick="window.outlinerApp.sidebar.navigateToPage('${page.path}')"
                     title="${page.path}">
                    <span class="page-icon">${icon}</span>
                    <span class="page-title">${displayTitle}</span>
                    <span class="page-date">${this.formatRelativeDate(page.updated_at)}</span>
                </div>
            `;
        }
        
        container.innerHTML = html;
    }

    async navigateToPage(pagePath) {
        await this.app.loadPage(pagePath);
    }

    // ================================
    // Task Management
    // ================================

    async loadTasks() {
        try {
            const response = await this.app.invokeCommand('search_tasks', {
                status: this.activeTaskFilter === 'all' ? null : this.activeTaskFilter.toUpperCase(),
                limit: 50
            });
            
            if (response.success) {
                this.currentTasks = response.data || [];
                this.renderTasks();
            }
        } catch (error) {
            console.error('Failed to load tasks:', error);
        }
    }

    renderTasks() {
        const container = document.getElementById('task-list');
        
        const filteredTasks = this.filterTasks(this.currentTasks);
        
        if (filteredTasks.length === 0) {
            container.innerHTML = `<div class="no-tasks">No ${this.activeTaskFilter === 'all' ? '' : this.activeTaskFilter + ' '}tasks</div>`;
            return;
        }
        
        let html = '';
        for (const task of filteredTasks) {
            const status = this.getTaskStatus(task.content);
            const content = this.getTaskContent(task.content);
            const statusClass = status ? status.toLowerCase() : 'unknown';
            
            html += `
                <div class="task-item task-${statusClass}" 
                     onclick="window.outlinerApp.sidebar.navigateToTask('${task.uuid}')">
                    <div class="task-status">
                        <input type="checkbox" 
                               ${status === 'DONE' ? 'checked' : ''} 
                               onchange="window.outlinerApp.sidebar.toggleTask('${task.id}', this.checked)"
                               onclick="event.stopPropagation()">
                    </div>
                    <div class="task-content">${content}</div>
                </div>
            `;
        }
        
        container.innerHTML = html;
    }

    setTaskFilter(status) {
        // Update active filter
        document.querySelectorAll('.task-filter').forEach(filter => {
            filter.classList.remove('active');
        });
        
        document.querySelector(`[data-status="${status}"]`).classList.add('active');
        
        this.activeTaskFilter = status;
        this.renderTasks();
    }

    filterTasks(tasks) {
        if (this.activeTaskFilter === 'all') {
            return tasks;
        }
        
        return tasks.filter(task => {
            const status = this.getTaskStatus(task.content);
            return status && status.toLowerCase() === this.activeTaskFilter;
        });
    }

    async toggleTask(blockId, isCompleted) {
        try {
            // This will be handled by the outliner component
            await this.app.outliner.toggleTaskStatus(blockId, isCompleted);
            
            // Reload tasks to reflect changes
            await this.loadTasks();
        } catch (error) {
            console.error('Failed to toggle task:', error);
            this.app.showError('Failed to update task: ' + error.message);
        }
    }

    async navigateToTask(blockUuid) {
        try {
            // Find the block and navigate to its page
            const response = await this.app.invokeCommand('get_block_by_uuid', { uuid: blockUuid });
            
            if (response.success && response.data) {
                const block = response.data;
                
                // Get the page for this block
                const pageResponse = await this.app.invokeCommand('get_page', { page_id: block.page_id });
                
                if (pageResponse.success && pageResponse.data) {
                    await this.app.loadPage(pageResponse.data.path);
                    
                    // Focus the specific block
                    setTimeout(() => {
                        const blockElement = document.querySelector(`[data-block-uuid="${blockUuid}"]`);
                        if (blockElement) {
                            const contentElement = blockElement.querySelector('.block-content');
                            contentElement.focus();
                            blockElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
                        }
                    }, 200);
                }
            }
        } catch (error) {
            console.error('Failed to navigate to task:', error);
            this.app.showError('Failed to open task: ' + error.message);
        }
    }

    // ================================
    // Backlinks Management
    // ================================

    async loadBacklinks(pageId) {
        try {
            const response = await this.app.invokeCommand('get_contextual_backlinks', { page_id: pageId });
            
            if (response.success) {
                this.renderBacklinks(response.data || []);
            }
        } catch (error) {
            console.error('Failed to load backlinks:', error);
            this.renderBacklinks([]);
        }
    }

    renderBacklinks(backlinks) {
        const container = document.getElementById('backlinks-container');
        
        if (backlinks.length === 0) {
            container.innerHTML = '<div class="no-backlinks">No pages link to this page</div>';
            return;
        }
        
        let html = '';
        for (const backlink of backlinks) {
            html += `
                <div class="backlink-item">
                    <div class="backlink-header" 
                         onclick="window.outlinerApp.sidebar.navigateToPage('${backlink.source_page_path}')">
                        <span class="backlink-page">${backlink.source_page_title}</span>
                    </div>
                    <div class="backlink-content">
                        ${this.renderBacklinkHierarchy(backlink.hierarchy)}
                    </div>
                </div>
            `;
        }
        
        container.innerHTML = html;
    }

    renderBacklinkHierarchy(hierarchy) {
        // Simplified rendering of the block hierarchy
        let html = `
            <div class="backlink-block" onclick="window.outlinerApp.sidebar.navigateToBlockUuid('${hierarchy.block.uuid}')">
                ${hierarchy.block.content}
            </div>
        `;
        
        if (hierarchy.children && hierarchy.children.length > 0) {
            html += '<div class="backlink-children">';
            for (const child of hierarchy.children) {
                html += this.renderBacklinkHierarchy(child);
            }
            html += '</div>';
        }
        
        return html;
    }

    async navigateToBlockUuid(blockUuid) {
        try {
            const response = await this.app.invokeCommand('get_block_by_uuid', { uuid: blockUuid });
            
            if (response.success && response.data) {
                const block = response.data;
                const pageResponse = await this.app.invokeCommand('get_page', { page_id: block.page_id });
                
                if (pageResponse.success && pageResponse.data) {
                    await this.app.loadPage(pageResponse.data.path);
                    
                    // Focus the block
                    setTimeout(() => {
                        const blockElement = document.querySelector(`[data-block-uuid="${blockUuid}"]`);
                        if (blockElement) {
                            blockElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
                            blockElement.classList.add('highlight');
                            
                            // Remove highlight after 2 seconds
                            setTimeout(() => {
                                blockElement.classList.remove('highlight');
                            }, 2000);
                        }
                    }, 200);
                }
            }
        } catch (error) {
            console.error('Failed to navigate to block:', error);
        }
    }

    // ================================
    // Utility Methods
    // ================================

    getTaskStatus(content) {
        const match = content.match(/^(TODO|DOING|DONE|WAITING|CANCELLED)\s+/);
        return match ? match[1] : null;
    }

    getTaskContent(content) {
        return content.replace(/^(TODO|DOING|DONE|WAITING|CANCELLED)\s+/, '');
    }

    isDatePath(path) {
        return /^\d{4}-\d{2}-\d{2}$/.test(path);
    }

    formatDateDisplay(dateString) {
        const date = new Date(dateString);
        const today = new Date();
        const yesterday = new Date(today);
        yesterday.setDate(yesterday.getDate() - 1);
        const tomorrow = new Date(today);
        tomorrow.setDate(tomorrow.getDate() + 1);
        
        if (this.isSameDay(date, today)) return 'Today';
        if (this.isSameDay(date, yesterday)) return 'Yesterday';
        if (this.isSameDay(date, tomorrow)) return 'Tomorrow';
        
        return date.toLocaleDateString('en-US', { 
            weekday: 'short', 
            month: 'short', 
            day: 'numeric' 
        });
    }

    formatRelativeDate(dateString) {
        const date = new Date(dateString);
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMins / 60);
        const diffDays = Math.floor(diffHours / 24);
        
        if (diffMins < 1) return 'now';
        if (diffMins < 60) return `${diffMins}m`;
        if (diffHours < 24) return `${diffHours}h`;
        if (diffDays < 7) return `${diffDays}d`;
        
        return date.toLocaleDateString();
    }

    isSameDay(date1, date2) {
        return date1.getFullYear() === date2.getFullYear() &&
               date1.getMonth() === date2.getMonth() &&
               date1.getDate() === date2.getDate();
    }

    // ================================
    // Public Interface
    // ================================

    refresh() {
        this.loadRecentPages();
        this.loadTasks();
        
        if (this.app.currentPage) {
            this.loadBacklinks(this.app.currentPage.id);
        }
    }

    updateCurrentPage(page) {
        if (page) {
            this.loadBacklinks(page.id);
        }
    }
}

// Add sidebar-specific CSS
const sidebarCSS = `
.page-item, .task-item, .backlink-item {
    padding: var(--spacing-xs) var(--spacing-sm);
    margin-bottom: 2px;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
}

.page-item:hover, .task-item:hover, .backlink-item:hover {
    background-color: var(--bg-hover);
}

.page-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
}

.page-icon {
    font-size: 14px;
    width: 20px;
    text-align: center;
}

.page-title {
    flex: 1;
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.page-date {
    font-size: 10px;
    color: var(--text-muted);
}

.task-item {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-xs);
}

.task-status input[type="checkbox"] {
    margin-top: 2px;
}

.task-content {
    flex: 1;
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-primary);
}

.task-item.task-done .task-content {
    text-decoration: line-through;
    color: var(--text-muted);
}

.backlink-header {
    font-weight: 500;
    font-size: 12px;
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
}

.backlink-block {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.3;
    padding: 2px 0;
    cursor: pointer;
}

.backlink-block:hover {
    color: var(--text-primary);
}

.backlink-children {
    margin-left: var(--spacing-md);
    border-left: 1px solid var(--border-light);
    padding-left: var(--spacing-sm);
}

.no-recent, .no-tasks, .no-backlinks {
    text-align: center;
    color: var(--text-muted);
    font-size: 11px;
    font-style: italic;
    padding: var(--spacing-md);
}

.block-item.highlight {
    background-color: rgba(255, 235, 59, 0.3);
    transition: background-color 0.5s ease;
}
`;

// Inject CSS
const style = document.createElement('style');
style.textContent = sidebarCSS;
document.head.appendChild(style);

// Export for module usage
export default SidebarComponent;
