use std::collections::{BTreeMap, BTreeSet};

use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_entity::{EntityId, EntityRange};
use prjcombine_interconnect::{
    coord::{BelCoord, BelPadCoord, ColId, ColIdExt, DieId, DieIdExt, RowId, RowIdExt, TileCoord},
    db::IntDb,
    dir::{DirH, DirHV, DirV},
};

use crate::{xc2000, xc3000, xc4000, xc5200};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum SharedCfgPad {
    Addr(u8),
    Data(u8),
    Ldc,
    Hdc,
    RclkB,
    Dout,
    M2, // dedicated on XC4000
    // XC3000+
    InitB,
    Cs0B,
    Cs1B,
    // XC4000+
    Tck,
    Tdi,
    Tms,
    // XC5200 only
    Tdo,
    M0,
    M1,
}

impl std::fmt::Display for SharedCfgPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharedCfgPad::Addr(i) => write!(f, "A{i}"),
            SharedCfgPad::Data(i) => write!(f, "D{i}"),
            SharedCfgPad::Ldc => write!(f, "LDC"),
            SharedCfgPad::Hdc => write!(f, "HDC"),
            SharedCfgPad::RclkB => write!(f, "RCLK_B"),
            SharedCfgPad::Dout => write!(f, "DOUT"),
            SharedCfgPad::M2 => write!(f, "M2"),
            SharedCfgPad::InitB => write!(f, "INIT_B"),
            SharedCfgPad::Cs0B => write!(f, "CS0_B"),
            SharedCfgPad::Cs1B => write!(f, "CS1_B"),
            SharedCfgPad::Tck => write!(f, "TCK"),
            SharedCfgPad::Tdi => write!(f, "TDI"),
            SharedCfgPad::Tms => write!(f, "TMS"),
            SharedCfgPad::Tdo => write!(f, "TDO"),
            SharedCfgPad::M0 => write!(f, "M0"),
            SharedCfgPad::M1 => write!(f, "M1"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum ChipKind {
    Xc2000,
    Xc3000,
    Xc3000A,
    // plain, D (no memory)
    Xc4000,
    Xc4000A,
    Xc4000H,
    // E, L, Spartan
    Xc4000E,
    // EX, XL
    Xc4000Ex,
    Xc4000Xla,
    Xc4000Xv,
    SpartanXl,
    Xc5200,
}

impl std::fmt::Display for ChipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipKind::Xc2000 => write!(f, "xc2000"),
            ChipKind::Xc3000 => write!(f, "xc3000"),
            ChipKind::Xc3000A => write!(f, "xc3000a"),
            ChipKind::Xc4000 => write!(f, "xc4000"),
            ChipKind::Xc4000A => write!(f, "xc4000a"),
            ChipKind::Xc4000H => write!(f, "xc4000h"),
            ChipKind::Xc4000E => write!(f, "xc4000e"),
            ChipKind::Xc4000Ex => write!(f, "xc4000ex"),
            ChipKind::Xc4000Xla => write!(f, "xc4000xla"),
            ChipKind::Xc4000Xv => write!(f, "xc4000xv"),
            ChipKind::SpartanXl => write!(f, "spartanxl"),
            ChipKind::Xc5200 => write!(f, "xc5200"),
        }
    }
}

