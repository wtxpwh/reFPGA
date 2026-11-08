use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_entity::{EntityId, EntityRange, EntityRangeVec};
use prjcombine_interconnect::{
    coord::{BelCoord, BelPadCoord, ColId, DieId, DieIdExt, RowId, TileCoord},
    db::IntDb,
    dir::{DirH, DirHV, DirV},
};
use std::collections::{BTreeMap, BTreeSet};

use crate::defs::{bcls, bslots, tslots};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum ChipKind {
    Spartan2,
    Virtex,
    VirtexE,
    VirtexEM,
}

impl std::fmt::Display for ChipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipKind::Spartan2 => write!(f, "spartan2"),
            ChipKind::Virtex => write!(f, "virtex"),
            ChipKind::VirtexE => write!(f, "virtexe"),
            ChipKind::VirtexEM => write!(f, "virtexem"),
        }
    }
}

impl ChipKind {
    pub fn is_virtexe(self) -> bool {
        matches!(self, ChipKind::VirtexE | ChipKind::VirtexEM)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum SharedCfgPad {
    Data(u8), // ×8
    CsB,
    InitB,
    RdWrB,
    Dout,
}

impl std::fmt::Display for SharedCfgPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharedCfgPad::Data(i) => write!(f, "D{i}"),
            SharedCfgPad::CsB => write!(f, "CS_B"),
            SharedCfgPad::RdWrB => write!(f, "RDWR_B"),
            SharedCfgPad::Dout => write!(f, "DOUT"),
            SharedCfgPad::InitB => write!(f, "INIT_B"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Chip {
    pub kind: ChipKind,
    pub cols_bram: BTreeSet<ColId>,
    // column -> clock column
    pub cols_clkv: EntityRangeVec<ColId, ColId>,
    pub rows: usize,
    pub cfg_io: BTreeMap<SharedCfgPad, BelPadCoord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub enum DisabledPart {
    // Virtex-E: primary DLLs are disabled
    PrimaryDlls,
    // Virtex-E: a BRAM column is disabled
    Bram(ColId),
}

impl std::fmt::Display for DisabledPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisabledPart::PrimaryDlls => write!(f, "PRIMARY_DLLS"),
            DisabledPart::Bram(col) => write!(f, "BRAM_COL:{col}"),
        }
    }
}

impl Chip {
    // always single chip
    pub const DIE: DieId = DieId::from_idx_const(0);

    pub fn row_mid(&self) -> RowId {
        RowId::from_idx(self.rows / 2)
    }

    pub fn row_clk(&self) -> RowId {
        match self.rows % 8 {
            2 => RowId::from_idx(self.rows / 2),
            6 => RowId::from_idx(self.rows / 2 - 2),
            _ => unreachable!(),
        }
    }

    pub fn col_clk(&self) -> ColId {
        ColId::from_idx(self.columns().len() / 2)
    }

    pub fn col_w(&self) -> ColId {
        self.columns().first().unwrap()
    }

    pub fn col_e(&self) -> ColId {
        self.columns().last().unwrap()
    }

    pub fn col_edge(&self, dir: DirH) -> ColId {
        match dir {
            DirH::W => self.col_w(),
            DirH::E => self.col_e(),
        }
    }

    pub fn row_s(&self) -> RowId {
        self.rows().first().unwrap()
    }

    pub fn row_n(&self) -> RowId {
        self.rows().last().unwrap()
    }

    pub fn row_edge(&self, dir: DirV) -> RowId {
        match dir {
            DirV::S => self.row_s(),
            DirV::N => self.row_n(),
        }
    }

    pub fn is_row_io(&self, row: RowId) -> bool {
        row == self.row_s() || row == self.row_n()
    }

    pub fn columns(&self) -> EntityRange<ColId> {
        EntityRange::new(0, self.cols_clkv.len())
    }

    pub fn rows(&self) -> EntityRange<RowId> {
        EntityRange::new(0, self.rows)
    }

    pub fn corner(&self, side: DirHV) -> TileCoord {
        Self::DIE
            .cell(self.col_edge(side.h), self.row_edge(side.v))
            .tile(tslots::MAIN)
    }

    pub fn bel_corner_misc(&self, side: DirHV) -> BelCoord {
        self.corner(side).bel(match side {
            DirHV::SW => bslots::MISC_SW,
            DirHV::SE => bslots::MISC_SE,
            DirHV::NW => bslots::MISC_NW,
            DirHV::NE => bslots::MISC_NE,
        })
    }

    pub fn pad_to_vcco(&self, pad: BelPadCoord) -> BelPadCoord {
        if bslots::IOB.contains(pad.slot) {
            if pad.col == self.col_w() || pad.col == self.col_e() {
                pad.with_row(if pad.row < self.row_mid() {
                    self.row_s()
                } else {
                    self.row_n()
                })
                .bel(bslots::BANK_WE)
                .pad(bcls::BANK::VCCO)
            } else if pad.row == self.row_s() || pad.row == self.row_n() {
                pad.with_col(if pad.col < self.col_clk() {
                    self.col_w()
                } else {
                    self.col_e()
                })
                .bel(bslots::BANK_SN)
                .pad(bcls::BANK::VCCO)
            } else {
                unreachable!()
            }
        } else if let Some(idx) = bslots::GCLK_IOB.index_of(pad.slot) {
            pad.with_col([self.col_e(), self.col_w()][idx])
                .bel(bslots::BANK_SN)
                .pad(bcls::BANK::VCCO)
        } else if pad == self.pad_tdo() || pad == self.pad_cclk() {
            self.pad_bank_vcco(2)
        } else if pad == self.pad_done() {
            self.pad_bank_vcco(3)
        } else {
            unreachable!()
        }
    }

