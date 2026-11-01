//! Tests for [`Grid`] queries, filtering and construction from images.

use grider::detection::SmallVecLine;
use grider::testing::{Pattern, test_image};
use grider::{
    Cell, Column, Grid, GridConfig, GridError, LineInfo, LineKind, LineTrait, Row, clamp_to_i32,
};
use image::DynamicImage;
use test_case::test_case;

fn grid_with_rows(heights: &[u32]) -> Grid {
    let rows: SmallVecLine<Row> = heights
        .iter()
        .scan(0u32, |y, &height| {
            let row = Row::new(LineInfo::new(*y, height, LineKind::Full));
            *y += height;
            Some(row)
        })
        .collect();
    Grid {
        rows,
        columns: SmallVecLine::from_vec(vec![Column::new(LineInfo::new(0, 1, LineKind::Full))]),
    }
}

fn grid_with_columns(widths: &[u32]) -> Grid {
    let columns: SmallVecLine<Column> = widths
        .iter()
        .scan(0u32, |x, &width| {
            let column = Column::new(LineInfo::new(*x, width, LineKind::Full));
            *x += width;
            Some(column)
        })
        .collect();
    Grid {
        rows: SmallVecLine::from_vec(vec![Row::new(LineInfo::new(0, 1, LineKind::Full))]),
        columns,
    }
}

#[cfg(feature = "serde")]
fn half_and_half_image() -> DynamicImage {
    let mut gray = grider::testing::gray_test_image(10, 10, Pattern::Empty);
    // Recreate the canonical 10x10 "top white, bottom black" fixture.
    for y in 5..10 {
        for x in 0..10 {
            gray.put_pixel(x, y, image::Luma([0]));
        }
    }
    DynamicImage::ImageLuma8(gray)
}

#[test_case(0, 100)]
#[test_case(100, 0)]
fn invalid_dimensions_are_rejected(width: u32, height: u32) {
    let dynamic = DynamicImage::ImageLuma8(image::GrayImage::new(width, height));
    let config = GridConfig::default();

    let result = Grid::try_from_image_with_config(&dynamic, config);
    assert_eq!(result, Err(GridError::InvalidDimensions { width, height }));
}

mod construction {
    use super::*;

    #[cfg(feature = "serde")]
    use insta::assert_yaml_snapshot;

    #[test]
    fn try_from_image_with_config_succeeds() {
        let img = test_image(100, 100, Pattern::Checkerboard);
        let config = GridConfig {
            threshold_block_size: 8,
            merge_threshold_ratio: 0.7,
            threshold_delta: 0,
            enable_parallel: true,
        };
        let result = Grid::try_from_image_with_config(&img, config);
        assert!(result.is_ok());
    }

    #[test]
    fn try_from_traits_cover_reference_and_value() {
        let img = test_image(10, 10, Pattern::Empty);
        let grid_from_ref = Grid::try_from(&img).unwrap();
        let grid_from_value = Grid::try_from(img.clone()).unwrap();
        assert_eq!(grid_from_ref, grid_from_value);

        let grid_from_try_into: Result<Grid, GridError> = img.try_into();
        assert!(grid_from_try_into.is_ok());
    }

    #[test]
    fn invalid_config_is_rejected() {
        let img = test_image(10, 10, Pattern::Empty);
        let config = GridConfig {
            threshold_block_size: 0,
            ..GridConfig::default()
        };
        assert_eq!(
            Grid::try_from_image_with_config(&img, config),
            Err(GridError::invalid_config(
                "threshold_block_size must be greater than 0"
            ))
        );
    }

    #[test]
    fn parallel_and_sequential_agree() {
        let img = test_image(64, 64, Pattern::Checkerboard);
        let parallel = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        let sequential = Grid::try_from_image_with_config(
            &img,
            GridConfig {
                enable_parallel: false,
                ..GridConfig::default()
            },
        )
        .unwrap();
        assert_eq!(parallel, sequential);
    }

