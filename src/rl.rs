use std::ffi::{c_char, c_float, c_int, c_uint, c_void, CString};

// Keyboard keys (US keyboard layout)
// NOTE: Use GetKeyPressed() to allow redefining
// required keys for alternative layouts
#[derive(Copy, Clone)]
pub enum KeyboardKey {
    Null = 0,
    Apostrophe = 39,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    Zero = 48,
    One = 49,
    Two = 50,
    Three = 51,
    Four = 52,
    Five = 53,
    Six = 54,
    Seven = 55,
    Eight = 56,
    Nine = 57,
    Semicolon = 59,
    Equal = 61,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    LeftBracket = 91,
    Backslash = 92,
    RightBracket = 93,
    Grave = 96,
    Space = 32,
    Escape = 256,
    Enter = 257,
    Tab = 258,
    Backspace = 259,
    Insert = 260,
    Delete = 261,
    Right = 262,
    Left = 263,
    Down = 264,
    Up = 265,
    PageUp = 266,
    PageDown = 267,
    Home = 268,
    End = 269,
    CapsLock = 280,
    ScrollLock = 281,
    NumLock = 282,
    PrintScreen = 283,
    Pause = 284,
    F1 = 290,
    F2 = 291,
    F3 = 292,
    F4 = 293,
    F5 = 294,
    F6 = 295,
    F7 = 296,
    F8 = 297,
    F9 = 298,
    F10 = 299,
    F11 = 300,
    F12 = 301,
    LeftShift = 340,
    LeftControl = 341,
    LeftAlt = 342,
    LeftSuper = 343,
    RightShift = 344,
    RightControl = 345,
    RightAlt = 346,
    RightSuper = 347,
    KbMenu = 348,
    Kp0 = 320,
    Kp1 = 321,
    Kp2 = 322,
    Kp3 = 323,
    Kp4 = 324,
    Kp5 = 325,
    Kp6 = 326,
    Kp7 = 327,
    Kp8 = 328,
    Kp9 = 329,
    KpDecimal = 330,
    KpDivide = 331,
    KpMultiply = 332,
    KpSubtract = 333,
    KpAdd = 334,
    KpEnter = 335,
    KpEqual = 336,
    Back = 4,
    Menu = 5,
    VolumeUp = 24,
    VolumeDown = 25,
}

#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Texture {
    id: u32,
    width: i32,
    height: i32,
    mipmaps: i32,
    format: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct C_Font {
    base_size: i32,
    glyph_count: i32,
    glyph_padding: i32,

    texture: Texture,
    recs: *const Rectangle,
    glyphs: *const c_void,
}

unsafe extern "C" {
    fn InitWindow(width: c_int, height: c_int, name: *const c_char);
    fn CloseWindow();
    fn WindowShouldClose() -> c_int;
    fn GetScreenWidth() -> c_int;
    fn GetScreenHeight() -> c_int;

    fn SetConfigFlags(flags: c_uint);

    fn ClearBackground(color: Color);
    fn BeginDrawing();
    fn EndDrawing();
    fn DrawRectangleRec(rec: Rectangle, color: Color);
    fn DrawRectangle(pos_x: c_int, pos_y: c_int, width: c_int, height: c_int, color: Color);
    fn DrawRectangleLines(pos_x: c_int, pos_y: c_int, width: c_int, height: c_int, color: Color);
    fn DrawLine(
        start_pos_x: c_int,
        start_pos_y: c_int,
        end_pos_x: c_int,
        end_pos_y: c_int,
        color: Color,
    );

    fn BeginScissorMode(x: c_int, y: c_int, w: c_int, h: c_int);
    fn EndScissorMode();

    fn SetTargetFPS(fps: i32);

    fn LoadCodepoints(text: *const c_char, codepointCount: *mut c_int) -> *const c_int;
    fn UnloadCodepoints(codepoint: *const c_int);

    fn LoadFontFromMemory(
        fileType: *const c_char,
        fileData: *const c_char,
        dataSize: c_int,
        fontSize: c_int,
        codepoints: *const c_int,
        codepointCount: c_int,
    ) -> C_Font;
    fn DrawTextEx(
        font: C_Font,
        text: *const c_char,
        position: Vector2,
        fontSize: c_float,
        spacing: c_float,
        tint: Color,
    );
    fn MeasureTextEx(
        font: C_Font,
        text: *const c_char,
        fontSize: c_float,
        spacing: c_float,
    ) -> Vector2;

    fn IsKeyPressed(key: c_int) -> bool;
    fn IsKeyPressedRepeat(key: c_int) -> bool;
    fn IsKeyDown(key: c_int) -> bool;
    fn GetCharPressed() -> c_int;
    fn SetExitKey(key: c_int);
}

