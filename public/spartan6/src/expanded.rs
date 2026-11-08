use prjcombine_entity::{EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, ColId, Rect, RowId, TileCoord},
    dir::Dir,
    grid::ExpandedGrid,
};
use prjcombine_types::{bitrect::BitRectId, enumerable::EnumMap};
use std::collections::{BTreeSet, HashMap};

use crate::{
    bitstream::{BitRect, FrameAddr, Reg},
    chip::{Chip, DisabledPart, RegId},
    defs::{self, bslots},
};

pub struct ExpandedDevice<'a> {
    pub chip: &'a Chip,
    pub disabled: BTreeSet<DisabledPart>,
    pub egrid: ExpandedGrid<'a>,
    pub site_holes: Vec<Rect>,
    pub frame_info: Vec<FrameAddr>,
    pub bram_frame_info: Vec<FrameAddr>,
    pub iob_frame_len: usize,
    pub col_frame: EntityVec<RegId, EntityVec<ColId, usize>>,
    pub col_width: EntityVec<ColId, usize>,
    pub spine_frame: EntityVec<RegId, usize>,
    pub bram_frame: EntityVec<RegId, EntityPartVec<ColId, usize>>,
    pub iob_frame: HashMap<CellCoord, usize>,
    pub reg_frame: EnumMap<Dir, usize>,
}

impl ExpandedDevice<'_> {
    pub fn in_site_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.site_holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    pub fn btile_main(&self, col: ColId, row: RowId) -> BitRect {
        let reg = self.chip.row_to_reg(row);
        let rd = row - self.chip.row_reg_s(reg);
        let bit = 64 * (rd as usize) + if rd < 8 { 0 } else { 16 };
        BitRect::Main(
            self.col_frame[reg][col],
            self.col_width[col],
            bit,
            64,
            false,
        )
    }

    pub fn btile_spine(&self, row: RowId) -> BitRect {
        let reg = self.chip.row_to_reg(row);
        let rd = row - self.chip.row_reg_s(reg);
        let bit = 64 * (rd as usize) + if rd < 8 { 0 } else { 16 };
        BitRect::Main(self.spine_frame[reg], 4, bit, 64, false)
    }

    pub fn btile_hclk(&self, col: ColId, row: RowId) -> BitRect {
        let reg = self.chip.row_to_reg(row);
        BitRect::Main(
            self.col_frame[reg][col],
            self.col_width[col],
            64 * 8,
            16,
            false,
        )
    }

    pub fn btile_bram(&self, col: ColId, row: RowId) -> BitRect {
        let reg = self.chip.row_to_reg(row);
        let rd: usize = (row - self.chip.row_reg_s(reg)).try_into().unwrap();
        BitRect::Bram(self.bram_frame[reg][col] + rd / 4)
    }

    pub fn btile_clk(&self, dir: Dir) -> BitRect {
        BitRect::Iob(self.reg_frame[dir], 384)
    }

    pub fn btile_iob(&self, cell: CellCoord) -> BitRect {
        BitRect::Iob(self.iob_frame[&cell], 128)
    }

    pub fn tile_bits(&self, tcrd: TileCoord) -> EntityVec<BitRectId, BitRect> {
        let tile = &self[tcrd];
        if tcrd.slot == defs::tslots::GLOBAL {
            EntityVec::from_iter([
                BitRect::Reg(Reg::Cor1),
                BitRect::Reg(Reg::Cor2),
                BitRect::Reg(Reg::Ctl0),
                BitRect::Reg(Reg::CclkFrequency),
                BitRect::Reg(Reg::HcOpt),
                BitRect::Reg(Reg::Powerdown),
                BitRect::Reg(Reg::PuGwe),
                BitRect::Reg(Reg::PuGts),
                BitRect::Reg(Reg::Mode),
                BitRect::Reg(Reg::General1),
                BitRect::Reg(Reg::General2),
                BitRect::Reg(Reg::General3),
                BitRect::Reg(Reg::General4),
                BitRect::Reg(Reg::General5),
                BitRect::Reg(Reg::SeuOpt),
                BitRect::Reg(Reg::EyeMask),
                BitRect::Reg(Reg::Timer),
                BitRect::Reg(Reg::Testmode),
            ])
        } else if tile.class == defs::tcls::BRAM {
            EntityVec::from_iter([
                self.btile_main(tcrd.col, tcrd.row),
                self.btile_main(tcrd.col, tcrd.row + 1),
                self.btile_main(tcrd.col, tcrd.row + 2),
                self.btile_main(tcrd.col, tcrd.row + 3),
                self.btile_bram(tcrd.col, tcrd.row),
            ])
        } else if matches!(tcrd.slot, defs::tslots::HCLK | defs::tslots::HCLK_BEL) {
            EntityVec::from_iter([self.btile_hclk(tcrd.col, tcrd.row)])
        } else if tile.class == defs::tcls::CLKC {
            EntityVec::from_iter([self.btile_main(tcrd.col, tcrd.row)])
        } else if tile.class == defs::tcls::CLK_W {
            EntityVec::from_iter([self.btile_clk(Dir::W)])
        } else if tile.class == defs::tcls::CLK_E {
            EntityVec::from_iter([self.btile_clk(Dir::E)])
        } else if tile.class == defs::tcls::CLK_S {
            EntityVec::from_iter([self.btile_clk(Dir::S)])
        } else if tile.class == defs::tcls::CLK_N {
            EntityVec::from_iter([self.btile_clk(Dir::N)])
        } else if tile.class == defs::tcls::HCLK_ROW {
            EntityVec::from_iter([self.btile_spine(tcrd.row - 1)])
        } else if tcrd.slot == defs::tslots::CMT_BUF {
            EntityVec::from_iter([self.btile_spine(tcrd.row - 7)])
        } else if tile.class == defs::tcls::IOB {
            EntityVec::from_iter([self.btile_iob(tcrd.cell)])
        } else if matches!(tile.class, defs::tcls::CMT_DCM | defs::tcls::CMT_PLL) {
            let mut res = EntityVec::new();
            for i in 0..16 {
                res.push(self.btile_main(tcrd.col, tcrd.row - 8 + i));
            }
            for i in 0..16 {
                res.push(self.btile_spine(tcrd.row - 8 + i));
            }
            res
        } else {
            EntityVec::from_iter(
                tile.cells
                    .values()
                    .map(|&cell| self.btile_main(cell.col, cell.row)),
            )
        }
    }

    pub fn bel_carry_prev(&self, bcrd: BelCoord) -> Option<BelCoord> {
        if bcrd.slot == bslots::SLICE[0] {
            let mut bcrd = bcrd;
            loop {
                bcrd.cell = self.cell_delta(bcrd.cell, 0, -1)?;
                if self.has_bel(bcrd) {
                    return Some(bcrd);
                }
            }
        } else if bcrd.slot == bslots::DSP {
            self.bel_delta(bcrd.cell, 0, -4, bcrd.slot)
        } else {
            panic!("not a carry-chain bel: {}", bcrd.to_string(self.db))
        }
    }
}

impl<'a> std::ops::Deref for ExpandedDevice<'a> {
    type Target = ExpandedGrid<'a>;

    fn deref(&self) -> &Self::Target {
        &self.egrid
    }
}
