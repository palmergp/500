//! Seats and partnerships (H1).

use std::fmt;

/// A seat at the table. Play goes clockwise: South → West → North → East.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Seat(u8);

impl Seat {
    pub const SOUTH: Seat = Seat(0);
    pub const WEST: Seat = Seat(1);
    pub const NORTH: Seat = Seat(2);
    pub const EAST: Seat = Seat(3);
    pub const ALL: [Seat; 4] = [Seat::SOUTH, Seat::WEST, Seat::NORTH, Seat::EAST];

    /// Seat `index` (taken modulo 4).
    pub fn new(index: usize) -> Seat {
        Seat((index % 4) as u8)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// The player to this seat's left, who acts next.
    pub fn next(self) -> Seat {
        Seat::new(self.index() + 1)
    }

    pub fn partner(self) -> Seat {
        Seat::new(self.index() + 2)
    }

    pub fn team(self) -> Team {
        if self.0.is_multiple_of(2) {
            Team::NorthSouth
        } else {
            Team::EastWest
        }
    }

    pub fn name(self) -> &'static str {
        ["South", "West", "North", "East"][self.index()]
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.name())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Team {
    NorthSouth,
    EastWest,
}

impl Team {
    pub const ALL: [Team; 2] = [Team::NorthSouth, Team::EastWest];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn other(self) -> Team {
        match self {
            Team::NorthSouth => Team::EastWest,
            Team::EastWest => Team::NorthSouth,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Team::NorthSouth => "North–South",
            Team::EastWest => "East–West",
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.name())
    }
}
