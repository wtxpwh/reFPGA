use bincode::{Decode, Encode};
use prjcombine_types::enumerable::Enumerable;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum DirH {
    W,
    E,
}

impl DirH {
    pub const DIRS: [DirH; 2] = [DirH::W, DirH::E];
}

impl core::ops::Not for DirH {
    type Output = DirH;
    fn not(self) -> DirH {
        match self {
            DirH::W => DirH::E,
            DirH::E => DirH::W,
        }
    }
}

impl std::fmt::Display for DirH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DirH::W => "W",
                DirH::E => "E",
            }
        )
    }
}

impl Enumerable for DirH {
    type Array<T> = [T; 2];
    const VALUES: [DirH; 2] = DirH::DIRS;

    fn to_enum_idx(self) -> usize {
        match self {
            DirH::W => 0,
            DirH::E => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum DirV {
    S,
    N,
}

impl DirV {
    pub const DIRS: [DirV; 2] = [DirV::S, DirV::N];
}

impl core::ops::Not for DirV {
    type Output = DirV;
    fn not(self) -> DirV {
        match self {
            DirV::S => DirV::N,
            DirV::N => DirV::S,
        }
    }
}

impl std::fmt::Display for DirV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DirV::S => "S",
                DirV::N => "N",
            }
        )
    }
}

impl Enumerable for DirV {
    type Array<T> = [T; 2];
    const VALUES: [DirV; 2] = DirV::DIRS;

    fn to_enum_idx(self) -> usize {
        match self {
            DirV::S => 0,
            DirV::N => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum Dir {
    H(DirH),
    V(DirV),
}

impl Dir {
    pub const DIRS: [Dir; 4] = [Dir::W, Dir::E, Dir::S, Dir::N];

    pub const W: Dir = Dir::H(DirH::W);
    pub const E: Dir = Dir::H(DirH::E);
    pub const S: Dir = Dir::V(DirV::S);
    pub const N: Dir = Dir::V(DirV::N);
}

impl core::ops::Not for Dir {
    type Output = Dir;
    fn not(self) -> Dir {
        match self {
            Dir::W => Dir::E,
            Dir::E => Dir::W,
            Dir::S => Dir::N,
            Dir::N => Dir::S,
        }
    }
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Dir::W => "W",
                Dir::E => "E",
                Dir::S => "S",
                Dir::N => "N",
            }
        )
    }
}

impl From<DirH> for Dir {
    fn from(value: DirH) -> Self {
        Dir::H(value)
    }
}

impl From<DirV> for Dir {
    fn from(value: DirV) -> Self {
        Dir::V(value)
    }
}

impl Enumerable for Dir {
    type Array<T> = [T; 4];
    const VALUES: [Dir; 4] = Dir::DIRS;

    fn to_enum_idx(self) -> usize {
        match self {
            Dir::W => 0,
            Dir::E => 1,
            Dir::S => 2,
            Dir::N => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct DirHV {
    pub h: DirH,
    pub v: DirV,
}

impl DirHV {
    pub const fn new(h: DirH, v: DirV) -> Self {
        Self { h, v }
    }

    pub const DIRS: [DirHV; 4] = [DirHV::SW, DirHV::SE, DirHV::NW, DirHV::NE];

    pub const SW: DirHV = DirHV {
        h: DirH::W,
        v: DirV::S,
    };
    pub const SE: DirHV = DirHV {
        h: DirH::E,
        v: DirV::S,
    };
    pub const NW: DirHV = DirHV {
        h: DirH::W,
        v: DirV::N,
    };
    pub const NE: DirHV = DirHV {
        h: DirH::E,
        v: DirV::N,
    };

    pub fn with_h(self, h: DirH) -> DirHV {
        DirHV { h, ..self }
    }

    pub fn with_v(self, v: DirV) -> DirHV {
        DirHV { v, ..self }
    }
}

impl std::fmt::Display for DirHV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DirHV::SW => "SW",
                DirHV::SE => "SE",
                DirHV::NW => "NW",
                DirHV::NE => "NE",
            }
        )
    }
}

impl core::ops::BitAnd<DirV> for DirH {
    type Output = DirHV;

    fn bitand(self, rhs: DirV) -> Self::Output {
        DirHV { h: self, v: rhs }
    }
}

impl core::ops::BitAnd<DirH> for DirV {
    type Output = DirHV;

    fn bitand(self, rhs: DirH) -> Self::Output {
        DirHV { h: rhs, v: self }
    }
}

impl Enumerable for DirHV {
    type Array<T> = [T; 4];
    const VALUES: [DirHV; 4] = DirHV::DIRS;

    fn to_enum_idx(self) -> usize {
        match self {
            DirHV::SW => 0,
            DirHV::SE => 1,
            DirHV::NW => 2,
            DirHV::NE => 3,
        }
    }
}