impl ChipKind {
    pub fn is_xc3000(self) -> bool {
        matches!(self, Self::Xc3000 | Self::Xc3000A)
    }
    pub fn is_xc4000(self) -> bool {
        matches!(
            self,
            Self::Xc4000
                | Self::Xc4000A
                | Self::Xc4000H
                | Self::Xc4000E
                | Self::Xc4000Ex
                | Self::Xc4000Xla
                | Self::Xc4000Xv
                | Self::SpartanXl
        )
    }
    pub fn is_xl(self) -> bool {
        matches!(self, Self::Xc4000Ex | Self::Xc4000Xla | Self::Xc4000Xv)
    }
    pub fn is_clb_xl(self) -> bool {
        matches!(
            self,
            Self::SpartanXl | Self::Xc4000Ex | Self::Xc4000Xla | Self::Xc4000Xv
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Chip {
    pub kind: ChipKind,
    pub columns: usize,
    pub rows: usize,
    // XC3000 only
    pub is_small: bool,
    // XC4000X only
    pub is_buff_large: bool,
    // XC2000 only
    pub cols_bidi: BTreeSet<ColId>,
    pub rows_bidi: BTreeSet<RowId>,
    pub cfg_io: BTreeMap<SharedCfgPad, BelPadCoord>,
    pub unbonded_io: BTreeSet<BelCoord>,
}

impl Chip {
    // always single chip
    pub const DIE: DieId = DieId::from_idx_const(0);

    pub fn col_w(&self) -> ColId {
        ColId::from_idx(0)
    }

    pub fn col_e(&self) -> ColId {
        ColId::from_idx(self.columns - 1)
    }

    pub fn col_mid(&self) -> ColId {
        ColId::from_idx(self.columns / 2)
    }

    pub fn col_edge(&self, edge: DirH) -> ColId {
        match edge {
            DirH::W => self.col_w(),
            DirH::E => self.col_e(),
        }
    }

    pub fn row_s(&self) -> RowId {
        RowId::from_idx(0)
    }

    pub fn row_n(&self) -> RowId {
        RowId::from_idx(self.rows - 1)
    }

    pub fn row_mid(&self) -> RowId {
        RowId::from_idx(self.rows / 2)
    }

    pub fn row_edge(&self, edge: DirV) -> RowId {
        match edge {
            DirV::S => self.row_s(),
            DirV::N => self.row_n(),
        }
    }

    pub fn corner(&self, side: DirHV) -> TileCoord {
        Self::DIE
            .cell(self.col_edge(side.h), self.row_edge(side.v))
            .tile(match self.kind {
                ChipKind::Xc2000 => xc2000::tslots::MAIN,
                ChipKind::Xc3000 | ChipKind::Xc3000A => xc3000::tslots::MAIN,
                ChipKind::Xc4000
                | ChipKind::Xc4000A
                | ChipKind::Xc4000H
                | ChipKind::Xc4000E
                | ChipKind::Xc4000Ex
                | ChipKind::Xc4000Xla
                | ChipKind::Xc4000Xv
                | ChipKind::SpartanXl => xc4000::tslots::MAIN,
                ChipKind::Xc5200 => xc5200::tslots::MAIN,
            })
    }

    pub fn col_q(&self, side: DirH) -> ColId {
        match side {
            DirH::W => ColId::from_idx((self.columns + 2) / 4),
            DirH::E => ColId::from_idx(3 * self.columns / 4),
        }
    }

    pub fn row_q(&self, side: DirV) -> RowId {
        match side {
            DirV::S => RowId::from_idx((self.rows + 2) / 4),
            DirV::N => RowId::from_idx(3 * self.rows / 4),
        }
    }

    pub fn col_side_of_mid(&self, col: ColId) -> DirH {
        col.side_of(self.col_mid())
    }

    pub fn row_side_of_mid(&self, row: RowId) -> DirV {
        row.side_of(self.row_mid())
    }

    pub fn columns(&self) -> EntityRange<ColId> {
        EntityRange::new(0, self.columns)
    }

    pub fn rows(&self) -> EntityRange<RowId> {
        EntityRange::new(0, self.rows)
    }

    pub fn pad_io_h(&self, edge: DirH, row: RowId, idx: usize) -> BelPadCoord {
        let cell = Self::DIE.cell(self.col_edge(edge), row);
        match self.kind {
            ChipKind::Xc2000 => cell
                .bel(
                    match edge {
                        DirH::W => xc2000::bslots::IO_W,
                        DirH::E => xc2000::bslots::IO_E,
                    }[idx],
                )
                .pad(xc2000::bcls::IO::PAD),
            ChipKind::Xc3000 | ChipKind::Xc3000A => cell
                .bel(
                    match edge {
                        DirH::W => xc3000::bslots::IO_W,
                        DirH::E => xc3000::bslots::IO_E,
                    }[idx],
                )
                .pad(xc3000::bcls::IO::PAD),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => cell.bel(xc4000::bslots::IO[idx]).pad(xc4000::bcls::IO::PAD),
            ChipKind::Xc4000H => cell
                .bel(xc4000::bslots::HIO[idx])
                .pad(xc4000::bcls::HIO::PAD),
            ChipKind::Xc5200 => cell.bel(xc5200::bslots::IO[idx]).pad(xc5200::bcls::IO::PAD),
        }
    }

    pub fn pad_io_v(&self, edge: DirV, col: ColId, idx: usize) -> BelPadCoord {
        let cell = Self::DIE.cell(col, self.row_edge(edge));
        match self.kind {
            ChipKind::Xc2000 => cell
                .bel(
                    match edge {
                        DirV::S => xc2000::bslots::IO_S,
                        DirV::N => xc2000::bslots::IO_N,
                    }[idx],
                )
                .pad(xc2000::bcls::IO::PAD),
            ChipKind::Xc3000 | ChipKind::Xc3000A => cell
                .bel(
                    match edge {
                        DirV::S => xc3000::bslots::IO_S,
                        DirV::N => xc3000::bslots::IO_N,
                    }[idx],
                )
                .pad(xc3000::bcls::IO::PAD),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => cell.bel(xc4000::bslots::IO[idx]).pad(xc4000::bcls::IO::PAD),
            ChipKind::Xc4000H => cell
                .bel(xc4000::bslots::HIO[idx])
                .pad(xc4000::bcls::HIO::PAD),
            ChipKind::Xc5200 => cell.bel(xc5200::bslots::IO[idx]).pad(xc5200::bcls::IO::PAD),
        }
    }

    pub fn pad_prog_b(&self) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => self
                .corner(DirHV::SE)
                .bel(xc2000::bslots::MISC_SE)
                .pad(xc2000::bcls::MISC_SE::PROG_B),
            ChipKind::Xc3000 | ChipKind::Xc3000A => self
                .corner(DirHV::SE)
                .bel(xc3000::bslots::MISC_SE)
                .pad(xc3000::bcls::MISC_SE::PROG_B),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => self
                .corner(DirHV::SE)
                .bel(xc4000::bslots::MISC_SE)
                .pad(xc4000::bcls::MISC_SE::PROG_B),
            ChipKind::Xc5200 => self
                .corner(DirHV::SE)
                .bel(xc5200::bslots::MISC_SE)
                .pad(xc5200::bcls::MISC_SE::PROG_B),
        }
    }

    pub fn pad_done(&self) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => self
                .corner(DirHV::SE)
                .bel(xc2000::bslots::MISC_SE)
                .pad(xc2000::bcls::MISC_SE::DONE),
            ChipKind::Xc3000 | ChipKind::Xc3000A => self
                .corner(DirHV::SE)
                .bel(xc3000::bslots::MISC_SE)
                .pad(xc3000::bcls::MISC_SE::DONE),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => self
                .corner(DirHV::SE)
                .bel(xc4000::bslots::MISC_SE)
                .pad(xc4000::bcls::MISC_SE::DONE),
            ChipKind::Xc5200 => self
                .corner(DirHV::SE)
                .bel(xc5200::bslots::MISC_SE)
                .pad(xc5200::bcls::MISC_SE::DONE),
        }
    }

    pub fn pad_cclk(&self) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => self
                .corner(DirHV::NE)
                .bel(xc2000::bslots::MISC_NE)
                .pad(xc2000::bcls::MISC_NE::CCLK),
            ChipKind::Xc3000 | ChipKind::Xc3000A => self
                .corner(DirHV::NE)
                .bel(xc3000::bslots::MISC_NE)
                .pad(xc3000::bcls::MISC_NE::CCLK),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => self
                .corner(DirHV::NE)
                .bel(xc4000::bslots::MISC_NE)
                .pad(xc4000::bcls::MISC_NE::CCLK),
            ChipKind::Xc5200 => self
                .corner(DirHV::NE)
                .bel(xc5200::bslots::MISC_NE)
                .pad(xc5200::bcls::MISC_NE::CCLK),
        }
    }

    pub fn pad_pwrdwn_b(&self) -> Option<BelPadCoord> {
        match self.kind {
            ChipKind::Xc2000 => Some(
                self.corner(DirHV::NW)
                    .bel(xc2000::bslots::MISC_NW)
                    .pad(xc2000::bcls::MISC_NW::PWRDWN_B),
            ),
            ChipKind::Xc3000 | ChipKind::Xc3000A => Some(
                self.corner(DirHV::NW)
                    .bel(xc3000::bslots::MISC_NW)
                    .pad(xc3000::bcls::MISC_NW::PWRDWN_B),
            ),
            ChipKind::SpartanXl => Some(
                self.corner(DirHV::SW)
                    .bel(xc4000::bslots::MISC_SW)
                    .pad(xc4000::bcls::MISC_SW::PWRDWN_B),
            ),
            _ => None,
        }
    }

    pub fn pad_tdo(&self) -> Option<BelPadCoord> {
        if self.kind.is_xc4000() {
            Some(
                self.corner(DirHV::NE)
                    .bel(xc4000::bslots::TDO)
                    .pad(xc4000::bcls::TDO::PAD),
            )
        } else if self.kind == ChipKind::Xc5200 {
            Some(self.cfg_io[&SharedCfgPad::Tdo])
        } else {
            None
        }
    }

    pub fn pad_mode(&self, idx: usize) -> Option<BelPadCoord> {
        Some(match self.kind {
            ChipKind::Xc2000 => {
                if idx == 2 {
                    self.cfg_io[&SharedCfgPad::M2]
                } else {
                    self.corner(DirHV::SW).bel(xc2000::bslots::MISC_SW).pad(
                        [xc2000::bcls::MISC_SW::M0, xc2000::bcls::MISC_SW::M1]
                            .get(idx)
                            .copied()?,
                    )
                }
            }
            ChipKind::Xc3000 | ChipKind::Xc3000A => {
                if idx == 2 {
                    self.cfg_io[&SharedCfgPad::M2]
                } else {
                    self.corner(DirHV::SW).bel(xc3000::bslots::MISC_SW).pad(
                        [xc3000::bcls::MISC_SW::M0, xc3000::bcls::MISC_SW::M1]
                            .get(idx)
                            .copied()?,
                    )
                }
            }
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => {
                let cell = self.corner(DirHV::SW);
                [
                    cell.bel(xc4000::bslots::MD0).pad(xc4000::bcls::IBUF::PAD),
                    cell.bel(xc4000::bslots::MD1).pad(xc4000::bcls::MD1::PAD),
                    cell.bel(xc4000::bslots::MD2).pad(xc4000::bcls::IBUF::PAD),
                ]
                .get(idx)
                .copied()?
            }
            ChipKind::Xc5200 => {
                self.cfg_io[&[SharedCfgPad::M0, SharedCfgPad::M1, SharedCfgPad::M2]
                    .get(idx)
                    .copied()?]
            }
        })
    }

    pub fn pad_gnd(&self) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => self.bel_power().pad(xc2000::bcls::POWER::GND),
            ChipKind::Xc3000 | ChipKind::Xc3000A => self.bel_power().pad(xc3000::bcls::POWER::GND),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => self.bel_power().pad(xc4000::bcls::POWER::GND),
            ChipKind::Xc5200 => self.bel_power().pad(xc5200::bcls::POWER::GND),
        }
    }

    pub fn pad_vccio(&self) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => self.bel_power().pad(xc2000::bcls::POWER::VCC),
            ChipKind::Xc3000 | ChipKind::Xc3000A => self.bel_power().pad(xc3000::bcls::POWER::VCC),
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::SpartanXl => self.bel_power().pad(xc4000::bcls::POWER::VCC),
            ChipKind::Xc4000Xv => self.bel_power().pad(xc4000::bcls::POWER::VCCIO),
            ChipKind::Xc5200 => self.bel_power().pad(xc5200::bcls::POWER::VCC),
        }
    }

    pub fn pad_vccint(&self) -> BelPadCoord {
        if self.kind == ChipKind::Xc4000Xv {
            self.bel_power().pad(xc4000::bcls::POWER::VCCINT)
        } else {
            self.pad_vccio()
        }
    }

    pub fn bel_is_io(&self, bel: BelCoord) -> bool {
        match self.kind {
            ChipKind::Xc2000 => {
                xc2000::bslots::IO_W.contains(bel.slot)
                    || xc2000::bslots::IO_E.contains(bel.slot)
                    || xc2000::bslots::IO_S.contains(bel.slot)
                    || xc2000::bslots::IO_N.contains(bel.slot)
            }
            ChipKind::Xc3000 | ChipKind::Xc3000A => {
                xc3000::bslots::IO_W.contains(bel.slot)
                    || xc3000::bslots::IO_E.contains(bel.slot)
                    || xc3000::bslots::IO_S.contains(bel.slot)
                    || xc3000::bslots::IO_N.contains(bel.slot)
            }
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => xc4000::bslots::IO.contains(bel.slot),
            ChipKind::Xc4000H => xc4000::bslots::HIO.contains(bel.slot),
            ChipKind::Xc5200 => xc5200::bslots::IO.contains(bel.slot),
        }
    }

    pub fn get_bonded_ios(&self) -> Vec<BelPadCoord> {
        let mut res = vec![];
        match self.kind {
            ChipKind::Xc2000 => {
                for col in self.columns() {
                    for iob in [0, 1] {
                        res.push(self.pad_io_v(DirV::N, col, iob));
                    }
                }
                for row in self.rows().rev() {
                    if row == self.row_s() {
                        res.push(self.pad_io_h(DirH::E, row, 0));
                    } else if row == self.row_n() || row == self.row_mid() - 1 {
                        res.push(self.pad_io_h(DirH::E, row, 1));
                    } else {
                        for iob in [0, 1] {
                            res.push(self.pad_io_h(DirH::E, row, iob));
                        }
                    }
                }
                for col in self.columns().rev() {
                    for iob in [1, 0] {
                        res.push(self.pad_io_v(DirV::S, col, iob));
                    }
                }
                for row in self.rows() {
                    if row == self.row_s() {
                        res.push(self.pad_io_h(DirH::W, row, 0));
                    } else if row == self.row_n() || row == self.row_mid() - 1 {
                        res.push(self.pad_io_h(DirH::W, row, 1));
                    } else {
                        for iob in [1, 0] {
                            res.push(self.pad_io_h(DirH::W, row, iob));
                        }
                    }
                }
            }
            ChipKind::Xc3000 | ChipKind::Xc3000A => {
                for col in self.columns() {
                    for iob in [0, 1] {
                        res.push(self.pad_io_v(DirV::N, col, iob));
                    }
                }
                for row in self.rows().rev() {
                    for iob in [0, 1] {
                        res.push(self.pad_io_h(DirH::E, row, iob));
                    }
                }
                for col in self.columns().rev() {
                    for iob in [1, 0] {
                        res.push(self.pad_io_v(DirV::S, col, iob));
                    }
                }
                for row in self.rows() {
                    for iob in [1, 0] {
                        res.push(self.pad_io_h(DirH::W, row, iob));
                    }
                }
            }
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => {
                let iobs = if self.kind == ChipKind::Xc4000H {
                    0..4
                } else {
                    0..2
                };
                for col in self.columns() {
                    if col == self.col_w() || col == self.col_e() {
                        continue;
                    }
                    for iob in iobs.clone() {
                        res.push(self.pad_io_v(DirV::N, col, iob));
                    }
                }
                for row in self.rows().rev() {
                    if row == self.row_s() || row == self.row_n() {
                        continue;
                    }
                    for iob in iobs.clone() {
                        res.push(self.pad_io_h(DirH::E, row, iob));
                    }
                }
                for col in self.columns().rev() {
                    if col == self.col_w() || col == self.col_e() {
                        continue;
                    }
                    for iob in iobs.clone().rev() {
                        res.push(self.pad_io_v(DirV::S, col, iob));
                    }
                }
                for row in self.rows() {
                    if row == self.row_s() || row == self.row_n() {
                        continue;
                    }
                    for iob in iobs.clone().rev() {
                        res.push(self.pad_io_h(DirH::W, row, iob));
                    }
                }
            }
            ChipKind::Xc5200 => {
                for col in self.columns() {
                    if col == self.col_w() || col == self.col_e() {
                        continue;
                    }
                    for iob in [3, 2, 1, 0] {
                        res.push(self.pad_io_v(DirV::N, col, iob));
                    }
                }
                for row in self.rows().rev() {
                    if row == self.row_s() || row == self.row_n() {
                        continue;
                    }
                    for iob in [3, 2, 1, 0] {
                        res.push(self.pad_io_h(DirH::E, row, iob));
                    }
                }
                for col in self.columns().rev() {
                    if col == self.col_w() || col == self.col_e() {
                        continue;
                    }
                    for iob in [0, 1, 2, 3] {
                        res.push(self.pad_io_v(DirV::S, col, iob));
                    }
                }
                for row in self.rows() {
                    if row == self.row_s() || row == self.row_n() {
                        continue;
                    }
                    for iob in [0, 1, 2, 3] {
                        res.push(self.pad_io_h(DirH::W, row, iob));
                    }
                }
            }
        }
        res
    }

    pub fn io_xtl1(&self) -> BelPadCoord {
        self.pad_io_v(DirV::S, self.col_e(), 1)
    }

    pub fn io_xtl2(&self) -> BelPadCoord {
        self.pad_io_h(DirH::E, self.row_s(), 0)
    }

    pub fn io_clock(&self, idx: usize) -> BelPadCoord {
        match self.kind {
            ChipKind::Xc2000 => unreachable!(),
            ChipKind::Xc3000 | ChipKind::Xc3000A => [
                self.pad_io_h(DirH::W, self.row_n(), 0),
                self.pad_io_h(DirH::E, self.row_s(), 0),
            ][idx],
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => {
                [
                    self.pad_io_h(DirH::W, self.row_n() - 1, 0),
                    self.pad_io_h(DirH::W, self.row_s() + 1, 1),
                    self.pad_io_v(DirV::S, self.col_w() + 1, 0),
                    self.pad_io_v(DirV::S, self.col_e() - 1, 1),
                    self.pad_io_h(DirH::E, self.row_s() + 1, 0), // skip one (D7)
                    self.pad_io_h(DirH::E, self.row_n() - 1, 0),
                    self.pad_io_v(DirV::N, self.col_e() - 1, 0), // skip one (A0???)
                    self.pad_io_v(DirV::N, self.col_w() + 1, 0),
                ][idx]
            }
            ChipKind::Xc4000H => {
                [
                    self.pad_io_h(DirH::W, self.row_n() - 1, 0),
                    self.pad_io_h(DirH::W, self.row_s() + 1, 3),
                    self.pad_io_v(DirV::S, self.col_w() + 1, 0),
                    self.pad_io_v(DirV::S, self.col_e() - 1, 3),
                    self.pad_io_h(DirH::E, self.row_s() + 1, 2), // skip one (D7)
                    self.pad_io_h(DirH::E, self.row_n() - 1, 0),
                    self.pad_io_v(DirV::N, self.col_e() - 1, 2), // skip one (A0???)
                    self.pad_io_v(DirV::N, self.col_w() + 1, 0),
                ][idx]
            }
            ChipKind::Xc5200 => {
                [
                    self.pad_io_h(DirH::W, self.row_n() - 1, 1), // skip two (2×unbonded)
                    self.pad_io_v(DirV::S, self.col_w() + 1, 2), // skip one (M2)
                    self.pad_io_h(DirH::E, self.row_s() + 1, 1), // skip one (D7)
                    self.pad_io_v(DirV::N, self.col_e() - 2, 2), // skip five (TDO, 2×unbonded, A0, unbonded)
                ][idx]
            }
        }
    }

    pub fn bel_buff_io(&self, which: DirHV) -> BelCoord {
        assert!(matches!(
            self.kind,
            ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv
        ));
        match (which, self.is_buff_large) {
            (DirHV::SW, _) => Self::DIE
                .cell(self.col_w(), self.row_q(DirV::S))
                .bel(xc4000::bslots::IO[1]),
            (DirHV::NW, _) => Self::DIE
                .cell(self.col_w(), self.row_q(DirV::N) - 1)
                .bel(xc4000::bslots::IO[0]),
            (DirHV::SE, false) => Self::DIE
                .cell(self.col_e(), self.row_q(DirV::S))
                .bel(xc4000::bslots::IO[1]),
            (DirHV::SE, true) => Self::DIE
                .cell(self.col_e(), self.row_q(DirV::S) + 1)
                .bel(xc4000::bslots::IO[1]),
            (DirHV::NE, false) => Self::DIE
                .cell(self.col_e(), self.row_q(DirV::N) - 1)
                .bel(xc4000::bslots::IO[0]),
            (DirHV::NE, true) => Self::DIE
                .cell(self.col_e(), self.row_q(DirV::N) - 2)
                .bel(xc4000::bslots::IO[0]),
        }
    }

    pub fn bel_power(&self) -> BelCoord {
        let power = match self.kind {
            ChipKind::Xc2000 => xc2000::bslots::POWER,
            ChipKind::Xc3000 | ChipKind::Xc3000A => xc3000::bslots::POWER,
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => xc4000::bslots::POWER,
            ChipKind::Xc5200 => xc5200::bslots::POWER,
        };
        Self::DIE.cell(self.col_w(), self.row_s()).bel(power)
    }

    pub fn btile_height_main(&self, row: RowId) -> usize {
        if row == self.row_s() {
            match self.kind {
                ChipKind::Xc2000 => 12,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 13,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => {
                    13
                }
                ChipKind::Xc4000A => 10,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla => 16,
                ChipKind::Xc4000Xv => 17,
                ChipKind::Xc5200 => 28,
            }
        } else if row == self.row_n() {
            match self.kind {
                ChipKind::Xc2000 => 9,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 10,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => 7,
                ChipKind::Xc4000A => 6,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla => 8,
                ChipKind::Xc4000Xv => 9,
                ChipKind::Xc5200 => 28,
            }
        } else {
            match self.kind {
                ChipKind::Xc2000 => 8,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 8,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => {
                    10
                }
                ChipKind::Xc4000A => 10,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla => 12,
                ChipKind::Xc4000Xv => 13,
                ChipKind::Xc5200 => 34,
            }
        }
    }

    pub fn btile_height_clk(&self) -> usize {
        match self.kind {
            ChipKind::Xc2000 => unreachable!(),
            ChipKind::Xc3000 | ChipKind::Xc3000A => 1,
            ChipKind::Xc4000 | ChipKind::Xc4000A | ChipKind::Xc4000H | ChipKind::Xc4000E => 1,
            ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv | ChipKind::SpartanXl => {
                2
            }
            ChipKind::Xc5200 => 4,
        }
    }

    pub fn btile_height_brk(&self) -> usize {
        if self.kind == ChipKind::Xc2000 { 1 } else { 2 }
    }

    pub fn btile_width_main(&self, col: ColId) -> usize {
        if col == self.col_w() {
            match self.kind {
                ChipKind::Xc2000 => 21,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 29,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => {
                    26
                }
                ChipKind::Xc4000A => 21,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv => 27,
                ChipKind::Xc5200 => 7,
            }
        } else if col == self.col_e() {
            match self.kind {
                ChipKind::Xc2000 => 27,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 36,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => {
                    41
                }
                ChipKind::Xc4000A => 32,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv => 52,
                ChipKind::Xc5200 => 8,
            }
        } else {
            match self.kind {
                ChipKind::Xc2000 => 18,
                ChipKind::Xc3000 | ChipKind::Xc3000A => 22,
                ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000E | ChipKind::SpartanXl => {
                    36
                }
                ChipKind::Xc4000A => 32,
                ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv => 47,
                ChipKind::Xc5200 => 12,
            }
        }
    }

    pub fn btile_width_clk(&self) -> usize {
        match self.kind {
            ChipKind::Xc2000 => unreachable!(),
            ChipKind::Xc3000 | ChipKind::Xc3000A => unreachable!(),
            ChipKind::Xc4000 | ChipKind::Xc4000H | ChipKind::Xc4000A | ChipKind::Xc4000E => 1,
            ChipKind::Xc4000Ex | ChipKind::Xc4000Xla | ChipKind::Xc4000Xv | ChipKind::SpartanXl => {
                2
            }
            ChipKind::Xc5200 => 1,
        }
    }

    pub fn btile_width_brk(&self) -> usize {
        if self.kind == ChipKind::Xc2000 { 2 } else { 1 }
    }

    pub fn bel_bufg(&self, idx: usize) -> BelCoord {
        assert!(self.kind.is_xc4000());
        let (col, row, slot) = [
            (self.col_w(), self.row_n(), xc4000::bslots::BUFG_V),
            (self.col_w(), self.row_s(), xc4000::bslots::BUFG_V),
            (self.col_w(), self.row_s(), xc4000::bslots::BUFG_H),
            (self.col_e(), self.row_s(), xc4000::bslots::BUFG_H),
            (self.col_e(), self.row_s(), xc4000::bslots::BUFG_V),
            (self.col_e(), self.row_n(), xc4000::bslots::BUFG_V),
            (self.col_e(), self.row_n(), xc4000::bslots::BUFG_H),
            (self.col_w(), self.row_n(), xc4000::bslots::BUFG_H),
        ][idx];
        Self::DIE.cell(col, row).bel(slot)
    }
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        writeln!(o, "\tkind {};", self.kind)?;
        writeln!(o, "\tcolumns {};", self.columns)?;
        writeln!(o, "\trows {};", self.rows)?;
        if self.is_small {
            writeln!(o, "\tsmall;")?;
        }
        if self.is_buff_large {
            writeln!(o, "\tbuff_large;")?;
        }
        if !self.cols_bidi.is_empty() {
            writeln!(
                o,
                "\tcols_bidi {};",
                self.cols_bidi.iter().map(|x| x.to_string()).join(", ")
            )?;
        }
        if !self.rows_bidi.is_empty() {
            writeln!(
                o,
                "\trows_bidi {};",
                self.rows_bidi.iter().map(|x| x.to_string()).join(", ")
            )?;
        }
        for (k, v) in &self.cfg_io {
            writeln!(o, "\tcfg_io {k} = {v};", v = v.to_string(db))?;
        }
        if !self.unbonded_io.is_empty() {
            writeln!(
                o,
                "\tunbonded_io {};",
                self.unbonded_io.iter().map(|x| x.to_string(db)).join(", ")
            )?;
        }
        Ok(())
    }
}
