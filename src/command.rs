use crate::{Event, View};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Clock,
    Calendar,
    Dismiss,
}

impl std::str::FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clock" => Ok(Self::Clock),
            "calendar" => Ok(Self::Calendar),
            "dismiss" => Ok(Self::Dismiss),
            _ => Err(format!("Unknown command: {}", s)),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clock => write!(f, "clock"),
            Self::Calendar => write!(f, "calendar"),
            Self::Dismiss => write!(f, "dismiss"),
        }
    }
}

impl From<Command> for Event {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Clock => Event::Toggle(View::Clock),
            Command::Calendar => Event::Toggle(View::Calendar),
            Command::Dismiss => Event::Quit,
        }
    }
}
