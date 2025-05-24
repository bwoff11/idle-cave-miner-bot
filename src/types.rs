#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<Position> for (i32, i32) {
    fn from(pos: Position) -> Self {
        (pos.x, pos.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Upgrades,
    Souls,
    Prestige,
}

impl TaskType {
    pub fn name(&self) -> &'static str {
        match self {
            TaskType::Upgrades => "Upgrades",
            TaskType::Souls => "Souls",
            TaskType::Prestige => "Prestige",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TaskType::Upgrades => "🔧",
            TaskType::Souls => "👻",
            TaskType::Prestige => "⭐",
        }
    }
}