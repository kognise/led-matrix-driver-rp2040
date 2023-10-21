// Depends on the performance of the display. Higher values result in a
// larger color space, but also a lot more flickering.
pub const COLOR_BITMASK: u32 = 0b00001111;

pub const PHYSICAL_WIDTH: usize = 128;
pub const PHYSICAL_HEIGHT: usize = 32;
pub const VIRTUAL_WIDTH: usize = 64;
pub const VIRTUAL_HEIGHT: usize = 64;

pub const fn physical_to_virtual(physical_x: usize, physical_y: usize) -> (usize, usize) {
	if physical_x < VIRTUAL_WIDTH {
		(
			VIRTUAL_WIDTH - physical_x - 1,
			PHYSICAL_HEIGHT - physical_y - 1,
		)
	} else {
		(physical_x - VIRTUAL_WIDTH, physical_y + PHYSICAL_HEIGHT)
	}
}