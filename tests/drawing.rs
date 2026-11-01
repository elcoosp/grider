//! Tests for the drawing module and the debug overlay helpers.

use grider::drawing::{Drawable, GridDrawingConfig, GridDrawingConfigBuilder};
use grider::testing::{Pattern, test_image};
use grider::{Cell, Column, Grid, LineInfo, LineKind, LineTrait, Row};
use image::{Rgba, RgbaImage};

fn sample_grid() -> Grid {
    grider::make_grid!(
        rows: [(0, 5), (5, 5, LineKind::Full)],
        columns: [(0, 5), (5, 5, LineKind::Full)]
    )
}

#[test]
fn default_config_values() {
    let config = GridDrawingConfig::default();
    assert_eq!(config.padding, 0);
    assert_eq!(config.row_color, Rgba([255, 0, 0, 255]));
    assert_eq!(config.column_color, Rgba([0, 0, 255, 255]));
    assert_eq!(config.cell_background_color, Rgba([200, 200, 200, 255]));
    assert_eq!(config.line_thickness, 2);
    assert!(config.row_color_provider.is_none());
    assert!(config.column_color_provider.is_none());
}

#[test]
fn builder_overrides_fields() {
    let config = GridDrawingConfig::builder()
        .padding(2)
        .row_color(Rgba([1, 2, 3, 4]))
        .column_color(Rgba([5, 6, 7, 8]))
        .cell_background_color(Rgba([9, 10, 11, 12]))
        .line_thickness(4)
        .build();
    assert_eq!(config.padding, 2);
    assert_eq!(config.row_color, Rgba([1, 2, 3, 4]));
    assert_eq!(config.column_color, Rgba([5, 6, 7, 8]));
    assert_eq!(config.cell_background_color, Rgba([9, 10, 11, 12]));
    assert_eq!(config.line_thickness, 4);
}

#[test]
fn builder_clamps_thickness_to_at_least_one() {
    let config = GridDrawingConfig::builder().line_thickness(0).build();
    assert_eq!(config.line_thickness, 1);
}

#[test]
fn builder_default_matches_config_default() {
    assert_eq!(
        GridDrawingConfigBuilder::default().build().padding,
        GridDrawingConfig::default().padding
    );
}

#[test]
fn builder_sets_color_providers() {
    let config = GridDrawingConfig::builder()
        .row_color_provider(Box::new(|index| {
            Rgba([u8::try_from(index).unwrap_or(u8::MAX), 1, 1, 255])
        }))
        .column_color_provider(Box::new(|index| {
            Rgba([1, u8::try_from(index).unwrap_or(u8::MAX), 1, 255])
        }))
        .build();
    assert_eq!(config.row_color_at(2), Rgba([2, 1, 1, 255]));
    assert_eq!(config.column_color_at(5), Rgba([1, 5, 1, 255]));
}

#[test]
fn debug_hides_provider_functions() {
    let config = GridDrawingConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GridDrawingConfig"));
    assert!(debug.contains("<function>"));
}

#[test]
fn color_providers_take_precedence() {
    let config = GridDrawingConfig {
        row_color_provider: Some(Box::new(|index| {
            Rgba([u8::try_from(index).unwrap_or(u8::MAX), 0, 0, 255])
        })),
        column_color_provider: Some(Box::new(|index| {
            Rgba([0, u8::try_from(index).unwrap_or(u8::MAX), 0, 255])
        })),
        ..GridDrawingConfig::default()
    };
    assert_eq!(config.row_color_at(0), Rgba([0, 0, 0, 255]));
    assert_eq!(config.row_color_at(3), Rgba([3, 0, 0, 255]));
    assert_eq!(config.column_color_at(2), Rgba([0, 2, 0, 255]));

    let plain = GridDrawingConfig::default();
    assert_eq!(plain.row_color_at(9), plain.row_color);
    assert_eq!(plain.column_color_at(9), plain.column_color);
}

#[test]
fn cell_draw_fills_the_background() {
    let mut image = RgbaImage::new(10, 10);
    let row = Row::new(LineInfo::new(2, 5, LineKind::Full));
    let column = Column::new(LineInfo::new(3, 4, LineKind::Full));
    let cell = Cell {
        row: &row,
        column: &column,
    };
    let config = GridDrawingConfig {
        cell_background_color: Rgba([1, 1, 1, 255]),
        ..GridDrawingConfig::default()
    };
    cell.draw(&mut image, &config);
    assert_eq!(image.get_pixel(4, 3), &Rgba([1, 1, 1, 255]));
}

