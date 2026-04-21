/// Physical keyboard layout definitions.
///
/// A layout describes where each key is physically positioned for rendering,
/// and maps each visual key to a (row, col) in the keyboard matrix.
use serde_json::Value;
use tracing::{debug, warn};

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

/// Parse a Vial keyboard definition JSON string into a KeyboardLayout.
///
/// The JSON format is:
/// ```json
/// {
///     "matrix": {"rows": N, "cols": M},
///     "layouts": {
///         "keymap": [["row,col", ...], [{properties}, "row,col", ...], ...]
///     }
/// }
/// ```
///
/// The keymap uses KLE (keyboard-layout-editor.com) format where:
/// - Each top-level array element is a row
/// - Within a row, JSON objects set properties (x, y, w, h, r, rx, ry offsets)
/// - Strings are key legends in "row,col" format (first legend = matrix position)
/// - Properties like `w`, `h` apply only to the next key then reset
/// - Properties like `x`, `y` are additive offsets
/// - `r`, `rx`, `ry` set rotation and persist until changed
pub fn parse_vial_definition(json: &str) -> Result<KeyboardLayout, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;

    let matrix = root.get("matrix").ok_or("missing 'matrix' field")?;
    let rows = matrix
        .get("rows")
        .and_then(|v| v.as_u64())
        .ok_or("missing matrix.rows")? as u8;
    let cols = matrix
        .get("cols")
        .and_then(|v| v.as_u64())
        .ok_or("missing matrix.cols")? as u8;

    let layouts = root.get("layouts").ok_or("missing 'layouts' field")?;
    let keymap = layouts
        .get("keymap")
        .and_then(|v| v.as_array())
        .ok_or("missing layouts.keymap array")?;

    let name = root
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Vial Keyboard")
        .to_string();

    let keys = parse_kle_keymap(keymap)?;

    debug!(
        name = %name,
        rows, cols,
        num_keys = keys.len(),
        "parsed Vial definition"
    );

    Ok(KeyboardLayout {
        name,
        vid_pid: vec![],
        rows,
        cols,
        keys,
    })
}

/// Parse KLE-format keymap rows into KeyPosition entries.
fn parse_kle_keymap(keymap: &[Value]) -> Result<Vec<KeyPosition>, String> {
    let mut keys = Vec::new();

    // Current position state
    let mut cur_x: f32;
    let mut cur_y: f32 = 0.0;

    // Per-key properties (reset after each key)
    let mut next_w: f32 = 1.0;
    let mut next_h: f32 = 1.0;

    // Rotation state (persists until changed)
    let mut cur_r: f32 = 0.0;
    let mut cur_rx: f32 = 0.0;
    let mut cur_ry: f32 = 0.0;

    for row_value in keymap {
        let row_arr = row_value.as_array().ok_or("keymap row is not an array")?;

        // Each new row: advance Y by 1, reset X to 0 (or rx if rotated)
        // But the first row starts at 0,0
        // KLE convention: new row = x resets, y increments
        // However, if rx/ry are set, x resets to rx and y to ry on rotation change

        cur_x = cur_rx; // reset x to rotation origin at start of each row
                        // y is incremented at the end of the previous row (handled below)

        for item in row_arr {
            match item {
                Value::Object(props) => {
                    // Properties object — sets state for next key(s)
                    if let Some(x) = props.get("x").and_then(|v| v.as_f64()) {
                        cur_x += x as f32;
                    }
                    if let Some(y) = props.get("y").and_then(|v| v.as_f64()) {
                        cur_y += y as f32;
                    }
                    if let Some(w) = props.get("w").and_then(|v| v.as_f64()) {
                        next_w = w as f32;
                    }
                    if let Some(h) = props.get("h").and_then(|v| v.as_f64()) {
                        next_h = h as f32;
                    }
                    if let Some(r) = props.get("r").and_then(|v| v.as_f64()) {
                        cur_r = r as f32;
                    }
                    if let Some(rx) = props.get("rx").and_then(|v| v.as_f64()) {
                        cur_rx = rx as f32;
                        cur_x = cur_rx; // reset x when rx changes
                    }
                    if let Some(ry) = props.get("ry").and_then(|v| v.as_f64()) {
                        cur_ry = ry as f32;
                        cur_y = cur_ry; // reset y when ry changes
                    }
                }
                Value::String(legend) => {
                    // Key — first legend line is "row,col" or could be a label
                    // Vial uses "row,col\n..." format
                    // Encoders use "row,col\n\n\n\n\n\n\n\n\ne" format (line 10 = "e")
                    let is_encoder = legend.lines().any(|line| line.trim() == "e");
                    let first_line = legend.lines().next().unwrap_or("");

                    if is_encoder {
                        // Encoder — skip, don't add to layout keys
                        debug!(legend = %legend, x = cur_x, y = cur_y, "skipping encoder key");
                    } else if let Some((matrix_row, matrix_col)) = parse_matrix_pos(first_line) {
                        let mut key = KeyPosition::new(cur_x, cur_y, matrix_row, matrix_col)
                            .with_size(next_w, next_h)
                            .with_rotation(cur_r, cur_rx, cur_ry);

                        // Apply rotation to position so the renderer doesn't need
                        // to handle rotated rectangles — just place keys at the
                        // pre-rotated coordinates.
                        if cur_r.abs() > 0.001 {
                            let angle = cur_r.to_radians();
                            let cos_a = angle.cos();
                            let sin_a = angle.sin();
                            // Rotate the key's top-left corner around (rx, ry)
                            let dx = cur_x - cur_rx;
                            let dy = cur_y - cur_ry;
                            key.x = cur_rx + dx * cos_a - dy * sin_a;
                            key.y = cur_ry + dx * sin_a + dy * cos_a;
                        }

                        keys.push(key);
                    } else if first_line.is_empty() {
                        debug!(legend = %legend, x = cur_x, y = cur_y, "skipping empty key");
                    } else {
                        warn!(legend = %legend, "unrecognized KLE legend format, skipping");
                    }

                    // Advance x by key width
                    cur_x += next_w;

                    // Reset per-key properties
                    next_w = 1.0;
                    next_h = 1.0;
                }
                _ => {
                    warn!(?item, "unexpected item type in KLE row");
                }
            }
        }

        // End of row: advance y
        cur_y += 1.0;
    }

    Ok(keys)
}

/// Parse a "row,col" string into (row, col) matrix coordinates.
fn parse_matrix_pos(s: &str) -> Option<(u8, u8)> {
    let s = s.trim();
    let (row_s, col_s) = s.split_once(',')?;
    let row = row_s.trim().parse::<u8>().ok()?;
    let col = col_s.trim().parse::<u8>().ok()?;
    Some((row, col))
}
