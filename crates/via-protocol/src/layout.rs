/// Physical keyboard layout definitions.
///
/// A layout describes where each key is physically positioned for rendering,
/// and maps each visual key to a (row, col) in the keyboard matrix.

/// A single physical key position.
#[derive(Debug, Clone)]
pub struct KeyPosition {
    /// X position in key units (1u = one standard key width).
    pub x: f32,
    /// Y position in key units.
    pub y: f32,
    /// Width in key units (default 1.0).
    pub w: f32,
    /// Height in key units (default 1.0).
    pub h: f32,
    /// Rotation angle in degrees (for thumb keys).
    pub r: f32,
    /// Rotation origin X (in key units, relative to layout origin).
    pub rx: f32,
    /// Rotation origin Y.
    pub ry: f32,
    /// Matrix row this key maps to.
    pub row: u8,
    /// Matrix column this key maps to.
    pub col: u8,
}

impl KeyPosition {
    pub fn new(x: f32, y: f32, row: u8, col: u8) -> Self {
        Self {
            x,
            y,
            w: 1.0,
            h: 1.0,
            r: 0.0,
            rx: 0.0,
            ry: 0.0,
            row,
            col,
        }
    }

    pub fn with_size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    pub fn with_rotation(mut self, r: f32, rx: f32, ry: f32) -> Self {
        self.r = r;
        self.rx = rx;
        self.ry = ry;
        self
    }
}

/// A complete keyboard layout definition.
#[derive(Debug, Clone)]
pub struct KeyboardLayout {
    /// Display name.
    pub name: String,
    /// VID:PID pairs this layout applies to (empty = generic).
    pub vid_pid: Vec<(u16, u16)>,
    /// Number of matrix rows.
    pub rows: u8,
    /// Number of matrix columns.
    pub cols: u8,
    /// Physical key positions.
    pub keys: Vec<KeyPosition>,
}

impl KeyboardLayout {
    /// Total width in key units.
    pub fn width(&self) -> f32 {
        self.keys.iter().map(|k| k.x + k.w).fold(0.0_f32, f32::max)
    }

    /// Total height in key units.
    pub fn height(&self) -> f32 {
        self.keys.iter().map(|k| k.y + k.h).fold(0.0_f32, f32::max)
    }
}

