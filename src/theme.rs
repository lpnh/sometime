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

pub trait Theme {
    const FACE: Bgra;
    const FRAME: Bgra;
    const HANDS: Bgra;
}

pub struct CatppuccinMocha;

impl Theme for CatppuccinMocha {
    const FACE: Bgra = Bgra::from_rgba(30, 30, 46, 216); // Base
    const FRAME: Bgra = Bgra::from_rgba(49, 50, 68, 208); // Surface0
    const HANDS: Bgra = Bgra::from_rgba(203, 166, 247, 208); // Mauve
}
