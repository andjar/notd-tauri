// Calendar Component
// Phase 1: Basic calendar navigation and daily page integration

class CalendarComponent {
    constructor(app) {
        this.app = app;
        this.currentDate = new Date();
        this.today = new Date();
        this.pagesWithContent = new Set(); // Will be populated from database
        
        this.monthNames = [
            'January', 'February', 'March', 'April', 'May', 'June',
            'July', 'August', 'September', 'October', 'November', 'December'
        ];
        
        this.dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
        
        this.init();
    }

    init() {
        this.render();
        this.loadPagesWithContent();
    }

    // ================================
    // Calendar Rendering
    // ================================

    render() {
        this.renderHeader();
        this.renderDays();
    }

    renderHeader() {
        const monthElement = document.getElementById('current-month');
        const monthName = this.monthNames[this.currentDate.getMonth()];
        const year = this.currentDate.getFullYear();
        
        monthElement.textContent = `${monthName} ${year}`;
    }

    renderDays() {
        const daysContainer = document.getElementById('calendar-days');
        const firstDay = new Date(this.currentDate.getFullYear(), this.currentDate.getMonth(), 1);
        const lastDay = new Date(this.currentDate.getFullYear(), this.currentDate.getMonth() + 1, 0);
        const startingDayOfWeek = firstDay.getDay();
        const daysInMonth = lastDay.getDate();
        
        let html = '';
        
        // Add day headers
        for (const dayName of this.dayNames) {
            html += `<div class="calendar-day-header">${dayName}</div>`;
        }
        
        // Add empty cells for days before the first day of the month
        for (let i = 0; i < startingDayOfWeek; i++) {
            html += '<div class="calendar-day empty"></div>';
        }
        
        // Add days of the month
        for (let day = 1; day <= daysInMonth; day++) {
            const date = new Date(this.currentDate.getFullYear(), this.currentDate.getMonth(), day);
            const dateString = this.formatDateString(date);
            const classes = this.getDayClasses(date, dateString);
            
            html += `
                <div class="calendar-day ${classes}" 
                     data-date="${dateString}"
                     onclick="window.outlinerApp.calendar.navigateToDate('${dateString}')">
                    ${day}
                </div>
            `;
        }
        
        daysContainer.innerHTML = html;
    }

    getDayClasses(date, dateString) {
        const classes = [];
        
        // Today
        if (this.isSameDay(date, this.today)) {
            classes.push('today');
        }
        
        // Has content
        if (this.pagesWithContent.has(dateString)) {
            classes.push('has-content');
        }
        
        // Weekend
        if (date.getDay() === 0 || date.getDay() === 6) {
            classes.push('weekend');
        }
        
        return classes.join(' ');
    }

    // ================================
    // Navigation
    // ================================

    prevMonth() {
        this.currentDate.setMonth(this.currentDate.getMonth() - 1);
        this.render();
        this.loadPagesWithContent();
    }

    nextMonth() {
        this.currentDate.setMonth(this.currentDate.getMonth() + 1);
        this.render();
        this.loadPagesWithContent();
    }

    async navigateToDate(dateString) {
        try {
            await this.app.loadPage(dateString);
        } catch (error) {
            console.error('Failed to navigate to date:', error);
            this.app.showError('Failed to open date: ' + error.message);
        }
    }

    // ================================
    // Data Loading
    // ================================

    async loadPagesWithContent() {
        try {
            // Get the first and last day of the current month
            const firstDay = new Date(this.currentDate.getFullYear(), this.currentDate.getMonth(), 1);
            const lastDay = new Date(this.currentDate.getFullYear(), this.currentDate.getMonth() + 1, 0);
            
            const startDate = this.formatDateString(firstDay);
            const endDate = this.formatDateString(lastDay);
            
            // Query for pages in this date range
            const response = await this.app.invokeCommand('list_pages', {
                filter: 'daily',
                limit: null
            });
            
            if (response.success) {
                const pages = response.data || [];
                
                // Filter pages for current month and check if they have content
                this.pagesWithContent.clear();
                
                for (const page of pages) {
                    const pageDate = page.path;
                    if (pageDate >= startDate && pageDate <= endDate) {
                        // Check if page has any blocks
                        const blocksResponse = await this.app.invokeCommand('get_page_blocks', { 
                            page_id: page.id 
                        });
                        
                        if (blocksResponse.success && blocksResponse.data && blocksResponse.data.length > 0) {
                            // Check if any blocks have content
                            const hasContent = blocksResponse.data.some(blockData => 
                                blockData.block.content && blockData.block.content.trim() !== ''
                            );
                            
                            if (hasContent) {
                                this.pagesWithContent.add(pageDate);
                            }
                        }
                    }
                }
                
                // Re-render calendar with updated content indicators
                this.renderDays();
            }
        } catch (error) {
            console.error('Failed to load pages with content:', error);
            // Continue without content indicators
        }
    }

    // ================================
    // Utility Methods
    // ================================

    formatDateString(date) {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        return `${year}-${month}-${day}`;
    }

    isSameDay(date1, date2) {
        return date1.getFullYear() === date2.getFullYear() &&
               date1.getMonth() === date2.getMonth() &&
               date1.getDate() === date2.getDate();
    }

    // ================================
    // Quick Navigation
    // ================================

    async goToToday() {
        const today = this.formatDateString(new Date());
        await this.navigateToDate(today);
        
        // Update calendar to show current month
        this.currentDate = new Date();
        this.render();
        this.loadPagesWithContent();
    }

    async goToYesterday() {
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);
        const dateString = this.formatDateString(yesterday);
        await this.navigateToDate(dateString);
    }

    async goToTomorrow() {
        const tomorrow = new Date();
        tomorrow.setDate(tomorrow.getDate() + 1);
        const dateString = this.formatDateString(tomorrow);
        await this.navigateToDate(dateString);
    }

    // ================================
    // Public Interface
    // ================================

    refresh() {
        this.render();
        this.loadPagesWithContent();
    }

    highlightDate(dateString) {
        // Remove previous highlights
        document.querySelectorAll('.calendar-day.current').forEach(day => {
            day.classList.remove('current');
        });
        
        // Add highlight to current date
        const dayElement = document.querySelector(`[data-date="${dateString}"]`);
        if (dayElement) {
            dayElement.classList.add('current');
        }
    }
}

// Add CSS for calendar enhancements
const calendarCSS = `
.calendar-day-header {
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-tertiary);
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
}

.calendar-day.empty {
    pointer-events: none;
    color: var(--text-muted);
}

.calendar-day.weekend {
    color: var(--text-secondary);
}

.calendar-day.current {
    background-color: var(--accent-blue);
    color: white;
    font-weight: 600;
}

.calendar-day.has-tasks::before {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 4px;
    height: 4px;
    background-color: var(--accent-orange);
    border-radius: 50%;
}
`;

// Inject CSS
const style = document.createElement('style');
style.textContent = calendarCSS;
document.head.appendChild(style);

// Export for module usage
export default CalendarComponent;
