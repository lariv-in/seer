//! Shared shell scaffold helpers for Seer UI plugins.

use maud::Markup;

use lariv_rs::components::{
    Crumb, LayoutMain, LayoutSidebar, ShellChrome, ShellScaffold, SidebarMenu, SidebarMenuItem,
    breadcrumbs, layout_main, layout_sidebar, shell_scaffold, sidebar_menu, sidebar_menu_item_pane,
};

/// Full-page shell with topbar, left sidebar, breadcrumbs, and body.
pub fn app_scaffold(
    title: &str,
    chrome: &ShellChrome,
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        breadcrumbs: crumbs,
        body,
        ..Default::default()
    })
}

/// HTMX `#app-layout` swap with sidebar + breadcrumbs + content.
pub fn scaffold_pane(
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> lariv_rs::components::AppLayoutHtml {
    layout_sidebar(LayoutSidebar {
        sidebar,
        breadcrumbs: crumbs,
        content: body,
    })
}

/// HTMX `#main-content` swap with breadcrumbs + content.
pub fn scaffold_main(crumbs: Markup, body: Markup) -> lariv_rs::components::MainContentHtml {
    layout_main(LayoutMain {
        breadcrumbs: crumbs,
        content: body,
    })
}

/// Single-segment breadcrumb for list / home pages.
pub fn list_crumbs(label: &'static str) -> Markup {
    breadcrumbs(&[Crumb {
        label,
        href: None,
    }])
}

/// List → detail breadcrumb trail.
pub fn detail_crumbs(list_label: &'static str, list_url: &str, detail_label: &str) -> Markup {
    breadcrumbs(&[
        Crumb {
            label: list_label,
            href: Some(list_url),
        },
        Crumb {
            label: detail_label,
            href: None,
        },
    ])
}

/// Pane-swapping sidebar nav item.
pub fn menu_item(title: &str, url: &str, active: bool) -> Markup {
    sidebar_menu_item_pane(SidebarMenuItem {
        title,
        url,
        active,
        ..Default::default()
    })
}

/// App sidebar with plugin nav items.
pub fn seer_menu(title: &str, items: Markup) -> Markup {
    sidebar_menu(SidebarMenu {
        title,
        children: items,
    })
}

/// Define a page struct that renders inside the Seer shell scaffold.
///
/// `$menu` and `$crumbs` are expressions evaluated in each render path
/// (typically `plugin_menu("key")` and `list_crumbs("Label")`).
#[macro_export]
macro_rules! seer_scaffold_page {
    ($name:ident, $page_title:expr, $menu:expr, $crumbs:expr) => {
        #[derive(::frunk::Generic)]
        pub struct $name {
            pub body: ::maud::Markup,
        }

        impl $name {
            fn pane_body(&self) -> ::maud::Markup {
                self.body.clone()
            }
        }

        impl ::lariv_rs::template::RenderAppPane for $name {
            fn render_pane(&self) -> ::lariv_rs::components::AppLayoutHtml {
                ::seer_common::templates::scaffold_pane($menu, $crumbs, self.pane_body())
            }

            fn render_main(&self) -> ::lariv_rs::components::MainContentHtml {
                ::seer_common::templates::scaffold_main($crumbs, self.pane_body())
            }
        }

        impl ::lariv_rs::template::RenderTemplate for $name {
            fn render(
                &self,
                chrome: &::lariv_rs::components::ShellChrome,
            ) -> ::maud::Markup {
                ::seer_common::templates::app_scaffold(
                    $page_title,
                    chrome,
                    $menu,
                    $crumbs,
                    self.pane_body(),
                )
            }
        }
    };
}
