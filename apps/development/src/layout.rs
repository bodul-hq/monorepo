//! Page shell shared by the home page and the dashboard: head, top bar and theme toggle.

/// The top-bar link marked as the current page.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    Home,
    Dashboard,
}

/// Applies a saved theme before the first paint so the page does not flash.
/// Without a saved choice, the stylesheet follows the system preference.
const THEME_INIT: &str = r#"<script>(function(){var t;try{t=localStorage.getItem("theme")}catch(e){}if(t==="light"||t==="dark")document.documentElement.dataset.theme=t})()</script>"#;

/// Reveals the toggle, which needs JavaScript, and stores the visitor's choice.
const THEME_TOGGLE: &str = r#"<script>(function(){var root=document.documentElement,group=document.querySelector(".theme"),buttons=group.querySelectorAll("button");function show(){var t=root.dataset.theme||"system";buttons.forEach(function(b){b.setAttribute("aria-pressed",String(b.dataset.themeChoice===t))})}buttons.forEach(function(b){b.addEventListener("click",function(){var t=b.dataset.themeChoice;if(t==="system")delete root.dataset.theme;else root.dataset.theme=t;try{if(t==="system")localStorage.removeItem("theme");else localStorage.setItem("theme",t)}catch(e){}show()})});show();group.hidden=false})()</script>"#;

/// Returns the document up to and including the top bar; close it with [`end`].
pub(crate) fn start(title: &str, active: Page) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>"#,
    );
    html.push_str(title);
    html.push_str("</title>");
    html.push_str(THEME_INIT);
    html.push_str("<style>");
    html.push_str(include_str!("style.css"));
    html.push_str(r#"</style></head><body><header class="topbar"><a class="brand" href="/"><span class="brand-mark" aria-hidden="true">b.</span> <span class="brand-name">Development</span> <span class="workspace">WORKSPACE</span></a><div class="topbar-end"><nav aria-label="Main">"#);
    for (page, href, label) in [
        (Page::Home, "/", "Home"),
        (Page::Dashboard, "/dashboard", "Dashboard"),
    ] {
        if page == active {
            html.push_str(&format!(
                r#"<a class="active" href="{href}" aria-current="page">{label}</a>"#
            ));
        } else {
            html.push_str(&format!(r#"<a href="{href}">{label}</a>"#));
        }
    }
    html.push_str(r#"</nav><div class="theme" role="group" aria-label="Color theme" hidden><button type="button" data-theme-choice="light" aria-label="Light theme" title="Light">☀</button><button type="button" data-theme-choice="dark" aria-label="Dark theme" title="Dark">☾</button><button type="button" data-theme-choice="system" aria-label="Use system theme" title="Use system theme">Auto</button></div></div></header>"#);
    html
}

/// Closes a document opened with [`start`], adding the theme toggle's script.
pub(crate) fn end(html: &mut String) {
    html.push_str(THEME_TOGGLE);
    html.push_str("</body></html>");
}
