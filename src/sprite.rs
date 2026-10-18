use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    /// Creates a new color with the provided RGBA values.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Changes the alpha value.
    pub fn change_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    /// Generates a transparent `Color`
    fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u8);

impl Index {
    /// Returns the internal `u8` value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Error, Debug)]
pub enum LayerError {
    #[error("Layer {0} not found in the Sprite")]
    NotFound(String),

    #[error("A Layer with this name already exists")]
    NameCollision,
}

/// A `Layer` is shared by every `Frame` of the `Sprite` and holds no pixels.
/// The pixels live in the `Cel`s.
#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    is_visible: bool,
}

impl Layer {
    /// Creates a new `Layer`.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_visible: true,
        }
    }

    /// Returns the name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Sets the visibility.
    pub fn visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
}

/// A `Cel` holds the pixels of ONE `Layer` in ONE `Frame`.
#[derive(Debug, Clone)]
pub struct Cel {
    pixels: Vec<Index>,
}

impl Cel {
    /// Creates a new transparent `Cel`.
    fn new(width: u32, height: u32) -> Self {
        let total_pixels = (width * height) as usize;
        let pixels = vec![Index(0); total_pixels];
        Self { pixels }
    }
}

/// A `Frame` holds one `Cel` for every `Layer` of the `Sprite`.
/// `cels[i]` always belongs to the `Sprite`'s `layers[i]`.
#[derive(Debug)]
pub struct Frame {
    cels: Vec<Cel>,
}

impl Frame {
    /// Creates a new `Frame` with one transparent `Cel` per `Layer`.
    fn new(width: u32, height: u32, layer_count: usize) -> Self {
        Self {
            cels: (0..layer_count).map(|_| Cel::new(width, height)).collect(),
        }
    }
}

#[derive(Error, Debug)]
pub enum SpriteError {
    #[error("Can't add more than 255 colors to a single palette!")]
    PaletteOverflow,

    #[error("Couldn't find the active layer")]
    MissingActiveLayer,

    #[error("Invalid column position: {0}")]
    InvalidColumn(u32),

    #[error("Invalid row position: {0}")]
    InvalidRow(u32),

    #[error("Invalid frame index: {0}")]
    InvalidFrameIndex(usize),
}

#[derive(Debug)]
pub struct Sprite {
    width: u32,
    height: u32,
    palette: Vec<Color>,
    layers: Vec<Layer>,
    frames: Vec<Frame>,
    current_frame: usize,
    active_layer: usize,
}

