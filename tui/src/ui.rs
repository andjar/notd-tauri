mod layout;
mod widgets;

pub use layout::render;
pub use widgets::{
    render_header,
    render_outline,
    render_page_switcher,
    render_status_bar,
    render_sidebar_pages,
    render_search_overlay,
    render_sidebar_tags_and_pages,
    render_backlinks_panel,
};

