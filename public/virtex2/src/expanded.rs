use prjcombine_entity::{EntityId, EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, ColId, Rect, RowId, TileCoord},
    grid::ExpandedGrid,
};
use prjcombine_types::bitrect::BitRectId;

use crate::{
    bitstream::{BitRect, FrameAddr, Reg},
    chip::{Chip, ChipKind},
    defs::{bslots, tcls, tslots},
};

pub struct ExpandedDevice<'a> {
    pub chip: &'a Chip,
    pub egrid: ExpandedGrid<'a>,
    pub frame_info: Vec<FrameAddr>,
    pub frame_len: usize,
    pub holes: Vec<Rect>,
    pub clkv_frame: usize,
    pub spine_frame: usize,
    pub term_w_frame: usize,
    pub term_e_frame: usize,
    pub col_frame: EntityVec<ColId, usize>,
    pub bram_frame: EntityPartVec<ColId, usize>,
}

impl ExpandedDevice<'_> {
    pub fn is_in_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    pub fn btile_main(&self, cell: CellCoord) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (22, 80)
        } else {
            (19, 64)
        };
        let bit = 16 + height * cell.row.to_idx();
        BitRect::Main(self.col_frame[cell.col], width, bit, height, false)
    }

    pub fn btile_bram(&self, cell: CellCoord) -> BitRect {
        let (width, height, height_single) = if self.chip.kind.is_virtex2() {
            (64, 80 * 4, 80)
        } else {
            (19 * 4, 64 * 4, 64)
        };
        let bit = 16 + height_single * cell.row.to_idx();
        BitRect::Main(self.bram_frame[cell.col], width, bit, height, false)
    }

    pub fn btile_term_h(&self, cell: CellCoord) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (4, 80)
        } else {
            (2, 64)
        };
        let bit = 16 + height * cell.row.to_idx();
        let frame = if cell.col == self.chip.col_w() {
            self.term_w_frame
        } else if cell.col == self.chip.col_e() {
            self.term_e_frame
        } else {
            unreachable!()
        };
        BitRect::Main(frame, width, bit, height, false)
    }

    pub fn btile_term_v(&self, cell: CellCoord) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (22, 80)
        } else {
            (19, 64)
        };
        let bit = if cell.row == self.chip.row_s() {
            if self.chip.kind.is_virtex2() {
                4
            } else if !self.chip.kind.is_spartan3a() {
                7
            } else {
                0
            }
        } else if cell.row == self.chip.row_n() {
            16 + height * self.chip.rows.len()
        } else {
            unreachable!()
        };
        BitRect::Main(
            self.col_frame[cell.col],
            width,
            bit,
            if self.chip.kind.is_virtex2() {
                12
            } else if !self.chip.kind.is_spartan3a() {
                5
            } else {
                6
            },
            false,
        )
    }

    pub fn btile_spine(&self, row: RowId) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (4, 80)
        } else if self.chip.has_ll || self.chip.kind.is_spartan3a() {
            (2, 64)
        } else {
            (1, 64)
        };
        let bit = 16 + height * row.to_idx();
        BitRect::Main(self.spine_frame, width, bit, height, false)
    }

    pub fn btile_clkv(&self, cell: CellCoord) -> BitRect {
        assert!(!self.chip.kind.is_virtex2());
        let bit = 16 + 64 * cell.row.to_idx();
        BitRect::Main(
            self.clkv_frame + if cell.col < self.chip.col_clk { 0 } else { 1 },
            1,
            bit,
            64,
            false,
        )
    }

    pub fn btile_spine_end(&self, row: RowId) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (4, 80)
        } else if self.chip.has_ll || self.chip.kind.is_spartan3a() {
            (2, 64)
        } else {
            (1, 64)
        };
        let bit = if row == self.chip.row_s() {
            0
        } else if row == self.chip.row_n() {
            16 + height * self.chip.rows.len()
        } else {
            unreachable!()
        };
        BitRect::Main(self.spine_frame, width, bit, 16, false)
    }

    pub fn btile_llv_s(&self, col: ColId) -> BitRect {
        assert_eq!(self.chip.kind, ChipKind::Spartan3E);
        assert!(self.chip.has_ll);
        let bit = self.chip.rows_hclk.len_ranges() / 2;
        BitRect::Main(self.col_frame[col], 19, bit, 1, false)
    }

    pub fn btile_llv_n(&self, col: ColId) -> BitRect {
        assert_eq!(self.chip.kind, ChipKind::Spartan3E);
        assert!(self.chip.has_ll);
        let bit = 16 + self.chip.rows.len() * 64 + 11 + self.chip.rows_hclk.len_ranges() / 2;
        BitRect::Main(self.col_frame[col], 19, bit, 2, false)
    }

    pub fn btile_llv(&self, col: ColId) -> BitRect {
        assert!(self.chip.kind.is_spartan3a());
        assert!(self.chip.has_ll);
        let bit = 16 + self.chip.rows.len() * 64 + 8;
        BitRect::Main(self.col_frame[col], 19, bit, 3, false)
    }

    pub fn btile_hclk(&self, cell: CellCoord) -> BitRect {
        let (width, height) = if self.chip.kind.is_virtex2() {
            (22, 80)
        } else {
            (19, 64)
        };
        let hclk_idx = self
            .chip
            .rows_hclk
            .values()
            .position(|&hrow| hrow == cell.row)
            .unwrap();
        let bit = if cell.row <= self.chip.row_mid() {
            if self.chip.kind.is_spartan3a() {
                11 + hclk_idx
            } else {
                hclk_idx
            }
        } else {
            let hclk_idx = self.chip.rows_hclk.len_ranges() - hclk_idx - 1;
            if self.chip.kind.is_spartan3a() || self.chip.has_ll {
                16 + height * self.chip.rows.len() + 11 + hclk_idx
            } else {
                16 + height * self.chip.rows.len() + 12 + hclk_idx
            }
        };
        BitRect::Main(self.col_frame[cell.col], width, bit, 1, false)
    }

    pub fn tile_bits(&self, tcrd: TileCoord) -> EntityVec<BitRectId, BitRect> {
        let col = tcrd.col;
        let row = tcrd.row;
        let tile = &self[tcrd];
        let tcls = &self.db.tile_classes[tile.class];
        if tcls.bitrects.is_empty() {
            EntityVec::new()
        } else if tcrd.slot == tslots::GLOBAL {
            if self.chip.kind.is_spartan3a() {
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
                    BitRect::Reg(Reg::SeuOpt),
                ])
            } else {
                EntityVec::from_iter([BitRect::Reg(Reg::Cor0), BitRect::Reg(Reg::Ctl0)])
            }
        } else if tcls.bels.contains_id(bslots::BRAM) {
            EntityVec::from_iter([
                self.btile_main(tcrd.delta(0, 0)),
                self.btile_main(tcrd.delta(0, 1)),
                self.btile_main(tcrd.delta(0, 2)),
                self.btile_main(tcrd.delta(0, 3)),
                self.btile_bram(tcrd.cell),
            ])
        } else if tcls.bels.contains_id(bslots::PCILOGICSE) {
            // CLK_W_*, CLK_E_*
            EntityVec::from_iter([
                self.btile_main(tcrd.delta(0, -1)),
                self.btile_main(tcrd.delta(0, 0)),
                self.btile_term_h(tcrd.delta(0, -2)),
                self.btile_term_h(tcrd.delta(0, -1)),
                self.btile_term_h(tcrd.delta(0, 0)),
                self.btile_term_h(tcrd.delta(0, 1)),
            ])
        } else if tcls.bels.contains_id(bslots::BUFGMUX[0]) {
            // CLK_S_*, CLK_N_*
            EntityVec::from_iter([self.btile_spine(row), self.btile_spine_end(row)])
        } else if tile.class == tcls::CLKC_50A {
            EntityVec::from_iter([self.btile_spine(row - 1)])
        } else if matches!(tile.class, tcls::CLKQC_S3 | tcls::CLKQC_S3E) {
            EntityVec::from_iter([
                self.btile_clkv(tcrd.delta(0, -1)),
                self.btile_clkv(tcrd.delta(0, 0)),
            ])
        } else if tcrd.slot == tslots::HROW {
            if row == self.chip.row_s() + 1 {
                EntityVec::from_iter([
                    self.btile_spine_end(row - 1),
                    self.btile_spine(row - 1),
                    self.btile_spine(row),
                    self.btile_spine(row + 1),
                ])
            } else if row == self.chip.row_n() {
                EntityVec::from_iter([
                    self.btile_spine(row - 2),
                    self.btile_spine(row - 1),
                    self.btile_spine(row),
                    self.btile_spine_end(row),
                ])
            } else {
                EntityVec::from_iter([
                    self.btile_spine(row - 2),
                    self.btile_spine(row - 1),
                    self.btile_spine(row),
                    self.btile_spine(row + 1),
                ])
            }
        } else if tcrd.slot == tslots::HCLK {
            EntityVec::from_iter([self.btile_hclk(tcrd.cell)])
        } else if tcrd.slot == tslots::IOB {
            if col == self.chip.col_w() || col == self.chip.col_e() {
                EntityVec::from_iter(
                    self[tcrd]
                        .cells
                        .values()
                        .map(|&cell| self.btile_term_h(cell)),
                )
            } else {
                EntityVec::from_iter(
                    self[tcrd]
                        .cells
                        .values()
                        .map(|&cell| self.btile_term_v(cell)),
                )
            }
        } else if matches!(tile.class, tcls::TERM_W | tcls::TERM_E) {
            EntityVec::from_iter([self.btile_term_h(tcrd.cell)])
        } else if matches!(
            tile.class,
            tcls::TERM_S | tcls::TERM_N | tcls::DCMCONN_S | tcls::DCMCONN_N
        ) {
            EntityVec::from_iter([self.btile_term_v(tcrd.cell)])
        } else if tcls.bels.contains_id(bslots::DCM) {
            if self.chip.kind.is_virtex2() {
                EntityVec::from_iter([self.btile_main(tcrd.cell), self.btile_term_v(tcrd.cell)])
            } else if self.chip.kind == ChipKind::Spartan3 {
                EntityVec::from_iter([self.btile_main(tcrd.cell)])
            } else {
                match tile.class {
                    tcls::DCM_S3E_SW | tcls::DCM_S3E_EN => EntityVec::from_iter([
                        self.btile_main(tcrd.delta(0, 0)),
                        self.btile_main(tcrd.delta(0, 1)),
                        self.btile_main(tcrd.delta(0, 2)),
                        self.btile_main(tcrd.delta(0, 3)),
                        self.btile_main(tcrd.delta(-3, 0)),
                        self.btile_main(tcrd.delta(-3, 1)),
                        self.btile_main(tcrd.delta(-3, 2)),
                        self.btile_main(tcrd.delta(-3, 3)),
                    ]),
                    tcls::DCM_S3E_SE | tcls::DCM_S3E_WN => EntityVec::from_iter([
                        self.btile_main(tcrd.delta(0, 0)),
                        self.btile_main(tcrd.delta(0, 1)),
                        self.btile_main(tcrd.delta(0, 2)),
                        self.btile_main(tcrd.delta(0, 3)),
                        self.btile_main(tcrd.delta(3, 0)),
                        self.btile_main(tcrd.delta(3, 1)),
                        self.btile_main(tcrd.delta(3, 2)),
                        self.btile_main(tcrd.delta(3, 3)),
                    ]),
                    tcls::DCM_S3E_NW | tcls::DCM_S3E_ES => EntityVec::from_iter([
                        self.btile_main(tcrd.delta(0, 0)),
                        self.btile_main(tcrd.delta(0, -3)),
                        self.btile_main(tcrd.delta(0, -2)),
                        self.btile_main(tcrd.delta(0, -1)),
                        self.btile_main(tcrd.delta(-3, -3)),
                        self.btile_main(tcrd.delta(-3, -2)),
                        self.btile_main(tcrd.delta(-3, -1)),
                        self.btile_main(tcrd.delta(-3, 0)),
                    ]),
                    tcls::DCM_S3E_NE | tcls::DCM_S3E_WS => EntityVec::from_iter([
                        self.btile_main(tcrd.delta(0, 0)),
                        self.btile_main(tcrd.delta(0, -3)),
                        self.btile_main(tcrd.delta(0, -2)),
                        self.btile_main(tcrd.delta(0, -1)),
                        self.btile_main(tcrd.delta(3, -3)),
                        self.btile_main(tcrd.delta(3, -2)),
                        self.btile_main(tcrd.delta(3, -1)),
                        self.btile_main(tcrd.delta(3, 0)),
                    ]),
                    _ => unreachable!(),
                }
            }
        } else if tile.class == tcls::RANDOR_FC {
            EntityVec::from_iter([self.btile_term_v(tcrd.cell)])
        } else if tcrd.slot == tslots::BEL
            && (col == self.chip.col_w() || col == self.chip.col_e())
            && (row == self.chip.row_s() || row == self.chip.row_n())
        {
            // CNR
            if self.chip.kind.is_virtex2() {
                EntityVec::from_iter([self.btile_term_h(tcrd.cell), self.btile_term_v(tcrd.cell)])
            } else {
                EntityVec::from_iter([self.btile_term_h(tcrd.cell)])
            }
        } else if tcrd.slot == tslots::RANDOR {
            EntityVec::from_iter([self.btile_main(tcrd.cell)])
        } else if tile.class == tcls::PPC_TERM_N {
            EntityVec::from_iter([self.btile_main(tcrd.delta(0, 1))])
        } else if tile.class == tcls::PPC_TERM_S {
            EntityVec::from_iter([self.btile_main(tcrd.delta(0, -1))])
        } else if !self.chip.kind.is_virtex2() && tcrd.slot == tslots::TERM_V {
            if self.chip.kind == ChipKind::Spartan3E {
                EntityVec::from_iter([self.btile_llv_s(col), self.btile_llv_n(col)])
            } else {
                EntityVec::from_iter([self.btile_llv(col)])
            }
        } else if !self.chip.kind.is_virtex2() && tcrd.slot == tslots::TERM_H {
            EntityVec::from_iter([self.btile_spine(row)])
        } else {
            EntityVec::from_iter(self[tcrd].cells.values().map(|&cell| self.btile_main(cell)))
        }
    }

    pub fn bel_carry_prev(&self, bcrd: BelCoord) -> Option<BelCoord> {
        if bslots::SLICE.contains(bcrd.slot) {
            todo!()
        } else if matches!(bcrd.slot, bslots::MULT | bslots::DSP) {
            let mut bcrd = bcrd;
            loop {
                bcrd.cell = self.cell_delta(bcrd.cell, 0, -4)?;
                if self.has_bel(bcrd) {
                    return Some(bcrd);
                }
            }
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