pub fn init_window(width: c_int, height: c_int, name: &str) {
    let cstr = CString::new(name).unwrap().into_raw();

    unsafe { InitWindow(width, height, cstr) };
}

pub fn close_window() {
    unsafe { CloseWindow() };
}

pub fn window_should_close() -> bool {
    unsafe { WindowShouldClose() == 1 }
}

pub fn get_screen_width() -> i32 {
    unsafe { GetScreenWidth() }
}

pub fn get_screen_height() -> i32 {
    unsafe { GetScreenHeight() }
}

pub fn set_config_flags(flags: u32) {
    unsafe { SetConfigFlags(flags) };
}

pub fn clear_background(color: Color) {
    unsafe { ClearBackground(color) };
}

pub fn begin_drawing() {
    unsafe { BeginDrawing() };
}

pub fn end_drawing() {
    unsafe { EndDrawing() };
}

pub fn draw_rectangle_rec(rec: Rectangle, color: Color) {
    unsafe { DrawRectangleRec(rec, color) };
}

pub fn draw_rectangle(pos_x: i32, pos_y: i32, width: i32, height: i32, color: Color) {
    unsafe { DrawRectangle(pos_x, pos_y, width, height, color) };
}

pub fn draw_rectangle_lines(pos_x: i32, pos_y: i32, width: i32, height: i32, color: Color) {
    unsafe { DrawRectangleLines(pos_x, pos_y, width, height, color) };
}

pub fn draw_line(start_pos_x: i32, start_pos_y: i32, end_pos_x: i32, end_pos_y: i32, color: Color) {
    unsafe { DrawLine(start_pos_x, start_pos_y, end_pos_x, end_pos_y, color) };
}

pub fn draw_vertical_line(x: i32, y1: i32, y2: i32, color: Color) {
    unsafe { DrawLine(x, y1, x, y2, color) };
}

pub fn draw_horizonal_line(y: i32, x1: i32, x2: i32, color: Color) {
    unsafe { DrawLine(x1, y, x2, y, color) };
}

pub fn begin_scissor_mode(x: i32, y: i32, w: i32, h: i32) {
    unsafe { BeginScissorMode(x, y, w, h) };
}

pub fn end_scissor_mode() {
    unsafe { EndScissorMode() };
}

pub fn set_target_fps(fps: i32) {
    unsafe { SetTargetFPS(fps) };
}

