// Outliner Application - Main JavaScript
// Phase 1: Core application initialization and state management
import BlockEditor from './editor.js';
import OutlinerManager from './outliner.js';
import CalendarComponent from './components/calendar.js';
import SearchComponent from './components/search.js';
import SidebarComponent from './components/sidebar.js';

class OutlinerApp {
    constructor() {
        this.currentPage = null;
        this.currentBlocks = [];
        this.selectedBlock = null;
        this.editingBlock = null;
        this.isEditMode = true; // Global edit/render toggle
        this.unsavedChanges = false;
        
        // Initialize application
        this.init();
    }
    
    async init() {
        console.log('Initializing Outliner Application...');
        
        try {
            // Show loading overlay
            this.showLoading();
            
            // Initialize Tauri API
            if (!window.__TAURI__) {
                console.error('Tauri API is not available. The application cannot function.');
                this.showError('Fatal Error: Tauri API not found. Please run this application in the Tauri environment.');
                return;
            }
            this.tauri = window.__TAURI__;
            console.log('Tauri API available');
            
            // Initialize UI components
            this.initUI();
            
            // Set up event listeners
            this.setupEventListeners();
            
            // Load today's page
            await this.loadTodayPage();
            
            // Hide loading overlay
            this.hideLoading();
            
            console.log('Application initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize application:', error);
            this.showError('Failed to initialize application: ' + error.message);
        }
    }
    
    initUI() {
        // Initialize component classes
        this.blockEditor = new BlockEditor(this);
        this.outliner = new OutlinerManager(this);
        this.calendar = new CalendarComponent(this);
        this.search = new SearchComponent(this);
        this.sidebar = new SidebarComponent(this);
        
        // Initialize sidebar components
        this.initSidebar();
        
        // Initialize calendar
        this.initCalendar();
        
        // Initialize search
        this.initSearch();
        
        // Initialize editor
        this.initEditor();
        
        // Initialize modals
        this.initModals();
        
        // Set initial UI state
        this.updateUI();
        
        // Setup outliner drag and drop
        this.outliner.setupEventListeners();
    }
    
    initSidebar() {
        // Navigation items
        document.getElementById('nav-today').addEventListener('click', () => this.navigateToToday());
        document.getElementById('nav-yesterday').addEventListener('click', () => this.navigateToYesterday());
        document.getElementById('nav-tomorrow').addEventListener('click', () => this.navigateToTomorrow());
        
        // Sidebar toggles are handled by the sidebar component
        
        // Task filters
        document.querySelectorAll('.task-filter').forEach(filter => {
            filter.addEventListener('click', (e) => this.filterTasks(e.target.dataset.status));
        });
    }
    
    initCalendar() {
        // Calendar navigation
        document.getElementById('prev-month').addEventListener('click', () => this.prevMonth());
        document.getElementById('next-month').addEventListener('click', () => this.nextMonth());
        
        // Render current month
        this.renderCalendar();
    }
    