    #[test]
    fn minimal_images_are_supported() {
        let img = test_image(1, 1, Pattern::Empty);
        let config = GridConfig {
            threshold_block_size: 1,
            ..GridConfig::default()
        };
        assert!(Grid::try_from_image_with_config(&img, config).is_ok());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_process_image() {
        let img = half_and_half_image();
        let grid: Grid = (&img).try_into().unwrap();
        assert_yaml_snapshot!("process_image", grid);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_process_image_with_redactions() {
        let img = half_and_half_image();
        let grid: Grid = (&img).try_into().unwrap();

        let expected_grid = grider::make_grid! {
            rows: [
                (0, 5),
                (5, 5, LineKind::Full),
            ],
            columns: [
                (0, 10, LineKind::Full),
            ]
        };
        assert_eq!(grid, expected_grid);
        assert_yaml_snapshot!("process_image_with_redactions", grid, {
            ".rows[0].y" => 0,
            ".rows[1].y" => 5,
        });
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_grid_processing_small_image() {
        let img = image::GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255])
            .unwrap();
        let dynamic_img = image::DynamicImage::ImageLuma8(img);
        let config = GridConfig {
            threshold_block_size: 3,
            merge_threshold_ratio: 0.5,
            threshold_delta: 0,
            enable_parallel: false,
        };
        let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();
        assert_yaml_snapshot!("grid_processing_small_image", grid);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_grid_processing_large_image() {
        let img = half_and_half_image();
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        assert_yaml_snapshot!("grid_processing_large_image", grid);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_grid_with_custom_config() {
        // The quadrant fixture of the original test suite.
        let img = DynamicImage::ImageLuma8(image::GrayImage::from_fn(20, 20, |x, y| {
            if (x < 10 && y < 10) || (x >= 10 && y >= 10) {
                image::Luma([0])
            } else {
                image::Luma([255])
            }
        }));

        let configs = [
            GridConfig {
                threshold_block_size: 5,
                merge_threshold_ratio: 0.3,
                threshold_delta: 0,
                enable_parallel: false,
            },
            GridConfig {
                threshold_block_size: 10,
                merge_threshold_ratio: 0.8,
                threshold_delta: 0,
                enable_parallel: true,
            },
        ];

        for (index, config) in configs.iter().enumerate() {
            let grid = Grid::try_from_image_with_config(&img, *config).unwrap();
            assert_yaml_snapshot!(format!("grid_custom_config_{index}"), grid);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_grid_with_redactions() {
        let img = half_and_half_image();
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        assert_yaml_snapshot!("grid_with_redactions", grid, {
            ".rows[0].y" => 0,
            ".rows[1].y" => 5,
        });
    }
}

mod queries {
    use super::*;

    #[test]
    fn iterators_yield_all_lines() {
        let grid = grid_with_rows(&[2, 3, 4]);
        assert_eq!(grid.rows().count(), 3);
        assert_eq!(grid.columns().count(), 1);
        assert_eq!(grid.rows().map(Row::end).collect::<Vec<_>>(), vec![2, 5, 9]);
    }

    #[test]
    fn slices_counts_and_empty() {
        let grid = grid_with_rows(&[2, 3]);
        assert_eq!(grid.row_count(), 2);
        assert_eq!(grid.column_count(), 1);
        assert_eq!(grid.rows_slice().len(), 2);
        assert_eq!(grid.columns_slice().len(), 1);
        assert!(!grid.is_empty());
    }

    #[test]
    fn filtered_rows_and_columns() {
        let img = test_image(10, 10, Pattern::Checkerboard);
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        let filtered: Vec<_> = grid
            .filtered_rows(|row| row.kind == LineKind::Full)
            .collect();
        assert!(!filtered.is_empty(), "expected full rows");
        assert!(filtered.iter().all(|row| row.kind == LineKind::Full));

        let filtered: Vec<_> = grid
            .filtered_columns(|column| column.kind == LineKind::Full)
            .collect();
        assert!(!filtered.is_empty(), "expected full columns");
        assert!(filtered.iter().all(|column| column.kind == LineKind::Full));
    }

    #[test]
    fn filtered_rows_empty_when_no_full_rows() {
        let img = test_image(10, 10, Pattern::Empty);
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        let filtered: Vec<_> = grid
            .filtered_rows(|row| row.kind == LineKind::Full)
            .collect();
        assert_eq!(filtered, Vec::<&Row>::new());

        let filtered: Vec<_> = grid
            .filtered_columns(|column| column.kind == LineKind::Full)
            .collect();
        assert_eq!(filtered, Vec::<&Column>::new());
    }

    #[test]
    fn rows_and_columns_by_kind() {
        let img = test_image(10, 10, Pattern::Checkerboard);
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        assert_eq!(
            grid.rows_by_kind(LineKind::Full).count(),
            grid.count_rows_by_kind(LineKind::Full)
        );
        assert!(
            grid.rows_by_kind(LineKind::Full)
                .all(|row| row.kind == LineKind::Full)
        );
        assert_eq!(
            grid.columns_by_kind(LineKind::Empty).count(),
            grid.count_columns_by_kind(LineKind::Empty)
        );
    }

    #[test]
    fn count_by_kind_matches_manual_count() {
        let img = test_image(10, 10, Pattern::Checkerboard);
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        let manual_full_rows = grid.rows().filter(|row| row.kind.is_full()).count();
        assert_eq!(grid.count_rows_by_kind(LineKind::Full), manual_full_rows);
        let manual_full_columns = grid
            .columns()
            .filter(|column| column.kind.is_full())
            .count();
        assert_eq!(
            grid.count_columns_by_kind(LineKind::Full),
            manual_full_columns
        );
    }

    #[test]
    fn find_row_and_column() {
        let grid = grid_with_rows(&[5, 5]);
        assert!(grid.find_row(0).is_some());
        assert!(grid.find_row(5).is_some());
        assert!(grid.find_row(10).is_none());

        let grid = grid_with_columns(&[5, 5]);
        assert!(grid.find_column(0).is_some());
        assert!(grid.find_column(10).is_none());
    }

    #[test]
    fn cell_returns_indexed_intersections() {
        let grid = grider::make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full)],
            columns: [(0, 2), (2, 3, LineKind::Full)]
        );
        let cell = grid.cell(0, 0).unwrap();
        assert_eq!(cell.row.y, 0);
        assert_eq!(cell.column.x, 0);
        let cell = grid.cell(1, 1).unwrap();
        assert_eq!(cell.row.y, 5);
        assert_eq!(cell.column.x, 2);
        assert!(grid.cell(2, 0).is_none());
        assert!(grid.cell(0, 2).is_none());
    }

    #[test]
    fn iter_cells_covers_row_major_grid() {
        let grid = grider::make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full)],
            columns: [(0, 2), (2, 3, LineKind::Full)]
        );
        let cells: Vec<Cell<'_>> = grid.iter_cells().collect();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0].row.y, 0);
        assert_eq!(cells[0].column.x, 0);
        assert_eq!(cells[1].row.y, 0);
        assert_eq!(cells[1].column.x, 2);
        assert_eq!(cells[3].row.y, 5);
        assert_eq!(cells[3].column.x, 2);
    }

    #[test]
    fn find_cells_matches_coordinates() {
        let grid = grid_with_rows(&[5, 5]);
        let columns = SmallVecLine::from_vec(vec![
            Column::new(LineInfo::new(0, 5, LineKind::Full)),
            Column::new(LineInfo::new(5, 5, LineKind::Full)),
        ]);
        let grid = Grid {
            rows: grid.rows,
            columns,
        };

        let cells: Vec<_> = grid.find_cells(&[0, 5], &[5]).collect();
        assert_eq!(cells.len(), 2);
        assert!(cells.iter().all(Result::is_ok));
        assert_eq!(cells[0].as_ref().unwrap().row.y, 0);
        assert_eq!(cells[1].as_ref().unwrap().row.y, 5);
    }

    #[test]
    fn find_cells_reports_missing_rows() {
        let grid = grid_with_rows(&[5]);
        let cells: Vec<_> = grid.find_cells(&[100], &[200]).collect();
        assert_eq!(cells.len(), 1);
        match &cells[0] {
            Err(GridError::RowNotFound { y: 100 }) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn find_cells_reports_missing_columns() {
        let grid = grid_with_rows(&[5]);
        let cells: Vec<_> = grid.find_cells(&[0], &[200]).collect();
        assert_eq!(cells.len(), 1);
        match &cells[0] {
            Err(GridError::ColumnNotFound { x: 200 }) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn dimensions_reports_extent() {
        let grid = grider::make_grid!(rows: [(0, 5), (5, 10)], columns: [(0, 4)]);
        assert_eq!(grid.dimensions(), (4, 15));
        assert_eq!(grid.to_string(), "Grid { rows: 2, columns: 1 }");
    }

    #[test]
    fn keep_kind_and_retain_agree() {
        let grid = grider::make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full)],
            columns: [(0, 5, LineKind::Full)]
        );
        let kept = grid.keep_kind(LineKind::Full);
        assert_eq!(kept.rows.len(), 1);
        assert_eq!(kept.columns.len(), 1);

        let mut retained = grid.clone();
        retained.retain_rows(|row| row.kind.is_full());
        retained.retain_columns(|column| column.kind.is_full());
        assert_eq!(kept, retained);
    }
}