impl Sprite {
    /// Creates a new `Sprite`
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            palette: vec![Color::transparent()],
            layers: vec![Layer::new("Background")],
            frames: vec![Frame::new(width, height, 1)],
            current_frame: 0,
            active_layer: 0,
        }
    }

    /// Returns the position of a `Layer` in the `Sprite`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    fn get_layer_index(&self, name: &str) -> Result<usize, LayerError> {
        self.layers
            .iter()
            .position(|layer| layer.name == name)
            .ok_or_else(|| LayerError::NotFound(name.to_string()))
    }

    /// Returns a reference to a `Layer`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn get_layer(&self, name: &str) -> Result<&Layer, LayerError> {
        let index = self.get_layer_index(name)?;
        Ok(&self.layers[index])
    }

    /// Returns a mutable reference to a `Layer`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn get_mut_layer(&mut self, name: &str) -> Result<&mut Layer, LayerError> {
        let index = self.get_layer_index(name)?;
        Ok(&mut self.layers[index])
    }

    /// Returns a reference to the `Cel` of a `Layer` within the current frame.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn get_cel(&self, name: &str) -> Result<&Cel, LayerError> {
        let index = self.get_layer_index(name)?;
        Ok(&self.frames[self.current_frame].cels[index])
    }

    /// Adds a `Layer` to the `Sprite`, along with an empty `Cel` in every `Frame`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NameCollision` if there's already a `Layer`
    /// with the same name.
    pub fn add_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if self.get_layer(name).is_ok() {
            Err(LayerError::NameCollision)
        } else {
            self.layers.push(Layer::new(name));
            for frame in &mut self.frames {
                frame.cels.push(Cel::new(self.width, self.height));
            }
            Ok(())
        }
    }

    /// Returns the index of the provided row and column.
    fn get_index(&self, row: u32, col: u32) -> usize {
        let width = self.width as usize;
        let row = row as usize;
        let col = col as usize;

        (row * width) + col
    }

    /// Changes the active `Layer` to another one.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn set_active_layer(&mut self, name: &str) -> Result<(), LayerError> {
        self.active_layer = self.get_layer_index(name)?;
        Ok(())
    }

    /// Sets the pixel at the provided row and column to the `Index` of the provided color.
    pub fn set_pixel(&mut self, row: u32, col: u32, palette_index: Index) {
        if row < self.height && col < self.width {
            let idx = self.get_index(row, col);
            let active_cel = &mut self.frames[self.current_frame].cels[self.active_layer];
            active_cel.pixels[idx] = palette_index;
        }
    }

    /// Resets a pixel to `Index` of 0, making it transparent.
    pub fn erase_pixel(&mut self, row: u32, col: u32) {
        self.set_pixel(row, col, Index(0));
    }

    /// Adds a color to the palette.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::PaletteOverflow` if the palette has more than 256 colors.
    // This migh have to be changed later.
    pub fn add_color(&mut self, color: Color) -> Result<Index, SpriteError> {
        if self.palette.len() >= 256 {
            Err(SpriteError::PaletteOverflow)
        } else {
            let idx = u8::try_from(self.palette.len()).expect("Should be always a valid u8");
            self.palette.push(color);
            Ok(Index(idx))
        }
    }

    /// Displays all the pixels of the selected `Layer` in a grid.
    fn display_layer_data(&self, layer: &Layer, cel: &Cel) {
        println!("Layer: '{}' (Visible: {})", layer.name, layer.is_visible);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let palette_index = cel.pixels[idx].value();
                print!("{palette_index} ");
            }
            println!();
        }
        println!();
    }

    /// Displays all the pixels of the selected `Layer` in a grid.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn display_layer(&self, name: &str) -> Result<(), LayerError> {
        let index = self.get_layer_index(name)?;
        let cel = &self.frames[self.current_frame].cels[index];
        self.display_layer_data(&self.layers[index], cel);
        Ok(())
    }

    /// Displays all the pixels of all the `Layer`s in the current `Frame` as a sequence of grids.
    pub fn display_all_layers(&self) {
        let current_frame = &self.frames[self.current_frame];
        for (layer, cel) in self.layers.iter().zip(&current_frame.cels) {
            self.display_layer_data(layer, cel);
        }
    }

    /// Adds a `Frame` to the `Sprite`, with an empty `Cel` for every `Layer`.
    pub fn add_frame(&mut self) {
        let new_frame = Frame::new(self.width, self.height, self.layers.len());
        self.frames.push(new_frame);
    }

    /// Switches the active `Frame` index.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::InvalidFrameIndex` if the `Frame` doesn't exist.
    pub fn set_current_frame(&mut self, index: usize) -> Result<(), SpriteError> {
        if index < self.frames.len() {
            self.current_frame = index;
            Ok(())
        } else {
            Err(SpriteError::InvalidFrameIndex(index))
        }
    }

    /// Returns the total number of frames in the sprite animation.
    pub fn get_frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Returns the index of the currently active frame.
    pub fn get_current_frame_index(&self) -> usize {
        self.current_frame
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Verifies that a new sprite is correctly initialized with the specified
    /// width and height, and automatically contains a default "Background" layer.
    #[test]
    fn test_sprite_initialization() {
        let sprite = Sprite::new(10, 5);
        assert_eq!(sprite.width, 10);
        assert_eq!(sprite.height, 5);
        assert!(sprite.get_layer("Background").is_ok());
    }

    /// Verifies that colors can be instantiated with custom RGBA values,
    /// their alpha channel can be modified, and transparent
    /// defaults are initialized with an alpha of zero.
    #[test]
    fn test_color_creation_and_alpha() {
        let mut color = Color::new(100, 150, 200, 255);
        assert_eq!(color.alpha, 255);

        color.change_alpha(128);
        assert_eq!(color.alpha, 128);

        let transparent_color = Color::transparent();
        assert_eq!(transparent_color.alpha, 0);
    }

    /// Verifies that the `Index` wrapper type correctly holds a `u8` value
    /// and that its `.value()` method returns the expected integer.
    #[test]
    fn test_index_value() {
        // yes I did this just for coverage
        let idx = Index(5);
        assert_eq!(idx.value(), 5);
    }

    /// Verifies that layer names can be correctly retrieved and updated,
    /// and that visibility toggles properly modify the layer's state.
    #[test]
    fn test_layer_getters_and_setters() {
        let mut layer = Layer::new("TestLayer");
        assert_eq!(layer.get_name(), "TestLayer");

        layer.set_name("RenamedLayer");
        assert_eq!(layer.get_name(), "RenamedLayer");

        layer.visible(false);
        assert!(!layer.is_visible);
    }

    /// Verifies that colors can be successfully added to the sprite's palette,
    /// and that exceeding the 256-color limit correctly triggers a
    ///  `SpriteError::PaletteOverflow`.
    #[test]
    fn test_color_addition_and_limit() {
        let mut sprite = Sprite::new(5, 5);

        let color = Color::new(255, 0, 0, 255);
        let idx = sprite.add_color(color);
        assert!(idx.is_ok());
        assert_eq!(idx.unwrap().value(), 1);

        // Fill up to the 256 limit (index 0 is already taken, so 255 more)
        for _ in 0..255 {
            let _ = sprite.add_color(Color::new(0, 0, 0, 255));
        }

        // The next one should fail with PaletteOverflow
        let overflow_result = sprite.add_color(Color::new(255, 255, 255, 255));
        assert!(matches!(overflow_result, Err(SpriteError::PaletteOverflow)));
    }

    /// Verifies that new layers can be successfully added and retrieved,
    /// that duplicate layer names trigger a `LayerError::NameCollision`,
    /// and that looking up non-existent layers returns a `LayerError::NotFound` error.
    #[test]
    fn test_layer_management_and_errors() {
        let mut sprite = Sprite::new(4, 4);

        // Add a new unique layer successfully
        assert!(sprite.add_layer("Foreground").is_ok());
        assert!(sprite.get_layer("Foreground").is_ok());

        // Attempting to add a duplicate layer name should cause a NameCollision
        let duplicate_result = sprite.add_layer("Foreground");
        assert!(matches!(duplicate_result, Err(LayerError::NameCollision)));

        // Requesting a non-existent layer should return NotFound
        let missing_result = sprite.get_layer("GhostLayer");
        assert!(matches!(missing_result, Err(LayerError::NotFound(_))));
    }

    /// Verifies that the active layer can be successfully switched to an existing layer,
    /// and that attempting to switch to a non-existent layer returns an error.
    #[test]
    fn test_active_layer_switching() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_layer("Foreground").unwrap();

        // Switch to Foreground successfully
        assert!(sprite.set_active_layer("Foreground").is_ok());

        // Switching to a non-existent layer should fail
        assert!(sprite.set_active_layer("BadLayer").is_err());
    }

    /// Verifies that pixels can be successfully drawn on the active layer using a palette index,
    /// erased back to the default transparent index (0), and that out-of-bounds pixel coordinates
    /// are safely ignored without panicking.
    #[test]
    fn test_pixel_drawing_and_erasing() {
        let mut sprite = Sprite::new(3, 3);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        // Draw a pixel
        sprite.set_pixel(1, 1, red);

        {
            let cel = sprite.get_cel("Background").unwrap();
            let idx = sprite.get_index(1, 1);
            assert_eq!(cel.pixels[idx], red);
        }

        // Erase the pixel
        sprite.erase_pixel(1, 1);

        // Check it again
        let cel = sprite.get_cel("Background").unwrap();
        let idx = sprite.get_index(1, 1);
        assert_eq!(cel.pixels[idx], Index(0));

        // Out of bounds should be silently ignored without crashing
        sprite.set_pixel(10, 10, red);
    }

    /// Verifies that the display and printing helper functions execute cleanly
    /// without panicking, correctly succeeding for existing layers and returning
    /// an error for non-existent layers.
    #[test]
    fn test_display_functions_dont_panic() {
        let mut sprite = Sprite::new(2, 2);
        let blue = sprite.add_color(Color::new(0, 0, 255, 255)).unwrap();
        sprite.set_pixel(0, 0, blue);

        // Ensure display methods run cleanly without throwing errors
        assert!(sprite.display_layer("Background").is_ok());
        assert!(sprite.display_layer("NonExistent").is_err());

        sprite.display_all_layers();
    }

    /// Verifies that new `Frame`s can be added, `Frame` counts increment correctly,
    /// and the initial frame index is zero.
    #[test]
    fn test_frame_initialization_and_addition() {
        let mut sprite = Sprite::new(8, 8);
        assert_eq!(sprite.get_frame_count(), 1);
        assert_eq!(sprite.get_current_frame_index(), 0);

        sprite.add_frame();
        assert_eq!(sprite.get_frame_count(), 2);
    }

    /// Verifies that switching between valid `Frame`s succeeds and
    /// attempting to switch to an out-of-bounds `Frame` returns an `InvalidFrameIndex` error.
    #[test]
    fn test_frame_navigation_and_errors() {
        let mut sprite = Sprite::new(8, 8);
        sprite.add_frame(); // Now has frames 0 and 1

        // Switch to `Frame` 1 successfully
        assert!(sprite.set_current_frame(1).is_ok());
        assert_eq!(sprite.get_current_frame_index(), 1);

        // Attempting to switch to a non-existent `Frame` should fail
        let invalid_switch = sprite.set_current_frame(5);
        assert!(matches!(
            invalid_switch,
            Err(SpriteError::InvalidFrameIndex(5))
        ));
    }

    /// Verifies that pixel changes are isolated to the currently active `Frame`.
    #[test]
    fn test_frame_pixel_isolation() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        // Draw on frame 0
        sprite.set_pixel(0, 0, red);

        // Add and switch to frame 1
        sprite.add_frame();
        sprite.set_current_frame(1).unwrap();

        // `Frame` 1's background cel should still be transparent (Index 0) at (0, 0)
        {
            let cel = sprite.get_cel("Background").unwrap();
            let idx = sprite.get_index(0, 0);
            assert_eq!(cel.pixels[idx], Index(0));
        }

        // Switch back to `Frame` 0 and verify the red pixel is still there
        sprite.set_current_frame(0).unwrap();
        {
            let cel = sprite.get_cel("Background").unwrap();
            let idx = sprite.get_index(0, 0);
            assert_eq!(cel.pixels[idx], red);
        }
    }

    /// Verifies that a `Layer` added after several `Frame`s exist gets a `Cel`
    /// in every `Frame`, and that a `Frame` added later gets a `Cel` for every `Layer`.
    #[test]
    fn test_layers_shared_across_frames() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_frame();
        sprite.add_layer("Foreground").unwrap();
        sprite.add_frame();

        // Every frame has one cel per layer (Background + Foreground)
        for frame in &sprite.frames {
            assert_eq!(frame.cels.len(), sprite.layers.len());
        }

        // The new layer is reachable from every frame
        sprite.set_current_frame(0).unwrap();
        assert!(sprite.get_cel("Foreground").is_ok());
        sprite.set_current_frame(2).unwrap();
        assert!(sprite.get_cel("Foreground").is_ok());
    }
}
