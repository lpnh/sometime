use crate::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Clock,
    Calendar,
}

impl std::str::FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clock" => Ok(Self::Clock),
            "calendar" => Ok(Self::Calendar),
            _ => Err(format!("Unknown command: {}", s)),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clock => write!(f, "clock"),
            Self::Calendar => write!(f, "calendar"),
        }
    }
}

impl From<Command> for View {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Clock => Self::Clock,
            Command::Calendar => Self::Calendar,
        }
    }
}
