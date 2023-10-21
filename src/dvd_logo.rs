use crate::display::Color;

pub const WIDTH: usize = 36;
pub const HEIGHT: usize = 16;

const LOGO_BASE: [bool; WIDTH * HEIGHT] = [
    false, false, false, true, true, true, true, true, true, true, true, true, true, true, true,
    true, false, false, false, false, false, false, true, true, true, true, true, true, true, true,
    true, true, false, false, false, false, false, false, false, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, false, false, false, false, false, true,
    true, true, true, true, true, true, true, true, true, true, true, false, false, false, false,
    false, false, false, false, false, false, false, true, true, true, true, true, true, true,
    true, false, false, false, false, true, true, true, true, false, false, false, false, false,
    false, true, true, true, true, true, false, false, true, true, true, true, false, false, false,
    false, true, true, false, true, true, true, true, true, false, false, true, true, true, true,
    false, true, true, true, false, false, false, false, true, true, true, true, false, false,
    true, true, true, false, false, false, false, false, true, true, true, false, true, true, true,
    true, false, false, true, true, true, false, false, true, true, true, false, false, false,
    false, true, true, true, true, false, false, true, true, true, false, false, false, false,
    true, true, true, true, false, false, true, true, true, true, true, true, true, false, false,
    true, true, true, true, false, false, false, true, true, true, true, false, false, true, true,
    true, true, false, false, false, true, true, true, true, false, false, false, true, true, true,
    true, true, true, true, false, false, true, true, true, true, false, false, false, true, true,
    true, false, false, false, true, true, true, true, true, true, true, true, true, true, false,
    false, false, false, false, true, true, true, true, true, false, false, false, true, true,
    true, true, true, true, true, true, true, false, false, false, true, true, true, true, true,
    true, true, true, false, false, false, false, false, false, false, false, true, true, true,
    true, false, false, false, true, true, true, true, true, true, true, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, true, true, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, true, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, false, false, false, false,
    false, false, false, false, true, true, true, true, true, false, true, false, true, true,
    false, true, true, false, false, true, true, true, false, false, true, true, false, false,
    false, true, true, true, true, true, true, false, false, false, true, true, true, true, true,
    true, true, false, true, false, true, true, false, true, true, false, true, false, true, true,
    false, true, true, true, false, true, false, true, true, true, true, true, true, true, false,
    false, false, false, false, true, true, true, true, true, false, true, true, true, false, true,
    true, false, false, true, true, true, false, false, true, true, false, false, false, true,
    true, true, true, false, false, false, false, false, false, false, false, false, false, false,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, false, false, false, false, false, false, false,
];

pub fn make_dvd_logo(color: Color) -> [[Color; WIDTH]; HEIGHT] {
    let mut logo = [[Color::black(); WIDTH]; HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if LOGO_BASE[y * WIDTH + x] {
                logo[y][x] = color;
            }
        }
    }
    logo
}
