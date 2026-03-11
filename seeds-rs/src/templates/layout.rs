use maud::{html, Markup, DOCTYPE};

pub fn layout(title: &str, content: Markup) -> Markup {
    layout_with_nav(title, "", content)
}

pub fn layout_with_nav(title: &str, active_nav: &str, content: Markup) -> Markup {
    // Determine which nav item is active; default to "seeds" if not specified
    let active = if active_nav.is_empty() { "seeds" } else { active_nav };

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " | Seeds" }
                link rel="stylesheet" href="/static/style.css";
                script src="/static/htmx.min.js" {}
                script {
                    "htmx.config.responseHandling = [\
                        {code:'204', swap: false},\
                        {code:'[23]..', swap: true},\
                        {code:'[45]..', swap: true, error: true}\
                    ];"
                }
            }
            body {
                header.app-header {
                    h1.logo { "Seeds" }
                    nav.main-nav {
                        a.nav-link href="/"
                            class=@if active == "seeds" { "active" } { "Seeds" }
                        a.nav-link href="/schedule"
                            class=@if active == "schedule" { "active" } { "Schedule" }
                        a.nav-link href="/settings"
                            class=@if active == "settings" { "active" } { "Settings" }
                    }
                }
                main.content {
                    (content)
                }
            }
        }
    }
}
