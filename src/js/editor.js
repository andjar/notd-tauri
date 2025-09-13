// Block Editor Component
// Phase 1: Individual block editing and content management

class BlockEditor {
    constructor(app) {
        this.app = app;
        this.currentBlock = null;
        this.isEditing = false;
        this.autoSaveTimeout = null;
        this.autoSaveDelay = 2000; // 2 seconds
    }

    // ================================
    // Block Creation
    // ================================

    async createNewBlock(parentId = null, position = null, content = '') {
        if (!this.app.currentPage) return null;

        try {
            const response = await this.app.invokeCommand('create_block', {
                request: {
                    page_id: this.app.currentPage.id,
                    parent_id: parentId,
                    content: content,
                    position: position
                }
            });

            if (response.success) {
                // Reload page blocks and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.updatePageHeader();
                this.app.renderBlocks();
                
                // Focus the new block
                const blockElement = document.querySelector(`[data-block-id="${response.data.id}"]`);
                if (blockElement) {
                    const contentElement = blockElement.querySelector('.block-content');
                    contentElement.focus();
                    // Place cursor at end
                    this.placeCursorAtEnd(contentElement);
                }

                return response.data;
            } else {
                throw new Error(response.error || 'Failed to create block');
            }
        } catch (error) {
            console.error('Failed to create block:', error);
            this.app.showError('Failed to create block: ' + error.message);
            return null;
        }
    }

    async createSiblingBlock(currentBlockElement) {
        const blockId = currentBlockElement.dataset.blockId;
        
        if (blockId === 'new') {
            const content = currentBlockElement.querySelector('.block-content').textContent;
            
            // Manually create the first block without triggering a full re-render yet.
            const response = await this.app.invokeCommand('create_block', {
                request: {
                    page_id: this.app.currentPage.id,
                    parent_id: null,
                    content: content,
                    position: null
                }
            });

            if (response.success) {
                // Now create a new empty block. This call will trigger load and render.
                await this.createNewBlock(null, null, '');
            } else {
                this.app.showError('Failed to save block: ' + response.error);
            }
            return;
        }

        const currentBlockData = this.findBlockById(parseInt(blockId));
        
        if (!currentBlockData) return;

        // Create sibling with same parent
        await this.createNewBlock(
            currentBlockData.block.parent_id,
            this.calculateSiblingPosition(currentBlockData)
        );
    }

    async createChildBlock(parentBlockElement) {
        const parentIdRaw = parentBlockElement.dataset.blockId;
        if (parentIdRaw === 'new') {
            // On a new block, Ctrl+Enter should first save the current block,
            // then create a new child block under it.
            const content = parentBlockElement.querySelector('.block-content').textContent;
            const response = await this.app.invokeCommand('create_block', {
                request: {
                    page_id: this.app.currentPage.id,
                    parent_id: null,
                    content: content,
                    position: null
                }
            });

            if (response.success) {
                const newParentId = response.data.id;
                // Now create a new empty block as a child.
                await this.createNewBlock(newParentId, null, '');
            } else {
                this.app.showError('Failed to save block: ' + response.error);
            }
            return;
        }
        const parentId = parseInt(parentBlockElement.dataset.blockId);
        
        // Create child block
        await this.createNewBlock(parentId, null);
    }

    calculateSiblingPosition(currentBlockData) {
        // Find current block's position among siblings
        const parentId = currentBlockData.block.parent_id;
        const siblings = this.getSiblingsOf(currentBlockData.block);
        const currentIndex = siblings.findIndex(s => s.block.id === currentBlockData.block.id);
        
        return currentIndex >= 0 ? currentIndex + 1 : null;
    }

    // ================================
    // Block Hierarchy Management
    // ================================

    async indentBlock(blockElement) {
        const blockId = parseInt(blockElement.dataset.blockId);
        
        console.log('🔄 Indenting block:', blockId, 'Element:', blockElement);
        
        if (!blockId || isNaN(blockId)) {
            console.error('❌ Invalid block ID for indentation:', blockId);
            return;
        }
        
        try {
            const response = await this.app.invokeCommand('indent_block', { block_id: blockId });
            
            if (response.success) {
                console.log('✅ Block indented successfully');
                // Reload and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.renderBlocks();
                
                // Restore focus
                this.restoreFocusToBlock(blockId);
            } else {
                throw new Error(response.error || 'Failed to indent block');
            }
        } catch (error) {
            console.error('Failed to indent block:', error);
            this.app.showError('Failed to indent block: ' + error.message);
        }
    }