impl Color {
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

/// Color constants
impl Color {
    pub const INDIANRED: Color = Color::new(205, 92, 92, 255);
    pub const LIGHTCORAL: Color = Color::new(240, 128, 128, 255);
    pub const SALMON: Color = Color::new(250, 128, 114, 255);
    pub const DARKSALMON: Color = Color::new(233, 150, 122, 255);
    pub const LIGHTSALMON: Color = Color::new(255, 160, 122, 255);
    pub const CRIMSON: Color = Color::new(220, 20, 60, 255);
    pub const RED: Color = Color::new(255, 0, 0, 255);
    pub const FIREBRICK: Color = Color::new(178, 34, 34, 255);
    pub const DARKRED: Color = Color::new(139, 0, 0, 255);
    pub const PINK: Color = Color::new(255, 192, 203, 255);
    pub const LIGHTPINK: Color = Color::new(255, 182, 193, 255);
    pub const HOTPINK: Color = Color::new(255, 105, 180, 255);
    pub const DEEPPINK: Color = Color::new(255, 20, 147, 255);
    pub const MEDIUMVIOLETRED: Color = Color::new(199, 21, 133, 255);
    pub const PALEVIOLETRED: Color = Color::new(219, 112, 147, 255);
    pub const CORAL: Color = Color::new(255, 127, 80, 255);
    pub const TOMATO: Color = Color::new(255, 99, 71, 255);
    pub const ORANGERED: Color = Color::new(255, 69, 0, 255);
    pub const DARKORANGE: Color = Color::new(255, 140, 0, 255);
    pub const ORANGE: Color = Color::new(255, 165, 0, 255);
    pub const GOLD: Color = Color::new(255, 215, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0, 255);
    pub const LIGHTYELLOW: Color = Color::new(255, 255, 224, 255);
    pub const LEMONCHIFFON: Color = Color::new(255, 250, 205, 255);
    pub const LIGHTGOLDENRODYELLOW: Color = Color::new(250, 250, 210, 255);
    pub const PAPAYAWHIP: Color = Color::new(255, 239, 213, 255);
    pub const MOCCASIN: Color = Color::new(255, 228, 181, 255);
    pub const PEACHPUFF: Color = Color::new(255, 218, 185, 255);
    pub const PALEGOLDENROD: Color = Color::new(238, 232, 170, 255);
    pub const KHAKI: Color = Color::new(240, 230, 140, 255);
    pub const DARKKHAKI: Color = Color::new(189, 183, 107, 255);
    pub const LAVENDER: Color = Color::new(230, 230, 250, 255);
    pub const THISTLE: Color = Color::new(216, 191, 216, 255);
    pub const PLUM: Color = Color::new(221, 160, 221, 255);
    pub const VIOLET: Color = Color::new(238, 130, 238, 255);
    pub const ORCHID: Color = Color::new(218, 112, 214, 255);
    pub const FUCHSIA: Color = Color::new(255, 0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255, 255);
    pub const MEDIUMORCHID: Color = Color::new(186, 85, 211, 255);
    pub const MEDIUMPURPLE: Color = Color::new(147, 112, 219, 255);
    pub const REBECCAPURPLE: Color = Color::new(102, 51, 153, 255);
    pub const BLUEVIOLET: Color = Color::new(138, 43, 226, 255);
    pub const DARKVIOLET: Color = Color::new(148, 0, 211, 255);
    pub const DARKORCHID: Color = Color::new(153, 50, 204, 255);
    pub const DARKMAGENTA: Color = Color::new(139, 0, 139, 255);
    pub const PURPLE: Color = Color::new(128, 0, 128, 255);
    pub const DARKPURPLE: Color = Color::new(112, 31, 126, 255);
    pub const INDIGO: Color = Color::new(75, 0, 130, 255);
    pub const SLATEBLUE: Color = Color::new(106, 90, 205, 255);
    pub const DARKSLATEBLUE: Color = Color::new(72, 61, 139, 255);
    pub const MEDIUMSLATEBLUE: Color = Color::new(123, 104, 238, 255);
    pub const GREENYELLOW: Color = Color::new(173, 255, 47, 255);
    pub const CHARTREUSE: Color = Color::new(127, 255, 0, 255);
    pub const LAWNGREEN: Color = Color::new(124, 252, 0, 255);
    pub const LIME: Color = Color::new(0, 255, 0, 255);
    pub const LIMEGREEN: Color = Color::new(50, 205, 50, 255);
    pub const PALEGREEN: Color = Color::new(152, 251, 152, 255);
    pub const LIGHTGREEN: Color = Color::new(144, 238, 144, 255);
    pub const MEDIUMSPRINGGREEN: Color = Color::new(0, 250, 154, 255);
    pub const SPRINGGREEN: Color = Color::new(0, 255, 127, 255);
    pub const MEDIUMSEAGREEN: Color = Color::new(60, 179, 113, 255);
    pub const SEAGREEN: Color = Color::new(46, 139, 87, 255);
    pub const FORESTGREEN: Color = Color::new(34, 139, 34, 255);
    pub const GREEN: Color = Color::new(0, 128, 0, 255);
    pub const DARKGREEN: Color = Color::new(0, 100, 0, 255);
    pub const YELLOWGREEN: Color = Color::new(154, 205, 50, 255);
    pub const OLIVEDRAB: Color = Color::new(107, 142, 35, 255);
    pub const OLIVE: Color = Color::new(128, 128, 0, 255);
    pub const DARKOLIVEGREEN: Color = Color::new(85, 107, 47, 255);
    pub const MEDIUMAQUAMARINE: Color = Color::new(102, 205, 170, 255);
    pub const DARKSEAGREEN: Color = Color::new(143, 188, 139, 255);
    pub const LIGHTSEAGREEN: Color = Color::new(32, 178, 170, 255);
    pub const DARKCYAN: Color = Color::new(0, 139, 139, 255);
    pub const TEAL: Color = Color::new(0, 128, 128, 255);
    pub const AQUA: Color = Color::new(0, 255, 255, 255);
    pub const CYAN: Color = Color::new(0, 255, 255, 255);
    pub const LIGHTCYAN: Color = Color::new(224, 255, 255, 255);
    pub const PALETURQUOISE: Color = Color::new(175, 238, 238, 255);
    pub const AQUAMARINE: Color = Color::new(127, 255, 212, 255);
    pub const TURQUOISE: Color = Color::new(64, 224, 208, 255);
    pub const MEDIUMTURQUOISE: Color = Color::new(72, 209, 204, 255);
    pub const DARKTURQUOISE: Color = Color::new(0, 206, 209, 255);
    pub const CADETBLUE: Color = Color::new(95, 158, 160, 255);
    pub const STEELBLUE: Color = Color::new(70, 130, 180, 255);
    pub const LIGHTSTEELBLUE: Color = Color::new(176, 196, 222, 255);
    pub const POWDERBLUE: Color = Color::new(176, 224, 230, 255);
    pub const LIGHTBLUE: Color = Color::new(173, 216, 230, 255);
    pub const SKYBLUE: Color = Color::new(135, 206, 235, 255);
    pub const LIGHTSKYBLUE: Color = Color::new(135, 206, 250, 255);
    pub const DEEPSKYBLUE: Color = Color::new(0, 191, 255, 255);
    pub const DODGERBLUE: Color = Color::new(30, 144, 255, 255);
    pub const CORNFLOWERBLUE: Color = Color::new(100, 149, 237, 255);
    pub const ROYALBLUE: Color = Color::new(65, 105, 225, 255);
    pub const BLUE: Color = Color::new(0, 0, 255, 255);
    pub const MEDIUMBLUE: Color = Color::new(0, 0, 205, 255);
    pub const DARKBLUE: Color = Color::new(0, 0, 139, 255);
    pub const NAVY: Color = Color::new(0, 0, 128, 255);
    pub const MIDNIGHTBLUE: Color = Color::new(25, 25, 112, 255);
    pub const CORNSILK: Color = Color::new(255, 248, 220, 255);
    pub const BLANCHEDALMOND: Color = Color::new(255, 235, 205, 255);
    pub const BISQUE: Color = Color::new(255, 228, 196, 255);
    pub const NAVAJOWHITE: Color = Color::new(255, 222, 173, 255);
    pub const WHEAT: Color = Color::new(245, 222, 179, 255);
    pub const BURLYWOOD: Color = Color::new(222, 184, 135, 255);
    pub const TAN: Color = Color::new(210, 180, 140, 255);
    pub const ROSYBROWN: Color = Color::new(188, 143, 143, 255);
    pub const SANDYBROWN: Color = Color::new(244, 164, 96, 255);
    pub const GOLDENROD: Color = Color::new(218, 165, 32, 255);
    pub const DARKGOLDENROD: Color = Color::new(184, 134, 11, 255);
    pub const PERU: Color = Color::new(205, 133, 63, 255);
    pub const CHOCOLATE: Color = Color::new(210, 105, 30, 255);
    pub const SADDLEBROWN: Color = Color::new(139, 69, 19, 255);
    pub const SIENNA: Color = Color::new(160, 82, 45, 255);
    pub const BROWN: Color = Color::new(165, 42, 42, 255);
    pub const DARKBROWN: Color = Color::new(76, 63, 47, 255);
    pub const MAROON: Color = Color::new(128, 0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    pub const SNOW: Color = Color::new(255, 250, 250, 255);
    pub const HONEYDEW: Color = Color::new(240, 255, 240, 255);
    pub const MINTCREAM: Color = Color::new(245, 255, 250, 255);
    pub const AZURE: Color = Color::new(240, 255, 255, 255);
    pub const ALICEBLUE: Color = Color::new(240, 248, 255, 255);
    pub const GHOSTWHITE: Color = Color::new(248, 248, 255, 255);
    pub const WHITESMOKE: Color = Color::new(245, 245, 245, 255);
    pub const SEASHELL: Color = Color::new(255, 245, 238, 255);
    pub const BEIGE: Color = Color::new(245, 245, 220, 255);
    pub const OLDLACE: Color = Color::new(253, 245, 230, 255);
    pub const FLORALWHITE: Color = Color::new(255, 250, 240, 255);
    pub const IVORY: Color = Color::new(255, 255, 240, 255);
    pub const ANTIQUEWHITE: Color = Color::new(250, 235, 215, 255);
    pub const LINEN: Color = Color::new(250, 240, 230, 255);
    pub const LAVENDERBLUSH: Color = Color::new(255, 240, 245, 255);
    pub const MISTYROSE: Color = Color::new(255, 228, 225, 255);
    pub const GAINSBORO: Color = Color::new(220, 220, 220, 255);
    pub const LIGHTGRAY: Color = Color::new(211, 211, 211, 255);
    pub const SILVER: Color = Color::new(192, 192, 192, 255);
    pub const DARKGRAY: Color = Color::new(169, 169, 169, 255);
    pub const GRAY: Color = Color::new(128, 128, 128, 255);
    pub const DIMGRAY: Color = Color::new(105, 105, 105, 255);
    pub const LIGHTSLATEGRAY: Color = Color::new(119, 136, 153, 255);
    pub const SLATEGRAY: Color = Color::new(112, 128, 144, 255);
    pub const DARKSLATEGRAY: Color = Color::new(47, 79, 79, 255);
    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const BLANK: Color = Color::new(0, 0, 0, 0);
    pub const RAYWHITE: Color = Color::new(245, 245, 245, 255);

    pub const DEEPGRAY: Color = Color::new(24, 24, 24, 255);
    pub const DEEPGRAY2: Color = Color::new(51, 51, 51, 255);
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_i32s(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }
}

pub struct Font {
    c_font: C_Font,
    spacing: f32,
}

impl Font {
    pub fn load_ttf_from_memory(data: &[u8], size: i32, spacing: f32) -> Self {
        let data_ptr = data.as_ptr() as *const c_char;
        let data_len = data.len() as i32;
        let cstr = CString::new(".ttf").unwrap().into_raw();

        let cp_text = CString::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789áàãâéêóôõíúüçÁÀÃÂÉÊÓÔÕÍÚÜÇ").unwrap().into_raw();

        let c_font =
            unsafe {
                let mut cp_count: c_int = 0;

                let cp = LoadCodepoints(cp_text, &mut cp_count);
                let f = LoadFontFromMemory(cstr, data_ptr, data_len, size, cp, cp_count);

                UnloadCodepoints(cp);
                f
            };

        Self {
            c_font: c_font,
            spacing,
        }
    }