mod filters {
    use super::*;

    #[test]
    fn filter_smallest_rows_with_tolerance() {
        let grid = grid_with_rows(&[5, 6, 10, 15]);
        let filtered = grid.filter_smallest_rows_with_tolerance(0.5);
        assert_eq!(filtered.rows.len(), 2);
        assert!(filtered.rows.iter().all(|row| row.height > 6));
        assert_eq!(filtered.columns, grid.columns);
    }

    #[test]
    fn filter_biggest_rows_with_tolerance() {
        let grid = grid_with_rows(&[5, 10, 14, 15]);
        let filtered = grid.filter_biggest_rows_with_tolerance(0.1);
        assert_eq!(filtered.rows.len(), 2);
        assert!(filtered.rows.iter().all(|row| row.height < 14));
    }

    #[test]
    fn filter_smallest_columns_with_tolerance() {
        let grid = grid_with_columns(&[5, 6, 10, 15]);
        let filtered = grid.filter_smallest_columns_with_tolerance(0.5);
        assert_eq!(filtered.columns.len(), 2);
        assert!(filtered.columns.iter().all(|column| column.width > 6));
        assert_eq!(filtered.rows, grid.rows);
    }

    #[test]
    fn filter_biggest_columns_with_tolerance() {
        let grid = grid_with_columns(&[5, 10, 14, 15]);
        let filtered = grid.filter_biggest_columns_with_tolerance(0.1);
        assert_eq!(filtered.columns.len(), 2);
        assert!(filtered.columns.iter().all(|column| column.width < 14));
    }