    async outdentBlock(blockElement) {
        const blockId = parseInt(blockElement.dataset.blockId);
        
        console.log('🔄 Outdenting block:', blockId, 'Element:', blockElement);
        
        if (!blockId || isNaN(blockId)) {
            console.error('❌ Invalid block ID for outdentation:', blockId);
            return;
        }
        
        try {
            const response = await this.app.invokeCommand('outdent_block', { block_id: blockId });
            
            if (response.success) {
                console.log('✅ Block outdented successfully');
                // Reload and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.renderBlocks();
                
                // Restore focus
                this.restoreFocusToBlock(blockId);
            } else {
                throw new Error(response.error || 'Failed to outdent block');
            }
        } catch (error) {
            console.error('Failed to outdent block:', error);
            this.app.showError('Failed to outdent block: ' + error.message);
        }
    }

    async moveBlockUp(blockElement) {
        const blockId = parseInt(blockElement.dataset.blockId);
        const blockData = this.findBlockById(blockId);
        
        if (!blockData) return;

        const siblings = this.getSiblingsOf(blockData.block);
        const currentIndex = siblings.findIndex(s => s.block.id === blockId);
        
        if (currentIndex > 0) {
            // Move to position before previous sibling
            await this.moveBlockToPosition(blockId, blockData.block.parent_id, currentIndex - 1);
        }
    }

    async moveBlockDown(blockElement) {
        const blockId = parseInt(blockElement.dataset.blockId);
        const blockData = this.findBlockById(blockId);
        
        if (!blockData) return;

        const siblings = this.getSiblingsOf(blockData.block);
        const currentIndex = siblings.findIndex(s => s.block.id === blockId);
        
        if (currentIndex < siblings.length - 1) {
            // Move to position after next sibling
            await this.moveBlockToPosition(blockId, blockData.block.parent_id, currentIndex + 2);
        }
    }

    async moveBlockToPosition(blockId, newParentId, newPosition) {
        try {
            const response = await this.app.invokeCommand('move_block', {
                block_id: blockId,
                new_parent_id: newParentId,
                new_position: newPosition
            });

            if (response.success) {
                // Reload and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.renderBlocks();
                
                // Restore focus
                this.restoreFocusToBlock(blockId);
            } else {
                throw new Error(response.error || 'Failed to move block');
            }
        } catch (error) {
            console.error('Failed to move block:', error);
            this.app.showError('Failed to move block: ' + error.message);
        }
    }

    // ================================
    // Block Deletion
    // ================================

    async deleteOrPromoteBlock(blockElement) {
        const blockId = parseInt(blockElement.dataset.blockId);
        const blockData = this.findBlockById(blockId);
        
        if (!blockData) return;

        // If block has content, confirm deletion
        if (blockData.block.content.trim() !== '') {
            if (!confirm('Delete this block and all its children?')) {
                return;
            }
        }

        try {
            const response = await this.app.invokeCommand('delete_block', { block_id: blockId });
            
            if (response.success) {
                // Find next block to focus on
                const nextBlock = this.findAdjacentBlock(blockElement, 'down') || 
                                 this.findAdjacentBlock(blockElement, 'up');

                // Reload and re-render
                await this.app.loadPageBlocks(this.app.currentPage.id);
                this.app.renderBlocks();
                
                // Focus next block
                if (nextBlock) {
                    const nextBlockId = nextBlock.dataset.blockId;
                    this.restoreFocusToBlock(parseInt(nextBlockId));
                } else {
                    // Create a new empty block if we deleted the last one
                    await this.createNewBlock();
                }
            } else {
                throw new Error(response.error || 'Failed to delete block');
            }
        } catch (error) {
            console.error('Failed to delete block:', error);
            this.app.showError('Failed to delete block: ' + error.message);
        }
    }