    initSearch() {
        const searchInput = document.getElementById('search-input');
        const searchBtn = document.getElementById('search-btn');
        
        // Search event listeners
        searchInput.addEventListener('input', (e) => this.handleSearch(e.target.value));
        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                this.performSearch(e.target.value);
            }
        });
        searchBtn.addEventListener('click', () => this.performSearch(searchInput.value));
    }
    
    initEditor() {
        // Editor buttons
        document.getElementById('new-page-btn').addEventListener('click', () => this.showNewPageModal());
        document.getElementById('quick-open-btn').addEventListener('click', () => this.showQuickOpenModal());
        document.getElementById('save-btn').addEventListener('click', () => this.savePage());
        
        // Add block button
        document.getElementById('add-block-btn').addEventListener('click', () => this.createNewBlock());
        
        // Edit mode toggle removed - always in edit mode now
        
        // Initialize first block if needed
        this.initFirstBlock();
    }
    
    initFirstBlock() {
        const firstBlock = document.getElementById('first-block');
        if (firstBlock) {
            const content = firstBlock.querySelector('.block-content');
            content.addEventListener('focus', () => this.handleBlockFocus(content));
            content.addEventListener('blur', () => this.handleBlockBlur(content));
            content.addEventListener('input', () => this.handleBlockInput(content));
            content.addEventListener('keydown', (e) => this.handleBlockKeydown(e));
        }
    }
    
    initModals() {
        // Quick Open Modal
        document.getElementById('quick-open-close').addEventListener('click', () => this.hideQuickOpenModal());
        document.getElementById('quick-open-input').addEventListener('input', (e) => this.searchPages(e.target.value));
        
        // New Page Modal
        document.getElementById('new-page-close').addEventListener('click', () => this.hideNewPageModal());
        document.getElementById('new-page-cancel').addEventListener('click', () => this.hideNewPageModal());
        document.getElementById('new-page-create').addEventListener('click', () => this.createNewPage());
        
        // Auto-generate path from title
        document.getElementById('page-title-input').addEventListener('input', (e) => {
            const pathInput = document.getElementById('page-path-input');
            if (!pathInput.value || pathInput.dataset.autoGenerated !== 'false') {
                pathInput.value = this.generatePagePath(e.target.value);
                pathInput.dataset.autoGenerated = 'true';
            }
        });
        
        document.getElementById('page-path-input').addEventListener('input', (e) => {
            e.target.dataset.autoGenerated = 'false';
        });
    }
    
    setupEventListeners() {
        // Global keyboard shortcuts
        document.addEventListener('keydown', (e) => this.handleGlobalKeydown(e));
        
        // Window events
        window.addEventListener('beforeunload', (e) => this.handleBeforeUnload(e));
        window.addEventListener('resize', () => this.handleWindowResize());
        
        // Click outside to close dropdowns
        document.addEventListener('click', (e) => this.handleDocumentClick(e));
    }
    
    // ================================
    // Tauri Integration
    // ================================
    
    async invokeCommand(command, args = {}) {
        if (!this.tauri || !this.tauri.invoke) {
            console.error(`Tauri API not available. Cannot invoke command: ${command}`);
            this.showError(`Cannot invoke command: ${command}. Tauri API is not available.`);
            throw new Error('Tauri API not available');
        }
        try {
            const result = await this.tauri.invoke(command, args);
            return result;
        } catch (error) {
            console.error(`Command ${command} failed:`, error);
            throw error;
        }
    }
    
    // ================================
    // Page Management
    // ================================
    
    async loadTodayPage() {
        const todayPath = this.getTodayPath();
        await this.loadPage(todayPath);
    }
    
    async loadPage(path) {
        try {
            this.showLoading();
            
            // Get page by path
            const pageResponse = await this.invokeCommand('get_page_by_path', { path });
            
            if (!pageResponse.success) {
                throw new Error(pageResponse.error || 'Failed to load page');
            }
            
            let page = pageResponse.data;
            
            // If page doesn't exist, create it (especially for daily pages)
            if (!page && this.isDatePath(path)) {
                const createResponse = await this.invokeCommand('get_daily_page', { date: path });
                if (createResponse.success) {
                    page = createResponse.data;
                }
            }
            
            if (!page) {
                throw new Error('Page not found and could not be created');
            }
            
            this.currentPage = page;
            
            // Load blocks for this page
            await this.loadPageBlocks(page.id);
            
            // Update UI
            this.updatePageHeader();
            this.renderBlocks();
            this.updateNavigationState();
            
            this.hideLoading();
            
        } catch (error) {
            console.error('Failed to load page:', error);
            this.showError('Failed to load page: ' + error.message);
            this.hideLoading();
        }
    }
    
    async loadPageBlocks(pageId) {
        try {
            const response = await this.invokeCommand('get_page_blocks', { page_id: pageId });
            
            if (!response.success) {
                throw new Error(response.error || 'Failed to load blocks');
            }
            
            this.currentBlocks = response.data || [];
            
        } catch (error) {
            console.error('Failed to load page blocks:', error);
            this.currentBlocks = [];
        }
    }
    
    updatePageHeader() {
        if (!this.currentPage) return;
        
        document.getElementById('page-title-display').textContent = this.currentPage.title;
        document.getElementById('page-path-display').textContent = this.currentPage.path;
        document.getElementById('current-page-title').textContent = this.currentPage.title;
        
        // Update page stats
        const blockCount = this.currentBlocks.length;
        document.getElementById('page-stats').textContent = `${blockCount} blocks`;
        document.getElementById('page-block-count').textContent = blockCount.toString();
        
        // Update page info
        document.getElementById('page-created').textContent = this.formatDate(this.currentPage.created_at);
        document.getElementById('page-modified').textContent = this.formatDate(this.currentPage.updated_at);
    }
    
    // ================================
    // Block Management
    // ================================
    
    renderBlocks() {
        const container = document.getElementById('blocks-container');
        
        if (!this.currentBlocks || this.currentBlocks.length === 0) {
            // Show empty state
            container.innerHTML = `
                <div class="empty-page">
                    <div class="empty-block block-item" data-block-id="new">
                        <div class="block-bullet">•</div>
                        <div class="block-content" contenteditable="true" data-placeholder="Start writing..."></div>
                    </div>
                </div>
            `;
        } else {
            // Render block hierarchy
            container.innerHTML = this.renderBlockHierarchy(this.currentBlocks);
        }
        
        // Attach event listeners to all blocks
        this.attachBlockEventListeners();
    }
    
    renderBlockHierarchy(blocks, level = 0) {
        return this.outliner.renderBlockHierarchy(blocks, level);
    }
    
    attachBlockEventListeners() {
        // Attach listeners to all block content elements
        document.querySelectorAll('.block-content').forEach(content => {
            content.addEventListener('focus', () => this.handleBlockFocus(content));
            content.addEventListener('blur', () => this.handleBlockBlur(content));
            content.addEventListener('input', () => this.handleBlockInput(content));
            content.addEventListener('keydown', (e) => this.handleBlockKeydown(e));
        });
        
        // Attach listeners to block bullets for drag & drop
        document.querySelectorAll('.block-bullet').forEach(bullet => {
            bullet.addEventListener('dragstart', (e) => this.handleBlockDragStart(e));
            bullet.addEventListener('dragover', (e) => this.handleBlockDragOver(e));
            bullet.addEventListener('drop', (e) => this.handleBlockDrop(e));
        });
    }
    
    async handleBlockFocus(contentElement) {
        this.outliner.handleBlockFocus(contentElement);
    }
    
    async handleBlockBlur(contentElement) {
        this.outliner.handleBlockBlur(contentElement);
    }
    
    handleBlockInput(contentElement) {
        this.blockEditor.handleContentInput(contentElement);
    }
    
    async handleBlockKeydown(e) {
        const contentElement = e.target;
        const blockElement = contentElement.closest('.block-item');
        
        switch (e.key) {
            case 'Enter':
                if (e.ctrlKey || e.metaKey) {
                    // Ctrl+Enter: Create child block
                    e.preventDefault();
                    await this.blockEditor.createChildBlock(blockElement);
                } else {
                    // Enter: Create sibling block
                    e.preventDefault();
                    await this.blockEditor.createSiblingBlock(blockElement);
                }
                break;
                
            case 'Tab':
                e.preventDefault();
                if (e.shiftKey) {
                    // Shift+Tab: Outdent
                    await this.blockEditor.outdentBlock(blockElement);
                } else {
                    // Tab: Indent
                    await this.blockEditor.indentBlock(blockElement);
                }
                break;
                
            case 'Backspace':
                if (contentElement.textContent === '' && e.target.selectionStart === 0) {
                    e.preventDefault();
                    await this.blockEditor.deleteOrPromoteBlock(blockElement);
                }
                break;
                
            case 'ArrowUp':
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    this.blockEditor.moveToAdjacentBlock(blockElement, 'up');
                }
                break;
                
            case 'ArrowDown':
                if (e.ctrlKey || e.metaKey) {
                    e.preventDefault();
                    this.blockEditor.moveToAdjacentBlock(blockElement, 'down');
                }
                break;
        }
    }
    
    async saveBlockContent(blockId, content) {
        return this.blockEditor.saveBlockContent(blockId, content);
    }
    
    async createNewBlockWithContent(content) {
        if (!this.currentPage || !content.trim()) return;
        
        return this.blockEditor.createNewBlock(null, null, content.trim());
    }
    
    // ================================
    // Navigation
    // ================================
    
    async navigateToToday() {
        await this.calendar.goToToday();
    }
    
    async navigateToYesterday() {
        await this.calendar.goToYesterday();
    }
    
    async navigateToTomorrow() {
        await this.calendar.goToTomorrow();
    }
    
    async navigateToPage(pagePath) {
        await this.loadPage(pagePath);
    }
    
    // ================================
    // Utility Functions
    // ================================
    
    getTodayPath() {
        return this.formatDatePath(new Date());
    }
    
    formatDatePath(date) {
        return date.toISOString().split('T')[0]; // YYYY-MM-DD
    }
    
    isDatePath(path) {
        return /^\d{4}-\d{2}-\d{2}$/.test(path);
    }
    
    formatDate(dateString) {
        return new Date(dateString).toLocaleDateString();
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    generatePagePath(title) {
        return title
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, '-')
            .replace(/^-|-$/g, '');
    }
    
    // ================================
    // UI State Management
    // ================================
    
    updateUI() {
        // Update navigation active states
        this.updateNavigationState();
        
        // Edit mode toggle removed - always in edit mode
        
        // Update save status
        this.updateSaveStatus(this.unsavedChanges ? 'Unsaved' : 'Saved');
        
        // Update sidebar with current page
        if (this.sidebar) {
            this.sidebar.updateCurrentPage(this.currentPage);
        }
        
        // Update calendar if current page is a daily page
        if (this.calendar && this.currentPage && this.isDatePath(this.currentPage.path)) {
            this.calendar.highlightDate(this.currentPage.path);
        }
    }
    
    updateNavigationState() {
        // Update active nav item (both old and new navigation styles)
        document.querySelectorAll('.nav-item, .nav-compact').forEach(item => item.classList.remove('active'));
        
        if (this.currentPage && this.isDatePath(this.currentPage.path)) {
            const today = this.getTodayPath();
            if (this.currentPage.path === today) {
                document.getElementById('nav-today').classList.add('active');
            }
        }
    }
    
    updateSaveStatus(status) {
        document.getElementById('save-status').textContent = status;
    }
    
    showLoading() {
        document.getElementById('loading-overlay').classList.add('active');
    }
    
    hideLoading() {
        document.getElementById('loading-overlay').classList.remove('active');
    }
    
    showError(message) {
        // Simple error display - could be enhanced with proper error modal
        alert(message);
    }
    
    // ================================
    // Global Event Handlers
    // ================================
    
    handleGlobalKeydown(e) {
        // Global shortcuts
        if (e.ctrlKey || e.metaKey) {
            switch (e.key) {
                case 'n':
                    e.preventDefault();
                    this.showNewPageModal();
                    break;
                case 'o':
                    e.preventDefault();
                    this.showQuickOpenModal();
                    break;
                case 's':
                    e.preventDefault();
                    this.savePage();
                    break;
                case 'd':
                    e.preventDefault();
                    this.navigateToToday();
                    break;
                case 'p':
                    e.preventDefault();
                    this.showCommandPalette();
                    break;
                // Edit mode toggle removed
            }
        }
        
        // Escape key
        if (e.key === 'Escape') {
            this.handleEscapeKey();
        }
    }
    
    handleEscapeKey() {
        // Close modals
        document.querySelectorAll('.modal-overlay.active').forEach(modal => {
            modal.classList.remove('active');
        });
    }
    
    handleBeforeUnload(e) {
        if (this.unsavedChanges) {
            e.preventDefault();
            e.returnValue = '';
        }
    }
    
    handleWindowResize() {
        // Handle responsive layout changes
        this.adjustLayoutForScreenSize();
    }
    
    handleDocumentClick(e) {
        // Close dropdowns and popovers when clicking outside
    }
    
    // ================================
    // Modal Management
    // ================================
    
    showQuickOpenModal() {
        document.getElementById('quick-open-modal').classList.add('active');
        document.getElementById('quick-open-input').focus();
    }
    
    hideQuickOpenModal() {
        document.getElementById('quick-open-modal').classList.remove('active');
    }
    
    showNewPageModal() {
        document.getElementById('new-page-modal').classList.add('active');
        document.getElementById('page-title-input').focus();
    }
    
    hideNewPageModal() {
        document.getElementById('new-page-modal').classList.remove('active');
        // Clear form
        document.getElementById('page-title-input').value = '';
        document.getElementById('page-path-input').value = '';
    }
    
    // ================================
    // Calendar Management
    // ================================
    
    renderCalendar() {
        // Basic calendar rendering - to be implemented
        const monthNames = [
            'January', 'February', 'March', 'April', 'May', 'June',
            'July', 'August', 'September', 'October', 'November', 'December'
        ];
        
        const now = new Date();
        const currentMonthName = monthNames[now.getMonth()];
        const currentYear = now.getFullYear();
        
        document.getElementById('current-month').textContent = `${currentMonthName} ${currentYear}`;
        
        // TODO: Implement full calendar rendering
    }
    
    // ================================
    // UI Interactions
    // ================================
    
    async createNewBlock() { 
        return this.blockEditor.createNewBlock();
    }
    
    async savePage() { 
        // Save any pending changes
        if (this.editingBlock && this.unsavedChanges) {
            const blockElement = document.querySelector(`[data-block-id="${this.editingBlock}"]`);
            if (blockElement) {
                const contentElement = blockElement.querySelector('.block-content');
                await this.saveBlockContent(this.editingBlock, contentElement.textContent);
            }
        }
        this.updateSaveStatus('Saved');
    }
    
    toggleEditMode() {
        this.isEditMode = !this.isEditMode;
        // Re-render blocks with new mode
        this.renderBlocks();
    }
    
    toggleSidebar(side) {
        if (side === 'left') {
            this.sidebar.toggleLeftSidebar();
        } else {
            this.sidebar.toggleRightSidebar();
        }
    }
    
    async performSearch(query) { 
        return this.search.performFullSearch(query);
    }
    
    async handleSearch(query) { 
        return this.search.performSearch(query);
    }
    
    async searchPages(query) { 
        return this.search.handleQuickOpenSearch(query);
    }
    
    async createNewPage() { 
        const titleInput = document.getElementById('page-title-input');
        const pathInput = document.getElementById('page-path-input');
        
        const title = titleInput.value.trim();
        const path = pathInput.value.trim();
        
        if (!title || !path) {
            this.showError('Please provide both title and path');
            return;
        }
        
        try {
            const response = await this.invokeCommand('create_page', { title, path });
            
            if (response.success) {
                this.hideNewPageModal();
                await this.loadPage(path);
                this.sidebar.refresh(); // Refresh recent pages
            } else {
                throw new Error(response.error || 'Failed to create page');
            }
        } catch (error) {
            console.error('Failed to create page:', error);
            this.showError('Failed to create page: ' + error.message);
        }
    }
    
    showCommandPalette() { 
        document.getElementById('command-palette').classList.add('active');
        document.getElementById('command-input').focus();
    }
    
    filterTasks(status) { 
        this.sidebar.setTaskFilter(status);
    }
    
    prevMonth() { 
        this.calendar.prevMonth();
    }
    
    nextMonth() { 
        this.calendar.nextMonth();
    }
    
    adjustLayoutForScreenSize() { 
        // Handle responsive layout adjustments
        const windowWidth = window.innerWidth;
        
        if (windowWidth < 768) {
            // Mobile layout
            if (!this.isLeftSidebarCollapsed) {
                this.sidebar.toggleLeftSidebar();
            }
        }
    }
    
    // Edit mode toggle function removed - always in edit mode now
    
    handleBlockDragStart(e) { 
        this.outliner.handleDragStart(e);
    }
    
    handleBlockDragOver(e) { 
        this.outliner.handleDragOver(e);
    }
    
    handleBlockDrop(e) { 
        this.outliner.handleDrop(e);
    }
    
    // Calendar rendering
    renderCalendar() {
        this.calendar.render();
    }
}

// Initialize application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.outlinerApp = new OutlinerApp();
});