    #[test]
    fn default_tolerance_filters_smallest_and_biggest() {
        let grid = grid_with_rows(&[5, 10, 15]);
        assert_eq!(grid.filter_smallest_rows().rows.len(), 2);
        assert!(
            grid.filter_smallest_rows()
                .rows
                .iter()
                .all(|row| row.height > 5)
        );

        let grid = grid_with_rows(&[5, 10, 15]);
        assert_eq!(grid.filter_biggest_rows().rows.len(), 2);
        assert!(
            grid.filter_biggest_rows()
                .rows
                .iter()
                .all(|row| row.height < 15)
        );

        let grid = grid_with_columns(&[5, 10, 15]);
        assert_eq!(grid.filter_smallest_columns().columns.len(), 2);
        assert!(
            grid.filter_smallest_columns()
                .columns
                .iter()
                .all(|column| column.width > 5)
        );

        let grid = grid_with_columns(&[5, 10, 15]);
        assert_eq!(grid.filter_biggest_columns().columns.len(), 2);
        assert!(
            grid.filter_biggest_columns()
                .columns
                .iter()
                .all(|column| column.width < 15)
        );
    }

    #[test]
    fn extremum_helpers_report_extremes() {
        let grid = grid_with_rows(&[2, 8, 4]);
        assert_eq!(grid.smallest_row_height(), Some(2));
        assert_eq!(grid.biggest_row_height(), Some(8));

        let grid = grid_with_columns(&[1, 7, 3]);
        assert_eq!(grid.smallest_column_width(), Some(1));
        assert_eq!(grid.biggest_column_width(), Some(7));
    }

    #[test]
    fn clamp_to_i32_is_reexported() {
        assert_eq!(clamp_to_i32(7), 7);
    }

    #[test]
    fn filter_pipeline_is_chainable() {
        let grid = grider::make_grid!(
            rows: [(0, 5), (5, 10, LineKind::Full), (15, 15)],
            columns: [(0, 5), (5, 10, LineKind::Full), (15, 15)]
        );
        let filtered = grid.filter_biggest_rows().filter_smallest_columns();
        assert_eq!(
            filtered
                .rows
                .iter()
                .map(|row| row.height)
                .collect::<Vec<_>>(),
            vec![5, 10]
        );
        assert_eq!(
            filtered
                .columns
                .iter()
                .map(|column| column.width)
                .collect::<Vec<_>>(),
            vec![10, 15]
        );
    }
}