    // ================================
    // Block Navigation
    // ================================

    moveToAdjacentBlock(currentBlockElement, direction) {
        const adjacentBlock = this.findAdjacentBlock(currentBlockElement, direction);
        
        if (adjacentBlock) {
            const contentElement = adjacentBlock.querySelector('.block-content');
            contentElement.focus();
            
            if (direction === 'up') {
                this.placeCursorAtEnd(contentElement);
            } else {
                this.placeCursorAtStart(contentElement);
            }
        }
    }

    findAdjacentBlock(currentBlockElement, direction) {
        const allBlocks = Array.from(document.querySelectorAll('.block-item'));
        const currentIndex = allBlocks.indexOf(currentBlockElement);
        
        if (direction === 'up' && currentIndex > 0) {
            return allBlocks[currentIndex - 1];
        } else if (direction === 'down' && currentIndex < allBlocks.length - 1) {
            return allBlocks[currentIndex + 1];
        }
        
        return null;
    }

    // ================================
    // Content Management
    // ================================

    async saveBlockContent(blockId, content) {
        // Clear any pending auto-save
        if (this.autoSaveTimeout) {
            clearTimeout(this.autoSaveTimeout);
        }

        if (blockId === 'new') {
            console.log('📝 Handling new block creation for content:', content);
            // Handle new block creation
            if (content.trim()) {
                console.log('📝 Creating new block with content:', content);
                const newBlock = await this.app.createNewBlockWithContent(content.trim());
                if (newBlock) {
                    console.log('✅ New block created successfully:', newBlock.id);
                    // Note: createNewBlock already handles re-rendering, so no need to update DOM here
                } else {
                    console.error('❌ Failed to create new block');
                }
            } else {
                console.log('⚠️ Empty content, skipping new block creation');
            }
            return;
        }

        if (isNaN(parseInt(blockId)) || !blockId) {
            console.log('⚠️ Attempting to save invalid block ID:', blockId, 'Content:', content);
            return;
        }

        try {
            console.log('💾 Saving block:', blockId, 'Content:', content);
            const response = await this.app.invokeCommand('update_block', {
                block_id: parseInt(blockId),
                request: { content: content }
            });

            console.log('📡 Save response:', response);

            if (response.success) {
                console.log('✅ Block saved successfully:', blockId);
                this.app.updateSaveStatus('Saved');
                this.app.unsavedChanges = false;
                
                // Update block data in memory
                this.updateBlockInMemory(parseInt(blockId), content);
            } else {
                console.error('❌ Save failed:', response.error);
                throw new Error(response.error || 'Failed to save block');
            }
        } catch (error) {
            console.error('❌ Failed to save block content:', error);
            this.app.showError('Failed to save: ' + error.message);
            this.app.updateSaveStatus('Save failed');
        }
    }

    scheduleAutoSave(blockId, content) {
        // Clear existing timeout
        if (this.autoSaveTimeout) {
            clearTimeout(this.autoSaveTimeout);
        }

        // Schedule new auto-save
        this.autoSaveTimeout = setTimeout(() => {
            this.saveBlockContent(blockId, content);
        }, this.autoSaveDelay);

        // Update UI to show unsaved state
        this.app.updateSaveStatus('Saving...');
        this.app.unsavedChanges = true;
    }

    updateBlockInMemory(blockId, content) {
        // Update the block content in the current blocks array
        const updateBlockRecursive = (blocks) => {
            for (const blockData of blocks) {
                if (blockData.block.id === blockId) {
                    blockData.block.content = content;
                    blockData.block.updated_at = new Date().toISOString();
                    return true;
                }
                if (blockData.children && updateBlockRecursive(blockData.children)) {
                    return true;
                }
            }
            return false;
        };

        updateBlockRecursive(this.app.currentBlocks);
    }

    // ================================
    // Utility Methods
    // ================================

    findBlockById(blockId) {
        const findBlockRecursive = (blocks) => {
            for (const blockData of blocks) {
                if (blockData.block.id === blockId) {
                    return blockData;
                }
                if (blockData.children) {
                    const found = findBlockRecursive(blockData.children);
                    if (found) return found;
                }
            }
            return null;
        };

        return findBlockRecursive(this.app.currentBlocks);
    }

