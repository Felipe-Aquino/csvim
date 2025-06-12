pub mod csv;
pub mod rl;

struct Buffer {
    data: [u8; 64],
    size: usize,
}

impl Buffer {
    fn new() -> Self {
        let data: [u8; 64] = [0; 64];

        Self { data, size: 0 }
    }

    fn write_u32(&mut self, value: u32, overwrite: bool) {
        if overwrite {
            self.size = 0;
        }

        let start = self.size;
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

        for i in start..(start + self.size) / 2 {
            let aux = self.data[i];
            self.data[i] = self.data[self.size - 1 + start - i];
            self.data[self.size - i + start - 1] = aux;
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

const CELL_DEFAULT_WIDTH: i32 = 215;

const CELL_PAD: i32 = 3;

const FONT_DATA: &[u8; 96964] = include_bytes!("../Inconsolata-Regular.ttf");
const BOLD_FONT_DATA: &[u8; 102148] = include_bytes!("../Inconsolata-Bold.ttf");

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

    let mut cell_default_height: i32 = 20;

    let mut font_size = 16;
    let mut font = rl::Font::load_ttf_from_memory(FONT_DATA, font_size, 1.0);
    let mut font_bold = rl::Font::load_ttf_from_memory(BOLD_FONT_DATA, font_size, 1.0);

    let top_headers_height: i32 = 3 * cell_default_height / 2;
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
        let row_count =
            (screen_height - top_headers_height - cell_default_height) / cell_default_height;

        rl::begin_drawing();

        rl::clear_background(rl::Color::DEEPGRAY);

        if rl::is_key_pressed_or_repeated(rl::KeyboardKey::H) {
            current_cell_col = (current_cell_col - 1).max(0);

            if current_cell_col < column_offset {
                column_offset -= 1;
            }
        } else if rl::is_key_pressed_or_repeated(rl::KeyboardKey::L) {
            current_cell_col += 1;

            if current_cell_col - column_offset > column_count - 1 {
                column_offset += 1;
            }
        } else if rl::is_key_pressed_or_repeated(rl::KeyboardKey::J) {
            current_cell_row += 1;

            if current_cell_row - row_offset > row_count - 1 {
                row_offset += 1;
            }
        } else if rl::is_key_pressed_or_repeated(rl::KeyboardKey::K) {
            current_cell_row = (current_cell_row - 1).max(0);

            if current_cell_row < row_offset {
                row_offset -= 1;
            }
        } else if rl::is_key_pressed(rl::KeyboardKey::Z) {
            cell_default_height = cell_default_height + 4;

            if cell_default_height > 28 {
                cell_default_height = 20;
            }

            font_size = cell_default_height - 4;
            font = rl::Font::load_ttf_from_memory(FONT_DATA, font_size, 1.0);
            font_bold =
                rl::Font::load_ttf_from_memory(BOLD_FONT_DATA, font_size, 1.0);
            let new_column_count = (screen_width - left_headers_width) / CELL_DEFAULT_WIDTH;
            let new_row_count =
                (screen_height - top_headers_height - cell_default_height) / cell_default_height;

            if current_cell_row < row_offset {
                current_cell_row = row_offset;
            } else if current_cell_row > row_offset + new_row_count {
                current_cell_row = row_offset + new_row_count - 1;
            }

            if current_cell_col < column_offset {
                current_cell_col = column_offset;
            } else if current_cell_col > column_offset + new_column_count {
                current_cell_col = column_offset + new_column_count - 1;
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
            start_y + (current_cell_row - row_offset) * cell_default_height,
            CELL_DEFAULT_WIDTH,
            cell_default_height,
            rl::Color::DARKSEAGREEN,
        );

        for i in 0..(column_count + 1) {
            let x = start_x + i * CELL_DEFAULT_WIDTH;

            buffer.write_letters_base26((column_offset + i) as u32);

            let w = font.measure_text(buffer.as_str());
            font.draw_text(
                buffer.as_str(),
                x as f32 + (CELL_DEFAULT_WIDTH as f32 - w) / 2.0,
                ((top_headers_height - 16) / 2) as f32,
                rl::Color::RAYWHITE,
            );

            rl::draw_vertical_line(x, 0, screen_height, rl::Color::RAYWHITE);
        }

        for i in 0..(row_count + 1) {
            let y = start_y + i * cell_default_height;

            buffer.write_u32((row_offset + i + 1) as u32, true);

            let w = font.measure_text(buffer.as_str());
            font.draw_text(
                buffer.as_str(),
                (left_headers_width as f32 - w) / 2.0,
                (y + 2) as f32,
                rl::Color::RAYWHITE,
            );

            rl::draw_horizonal_line(y, 0, screen_width, rl::Color::RAYWHITE);
        }

        for j in 0..(row_count + 1) {
            for i in 0..(column_count + 1) {
                let pos = ((row_offset + j) as usize, (column_offset + i) as usize);

                if let Some(value) = csv.map.get(&pos) {
                    let x = start_x + i * CELL_DEFAULT_WIDTH + CELL_PAD;
                    let y = start_y + j * cell_default_height;

                    buffer.write_str(value.as_str());

                    rl::begin_scissor_mode(
                        x,
                        y,
                        CELL_DEFAULT_WIDTH - 2 * CELL_PAD,
                        cell_default_height,
                    );
                    font.draw_text(
                        buffer.as_str(),
                        x as f32,
                        (y + 2) as f32,
                        rl::Color::RAYWHITE,
                    );
                    rl::end_scissor_mode();
                }
            }
        }

        rl::draw_rectangle(
            0,
            screen_height - cell_default_height,
            screen_width,
            cell_default_height,
            rl::Color::DEEPGRAY2,
        );
        rl::draw_rectangle_lines(
            0,
            screen_height - cell_default_height,
            screen_width + 2,
            cell_default_height + 2,
            rl::Color::DIMGRAY,
        );

        buffer.write_letters_base26(current_cell_col as u32);
        buffer.write_u32((current_cell_row + 1) as u32, false);

        let w = font.measure_text(buffer.as_str());

        font_bold.draw_text(
            buffer.as_str(),
            screen_width as f32 - w - 5.0,
            (screen_height - cell_default_height + 2) as f32,
            rl::Color::WHITE,
        );

        let pos = (current_cell_row as usize, current_cell_col as usize);

        if let Some(value) = csv.map.get(&pos) {
            let x = start_x + (current_cell_col - column_offset) * CELL_DEFAULT_WIDTH + CELL_PAD;
            let y = start_y + (current_cell_row - row_offset) * cell_default_height;

            buffer.write_str(value.as_str());

            rl::begin_scissor_mode(x, y, CELL_DEFAULT_WIDTH - 2 * CELL_PAD, cell_default_height);
            font_bold.draw_text(
                buffer.as_str(),
                x as f32,
                (y + 2) as f32,
                rl::Color::BLACK,
            );
            rl::end_scissor_mode();

            let x = CELL_PAD;
            let y = screen_height - cell_default_height + 2;

            rl::begin_scissor_mode(x, y, screen_width - 2 * CELL_PAD, cell_default_height);
            font_bold.draw_text(
                buffer.as_str(),
                x as f32,
                y as f32,
                rl::Color::WHITE,
            );
            rl::end_scissor_mode();

        }

        rl::end_drawing();
    }

    rl::close_window();
}
