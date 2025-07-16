use gio::ListStore;
use gtk4::{Align::Fill, Box, ColumnView, ColumnViewColumn, CssProvider, Label, MultiSelection, Orientation::Horizontal, SignalListItemFactory, StringObject, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4::pango::EllipsizeMode::End;
use libadwaita::prelude::{BoxExt, CastNone, ListItemExt, StyleContextExt, WidgetExt};

use crate::utils::constants::DR_COLORS;

/// Creates and configures a `ColumnView` for displaying file analysis results.
///
/// This function initializes a three-column layout:
/// 1. File Name (fixed width)
/// 2. Full Path (expandable)
/// 3. DR Value (fixed width)
///
/// This setup ensures that the "Path" column is the only one that grows or shrinks
/// with the window size, preventing horizontal scrolling. User resizing of columns
/// is disabled to maintain a stable layout.
pub fn create_column_view() -> (ColumnView, ListStore, MultiSelection) {
    let list_store = ListStore::new::<StringObject>();
    let selection_model = MultiSelection::new(Some(list_store.clone()));
    let column_view = ColumnView::new(Some(selection_model.clone()));
    
    // Configure the appearance and behavior of the column view.
    column_view.set_show_row_separators(true);
    column_view.set_enable_rubberband(true);
    column_view.set_hexpand(true);
    column_view.set_vexpand(true);
    column_view.set_valign(Fill);

    // Add the columns to the view.
    add_column(&column_view, "File Name", Some(250), false, |text| text.split('\t').next().unwrap_or(""));
    add_column(&column_view, "Path", None, true, |text| text.split('\t').nth(1).unwrap_or(""));
    add_dr_column(&column_view);

    (column_view, list_store, selection_model)
}

/// Adds a generic text column to the `ColumnView`.
///
/// This helper function creates a column with a specified `title` and uses a
/// `text_extractor` closure to determine which part of the row's data string to display.
/// The cell's label is configured to ellipsize long text.
pub fn add_column(column_view: &ColumnView, title: &str, fixed_width: Option<i32>, expand: bool, text_extractor: impl Fn(&str) -> &str + 'static) {
    let factory = SignalListItemFactory::new();
    
    // The setup handler creates the label widget for the cell.
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        label.set_xalign(0.0); // Left-align text.
        label.set_margin_start(5);
        label.set_margin_end(5);
        label.set_ellipsize(End); // Truncate long text.
        list_item.set_child(Some(&label));
    });

    // The bind handler updates the label's text when the cell is bound to data.
    factory.connect_bind(move |_, list_item| {
        let string_object = list_item.item().and_downcast::<StringObject>().unwrap();
        let label = list_item.child().and_downcast::<Label>().unwrap();
        let text = string_object.string();
        label.set_text(text_extractor(&text));
    });

    let column = ColumnViewColumn::new(Some(title), Some(factory));
    column.set_resizable(false); // Disable user resizing for this column.
    column.set_expand(expand);
    if let Some(width) = fixed_width {
        column.set_fixed_width(width);
    }
    column_view.append_column(&column);
}

/// Adds the specialized "DR Value" column to the `ColumnView`.
///
/// This column displays the DR value and a colored box next to it. The color of the box
/// - `PENDING` is gray.                                                                                                                                                   │                                                                                                                                       │
/// - `ERR` is dark gray.                                                                                                                                                  │                                                                                                                                                                   │
/// - Numeric values are colored based on the `DR_COLORS` constant.
/// is determined by the DR value, providing a quick visual indicator of the audio quality.
pub fn add_dr_column(column_view: &ColumnView) {
    let factory = SignalListItemFactory::new();
    
    // The setup handler creates a horizontal box containing the color indicator and the label.
    factory.connect_setup(move |_, list_item| {
        let hbox = Box::new(Horizontal, 5);
        let color_box = Box::new(Horizontal, 0);
        let label = Label::new(None);
        
        color_box.set_size_request(16, 16);
        color_box.add_css_class("color-box");
        
        hbox.append(&color_box);
        hbox.append(&label);
        hbox.set_spacing(5);
        list_item.set_child(Some(&hbox));
    });

    // The bind handler updates the label text and the color of the indicator box.
    factory.connect_bind(move |_, list_item| {
        let string_object = list_item.item().and_downcast::<StringObject>().unwrap();
        let hbox = list_item.child().and_downcast::<Box>().unwrap();
        let color_box = hbox.first_child().and_downcast::<Box>().unwrap();
        let label = hbox.last_child().and_downcast::<Label>().unwrap();

        let text = string_object.string();
        let dr_text = text.split('\t').nth(2).unwrap_or("PENDING");
        label.set_text(dr_text);
        
        // Determine the color based on the DR text.
        let (r, g, b) = match dr_text {
            "PENDING" => (180, 180, 180),
            "ERR" => (128, 128, 128),
            _ => dr_text.parse::<u8>()
                .map(|dr| {
                    if dr >= DR_COLORS.len() as u8 { // Handle DR values outside the defined color range.
                        DR_COLORS[DR_COLORS.len() - 1]
                    } else {
                        DR_COLORS[dr as usize]
                    }
                })
                .unwrap_or((128, 128, 128)) // Default to error color on parse failure.
        };
        
        // Apply the color to the indicator box using a dynamic CSS provider.
        let css = format!("box.color-box {{ background-color: rgb({}, {}, {}); }}", r, g, b);
        let css_provider = CssProvider::new();
        css_provider.load_from_data(&css);
        color_box.style_context().add_provider(&css_provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    });

    let column = ColumnViewColumn::new(Some("DR Value"), Some(factory));
    column.set_resizable(false); // Disable user resizing for this column.
    column.set_expand(false);
    column.set_fixed_width(120);
    column_view.append_column(&column);
}