#[test]
fn cell_draw_skips_fully_padded_cells() {
    let mut image = RgbaImage::new(10, 10);
    let row = Row::new(LineInfo::new(2, 2, LineKind::Full));
    let column = Column::new(LineInfo::new(2, 2, LineKind::Full));
    let cell = Cell {
        row: &row,
        column: &column,
    };
    let config = GridDrawingConfig {
        padding: 10,
        cell_background_color: Rgba([1, 1, 1, 255]),
        ..GridDrawingConfig::default()
    };
    cell.draw(&mut image, &config);
    assert_eq!(image.get_pixel(2, 2), &Rgba([0, 0, 0, 0]));
}

#[test]
fn row_and_column_draw_their_edges() {
    let mut image = RgbaImage::new(10, 10);
    let row = Row::new(LineInfo::new(0, 5, LineKind::Full));
    let column = Column::new(LineInfo::new(0, 5, LineKind::Full));
    row.draw(&mut image, &GridDrawingConfig::default());
    column.draw(&mut image, &GridDrawingConfig::default());
    // Row end at y=5 with thickness 2 covers y in [4, 6) — red row color.
    assert_eq!(image.get_pixel(7, 5), &Rgba([255, 0, 0, 255]));
    // Column end at x=5 with thickness 2 covers x in [4, 6) — blue column color.
    assert_eq!(image.get_pixel(5, 7), &Rgba([0, 0, 255, 255]));
}

#[test]
fn grid_draw_paints_cells_and_lines() {
    let mut image = RgbaImage::new(12, 12);
    sample_grid().draw(&mut image, &GridDrawingConfig::default());
    // Bottom row band (y=10, thickness 2) must be red.
    assert_eq!(image.get_pixel(11, 10), &Rgba([255, 0, 0, 255]));
    // Right column band (x=10, thickness 2) must be blue.
    assert_eq!(image.get_pixel(10, 11), &Rgba([0, 0, 255, 255]));
    // Cell background (top-left cell interior) is light gray.
    assert_eq!(image.get_pixel(2, 2), &Rgba([200, 200, 200, 255]));
}

#[test]
fn grid_draw_with_providers() {
    let config = GridDrawingConfig {
        row_color_provider: Some(Box::new(|index| {
            Rgba([u8::try_from(index).unwrap_or(u8::MAX), 0, 0, 255])
        })),
        column_color_provider: Some(Box::new(|_index| Rgba([0, 9, 9, 255]))),
        ..GridDrawingConfig::default()
    };
    let mut image = RgbaImage::new(12, 12);
    sample_grid().draw(&mut image, &config);
    assert_eq!(image.get_pixel(11, 10), &Rgba([1, 0, 0, 255]));
    assert_eq!(image.get_pixel(10, 11), &Rgba([0, 9, 9, 255]));
}

#[test]
fn grid_draw_with_zero_thickness_never_panics_on_edges() {
    let config = GridDrawingConfig::builder().line_thickness(1).build();
    let mut image = RgbaImage::new(10, 10);
    let grid = grider::make_grid!(
        rows: [(0, 10, LineKind::Full)],
        columns: [(0, 10, LineKind::Full)]
    );
    grid.draw(&mut image, &config);
}

#[test]
fn drawing_on_empty_grid_is_a_noop() {
    let mut image = RgbaImage::new(4, 4);
    let grid = Grid {
        rows: [].into_iter().collect(),
        columns: [].into_iter().collect(),
    };
    grid.draw(&mut image, &GridDrawingConfig::default());
}

#[test]
fn save_image_with_grid_writes_a_file() {
    let img = test_image(10, 10, Pattern::Checkerboard);
    let grid = sample_grid();
    let output_path = std::env::temp_dir().join("grider_test_overlay.png");
    let output_str = output_path.to_string_lossy().to_string();
    grider::debug::save_image_with_grid(&img, &grid, &output_str, &GridDrawingConfig::default())
        .unwrap();
    assert!(output_path.exists());
    std::fs::remove_file(&output_path).unwrap();
}

#[test]
fn save_image_with_grid_reports_io_errors() {
    let img = test_image(10, 10, Pattern::Empty);
    let grid = sample_grid();
    let result = grider::debug::save_image_with_grid(
        &img,
        &grid,
        "/nonexistent-directory/grider.png",
        &GridDrawingConfig::default(),
    );
    assert!(result.is_err());
}
