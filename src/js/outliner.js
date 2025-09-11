// Outliner Management Component
// Phase 1: Block hierarchy visualization and manipulation

class OutlinerManager {
    constructor(app) {
        this.app = app;
        this.draggedBlock = null;
        this.dragOverTarget = null;
        this.dropIndicator = null;
    }

    // ================================
    // Block Rendering
    // ================================

    renderBlockHierarchy(blocks, level = 0) {
        let html = '';
        
        for (const blockData of blocks) {
            html += this.renderBlock(blockData, level);
        }
        
        return html;
    }

    renderBlock(blockData, level = 0) {
        const block = blockData.block;
        const children = blockData.children || [];
        const properties = blockData.properties || [];
        
        // Determine block classes
        const classes = ['block-item'];
        
        // Task styling
        const taskStatus = this.detectTaskStatus(block.content);
        if (taskStatus) {
            classes.push('task', `task-${taskStatus.toLowerCase()}`);
        }
        
        // Selection and editing states
        if (this.app.selectedBlock === block.id) classes.push('selected');
        if (this.app.editingBlock === block.id) classes.push('editing');
        
        // Render block
        let html = `
            <div class="${classes.join(' ')}" 
                 data-block-id="${block.id}" 
                 data-level="${level}"
                 data-block-uuid="${block.uuid}">
                <div class="block-bullet" 
                     draggable="true" 
                     data-block-id="${block.id}">
                    ${this.renderBullet(block, taskStatus)}
                </div>
                <div class="block-content" 
                     contenteditable="true" 
                     data-placeholder="${level === 0 ? 'Start writing...' : 'Empty block'}"
                     data-block-id="${block.id}">
                    ${this.renderBlockContent(block.content)}
                </div>
            </div>
        `;
        
        // Render children recursively
        if (children.length > 0) {
            html += `<div class="block-children">`;
            html += this.renderBlockHierarchy(children, level + 1);
            html += `</div>`;
        }
        
        return html;
    }

    renderBullet(block, taskStatus) {
        if (taskStatus) {
            // Task checkbox
            const isCompleted = taskStatus === 'DONE' || taskStatus === 'CANCELLED';
            return `<input type="checkbox" 
                           class="task-checkbox" 
                           ${isCompleted ? 'checked' : ''}
                           data-status="${taskStatus.toLowerCase()}"
                           onchange="window.outlinerApp.outliner.toggleTaskStatus('${block.id}', this.checked)">`;
        } else {
            // Regular bullet
            return '•';
        }
    }

    renderBlockContent(content) {
        if (!content) return '';
        
        // Escape HTML for safety
        let escaped = this.escapeHtml(content);
        
        // Process in render mode vs edit mode
        if (!this.app.isEditMode) {
            // Render mode: process links, etc.
            escaped = this.processLinksForRendering(escaped);
        }
        
        return escaped;
    }