    pub fn pad_bank_vcco(&self, bank: u32) -> BelPadCoord {
        let bel = match bank {
            0 => self.corner(DirHV::NW).bel(bslots::BANK_SN),
            1 => self.corner(DirHV::NE).bel(bslots::BANK_SN),
            2 => self.corner(DirHV::NE).bel(bslots::BANK_WE),
            3 => self.corner(DirHV::SE).bel(bslots::BANK_WE),
            4 => self.corner(DirHV::SE).bel(bslots::BANK_SN),
            5 => self.corner(DirHV::SW).bel(bslots::BANK_SN),
            6 => self.corner(DirHV::SW).bel(bslots::BANK_WE),
            7 => self.corner(DirHV::NW).bel(bslots::BANK_WE),
            _ => unreachable!(),
        };
        bel.pad(bcls::BANK::VCCO)
    }

    pub fn pad_io_h(&self, edge: DirH, row: RowId, idx: usize) -> BelPadCoord {
        Self::DIE
            .cell(self.col_edge(edge), row)
            .bel(bslots::IOB[idx])
            .pad(bcls::IOB::PAD)
    }

    pub fn pad_io_v(&self, edge: DirV, col: ColId, idx: usize) -> BelPadCoord {
        Self::DIE
            .cell(col, self.row_edge(edge))
            .bel(bslots::IOB[idx])
            .pad(bcls::IOB::PAD)
    }

    pub fn pad_clk(&self, hv: DirHV) -> BelPadCoord {
        Self::DIE
            .cell(self.col_clk(), self.row_edge(hv.v))
            .bel(
                bslots::GCLK_IOB[match hv.h {
                    // sigh.
                    DirH::W => 1,
                    DirH::E => 0,
                }],
            )
            .pad(bcls::GCLK_IOB::PAD)
    }

    pub fn pads_mode(&self) -> [BelPadCoord; 3] {
        let bel = self.bel_corner_misc(DirHV::SW);
        [
            bel.pad(bcls::MISC_SW::M0),
            bel.pad(bcls::MISC_SW::M1),
            bel.pad(bcls::MISC_SW::M2),
        ]
    }

    pub fn pad_done(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SE).pad(bcls::MISC_SE::DONE)
    }

    pub fn pad_prog_b(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SE).pad(bcls::MISC_SE::PROG_B)
    }

    pub fn pad_cclk(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::CCLK)
    }

    pub fn pad_tck(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::TCK)
    }

    pub fn pad_tms(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::TMS)
    }

    pub fn pad_tdi(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::TDI)
    }

    pub fn pad_tdo(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::TDO)
    }

    pub fn pad_gnd(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::GND)
    }

    pub fn pad_vccint(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::VCCINT)
    }

    pub fn pad_vccaux(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::VCCAUX)
    }

    pub fn pad_dxp(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SW).pad(bcls::MISC_SW::DXP)
    }

    pub fn pad_dxn(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SW).pad(bcls::MISC_SW::DXN)
    }

    pub fn iob_to_ioi(&self, iob: BelCoord) -> BelCoord {
        let idx = bslots::IOB.index_of(iob.slot).unwrap();
        iob.bel(bslots::IOI[idx])
    }

    pub fn io_to_pad(&self, bel: BelCoord) -> BelPadCoord {
        if let Some(idx) = bslots::IOI.index_of(bel.slot) {
            bel.bel(bslots::IOB[idx]).pad(bcls::IOB::PAD)
        } else if bslots::IOB.contains(bel.slot) {
            bel.pad(bcls::IOB::PAD)
        } else if bslots::GCLK_IOB.contains(bel.slot) {
            bel.pad(bcls::GCLK_IOB::PAD)
        } else {
            unreachable!()
        }
    }

    pub fn get_all_ios(&self) -> Vec<BelPadCoord> {
        self.get_bscan()
            .io
            .into_keys()
            .filter(|pad| bslots::IOB.contains(pad.slot) || bslots::GCLK_IOB.contains(pad.slot))
            .collect()
    }

    pub fn bel_pci(&self, side: DirH) -> BelCoord {
        Self::DIE
            .cell(self.col_edge(side), self.row_clk())
            .bel(bslots::PCILOGIC)
    }

    pub fn tile_global(&self) -> TileCoord {
        self.corner(DirHV::SW).tile(tslots::GLOBAL)
    }

    pub fn bel_global(&self) -> BelCoord {
        self.tile_global().bel(bslots::GLOBAL)
    }
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        writeln!(o, "\tkind {};", self.kind)?;
        writeln!(o, "\trows {};", self.rows)?;
        writeln!(
            o,
            "\tcols_bram {};",
            self.cols_bram.iter().map(|x| x.to_string()).join(", ")
        )?;
        for (cols, &col_hclk) in &self.cols_clkv {
            writeln!(o, "\tcol_clkv {col_hclk} = {cols};")?;
        }
        for (k, pad) in &self.cfg_io {
            writeln!(o, "\tcfg_io {k} = {pad};", pad = pad.to_string(db))?;
        }
        Ok(())
    }
}
