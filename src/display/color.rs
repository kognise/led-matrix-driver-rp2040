#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(u32);

impl Color {
	pub const fn from_hex(hex: u32) -> Self {
		Self(hex & 0xffffff)
	}

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
		Self((r as u32) << 16 | (g as u32) << 8 | b as u32)
	}

	pub const fn black() -> Self {
		Self(0)
	}

	pub const fn white() -> Self {
		Self(0xffffff)
	}

	pub const fn hex(&self) -> u32 {
		self.0
	}
}