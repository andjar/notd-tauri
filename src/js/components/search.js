// Search Component
// Phase 1: Full-text search across pages and blocks

class SearchComponent {
    constructor(app) {
        this.app = app;
        this.searchInput = document.getElementById('search-input');
        this.searchResults = document.getElementById('search-results');
        this.quickOpenInput = document.getElementById('quick-open-input');
        this.quickOpenResults = document.getElementById('quick-open-results');
        
        this.searchTimeout = null;
        this.searchDelay = 300; // ms
        this.isSearching = false;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Main search input
        this.searchInput.addEventListener('input', (e) => {
            this.handleSearchInput(e.target.value);
        });
        
        this.searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                this.performFullSearch(e.target.value);
            } else if (e.key === 'Escape') {
                this.clearSearch();
            }
        });
        
        // Quick open search
        this.quickOpenInput.addEventListener('input', (e) => {
            this.handleQuickOpenSearch(e.target.value);
        });
        
        this.quickOpenInput.addEventListener('keydown', (e) => {
            this.handleQuickOpenKeydown(e);
        });
        
        // Click outside to close results
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.search-container')) {
                this.hideSearchResults();
            }
        });
    }

    // ================================
    // Main Search
    // ================================

    handleSearchInput(query) {
        // Clear previous timeout
        if (this.searchTimeout) {
            clearTimeout(this.searchTimeout);
        }
        
        if (query.trim() === '') {
            this.hideSearchResults();
            return;
        }
        
        // Debounce search
        this.searchTimeout = setTimeout(() => {
            this.performSearch(query);
        }, this.searchDelay);
    }

    async performSearch(query) {
        if (this.isSearching) return;
        
        this.isSearching = true;
        this.showSearchLoading();
        
        try {
            const response = await this.app.invokeCommand('global_search', {
                request: {
                    query: query,
                    limit: 20
                }
            });
            
            if (response.success) {
                this.displaySearchResults(response.data, query);
            } else {
                this.showSearchError(response.error || 'Search failed');
            }
        } catch (error) {
            console.error('Search failed:', error);
            this.showSearchError('Search failed: ' + error.message);
        } finally {
            this.isSearching = false;
        }
    }

    async performFullSearch(query) {
        if (!query.trim()) return;
        
        // TODO: Show full search results in a dedicated modal/page
        await this.performSearch(query);
        this.showSearchResults();
    }

    displaySearchResults(results, query) {
        const { pages, blocks } = results;
        let html = '';
        
        // Pages results
        if (pages && pages.length > 0) {
            html += '<div class="search-section"><h5>Pages</h5>';
            for (const page of pages) {
                html += this.renderPageResult(page, query);
            }
            html += '</div>';
        }
        
        // Blocks results
        if (blocks && blocks.length > 0) {
            html += '<div class="search-section"><h5>Blocks</h5>';
            for (const blockResult of blocks) {
                html += this.renderBlockResult(blockResult, query);
            }
            html += '</div>';
        }
        
        if (!html) {
            html = '<div class="no-results">No results found</div>';
        }
        
        this.searchResults.innerHTML = html;
        this.showSearchResults();
    }

    renderPageResult(page, query) {
        const title = this.highlightQuery(page.title, query);
        const path = this.highlightQuery(page.path, query);
        
        return `
            <div class="search-result" onclick="window.outlinerApp.search.navigateToPage('${page.path}')">
                <div class="search-result-title">${title}</div>
                <div class="search-result-snippet">${path}</div>
            </div>
        `;
    }

    renderBlockResult(blockResult, query) {
        const content = blockResult.snippet || blockResult.block.content;
        const highlightedContent = this.highlightQuery(content, query);
        
        return `
            <div class="search-result" onclick="window.outlinerApp.search.navigateToBlock('${blockResult.page_path}', '${blockResult.block.uuid}')">
                <div class="search-result-title">${blockResult.page_title}</div>
                <div class="search-result-snippet">${highlightedContent}</div>
            </div>
        `;
    }

    highlightQuery(text, query) {
        if (!query || !text) return text;
        
        const regex = new RegExp(`(${this.escapeRegExp(query)})`, 'gi');
        return text.replace(regex, '<span class="search-highlight">$1</span>');
    }

    escapeRegExp(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    // ================================
    // Quick Open Search
    // ================================

    async handleQuickOpenSearch(query) {
        if (query.trim() === '') {
            this.clearQuickOpenResults();
            return;
        }
        
        try {
            const response = await this.app.invokeCommand('search_pages', {
                query: query,
                limit: 10
            });
            
            if (response.success) {
                this.displayQuickOpenResults(response.data);
            }
        } catch (error) {
            console.error('Quick open search failed:', error);
        }
    }

    displayQuickOpenResults(pages) {
        if (!pages || pages.length === 0) {
            this.quickOpenResults.innerHTML = '<div class="no-results">No pages found</div>';
            return;
        }
        
        let html = '';
        for (const page of pages) {
            html += `
                <div class="quick-open-result" 
                     data-page-path="${page.path}"
                     onclick="window.outlinerApp.search.selectQuickOpenResult('${page.path}')">
                    <div class="result-title">${page.title}</div>
                    <div class="result-path">${page.path}</div>
                </div>
            `;
        }
        
        this.quickOpenResults.innerHTML = html;
    }

    handleQuickOpenKeydown(e) {
        const results = this.quickOpenResults.querySelectorAll('.quick-open-result');
        const selected = this.quickOpenResults.querySelector('.quick-open-result.selected');
        let selectedIndex = selected ? Array.from(results).indexOf(selected) : -1;
        
        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
                this.selectQuickOpenItem(results, selectedIndex);
                break;
                
            case 'ArrowUp':
                e.preventDefault();
                selectedIndex = Math.max(selectedIndex - 1, 0);
                this.selectQuickOpenItem(results, selectedIndex);
                break;
                
            case 'Enter':
                e.preventDefault();
                if (selected) {
                    const pagePath = selected.dataset.pagePath;
                    this.selectQuickOpenResult(pagePath);
                } else if (results.length > 0) {
                    const pagePath = results[0].dataset.pagePath;
                    this.selectQuickOpenResult(pagePath);
                }
                break;
                
            case 'Escape':
                e.preventDefault();
                this.app.hideQuickOpenModal();
                break;
        }
    }

    selectQuickOpenItem(results, index) {
        // Remove previous selection
        results.forEach(result => result.classList.remove('selected'));
        
        // Add selection to new item
        if (results[index]) {
            results[index].classList.add('selected');
        }
    }

    async selectQuickOpenResult(pagePath) {
        this.app.hideQuickOpenModal();
        await this.app.loadPage(pagePath);
    }

    // ================================
    // Navigation
    // ================================

    async navigateToPage(pagePath) {
        this.hideSearchResults();
        await this.app.loadPage(pagePath);
    }

    async navigateToBlock(pagePath, blockUuid) {
        this.hideSearchResults();
        await this.app.loadPage(pagePath);
        
        // Focus the specific block
        setTimeout(() => {
            const blockElement = document.querySelector(`[data-block-uuid="${blockUuid}"]`);
            if (blockElement) {
                const contentElement = blockElement.querySelector('.block-content');
                contentElement.focus();
                blockElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }, 100);
    }

    // ================================
    // UI State Management
    // ================================

    showSearchResults() {
        this.searchResults.classList.add('active');
    }

    hideSearchResults() {
        this.searchResults.classList.remove('active');
    }

    showSearchLoading() {
        this.searchResults.innerHTML = '<div class="search-loading">Searching...</div>';
        this.showSearchResults();
    }

    showSearchError(message) {
        this.searchResults.innerHTML = `<div class="search-error">Error: ${message}</div>`;
        this.showSearchResults();
    }

    clearSearch() {
        this.searchInput.value = '';
        this.hideSearchResults();
    }

    clearQuickOpenResults() {
        this.quickOpenResults.innerHTML = '';
    }

    // ================================
    // Advanced Search (Future Enhancement)
    // ================================

    async performAdvancedSearch(criteria) {
        try {
            const response = await this.app.invokeCommand('advanced_search', { criteria });
            
            if (response.success) {
                // TODO: Display advanced search results in dedicated interface
                console.log('Advanced search results:', response.data);
            }
        } catch (error) {
            console.error('Advanced search failed:', error);
        }
    }

    // Search by tags
    async searchByTag(tag) {
        await this.performSearch(`#${tag}`);
    }

    // Search by property
    async searchByProperty(key, value = null) {
        try {
            const response = await this.app.invokeCommand('search_blocks_by_property', {
                request: {
                    key: key,
                    value: value
                }
            });
            
            if (response.success) {
                // Display property search results
                this.displayPropertySearchResults(response.data, key, value);
            }
        } catch (error) {
            console.error('Property search failed:', error);
        }
    }

    displayPropertySearchResults(blocks, key, value) {
        let html = `<div class="search-section"><h5>Blocks with ${key}${value ? `: ${value}` : ''}</h5>`;
        
        for (const block of blocks) {
            html += `
                <div class="search-result" onclick="window.outlinerApp.search.navigateToBlock('${block.page_path || ''}', '${block.uuid}')">
                    <div class="search-result-snippet">${block.content}</div>
                </div>
            `;
        }
        
        html += '</div>';
        
        this.searchResults.innerHTML = html;
        this.showSearchResults();
    }
}

// Add search-specific CSS
const searchCSS = `
.search-section {
    margin-bottom: var(--spacing-md);
}

.search-section h5 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    background-color: var(--bg-tertiary);
    border-radius: var(--border-radius-sm);
}

.quick-open-result {
    padding: var(--spacing-sm);
    border-bottom: 1px solid var(--border-light);
    cursor: pointer;
    transition: background-color var(--transition-fast);
}

.quick-open-result:last-child {
    border-bottom: none;
}

.quick-open-result:hover,
.quick-open-result.selected {
    background-color: var(--bg-active);
}

.result-title {
    font-weight: 500;
    color: var(--text-primary);
    font-size: 13px;
}

.result-path {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: monospace;
}

.search-loading,
.search-error,
.no-results {
    padding: var(--spacing-md);
    text-align: center;
    color: var(--text-secondary);
    font-size: 12px;
    font-style: italic;
}

.search-error {
    color: var(--accent-red);
}
`;

// Inject CSS
const style = document.createElement('style');
style.textContent = searchCSS;
document.head.appendChild(style);

// Export for module usage
export default SearchComponent;
