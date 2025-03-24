Debug module for visualizing the grid on the image.

# Example

```rust
use grider::{*, drawing::*};
use image::DynamicImage;

// Replace with actual image loading
let img: DynamicImage = image::open("tests/large.png").unwrap();
let config = GridConfig::default();
let grid = Grid::try_from_image_with_config(&img, config).unwrap();

grider::debug::save_image_with_grid(&img, &grid, "output.png", &GridDrawingConfig::default());
```