    getSiblingsOf(block) {
        const findSiblingsRecursive = (blocks, parentId) => {
            if (parentId === null) {
                // Root level blocks
                return blocks.filter(b => b.block.parent_id === null);
            }
            
            for (const blockData of blocks) {
                if (blockData.block.id === parentId) {
                    return blockData.children || [];
                }
                if (blockData.children) {
                    const found = findSiblingsRecursive(blockData.children, parentId);
                    if (found.length > 0) return found;
                }
            }
            return [];
        };

        return findSiblingsRecursive(this.app.currentBlocks, block.parent_id);
    }

    restoreFocusToBlock(blockId) {
        // Wait for DOM update
        setTimeout(() => {
            const blockElement = document.querySelector(`[data-block-id="${blockId}"]`);
            if (blockElement) {
                const contentElement = blockElement.querySelector('.block-content');
                contentElement.focus();
            }
        }, 100);
    }

    placeCursorAtEnd(element) {
        const range = document.createRange();
        const selection = window.getSelection();
        range.selectNodeContents(element);
        range.collapse(false);
        selection.removeAllRanges();
        selection.addRange(range);
    }

    placeCursorAtStart(element) {
        const range = document.createRange();
        const selection = window.getSelection();
        range.selectNodeContents(element);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
    }

    // ================================
    // Content Processing
    // ================================

    processBlockContent(content) {
        // Basic content processing - link detection, task detection, etc.
        let processed = content;

        // Detect and mark page links
        processed = this.processPageLinks(processed);
        
        // Detect task status
        const taskStatus = this.detectTaskStatus(processed);
        
        return {
            content: processed,
            taskStatus: taskStatus,
            hasLinks: this.hasPageLinks(content)
        };
    }

    processPageLinks(content) {
        // Convert [[Page Name]] to clickable links in render mode
        return content.replace(/\[\[([^\]]+)\]\]/g, (match, pageName) => {
            return `<a href="#" class="page-link" data-page="${pageName}">${pageName}</a>`;
        });
    }

    detectTaskStatus(content) {
        const taskRegex = /^(TODO|DOING|DONE|WAITING|CANCELLED)\s+/;
        const match = content.match(taskRegex);
        return match ? match[1] : null;
    }

    hasPageLinks(content) {
        return /\[\[[^\]]+\]\]/.test(content);
    }

    // ================================
    // Event Handlers
    // ================================

    handleContentInput(contentElement) {
        const blockElement = contentElement.closest('.block-item');
        const blockId = blockElement.dataset.blockId;
        const content = contentElement.textContent || contentElement.innerHTML;

        console.log('📝 Content input for block:', blockId, 'Content:', content);

        // Process content for task detection, etc.
        const processed = this.processBlockContent(content);
        
        // Update block styling based on content
        this.updateBlockStyling(blockElement, processed);
        
        // Schedule auto-save for valid blocks (including new blocks)
        if (blockId && (blockId === 'new' || !isNaN(parseInt(blockId)))) {
            this.scheduleAutoSave(blockId, content);
        } else {
            console.log('⚠️ Skipping auto-save for block ID:', blockId, '(will save on blur)');
        }

        // Update word count
        this.updateWordCount();
    }

    updateBlockStyling(blockElement, processedContent) {
        // Update task styling
        blockElement.classList.remove('task', 'task-todo', 'task-doing', 'task-done', 'task-waiting', 'task-cancelled');
        
        if (processedContent.taskStatus) {
            blockElement.classList.add('task', `task-${processedContent.taskStatus.toLowerCase()}`);
        }

        // Update other styling based on content
        if (processedContent.hasLinks) {
            blockElement.classList.add('has-links');
        } else {
            blockElement.classList.remove('has-links');
        }
    }

    updateWordCount() {
        const allContent = Array.from(document.querySelectorAll('.block-content'))
            .map(el => el.textContent || '')
            .join(' ');
        
        const wordCount = allContent.trim() ? allContent.trim().split(/\s+/).length : 0;
        document.getElementById('word-count').textContent = `${wordCount} words`;
    }
}

// Export for module usage
export default BlockEditor;
