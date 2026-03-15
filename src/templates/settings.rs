use maud::{html, Markup};

use super::layout::layout_with_nav;

pub fn settings_page() -> Markup {
    let content = html! {
        section.detail-section {
            h2 { "Settings" }
        }

        section.detail-section {
            h2 { "Data Management" }
            p.settings-danger-desc {
                "Re-parse all seeds from stored HTML. Useful after parser improvements to update growing details."
            }
            button.btn #reparse-btn
                   hx-post="/seeds/reparse"
                   hx-confirm="Re-parse all seeds from stored HTML?"
                   hx-target="#reparse-result"
                   hx-swap="innerHTML"
                   hx-indicator="#reparse-btn"
            {
                "Re-parse All Seeds"
            }
            div #reparse-result {}
        }

        section.detail-section {
            h2 { "Danger Zone" }
            p.settings-danger-desc {
                "This will permanently delete all seeds, purchases, images, and plans. This cannot be undone."
            }
            button.btn.btn-delete #reset-all-btn
                   hx-post="/settings/reset-all-data"
                   hx-confirm="Are you sure you want to delete ALL data? This cannot be undone."
                   hx-target="#reset-result"
                   hx-swap="innerHTML"
            {
                "Reset All Data"
            }
            div #reset-result {}
        }
    };

    layout_with_nav("Settings", "settings", content)
}

pub fn reparse_success(count: u64) -> Markup {
    html! {
        p.reset-success { "Re-parsed " (count) " seeds successfully." }
    }
}

pub fn reset_success() -> Markup {
    html! {
        p.reset-success { "All data has been reset successfully." }
        script { "setTimeout(()=>window.location.href='/',1500)" }
    }
}
