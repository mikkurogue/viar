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
    let mut cur_y: f32 = -1.0; // will be incremented to 0.0 on first row

    // Per-key properties (reset after each key)
    let mut next_w: f32 = 1.0;
    let mut next_h: f32 = 1.0;

    // Rotation state (persists until changed)
    let mut cur_r: f32 = 0.0;
    let mut cur_rx: f32 = 0.0;
    let mut cur_ry: f32 = 0.0;

    for row_value in keymap {
        let row_arr = row_value.as_array().ok_or("keymap row is not an array")?;

        // KLE convention: each new row resets x and increments y by 1
        cur_x = cur_rx;
        cur_y += 1.0;

        for item in row_arr {
            match item {
                Value::Object(props) => {
                    // Properties object — sets state for next key(s)

                    // Rotation properties: when r, rx, or ry change,
                    // position resets to (rx, ry).
                    let mut rotation_changed = false;
                    if let Some(r) = props.get("r").and_then(|v| v.as_f64()) {
                        cur_r = r as f32;
                        rotation_changed = true;
                    }
                    if let Some(rx) = props.get("rx").and_then(|v| v.as_f64()) {
                        cur_rx = rx as f32;
                        rotation_changed = true;
                    }
                    if let Some(ry) = props.get("ry").and_then(|v| v.as_f64()) {
                        cur_ry = ry as f32;
                        rotation_changed = true;
                    }
                    if rotation_changed {
                        cur_x = cur_rx;
                        cur_y = cur_ry;
                    }

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
                }
                Value::String(legend) => {
                    // Key — first legend line is "row,col" or could be a label
                    // Vial uses "row,col\n..." format
                    // Encoders use "row,col\n\n\n\n\n\n\n\n\ne" format (line 10 = "e")
                    // Layout options use "row,col\n\n\nopt,choice" format (line 4)
                    let is_encoder = legend.lines().any(|line| line.trim() == "e");
                    let lines: Vec<&str> = legend.lines().collect();
                    let first_line = lines.first().copied().unwrap_or("");

                    // Check layout option (4th legend line, 0-indexed line 3)
                    // If present, only keep choice 0 (default)
                    let layout_option_line = lines.get(3).copied().unwrap_or("");
                    let is_non_default_option = if !layout_option_line.is_empty() {
                        // Format: "option_idx,choice_idx" — keep only choice 0
                        layout_option_line
                            .split_once(',')
                            .map(|(_, choice)| choice != "0")
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if is_encoder || is_non_default_option {
                        debug!(legend = %legend, x = cur_x, y = cur_y, "skipping encoder/non-default key");
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
                            // Rotate the key's center around (rx, ry), then
                            // position the axis-aligned rect so its center
                            // matches the rotated center. This gives better
                            // visual alignment than rotating the top-left corner.
                            let center_x = cur_x + next_w / 2.0;
                            let center_y = cur_y + next_h / 2.0;
                            let dx = center_x - cur_rx;
                            let dy = center_y - cur_ry;
                            let rot_cx = cur_rx + dx * cos_a - dy * sin_a;
                            let rot_cy = cur_ry + dx * sin_a + dy * cos_a;
                            key.x = rot_cx - next_w / 2.0;
                            key.y = rot_cy - next_h / 2.0;
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
