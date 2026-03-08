use maud::{html, Markup, DOCTYPE};

pub fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " | Seeds" }
                link rel="stylesheet" href="/static/style.css";
                script src="/static/htmx.min.js" {}
            }
            body {
                header.app-header {
                    h1.logo { "Seeds" }
                    nav.main-nav {
                        a.nav-link.active href="/" { "Seeds" }
                        span.nav-link.disabled { "Inventory" }
                        span.nav-link.disabled { "Schedule" }
                    }
                }
                main.content {
                    (content)
                }
            }
        }
    }
}