    processLinksForRendering(content) {
        // Convert [[Page Name]] to clickable links
        return content.replace(/\[\[([^\]]+)\]\]/g, (match, pageName) => {
            return `<a href="#" class="page-link" data-page="${pageName.trim()}" onclick="window.outlinerApp.navigateToPage('${pageName.trim()}')">${pageName.trim()}</a>`;
        });
    }

    detectTaskStatus(content) {
        const taskRegex = /^(TODO|DOING|DONE|WAITING|CANCELLED)\s+/;
        const match = content.match(taskRegex);
        return match ? match[1] : null;
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // ================================
    // Drag and Drop
    // ================================

    initializeDragAndDrop() {
        // Create drop indicator element
        this.dropIndicator = document.createElement('div');
        this.dropIndicator.className = 'drop-indicator';
        this.dropIndicator.style.cssText = `
            height: 2px;
            background-color: var(--accent-blue);
            margin: 2px 0;
            border-radius: 1px;
            display: none;
        `;
    }

    handleDragStart(e) {
        const bullet = e.target;
        const blockElement = bullet.closest('.block-item');
        
        if (!blockElement) return;
        
        this.draggedBlock = {
            element: blockElement,
            id: parseInt(blockElement.dataset.blockId),
            level: parseInt(blockElement.dataset.level)
        };
        
        // Add dragging class
        blockElement.classList.add('dragging');
        
        // Set drag data
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('text/plain', blockElement.dataset.blockId);
        
        console.log('Drag started for block:', this.draggedBlock.id);
    }

    handleDragOver(e) {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        
        const blockElement = e.target.closest('.block-item');
        if (!blockElement || blockElement === this.draggedBlock?.element) {
            this.hideDropIndicator();
            return;
        }
        
        const rect = blockElement.getBoundingClientRect();
        const mouseY = e.clientY;
        const blockMiddle = rect.top + rect.height / 2;
        
        let dropPosition;
        let insertBefore;
        
        if (mouseY < blockMiddle) {
            // Drop above the block
            dropPosition = 'before';
            insertBefore = blockElement;
        } else {
            // Drop below the block
            dropPosition = 'after';
            insertBefore = blockElement.nextElementSibling;
        }
        
        this.showDropIndicator(insertBefore || blockElement.parentNode, dropPosition);
    }

    handleDrop(e) {
        e.preventDefault();
        
        if (!this.draggedBlock) return;
        
        const targetElement = e.target.closest('.block-item');
        if (!targetElement || targetElement === this.draggedBlock.element) {
            this.cleanupDrag();
            return;
        }
        
        // Calculate drop position
        const rect = targetElement.getBoundingClientRect();
        const mouseY = e.clientY;
        const blockMiddle = rect.top + rect.height / 2;
        
        const isDroppedAbove = mouseY < blockMiddle;
        const targetBlockId = parseInt(targetElement.dataset.blockId);
        
        // Move the block
        this.moveBlockRelativeTo(this.draggedBlock.id, targetBlockId, isDroppedAbove);
        
        this.cleanupDrag();
    }

    handleDragEnd(e) {
        this.cleanupDrag();
    }

    async moveBlockRelativeTo(blockId, targetBlockId, insertBefore) {
        try {
            // Find target block data to determine new parent and position
            const targetBlockData = this.app.blockEditor.findBlockById(targetBlockId);
            if (!targetBlockData) return;
            
            // Calculate new position
            const siblings = this.app.blockEditor.getSiblingsOf(targetBlockData.block);
            const targetIndex = siblings.findIndex(s => s.block.id === targetBlockId);
            const newPosition = insertBefore ? targetIndex : targetIndex + 1;
            
            const response = await this.app.invokeCommand('move_block', {
                block_id: blockId,
                new_parent_id: targetBlockData.block.parent_id,
                new_position: newPosition
            });

            if (response.success) {
                // Reload and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.renderBlocks();
            } else {
                throw new Error(response.error || 'Failed to move block');
            }
        } catch (error) {
            console.error('Failed to move block:', error);
            this.app.showError('Failed to move block: ' + error.message);
        }
    }

    showDropIndicator(referenceElement, position) {
        this.hideDropIndicator();
        
        if (!this.dropIndicator) this.initializeDragAndDrop();
        
        if (position === 'before') {
            referenceElement.parentNode.insertBefore(this.dropIndicator, referenceElement);
        } else {
            referenceElement.parentNode.insertBefore(this.dropIndicator, referenceElement.nextSibling);
        }
        
        this.dropIndicator.style.display = 'block';
    }

    hideDropIndicator() {
        if (this.dropIndicator && this.dropIndicator.parentNode) {
            this.dropIndicator.style.display = 'none';
            this.dropIndicator.parentNode.removeChild(this.dropIndicator);
        }
    }

    cleanupDrag() {
        if (this.draggedBlock) {
            this.draggedBlock.element.classList.remove('dragging');
            this.draggedBlock = null;
        }
        
        this.hideDropIndicator();
    }

    // ================================
    // Task Management
    // ================================

    async toggleTaskStatus(blockId, isCompleted) {
        try {
            const blockData = this.app.blockEditor.findBlockById(parseInt(blockId));
            if (!blockData) return;
            
            const currentContent = blockData.block.content;
            const currentStatus = this.detectTaskStatus(currentContent);
            
            if (!currentStatus) return;
            
            // Determine new status
            let newStatus;
            if (isCompleted) {
                newStatus = 'DONE';
            } else {
                newStatus = 'TODO';
            }
            
            // Update content with new status
            const contentWithoutStatus = currentContent.replace(/^(TODO|DOING|DONE|WAITING|CANCELLED)\s+/, '');
            const newContent = `${newStatus} ${contentWithoutStatus}`;
            
            // Save the updated content
            await this.app.blockEditor.saveBlockContent(parseInt(blockId), newContent);
            
            // Update UI
            const blockElement = document.querySelector(`[data-block-id="${blockId}"]`);
            if (blockElement) {
                blockElement.className = blockElement.className.replace(/task-\w+/g, '');
                blockElement.classList.add('task', `task-${newStatus.toLowerCase()}`);
            }
            
        } catch (error) {
            console.error('Failed to toggle task status:', error);
            this.app.showError('Failed to update task: ' + error.message);
        }
    }

    // ================================
    // Block Focus Management
    // ================================

    handleBlockFocus(contentElement) {
        const blockElement = contentElement.closest('.block-item');
        const blockId = parseInt(blockElement.dataset.blockId);
        
        // Update selection state
        this.app.selectedBlock = blockId;
        this.app.editingBlock = blockId;
        
        // Update visual state
        document.querySelectorAll('.block-item.selected').forEach(el => {
            el.classList.remove('selected');
        });
        blockElement.classList.add('selected', 'editing');
        
        // Update properties panel
        this.updatePropertiesPanel(blockId);
        
        // Update current block path in status bar
        this.updateCurrentBlockPath(blockElement);
    }

    handleBlockBlur(contentElement) {
        const blockElement = contentElement.closest('.block-item');
        const blockId = parseInt(blockElement.dataset.blockId);
        
        blockElement.classList.remove('editing');
        
        // Save content
        const content = contentElement.textContent || '';
        this.app.blockEditor.saveBlockContent(blockId, content);
        
        this.app.editingBlock = null;
    }

    updateCurrentBlockPath(blockElement) {
        const level = parseInt(blockElement.dataset.level);
        const path = `Block (Level ${level})`;
        document.getElementById('current-block-path').textContent = path;
    }

    // ================================
    // Properties Panel
    // ================================

    async updatePropertiesPanel(blockId) {
        const container = document.getElementById('properties-container');
        
        try {
            const response = await this.app.invokeCommand('get_block_properties', { block_id: blockId });
            
            if (response.success) {
                const properties = response.data || [];
                this.renderProperties(container, properties, blockId);
            } else {
                container.innerHTML = '<div class="no-selection">Failed to load properties</div>';
            }
        } catch (error) {
            console.error('Failed to load block properties:', error);
            container.innerHTML = '<div class="no-selection">Error loading properties</div>';
        }
    }

    renderProperties(container, properties, blockId) {
        if (properties.length === 0) {
            container.innerHTML = '<div class="no-selection">No properties set</div>';
            return;
        }
        
        let html = '';
        for (const property of properties) {
            html += `
                <div class="property-item" data-property-key="${property.key}">
                    <span class="property-key">${property.key}</span>
                    <input type="text" 
                           class="property-value" 
                           value="${this.escapeHtml(property.value)}"
                           data-block-id="${blockId}"
                           data-property-key="${property.key}"
                           onchange="window.outlinerApp.outliner.updateProperty(${blockId}, '${property.key}', this.value)">
                    <button class="property-delete" 
                            onclick="window.outlinerApp.outliner.deleteProperty(${blockId}, '${property.key}')">
                        ×
                    </button>
                </div>
            `;
        }
        
        container.innerHTML = html;
    }

    async updateProperty(blockId, key, value) {
        try {
            const response = await this.app.invokeCommand('set_block_property', {
                block_id: blockId,
                key: key,
                value: value,
                value_type: 'text'
            });
            
            if (!response.success) {
                throw new Error(response.error || 'Failed to update property');
            }
        } catch (error) {
            console.error('Failed to update property:', error);
            this.app.showError('Failed to update property: ' + error.message);
        }
    }

    async deleteProperty(blockId, key) {
        if (!confirm(`Delete property "${key}"?`)) return;
        
        try {
            const response = await this.app.invokeCommand('delete_block_property', {
                block_id: blockId,
                key: key
            });
            
            if (response.success) {
                // Refresh properties panel
                await this.updatePropertiesPanel(blockId);
            } else {
                throw new Error(response.error || 'Failed to delete property');
            }
        } catch (error) {
            console.error('Failed to delete property:', error);
            this.app.showError('Failed to delete property: ' + error.message);
        }
    }

    async addNewProperty(blockId) {
        const key = prompt('Property name:');
        if (!key) return;
        
        const value = prompt('Property value:');
        if (value === null) return;
        
        await this.updateProperty(blockId, key, value || '');
        await this.updatePropertiesPanel(blockId);
    }

    // ================================
    // Event Listeners Setup
    // ================================

    setupEventListeners() {
        // Add property button
        document.getElementById('add-property-btn').addEventListener('click', () => {
            if (this.app.selectedBlock) {
                this.addNewProperty(this.app.selectedBlock);
            }
        });
        
        // Global drag and drop
        document.addEventListener('dragstart', (e) => {
            if (e.target.classList.contains('block-bullet')) {
                this.handleDragStart(e);
            }
        });
        
        document.addEventListener('dragover', (e) => {
            if (this.draggedBlock) {
                this.handleDragOver(e);
            }
        });
        
        document.addEventListener('drop', (e) => {
            if (this.draggedBlock) {
                this.handleDrop(e);
            }
        });
        
        document.addEventListener('dragend', (e) => {
            this.handleDragEnd(e);
        });
    }
}

// Export for module usage
export default OutlinerManager;
