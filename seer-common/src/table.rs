//! Thin wrappers around lariv-rs data tables for Seer list UIs.

use lariv_rs::components::{
    ButtonLink, FieldLink, FieldText, HtmlAttrs, SwapKey, TableColumnHeader, TableRow, button_link,
    data_table_list, data_table_list_refresh, field_link, field_text, row_attr_navigate,
};
use lariv_rs::http::RouteUrl;
use maud::{html, Markup};

/// Column header without sort links (Seer lists are not yet sortable).
pub fn col(key: &'static str, label: &'static str) -> TableColumnHeader<'static> {
    TableColumnHeader {
        key,
        label,
        sort_url: None,
        push_url: false,
    }
}

/// Plain text cell.
pub fn cell(value: &str) -> Markup {
    field_text(FieldText {
        value,
        classes: "",
    })
}

/// Link cell (HTMX app-layout nav for in-app hrefs).
pub fn cell_link(href: &str, label: &str) -> Markup {
    field_link(FieldLink {
        href,
        label,
        classes: "",
    })
}

/// Non-interactive row.
pub fn row(cells: Vec<Markup>) -> TableRow {
    TableRow {
        attrs: HtmlAttrs::default(),
        cells,
    }
}

/// Clickable row that navigates to `href`.
pub fn row_nav(href: &str, cells: Vec<Markup>) -> TableRow {
    TableRow {
        attrs: row_attr_navigate(href),
        cells,
    }
}

/// Table toolbar create control: square outline button with a plus icon (lariv table style).
pub fn seer_table_create(route: impl RouteUrl) -> Markup {
    let href = route.url();
    button_link(ButtonLink {
        label: "",
        href: &href,
        icon_name: Some("plus"),
        classes: "btn-square btn-outline btn-sm",
        ..Default::default()
    })
}

/// Standard List/Grid data table keyed by a [`SwapKey`].
pub fn seer_table<K: SwapKey>(
    title: &str,
    headers: &[TableColumnHeader<'_>],
    rows: &[TableRow],
) -> Markup {
    data_table_list::<K>(title, html! {}, headers, rows, html! {})
}

/// Like [`seer_table`] with a toolbar actions slot (e.g. Create button).
pub fn seer_table_with_actions<K: SwapKey>(
    title: &str,
    actions: Markup,
    headers: &[TableColumnHeader<'_>],
    rows: &[TableRow],
) -> Markup {
    data_table_list::<K>(title, actions, headers, rows, html! {})
}

/// Like [`seer_table_with_actions`], listening for create-modal refresh on `refresh_url`.
///
/// Pass the list page `path_and_query` so the table re-GETs itself after
/// [`lariv_rs::web::respond_create_modal_done`].
pub fn seer_table_with_actions_refresh<K: SwapKey>(
    title: &str,
    actions: Markup,
    headers: &[TableColumnHeader<'_>],
    rows: &[TableRow],
    refresh_url: &str,
) -> Markup {
    data_table_list_refresh::<K>(title, actions, headers, rows, html! {}, refresh_url)
}

/// Like [`seer_table`] with a subtitle under the title.
pub fn seer_table_with_subtitle<K: SwapKey>(
    title: &str,
    subtitle: &str,
    headers: &[TableColumnHeader<'_>],
    rows: &[TableRow],
) -> Markup {
    lariv_rs::components::data_table_list_with_subtitle::<K>(
        title,
        subtitle,
        html! {},
        headers,
        rows,
        html! {},
    )
}
