use maud::{html, Markup};

use super::layout::layout_with_nav;

pub fn settings_page() -> Markup {
    let content = html! {
        section.detail-section {
            h2 { "Settings" }
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

pub fn reset_success() -> Markup {
    html! {
        p.reset-success { "All data has been reset successfully." }
        script { "setTimeout(()=>window.location.href='/',1500)" }
    }
}
