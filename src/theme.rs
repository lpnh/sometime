#[derive(Debug, Clone, Copy)]
pub struct Bgra([u8; 4]);

impl Bgra {
    pub const fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([blue, green, red, alpha])
    }
}

impl AsRef<[u8]> for Bgra {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Bgra,
    pub frame: Bgra,
    pub primary: Bgra,
    pub secondary: Bgra,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            background: Bgra::from_rgba(30, 30, 46, 210),   // Base
            frame: Bgra::from_rgba(49, 50, 68, 210),        // Surface0
            primary: Bgra::from_rgba(203, 166, 247, 210),   // Mauve
            secondary: Bgra::from_rgba(180, 190, 254, 210), // Lavender
        }
    }
}
