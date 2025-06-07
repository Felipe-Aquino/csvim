mod csv;
mod rl;

struct Buffer {
    data: [u8; 64],
    size: usize,
}

impl Buffer {
    fn new() -> Self {
        let data: [u8; 64] = [0; 64];

        Self { data, size: 0 }
    }

    fn write_u32(&mut self, value: u32) {
        self.size = 0;

        let mut v = value;

        loop {
            let c = '0' as u8 + (v % 10) as u8;

            self.data[self.size] = c;
            self.size += 1;

            v = v / 10;

            if v == 0 {
                break;
            }
        }

        for i in 0..self.size / 2 {
            let aux = self.data[i];
            self.data[i] = self.data[self.size - 1 - i];
            self.data[self.size - i - 1] = aux;
        }
    }

    fn write_letters_base26(&mut self, value: u32) {
        self.size = 0;

        let mut v = value;

        loop {
            let c = 'A' as u8 + (v % 26) as u8;

            self.data[self.size] = c;
            self.size += 1;

            v = v / 26;

            if v == 0 {
                break;
            }

            v -= 1;
        }

        for i in 0..self.size / 2 {
            let aux = self.data[i];
            self.data[i] = self.data[self.size - 1 - i];
            self.data[self.size - i - 1] = aux;
        }
    }

    fn write_str(&mut self, s: &str) {
        for (i, c) in s.chars().enumerate() {
            if i >= self.data.len() {
                break;
            }

            self.data[i] = c as u8;
        }

        self.size = s.len().min(self.data.len());
    }

    fn as_str(&self) -> &str {
        if self.size == 0 {
            ""
        } else {
            str::from_utf8(&self.data[0..self.size]).unwrap()
        }
    }
}

const CELL_DEFAULT_HEIGHT: i32 = 20;
const CELL_DEFAULT_WIDTH: i32 = 215;

const CELL_PAD: i32 = 3;

const font_data: &[u8; 96964] = include_bytes!("../Inconsolata-Regular.ttf");
const bold_font_data: &[u8; 102148] = include_bytes!("../Inconsolata-Bold.ttf");

fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: csvim FILENAME");
        return;
    }

    let filepath = std::env::args().nth(1).unwrap();

    let csv = csv::read_csv_file_as_hashmap(&filepath, ',', csv::Delimiter::DoubleQuote);

    rl::set_config_flags(0x00000004 | 0x00000400);
    rl::init_window(200, 200, "csvim");

    rl::set_target_fps(30);

    let font16 = rl::Font::load_ttf_from_memory(font_data, 16, 1.0);
    let font16bold = rl::Font::load_ttf_from_memory(bold_font_data, 16, 1.0);

    let top_headers_height: i32 = 2 * CELL_DEFAULT_HEIGHT;
    let left_headers_width: i32 = 100 / 2;

    let mut column_offset: i32 = 0;
    let mut row_offset: i32 = 0;

    let mut current_cell_row: i32 = 0;
    let mut current_cell_col: i32 = 0;

    let mut buffer = Buffer::new();

    while !rl::window_should_close() {
        let screen_width = rl::get_screen_width();
        let screen_height = rl::get_screen_height();

        let column_count = (screen_width - left_headers_width) / CELL_DEFAULT_WIDTH;
        let row_count = (screen_height - top_headers_height) / CELL_DEFAULT_HEIGHT;

        rl::begin_drawing();

        rl::clear_background(rl::Color::DEEPGRAY);

        if rl::is_key_pressed_or_repeat(rl::KeyboardKey::H) {
            current_cell_col = (current_cell_col - 1).max(0);

            if current_cell_col < column_offset {
                column_offset -= 1;
            }
        } else if rl::is_key_pressed_or_repeat(rl::KeyboardKey::L) {
            current_cell_col += 1;

            if current_cell_col - column_offset > column_count - 1 {
                column_offset += 1;
            }

            println!("{current_cell_col}, {column_offset}");
        } else if rl::is_key_pressed_or_repeat(rl::KeyboardKey::J) {
            current_cell_row += 1;

            if current_cell_row - row_offset > row_count - 1 {
                row_offset += 1;
            }
        } else if rl::is_key_pressed_or_repeat(rl::KeyboardKey::K) {
            current_cell_row = (current_cell_row - 1).max(0);

            if current_cell_row < row_offset {
                row_offset -= 1;
            }
        }

        rl::draw_rectangle(0, 0, screen_width, top_headers_height, rl::Color::DEEPGRAY2);
        rl::draw_rectangle(
            0,
            0,
            left_headers_width,
            screen_height,
            rl::Color::DEEPGRAY2,
        );

        let start_x = left_headers_width;
        let start_y = top_headers_height;

        rl::draw_rectangle(
            start_x + (current_cell_col - column_offset) * CELL_DEFAULT_WIDTH,
            start_y + (current_cell_row - row_offset) * CELL_DEFAULT_HEIGHT,
            CELL_DEFAULT_WIDTH,
            CELL_DEFAULT_HEIGHT,
            rl::Color::DARKSEAGREEN,
        );

        for i in 0..(column_count + 1) {
            let x = start_x + i * CELL_DEFAULT_WIDTH;

            buffer.write_letters_base26((column_offset + i) as u32);

            let w = font16.measure_text(buffer.as_str());
            font16.draw_text(
                buffer.as_str(),
                x as f32 + (CELL_DEFAULT_WIDTH as f32 - w) / 2.0,
                ((top_headers_height - 16) / 2) as f32,
                rl::Color::RAYWHITE,
            );

            rl::draw_vertical_line(x, 0, screen_height, rl::Color::RAYWHITE);
        }

        for i in 0..(row_count + 1) {
            let y = start_y + i * CELL_DEFAULT_HEIGHT;

            buffer.write_u32((row_offset + i + 1) as u32);

            let w = font16.measure_text(buffer.as_str());
            font16.draw_text(
                buffer.as_str(),
                (left_headers_width as f32 - w) / 2.0,
                (y + (CELL_DEFAULT_HEIGHT - 16) / 2) as f32,
                rl::Color::RAYWHITE,
            );

            rl::draw_horizonal_line(y, 0, screen_width, rl::Color::RAYWHITE);
        }

        for j in 0..(row_count + 1) {
            for i in 0..(column_count + 1) {
                let pos = ((row_offset + j) as usize, (column_offset + i) as usize);

                if let Some(value) = csv.map.get(&pos) {
                    let x = start_x + i * CELL_DEFAULT_WIDTH + CELL_PAD;
                    let y = start_y + j * CELL_DEFAULT_HEIGHT;

                    buffer.write_str(value.as_str());

                    rl::begin_scissor_mode(
                        x,
                        y,
                        CELL_DEFAULT_WIDTH - 2 * CELL_PAD,
                        CELL_DEFAULT_HEIGHT,
                    );
                    font16.draw_text(
                        buffer.as_str(),
                        x as f32,
                        (y + (CELL_DEFAULT_HEIGHT - 16) / 2) as f32,
                        rl::Color::RAYWHITE,
                    );
                    rl::end_scissor_mode();
                }
            }
        }

        let pos = (current_cell_row as usize, current_cell_col as usize);

        if let Some(value) = csv.map.get(&pos) {
            let x = start_x + (current_cell_col - column_offset) * CELL_DEFAULT_WIDTH + CELL_PAD;
            let y = start_y + (current_cell_row - row_offset) * CELL_DEFAULT_HEIGHT;

            buffer.write_str(value.as_str());

            rl::begin_scissor_mode(x, y, CELL_DEFAULT_WIDTH - 2 * CELL_PAD, CELL_DEFAULT_HEIGHT);
            font16bold.draw_text(
                buffer.as_str(),
                x as f32,
                (y + (CELL_DEFAULT_HEIGHT - 16) / 2) as f32,
                rl::Color::BLACK,
            );
            rl::end_scissor_mode();
        }

        rl::end_drawing();
    }

    rl::close_window();
}
