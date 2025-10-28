#[derive(Debug, Clone, Copy)]
pub struct Bgra([u8; 4]);

impl Bgra {
    const fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([blue, green, red, alpha])
    }

    #[inline]
    pub const fn r(self) -> u8 {
        self.0[2]
    }

    #[inline]
    pub const fn g(self) -> u8 {
        self.0[1]
    }

    #[inline]
    pub const fn b(self) -> u8 {
        self.0[0]
    }

    #[inline]
    pub const fn a(self) -> u8 {
        self.0[3]
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
            background: Bgra::from_rgba(24, 24, 37, 216),   // Mantle
            frame: Bgra::from_rgba(49, 50, 68, 216),        // Surface0
            primary: Bgra::from_rgba(235, 160, 172, 216),   // Maroon
            secondary: Bgra::from_rgba(245, 194, 231, 216), // Pink
        }
    }
}