    pub fn draw_text(&self, text: &str, x: f32, y: f32, tint: Color) {
        let cstr = CString::new(text).unwrap().into_raw();
        let size = self.c_font.base_size as f32;

        unsafe {
            DrawTextEx(
                self.c_font,
                cstr,
                Vector2::new(x, y),
                size,
                self.spacing,
                tint,
            )
        };
    }

    pub fn measure_text(&self, text: &str) -> f32 {
        let cstr = CString::new(text).unwrap().into_raw();
        let size = self.c_font.base_size as f32;

        unsafe { MeasureTextEx(self.c_font, cstr, size, self.spacing).x }
    }
}

pub fn is_key_pressed(key: KeyboardKey) -> bool {
    unsafe { IsKeyPressed(key as c_int) }
}

pub fn is_key_down(key: KeyboardKey) -> bool {
    unsafe { IsKeyDown(key as c_int) }
}

pub fn is_key_pressed_repeat(key: KeyboardKey) -> bool {
    unsafe { IsKeyPressedRepeat(key as c_int) }
}

pub fn is_key_pressed_or_repeated(key: KeyboardKey) -> bool {
    unsafe { IsKeyPressed(key as c_int) || IsKeyPressedRepeat(key as c_int) }
}

pub fn get_char_pressed() -> Option<char> {
    unsafe {
        let c = GetCharPressed();

        if c == 0 {
            None
        } else {
            char::from_u32(c as u32)
        }
    }
}

pub fn set_exit_key(key: KeyboardKey) {
    unsafe { SetExitKey(key as i32) };
}