/// Built-in layout for the foostan Corne v4.1 keyboard.
///
/// Uses the LAYOUT_split_3x6_3_ex2 layout from QMK rev4_1/info.json.
/// Matrix is 8 rows x 7 columns:
///   - Left half: rows 0-3, cols 0-6 (col 6 = extra inner key on rows 0-1)
///   - Right half: rows 4-7, cols 0-6 (col 6 = extra inner key on rows 4-5)
/// Row 3 (left) and row 7 (right) are thumb clusters.
/// Rows 2-3 and 6-7 have no col 6 (null in matrix_pins).
pub fn corne_layout() -> KeyboardLayout {
    // From QMK rev4_1/info.json LAYOUT_split_3x6_3_ex2
    let keys = vec![
        // Left half - row 0 (cols 0-5) + extra inner key (col 6)
        KeyPosition::new(0.0, 0.3, 0, 0),
        KeyPosition::new(1.0, 0.3, 0, 1),
        KeyPosition::new(2.0, 0.1, 0, 2),
        KeyPosition::new(3.0, 0.0, 0, 3),
        KeyPosition::new(4.0, 0.1, 0, 4),
        KeyPosition::new(5.0, 0.2, 0, 5),
        KeyPosition::new(6.0, 0.7, 0, 6), // extra inner
        // Right half - row 0 (matrix row 4, cols reversed) + extra inner (col 6)
        KeyPosition::new(8.0, 0.7, 4, 6), // extra inner
        KeyPosition::new(9.0, 0.2, 4, 5),
        KeyPosition::new(10.0, 0.1, 4, 4),
        KeyPosition::new(11.0, 0.0, 4, 3),
        KeyPosition::new(12.0, 0.1, 4, 2),
        KeyPosition::new(13.0, 0.3, 4, 1),
        KeyPosition::new(14.0, 0.3, 4, 0),
        // Left half - row 1 (cols 0-5) + extra inner (col 6)
        KeyPosition::new(0.0, 1.3, 1, 0),
        KeyPosition::new(1.0, 1.3, 1, 1),
        KeyPosition::new(2.0, 1.1, 1, 2),
        KeyPosition::new(3.0, 1.0, 1, 3),
        KeyPosition::new(4.0, 1.1, 1, 4),
        KeyPosition::new(5.0, 1.2, 1, 5),
        KeyPosition::new(6.0, 1.7, 1, 6), // extra inner
        // Right half - row 1 (matrix row 5) + extra inner (col 6)
        KeyPosition::new(8.0, 1.7, 5, 6), // extra inner
        KeyPosition::new(9.0, 1.2, 5, 5),
        KeyPosition::new(10.0, 1.1, 5, 4),
        KeyPosition::new(11.0, 1.0, 5, 3),
        KeyPosition::new(12.0, 1.1, 5, 2),
        KeyPosition::new(13.0, 1.3, 5, 1),
        KeyPosition::new(14.0, 1.3, 5, 0),
        // Left half - row 2 (cols 0-5, no col 6)
        KeyPosition::new(0.0, 2.3, 2, 0),
        KeyPosition::new(1.0, 2.3, 2, 1),
        KeyPosition::new(2.0, 2.1, 2, 2),
        KeyPosition::new(3.0, 2.0, 2, 3),
        KeyPosition::new(4.0, 2.1, 2, 4),
        KeyPosition::new(5.0, 2.2, 2, 5),
        // Right half - row 2 (matrix row 6, no col 6)
        KeyPosition::new(9.0, 2.2, 6, 5),
        KeyPosition::new(10.0, 2.1, 6, 4),
        KeyPosition::new(11.0, 2.0, 6, 3),
        KeyPosition::new(12.0, 2.1, 6, 2),
        KeyPosition::new(13.0, 2.3, 6, 1),
        KeyPosition::new(14.0, 2.3, 6, 0),
        // Left thumb cluster - row 3 (cols 3-5)
        KeyPosition::new(4.0, 3.7, 3, 3),
        KeyPosition::new(5.0, 3.7, 3, 4),
        KeyPosition::new(6.0, 3.2, 3, 5).with_size(1.0, 1.5),
        // Right thumb cluster - row 7 (cols 3-5)
        KeyPosition::new(8.0, 3.2, 7, 5).with_size(1.0, 1.5),
        KeyPosition::new(9.0, 3.7, 7, 4),
        KeyPosition::new(10.0, 3.7, 7, 3),
    ];

    KeyboardLayout {
        name: "Corne v4.1 (crkbd)".to_string(),
        vid_pid: vec![(0x4653, 0x0004)], // foostan Corne v4
        rows: 8,
        cols: 7,
        keys,
    }
}

/// Try to find a built-in layout matching the given VID:PID.
pub fn find_layout(vid: u16, pid: u16) -> Option<KeyboardLayout> {
    let layouts = [corne_layout()];
    layouts
        .into_iter()
        .find(|l| l.vid_pid.iter().any(|&(v, p)| v == vid && p == pid))
}

/// Create a generic grid layout for an unknown keyboard.
/// Falls back to a simple grid based on matrix dimensions.
pub fn generic_layout(rows: u8, cols: u8) -> KeyboardLayout {
    let mut keys = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            keys.push(KeyPosition::new(col as f32, row as f32, row, col));
        }
    }
    KeyboardLayout {
        name: format!("Generic {rows}x{cols}"),
        vid_pid: vec![],
        rows,
        cols,
        keys,
    }
}
