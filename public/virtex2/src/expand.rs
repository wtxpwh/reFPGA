use prjcombine_entity::{EntityId, EntityIdArith, EntityPartVec, EntityVec};
use prjcombine_interconnect::db::IntDb;
use prjcombine_interconnect::dir::DirHV;
use prjcombine_interconnect::grid::builder::GridBuilder;
use prjcombine_interconnect::grid::{CellCoord, ColId, DieId, DieIdExt, Rect};

use crate::bitstream::FrameAddr;
use crate::chip::{Chip, ChipKind, ColumnIoKind, ColumnKind, DcmPairKind};
use crate::defs::{ccls, cslots, rslots, tcls, tslots};
use crate::expanded::ExpandedDevice;
use crate::iob::{get_iob_data_e, get_iob_data_n, get_iob_data_s, get_iob_data_w};

struct Expander<'a, 'b> {
    chip: &'b Chip,
    die: DieId,
    egrid: &'a mut GridBuilder<'b>,
    holes: Vec<Rect>,
    frame_info: Vec<FrameAddr>,
    clkv_frame: usize,
    spine_frame: usize,
    term_w_frame: usize,
    term_e_frame: usize,
    col_frame: EntityVec<ColId, usize>,
    bram_frame: EntityPartVec<ColId, usize>,
}

impl Expander<'_, '_> {
    fn is_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    fn fill_cnr(&mut self) {
        let tcid_int = if self.chip.kind.is_virtex2() {
            tcls::INT_CNR
        } else if self.chip.kind == ChipKind::FpgaCore {
            tcls::INT_CLB_FC
        } else {
            tcls::INT_CLB
        };
        let (tcid_sw, tcid_se, tcid_nw, tcid_ne) = match self.chip.kind {
            ChipKind::Virtex2 => (
                tcls::CNR_SW_V2,
                tcls::CNR_SE_V2,
                tcls::CNR_NW_V2,
                tcls::CNR_NE_V2,
            ),
            ChipKind::Virtex2P | ChipKind::Virtex2PX => (
                tcls::CNR_SW_V2P,
                tcls::CNR_SE_V2P,
                tcls::CNR_NW_V2P,
                tcls::CNR_NE_V2P,
            ),
            ChipKind::Spartan3 => (
                tcls::CNR_SW_S3,
                tcls::CNR_SE_S3,
                tcls::CNR_NW_S3,
                tcls::CNR_NE_S3,
            ),
            ChipKind::FpgaCore => (
                tcls::CNR_SW_FC,
                tcls::CNR_SE_FC,
                tcls::CNR_NW_FC,
                tcls::CNR_NE_FC,
            ),
            ChipKind::Spartan3E => (
                tcls::CNR_SW_S3E,
                tcls::CNR_SE_S3E,
                tcls::CNR_NW_S3E,
                tcls::CNR_NE_S3E,
            ),
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => (
                tcls::CNR_SW_S3A,
                tcls::CNR_SE_S3A,
                tcls::CNR_NW_S3A,
                tcls::CNR_NE_S3A,
            ),
        };

        self.egrid
            .add_tile_single(self.chip.corner(DirHV::SW).cell, tcid_int);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::SE).cell, tcid_int);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::NW).cell, tcid_int);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::NE).cell, tcid_int);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::SW).cell, tcid_sw);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::SE).cell, tcid_se);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::NW).cell, tcid_nw);
        self.egrid
            .add_tile_single(self.chip.corner(DirHV::NE).cell, tcid_ne);

        if !self.chip.kind.is_virtex2() {
            self.egrid.add_tile(
                self.chip.corner(DirHV::NW).cell,
                if self.chip.kind == ChipKind::FpgaCore {
                    tcls::RANDOR_INIT_FC
                } else {
                    tcls::RANDOR_INIT
                },
                &[],
            );
        }
    }

    fn fill_term(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            if self.chip.kind.is_virtex2() {
                self.egrid.add_tile_single(cell, tcls::TERM_S);
            }
            self.egrid.fill_conn_term(cell, ccls::TERM_S);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            if self.chip.kind.is_virtex2() {
                self.egrid.add_tile_single(cell, tcls::TERM_N);
            }
            self.egrid.fill_conn_term(cell, ccls::TERM_N);
        }
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            if self.chip.kind.is_virtex2() {
                self.egrid.add_tile_single(cell, tcls::TERM_W);
            }
            self.egrid.fill_conn_term(cell, ccls::TERM_W);
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if self.chip.kind.is_virtex2() {
                self.egrid.add_tile_single(cell, tcls::TERM_E);
            }
            self.egrid.fill_conn_term(cell, ccls::TERM_E);
        }
    }

    fn fill_io_n(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            let cd = &self.chip.columns[cell.col];
            if self.chip.kind.is_spartan3ea() {
                if cd.kind == ColumnKind::Io {
                    continue;
                }
            } else {
                if cd.kind != ColumnKind::Clb {
                    continue;
                }
            }

            let (int_kind, ioi_kind) = match self.chip.kind {
                ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                    if cd.io == ColumnIoKind::DoubleEClk(1) {
                        (tcls::INT_IOI_CLK_N, tcls::IOI_CLK_N)
                    } else {
                        (tcls::INT_IOI, tcls::IOI)
                    }
                }
                ChipKind::Spartan3 => (tcls::INT_IOI_S3, tcls::IOI_S3),
                ChipKind::FpgaCore => (tcls::INT_IOI_FC, tcls::IOI_FC),
                ChipKind::Spartan3E => (tcls::INT_IOI_S3E, tcls::IOI_S3E),
                ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                    (tcls::INT_IOI_S3A_SN, tcls::IOI_S3A_N)
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, ioi_kind);
            if cd.io != ColumnIoKind::None {
                let (data, tidx) = get_iob_data_n(self.chip.kind, cd.io);
                if tidx.to_idx() == 0 {
                    self.egrid.add_tile_e(cell, data.tcid, data.cells);
                }
            }
            if !self.chip.kind.is_virtex2() {
                self.egrid.add_tile(
                    cell,
                    if self.chip.kind == ChipKind::FpgaCore {
                        tcls::RANDOR_FC
                    } else {
                        tcls::RANDOR
                    },
                    &[],
                );
            }
        }
    }

    fn fill_io_e(&mut self) {
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            if self.chip.is_row_io(cell.row) {
                continue;
            }
            let (int_kind, ioi_kind) = match self.chip.kind {
                ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                    (tcls::INT_IOI, tcls::IOI)
                }
                ChipKind::Spartan3 => (tcls::INT_IOI_S3, tcls::IOI_S3),
                ChipKind::FpgaCore => (tcls::INT_IOI_FC, tcls::IOI_FC),
                ChipKind::Spartan3E => (tcls::INT_IOI_S3E, tcls::IOI_S3E),
                ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                    (tcls::INT_IOI_S3A_WE, tcls::IOI_S3A_WE)
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, ioi_kind);
            let (data, tidx) = get_iob_data_e(self.chip.kind, self.chip.rows[cell.row]);
            if tidx.to_idx() == 0 {
                self.egrid.add_tile_n(cell, data.tcid, data.cells);
            }
        }
    }

    fn fill_io_s(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            let cd = &self.chip.columns[cell.col];
            if self.chip.kind.is_spartan3ea() {
                if cd.kind == ColumnKind::Io {
                    continue;
                }
            } else {
                if cd.kind != ColumnKind::Clb {
                    continue;
                }
            }

            let (int_kind, ioi_kind) = match self.chip.kind {
                ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                    if cd.io == ColumnIoKind::DoubleEClk(1) {
                        (tcls::INT_IOI_CLK_S, tcls::IOI_CLK_S)
                    } else {
                        (tcls::INT_IOI, tcls::IOI)
                    }
                }
                ChipKind::Spartan3 => (tcls::INT_IOI_S3, tcls::IOI_S3),
                ChipKind::FpgaCore => (tcls::INT_IOI_FC, tcls::IOI_FC),
                ChipKind::Spartan3E => (tcls::INT_IOI_S3E, tcls::IOI_S3E),
                ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                    (tcls::INT_IOI_S3A_SN, tcls::IOI_S3A_S)
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, ioi_kind);
            if cd.io != ColumnIoKind::None {
                let (data, tidx) = get_iob_data_s(self.chip.kind, cd.io);
                if tidx.to_idx() == 0 {
                    self.egrid.add_tile_e(cell, data.tcid, data.cells);
                }
            }
            if !self.chip.kind.is_virtex2() && self.chip.kind != ChipKind::FpgaCore {
                self.egrid.add_tile(cell, tcls::RANDOR, &[]);
            }
        }
    }

    fn fill_io_w(&mut self) {
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            if self.chip.is_row_io(cell.row) {
                continue;
            }
            let (int_kind, ioi_kind) = match self.chip.kind {
                ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                    (tcls::INT_IOI, tcls::IOI)
                }
                ChipKind::Spartan3 => (tcls::INT_IOI_S3, tcls::IOI_S3),
                ChipKind::FpgaCore => (tcls::INT_IOI_FC, tcls::IOI_FC),
                ChipKind::Spartan3E => (tcls::INT_IOI_S3E, tcls::IOI_S3E),
                ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                    (tcls::INT_IOI_S3A_WE, tcls::IOI_S3A_WE)
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, ioi_kind);
            let (data, tidx) = get_iob_data_w(self.chip.kind, self.chip.rows[cell.row]);
            if tidx.to_idx() == 0 {
                self.egrid.add_tile_n(cell, data.tcid, data.cells);
            }
        }
    }

    fn fill_clb(&mut self) {
        for (col, &cd) in self.chip.columns.iter() {
            if self.chip.kind == ChipKind::Spartan3E {
                if cd.kind == ColumnKind::Io {
                    continue;
                }
            } else {
                if cd.kind != ColumnKind::Clb {
                    continue;
                }
            }
            for cell in self.egrid.column(self.die, col) {
                if self.chip.is_row_io(cell.row) {
                    continue;
                }
                if self.is_hole(cell) {
                    continue;
                }
                if self.chip.kind.is_virtex2() {
                    self.egrid.add_tile_single(cell, tcls::INT_CLB);
                    self.egrid.add_tile_single(cell, tcls::CLB);
                } else {
                    if self.chip.kind == ChipKind::FpgaCore {
                        self.egrid.add_tile_single(cell, tcls::INT_CLB_FC);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::INT_CLB);
                    }
                    self.egrid.add_tile_single(cell, tcls::CLB);
                }
            }
        }
    }

    fn fill_bram_dsp(&mut self) {
        let bram_kind = match self.chip.kind {
            ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => [tcls::INT_BRAM; 4],
            ChipKind::Spartan3 => [tcls::INT_BRAM_S3; 4],
            ChipKind::FpgaCore => return,
            ChipKind::Spartan3E => [tcls::INT_BRAM_S3E; 4],
            ChipKind::Spartan3A => [
                tcls::INT_BRAM_S3A_03,
                tcls::INT_BRAM_S3A_12,
                tcls::INT_BRAM_S3A_12,
                tcls::INT_BRAM_S3A_03,
            ],
            ChipKind::Spartan3ADsp => [tcls::INT_BRAM_S3ADSP; 4],
        };
        for (col, &cd) in self.chip.columns.iter() {
            if cd.kind != ColumnKind::Bram {
                continue;
            }
            if let Some((b, t)) = self.chip.rows_ram {
                self.holes.push(Rect {
                    die: self.die,
                    col_w: col,
                    col_e: col + 4,
                    row_s: b,
                    row_n: t + 1,
                });
                for d in 1..4 {
                    self.egrid.fill_conn_pair(
                        self.die.cell(col + d, b - 1),
                        self.die.cell(col + d, t + 1),
                        ccls::TERM_BRAM_N,
                        ccls::TERM_BRAM_S,
                    );
                }
            }
            for cell in self.egrid.column(self.die, col) {
                if self.chip.kind != ChipKind::Spartan3E && self.is_hole(cell) {
                    continue;
                }
                let Some(idx) = self.chip.bram_row(cell.row) else {
                    continue;
                };
                self.egrid.add_tile_single(cell, bram_kind[idx]);
                if self.chip.kind == ChipKind::Spartan3ADsp {
                    self.egrid
                        .add_tile_single(cell.delta(3, 0), tcls::INT_BRAM_S3ADSP);
                }
                if idx == 0 {
                    let kind = match self.chip.kind {
                        ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => tcls::BRAM,
                        ChipKind::Spartan3 => tcls::BRAM_S3,
                        ChipKind::FpgaCore => unreachable!(),
                        ChipKind::Spartan3E => tcls::BRAM_S3E,
                        ChipKind::Spartan3A => tcls::BRAM_S3A,
                        ChipKind::Spartan3ADsp => tcls::BRAM_S3ADSP,
                    };
                    self.egrid.add_tile_n(cell, kind, 4);
                    if self.chip.kind == ChipKind::Spartan3ADsp {
                        let cell = cell.delta(3, 0);
                        self.egrid.add_tile_n(cell, tcls::DSP, 4);
                    }
                }
            }
        }
    }

    fn fill_dcm(&mut self) {
        if self.chip.kind.is_spartan3ea() {
            for pair in self.chip.get_dcm_pairs() {
                match pair.kind {
                    DcmPairKind::S => {
                        self.holes.push(pair.cell.delta(-4, 0).rect(8, 4));
                        let cell = pair.cell.delta(-1, 0);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_SW);
                        let cell_root = cell.with_row(self.chip.row_s());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_SE);
                        let cell_root = cell.with_row(self.chip.row_s());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::SingleS => {
                        self.holes.push(pair.cell.delta(-1, 0).rect(5, 4));
                        let cell = pair.cell.delta(-1, 0);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM_S3E_DUMMY);
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_SE);
                        let cell_root = cell.with_row(self.chip.row_s());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::N => {
                        self.holes.push(pair.cell.delta(-4, -3).rect(8, 4));
                        let cell = pair.cell.delta(-1, 0);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_NW);
                        let cell_root = cell.with_row(self.chip.row_n());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_NE);
                        let cell_root = cell.with_row(self.chip.row_n());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::SingleN => {
                        self.holes.push(pair.cell.delta(-1, -3).rect(5, 4));
                        let cell = pair.cell.delta(-1, 0);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM_S3E_DUMMY);
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_NE);
                        let cell_root = cell.with_row(self.chip.row_n());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::W => {
                        self.holes.push(pair.cell.delta(0, -4).rect(4, 8));
                        let cell = pair.cell.delta(0, -1);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_WS);
                        let cell_root = cell.with_col(self.chip.col_w());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_WN);
                        let cell_root = cell.with_col(self.chip.col_w());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::E => {
                        self.holes.push(pair.cell.delta(-3, -4).rect(4, 8));
                        let cell = pair.cell.delta(0, -1);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_ES);
                        let cell_root = cell.with_col(self.chip.col_e());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_EN);
                        let cell_root = cell.with_col(self.chip.col_e());
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                    DcmPairKind::Bram => {
                        let col_root = if pair.cell.col < self.chip.col_clk {
                            self.chip.col_w()
                        } else {
                            self.chip.col_e()
                        };
                        self.holes.push(pair.cell.delta(0, -4).rect(4, 8));
                        let cell = pair.cell.delta(0, -1);
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_WS);
                        let cell_root = cell.with_col(col_root);
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                        let cell = pair.cell;
                        self.egrid.add_tile_single(cell, tcls::INT_DCM);
                        self.egrid.add_tile_single(cell, tcls::DCM_S3E_WN);
                        let cell_root = cell.with_col(col_root);
                        self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
                        self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
                    }
                }
            }
        } else {
            let row_b = self.chip.row_s();
            let row_t = self.chip.row_n();
            for (col, &cd) in self.chip.columns.iter() {
                if cd.kind != ColumnKind::Bram {
                    continue;
                }
                if !self.chip.cols_gt.contains_key(&col) {
                    let (kind, dcm) = match self.chip.kind {
                        ChipKind::Virtex2 => (tcls::INT_DCM_V2, Some(tcls::DCM_V2)),
                        ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                            (tcls::INT_DCM_V2P, Some(tcls::DCM_V2P))
                        }
                        ChipKind::Spartan3 => {
                            if col == self.chip.col_w() + 3 || col == self.chip.col_e() - 3 {
                                (tcls::INT_DCM, Some(tcls::DCM_S3))
                            } else {
                                (tcls::INT_DCM_S3_DUMMY, None)
                            }
                        }
                        _ => unreachable!(),
                    };
                    for row in [self.chip.row_s(), self.chip.row_n()] {
                        let cell = self.die.cell(col, row);
                        self.egrid.add_tile_single(cell, kind);
                        if let Some(dcm) = dcm {
                            self.egrid.add_tile_single(cell, dcm);
                        }
                    }
                }
                if self.chip.kind.is_virtex2() {
                    self.egrid
                        .add_tile_single(self.die.cell(col, row_b), tcls::DCMCONN_S);
                    self.egrid
                        .add_tile_single(self.die.cell(col, row_t), tcls::DCMCONN_N);
                } else {
                    if col == self.chip.col_w() + 3 || col == self.chip.col_e() - 3 {
                        self.egrid
                            .add_tile_single(self.die.cell(col, row_b), tcls::DCMCONN_S);
                        self.egrid
                            .add_tile_single(self.die.cell(col, row_t), tcls::DCMCONN_N);
                    }
                }
            }
        }
    }

    fn fill_ppc(&mut self) {
        for &(bc, br) in &self.chip.holes_ppc {
            let cell = self.die.cell(bc, br);
            self.holes.push(cell.rect(10, 16));

            let mut tcells = vec![];
            // left side
            tcells.extend(cell.cells_n_const::<16>());
            // right side
            tcells.extend(cell.delta(9, 0).cells_n_const::<16>());
            // bottom
            tcells.extend(cell.delta(1, 0).cells_e_const::<8>());
            // top
            tcells.extend(cell.delta(1, 15).cells_e_const::<8>());

            for &cell in &tcells {
                self.egrid.add_tile_single(cell, tcls::INT_PPC);
                self.egrid.add_tile_single(cell, tcls::INTF_PPC);
            }
            // horiz passes
            for d in 1..15 {
                let cell_w = cell.delta(0, d);
                let cell_e = cell.delta(9, d);
                self.egrid
                    .add_tile(cell_w, tcls::PPC_TERM_E, &[cell_w, cell_e]);
                self.egrid
                    .add_tile(cell_e, tcls::PPC_TERM_W, &[cell_e, cell_w]);
                self.egrid
                    .fill_conn_pair(cell_w, cell_e, ccls::PPC_E, ccls::PPC_W);
            }
            // vert passes
            for d in 1..9 {
                let cell_s = cell.delta(d, 0);
                let cell_n = cell.delta(d, 15);
                self.egrid
                    .add_tile(cell_s, tcls::PPC_TERM_N, &[cell_s, cell_n]);
                self.egrid
                    .add_tile(cell_n, tcls::PPC_TERM_S, &[cell_n, cell_s]);
                self.egrid
                    .fill_conn_pair(cell_s, cell_n, ccls::PPC_N, ccls::PPC_S);
            }
            let tcid = if bc < self.chip.col_clk {
                tcls::PPC_W
            } else {
                tcls::PPC_E
            };
            self.egrid.add_tile(cell, tcid, &tcells);
        }
    }

    fn fill_gt(&mut self) {
        for col in self.chip.cols_gt.keys().copied() {
            let cell_s = self.die.cell(col, self.chip.row_s());
            let cell_n = self.die.cell(col, self.chip.row_n());
            if self.chip.kind == ChipKind::Virtex2PX {
                self.holes.push(cell_s.rect(1, 9));
                self.holes.push(cell_n.delta(0, -8).rect(1, 9));
            } else {
                self.holes.push(cell_s.rect(1, 5));
                self.holes.push(cell_n.delta(0, -4).rect(1, 5));
            }
            self.egrid.add_tile_single(cell_s, tcls::INT_GT_CLKPAD);
            self.egrid.add_tile_single(cell_n, tcls::INT_GT_CLKPAD);
            self.egrid.add_tile_single(cell_s, tcls::INTF_GT_S_CLKPAD);
            self.egrid.add_tile_single(cell_n, tcls::INTF_GT_N_CLKPAD);
            let n = match self.chip.kind {
                ChipKind::Virtex2P => 4,
                ChipKind::Virtex2PX => 8,
                _ => unreachable!(),
            };
            for d in 0..n {
                let cell = cell_s.delta(0, 1 + d);
                self.egrid.add_tile_single(cell, tcls::INT_PPC);
                self.egrid.add_tile_single(
                    cell,
                    if d % 4 == 0 {
                        tcls::INTF_GT_S0
                    } else {
                        tcls::INTF_GT_S123
                    },
                );
            }
            for d in 0..n {
                let cell = cell_n.delta(0, -n + d);
                self.egrid.add_tile_single(cell, tcls::INT_PPC);
                self.egrid.add_tile_single(
                    cell,
                    if d % 4 == 0 {
                        tcls::INTF_GT_N0
                    } else {
                        tcls::INTF_GT_N123
                    },
                );
            }
            if self.chip.kind == ChipKind::Virtex2P {
                self.egrid.add_tile(
                    cell_s,
                    tcls::GIGABIT_S,
                    &[
                        cell_s,
                        cell_s.delta(0, 1),
                        cell_s.delta(0, 2),
                        cell_s.delta(0, 3),
                        cell_s.delta(0, 4),
                    ],
                );
                self.egrid.add_tile(
                    cell_n,
                    tcls::GIGABIT_N,
                    &[
                        cell_n,
                        cell_n.delta(0, -4),
                        cell_n.delta(0, -3),
                        cell_n.delta(0, -2),
                        cell_n.delta(0, -1),
                    ],
                );
            } else {
                self.egrid.add_tile(
                    cell_s,
                    tcls::GIGABIT10_S,
                    &[
                        cell_s,
                        cell_s.delta(0, 1),
                        cell_s.delta(0, 2),
                        cell_s.delta(0, 3),
                        cell_s.delta(0, 4),
                        cell_s.delta(0, 5),
                        cell_s.delta(0, 6),
                        cell_s.delta(0, 7),
                        cell_s.delta(0, 8),
                    ],
                );
                self.egrid.add_tile(
                    cell_n,
                    tcls::GIGABIT10_N,
                    &[
                        cell_n,
                        cell_n.delta(0, -8),
                        cell_n.delta(0, -7),
                        cell_n.delta(0, -6),
                        cell_n.delta(0, -5),
                        cell_n.delta(0, -4),
                        cell_n.delta(0, -3),
                        cell_n.delta(0, -2),
                        cell_n.delta(0, -1),
                    ],
                );
            }
        }
    }

    fn fill_llv(&mut self) {
        for cell in self.egrid.row(self.die, self.chip.row_mid()) {
            if matches!(self.chip.columns[cell.col].kind, ColumnKind::BramCont(_)) {
                continue;
            }
            let mut cell_s = cell.delta(0, -1);
            let mut cell_n = cell;
            while !self.egrid[cell_s].tiles.contains_id(tslots::INT) {
                cell_s.row -= 1;
            }
            while !self.egrid[cell_n].tiles.contains_id(tslots::INT) {
                cell_n.row += 1;
            }
            let mut term_s = ccls::LLV_S;
            let mut term_n = ccls::LLV_N;
            if self.chip.is_col_io(cell.col) && self.chip.kind != ChipKind::Spartan3A {
                term_s = ccls::LLV_CLK_WE_S3E_S;
                term_n = ccls::LLV_CLK_WE_S3E_N;
            }
            self.egrid.fill_conn_pair(cell_s, cell_n, term_n, term_s);
            self.egrid.add_tile(
                cell_n,
                if self.chip.kind.is_spartan3a() {
                    tcls::LLV_S3A
                } else {
                    tcls::LLV_S3E
                },
                &[cell_s, cell_n],
            );
        }
    }

    fn fill_llh(&mut self) {
        for cell in self.egrid.column(self.die, self.chip.col_clk) {
            let mut cell_w = cell.delta(-1, 0);
            let mut cell_e = cell;
            while !self.egrid[cell_w].tiles.contains_id(tslots::INT) {
                cell_w.col -= 1;
            }
            while !self.egrid[cell_e].tiles.contains_id(tslots::INT) {
                cell_e.col += 1;
            }
            let mut term_w = ccls::LLH_W;
            let mut term_e = ccls::LLH_E;
            if self.chip.kind == ChipKind::Spartan3ADsp
                && [
                    self.chip.row_s() + 2,
                    self.chip.row_s() + 3,
                    self.chip.row_s() + 4,
                    self.chip.row_n() - 4,
                    self.chip.row_n() - 3,
                    self.chip.row_n() - 2,
                ]
                .into_iter()
                .any(|x| x == cell.row)
            {
                term_w = ccls::LLH_DCM_S3ADSP_W;
                term_e = ccls::LLH_DCM_S3ADSP_E;
            }
            self.egrid.fill_conn_pair(cell_w, cell_e, term_e, term_w);
            self.egrid.add_tile(
                cell_e,
                if self.chip.kind.is_spartan3a() && cell.row == self.chip.row_s() {
                    tcls::LLH_S_S3A
                } else if self.chip.kind.is_spartan3a() && cell.row == self.chip.row_n() {
                    tcls::LLH_N_S3A
                } else {
                    tcls::LLH
                },
                &[cell_w, cell_e],
            );
        }
    }

    fn fill_misc_passes(&mut self) {
        if self.chip.kind == ChipKind::Spartan3E && !self.chip.has_ll {
            for col in [self.chip.col_w(), self.chip.col_e()] {
                self.egrid.fill_conn_pair(
                    self.die.cell(col, self.chip.row_mid() - 1),
                    self.die.cell(col, self.chip.row_mid()),
                    ccls::CLK_WE_S3E_N,
                    ccls::CLK_WE_S3E_S,
                );
            }
        }
        if self.chip.kind == ChipKind::Spartan3 {
            for &(_, _, row) in &self.chip.rows_hclk {
                if row == self.chip.row_mid() {
                    continue;
                }
                if row - 1 == self.chip.row_n() {
                    continue;
                }
                for cell in self.egrid.row(self.die, row) {
                    self.egrid.fill_conn_pair(
                        cell.delta(0, -1),
                        cell,
                        ccls::BRKH_S3_N,
                        ccls::BRKH_S3_S,
                    );
                }
            }
        }
        if self.chip.kind == ChipKind::Spartan3ADsp {
            for (col, cd) in &self.chip.columns {
                if cd.kind == ColumnKind::Dsp {
                    for row in [self.chip.row_s(), self.chip.row_n()] {
                        self.egrid.fill_conn_pair(
                            self.die.cell(col, row),
                            self.die.cell(col + 1, row),
                            ccls::DSPHOLE_E,
                            ccls::DSPHOLE_W,
                        );
                    }
                }
            }
            for col in [self.chip.col_w() + 3, self.chip.col_e() - 6] {
                for row in [self.chip.row_mid() - 1, self.chip.row_mid()] {
                    self.egrid.fill_conn_pair(
                        self.die.cell(col, row),
                        self.die.cell(col + 4, row),
                        ccls::DSPHOLE_E,
                        ccls::DSPHOLE_W,
                    );
                }
                for row in [
                    self.chip.row_mid() - 4,
                    self.chip.row_mid() - 3,
                    self.chip.row_mid() - 2,
                    self.chip.row_mid() + 1,
                    self.chip.row_mid() + 2,
                    self.chip.row_mid() + 3,
                ] {
                    self.egrid.fill_conn_pair(
                        self.die.cell(col - 1, row),
                        self.die.cell(col + 4, row),
                        ccls::HDCM_E,
                        ccls::HDCM_W,
                    );
                }
            }
        }
    }

    fn fill_main_passes(&mut self) {
        let (pass_w, pass_e, pass_s, pass_n) = if self.chip.kind.is_virtex2() {
            (ccls::PASS_W, ccls::PASS_E, ccls::PASS_S, ccls::PASS_N)
        } else if self.chip.kind == ChipKind::FpgaCore {
            (ccls::PASS_W_FC, ccls::PASS_E, ccls::PASS_S_FC, ccls::PASS_N)
        } else {
            (ccls::PASS_W, ccls::PASS_E, ccls::PASS_S, ccls::PASS_N)
        };
        let slot_w = cslots::W;
        let slot_e = cslots::E;
        let slot_s = cslots::S;
        let slot_n = cslots::N;
        let die = DieId::from_idx(0);
        // horizontal
        for row in self.egrid.rows(die) {
            let mut prev = None;
            for cell in self.egrid.row(die, row) {
                if !self.egrid[cell].tiles.contains_id(tslots::INT) {
                    continue;
                }
                if let Some(prev) = prev
                    && !self.egrid[cell].conns.contains_id(slot_w)
                {
                    self.egrid.fill_conn_pair(prev, cell, pass_e, pass_w);
                }
                if !self.egrid[cell].conns.contains_id(slot_e) {
                    prev = Some(cell);
                } else {
                    prev = None;
                }
            }
        }
        // vertical
        for col in self.egrid.cols(die) {
            let mut prev = None;
            for cell in self.egrid.column(die, col) {
                if !self.egrid[cell].tiles.contains_id(tslots::INT) {
                    continue;
                }
                if let Some(prev) = prev
                    && !self.egrid[cell].conns.contains_id(slot_s)
                {
                    self.egrid.fill_conn_pair(prev, cell, pass_n, pass_s);
                }
                if !self.egrid[cell].conns.contains_id(slot_n) {
                    prev = Some(cell);
                } else {
                    prev = None;
                }
            }
        }
    }

    fn fill_bram_passes(&mut self) {
        if matches!(self.chip.kind, ChipKind::Spartan3A | ChipKind::Spartan3ADsp) {
            let slot_n = cslots::N;
            let slot_s = cslots::S;
            for (col, cd) in &self.chip.columns {
                if matches!(cd.kind, ColumnKind::BramCont(_)) {
                    self.egrid[self.die.cell(col, self.chip.row_s())]
                        .conns
                        .remove(slot_n);
                    self.egrid[self.die.cell(col, self.chip.row_n())]
                        .conns
                        .remove(slot_s);
                }
            }
        }
    }

    fn fill_clk_sn_v2(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            let cell_root = cell.with_col(if cell.col < self.chip.col_clk {
                self.chip.col_clk - 1
            } else {
                self.chip.col_clk
            });
            self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root;
            self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell_root;
        }
        {
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
            self.egrid.add_tile_we(cell, tcls::CLK_S, 1, 2);
        }
        {
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
            self.egrid.add_tile_we(cell, tcls::CLK_N, 1, 2);
        }

        {
            let cell = self.die.cell(self.chip.col_w(), self.chip.row_pci.unwrap());
            self.egrid.add_tile_sn(cell, tcls::PCI_W, 2, 4);
        }
        {
            let cell = self.die.cell(self.chip.col_e(), self.chip.row_pci.unwrap());
            self.egrid.add_tile_sn(cell, tcls::PCI_E, 2, 4);
        }
    }

    fn fill_clk_sn_s3(&mut self) {
        if !self.chip.kind.is_spartan3ea() {
            for cell in self.egrid.die_cells(self.die) {
                let cell_root_bus = cell.with_col(if cell.col < self.chip.col_clk {
                    self.chip.col_clk - 1
                } else {
                    self.chip.col_clk
                });
                self.egrid[cell].region_root[rslots::DCM_BUS] = cell_root_bus;
                self.egrid[cell].region_root[rslots::DCM_CLKPAD] = cell.with_col(self.chip.col_clk);
            }
        }
        let (tcid_s, tcid_n) = match self.chip.kind {
            ChipKind::Spartan3 => (tcls::CLK_S_S3, tcls::CLK_N_S3),
            ChipKind::FpgaCore => (tcls::CLK_S_FC, tcls::CLK_N_FC),
            ChipKind::Spartan3E => (tcls::CLK_S_S3E, tcls::CLK_N_S3E),
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => (tcls::CLK_S_S3A, tcls::CLK_N_S3A),
            _ => unreachable!(),
        };
        let (w, n) = if self.chip.kind.is_spartan3ea() {
            (4, 8)
        } else {
            (1, 2)
        };
        {
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_s());
            self.egrid.add_tile_we(cell, tcid_s, w, n);
        }
        {
            let cell = self.die.cell(self.chip.col_clk, self.chip.row_n());
            self.egrid.add_tile_we(cell, tcid_n, w, n);
        }
    }

    fn fill_clk_we_s3e(&mut self) {
        let (tcid_w, tcid_e) = if self.chip.kind == ChipKind::Spartan3E {
            (tcls::CLK_W_S3E, tcls::CLK_E_S3E)
        } else {
            (tcls::CLK_W_S3A, tcls::CLK_E_S3A)
        };
        {
            let cell = self.die.cell(self.chip.col_w(), self.chip.row_mid());
            self.egrid.add_tile_sn(cell, tcid_w, 4, 8);
        }
        {
            let cell = self.die.cell(self.chip.col_e(), self.chip.row_mid());
            self.egrid.add_tile_sn(cell, tcid_e, 4, 8);
        }
    }

    fn fill_hclk(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            self.egrid[cell].region_root[rslots::GLOBAL] =
                self.die.cell(self.chip.col_clk, self.chip.row_mid());
        }
        for col in self.egrid.cols(self.die) {
            let col_q = if col < self.chip.col_clk {
                if let Some((col_q, _)) = self.chip.cols_clkv {
                    col_q
                } else {
                    self.chip.col_clk - 1
                }
            } else {
                if let Some((_, col_q)) = self.chip.cols_clkv {
                    col_q
                } else {
                    self.chip.col_clk
                }
            };
            for &(row_m, row_b, row_t) in &self.chip.rows_hclk {
                let row_q = if self.chip.kind.is_virtex2() {
                    row_m
                } else if row_m < self.chip.row_mid() {
                    self.chip.row_mid() - 1
                } else {
                    self.chip.row_mid()
                };
                for row in row_b.range(row_m) {
                    let row_root = if matches!(self.chip.columns[col].kind, ColumnKind::BramCont(_))
                        && self.chip.kind.is_spartan3a()
                    {
                        row_m
                    } else {
                        row_m - 1
                    };
                    let cell = self.die.cell(col, row);
                    self.egrid[cell].region_root[rslots::LEAF] = cell.with_row(row_root);
                    self.egrid[cell].region_root[rslots::HCLK] = self.die.cell(col_q, row_q);
                }
                for row in row_m.range(row_t) {
                    let cell = self.die.cell(col, row);
                    self.egrid[cell].region_root[rslots::LEAF] = cell.with_row(row_m);
                    self.egrid[cell].region_root[rslots::HCLK] = self.die.cell(col_q, row_q);
                }
                let kind = if matches!(self.chip.columns[col].kind, ColumnKind::BramCont(_))
                    && self.chip.kind.is_spartan3a()
                {
                    tcls::HCLK_UNI
                } else {
                    tcls::HCLK
                };
                let cell = self.die.cell(col, row_m);
                self.egrid.add_tile(cell, kind, &[cell.delta(0, -1), cell]);
            }
        }
    }

    fn fill_hrow(&mut self) {
        for &(row_m, _, _) in &self.chip.rows_hclk {
            if self.chip.kind.is_virtex2() {
                let kind = if row_m == self.chip.row_s() + 1 {
                    tcls::HROW_S
                } else if row_m == self.chip.row_n() {
                    tcls::HROW_N
                } else {
                    tcls::HROW
                };
                self.egrid
                    .add_tile_we(self.die.cell(self.chip.col_clk, row_m), kind, 1, 2);
            }
        }
    }

    fn fill_clkc(&mut self) {
        if !self.chip.kind.is_virtex2() && self.chip.cols_clkv.is_none() {
            self.egrid.add_tile_we(
                self.die.cell(self.chip.col_clk, self.chip.row_mid()),
                tcls::CLKC_50A,
                1,
                2,
            );
        }
    }

    fn fill_clkqc(&mut self) {
        if let Some((col_cl, col_cr)) = self.chip.cols_clkv {
            if matches!(self.chip.kind, ChipKind::Spartan3 | ChipKind::FpgaCore) {
                self.egrid.add_tile_sn(
                    self.die.cell(col_cl, self.chip.row_mid()),
                    tcls::CLKQC_S3,
                    1,
                    2,
                );
                self.egrid.add_tile_sn(
                    self.die.cell(col_cr, self.chip.row_mid()),
                    tcls::CLKQC_S3,
                    1,
                    2,
                );
            } else {
                self.egrid.add_tile_sn(
                    self.die.cell(col_cl, self.chip.row_mid()),
                    tcls::CLKQC_S3E,
                    1,
                    2,
                );
                self.egrid.add_tile_sn(
                    self.die.cell(col_cr, self.chip.row_mid()),
                    tcls::CLKQC_S3E,
                    1,
                    2,
                );
            }
        }
    }

    fn fill_global(&mut self) {
        let tcid = match self.chip.kind {
            ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => tcls::GLOBAL,
            ChipKind::Spartan3 => tcls::GLOBAL_S3,
            ChipKind::FpgaCore => tcls::GLOBAL_FC,
            ChipKind::Spartan3E => tcls::GLOBAL_S3E,
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => tcls::GLOBAL_S3A,
        };
        self.egrid.add_tile(self.chip.tile_global().cell, tcid, &[]);
    }

    fn fill_frame_info(&mut self) {
        let mut major = 0;
        // spine
        self.clkv_frame = 0;
        let num_spine = if self.chip.kind.is_virtex2() {
            self.spine_frame = 0;
            4
        } else if self.chip.cols_clkv.is_none() {
            self.spine_frame = 0;
            2
        } else if self.chip.has_ll || self.chip.kind.is_spartan3a() {
            self.spine_frame = 2;
            4
        } else {
            self.spine_frame = 2;
            3
        };
        for minor in 0..num_spine {
            self.frame_info.push(FrameAddr {
                typ: 0,
                major,
                minor,
            });
        }
        major += 1;
        let num_term = if self.chip.kind.is_virtex2() { 4 } else { 2 };
        self.term_w_frame = self.frame_info.len();
        for minor in 0..num_term {
            self.frame_info.push(FrameAddr {
                typ: 0,
                major,
                minor,
            });
        }
        major += 1;
        let num_main = if self.chip.kind.is_virtex2() { 22 } else { 19 };
        for (_, cd) in &self.chip.columns {
            // For Bram and BramCont, to be fixed later.
            self.col_frame.push(self.frame_info.len());
            if matches!(cd.kind, ColumnKind::BramCont(_) | ColumnKind::Bram) {
                continue;
            }
            for minor in 0..num_main {
                self.frame_info.push(FrameAddr {
                    typ: 0,
                    major,
                    minor,
                });
            }
            major += 1;
        }
        self.term_e_frame = self.frame_info.len();
        for minor in 0..num_term {
            self.frame_info.push(FrameAddr {
                typ: 0,
                major,
                minor,
            });
        }

        major = 0;
        let num_bram = if self.chip.kind.is_virtex2() { 64 } else { 76 };
        for (col, cd) in &self.chip.columns {
            if cd.kind != ColumnKind::Bram {
                continue;
            }
            self.bram_frame.insert(col, self.frame_info.len());
            for minor in 0..num_bram {
                self.frame_info.push(FrameAddr {
                    typ: 1,
                    major,
                    minor,
                });
            }
            major += 1;
        }

        major = 0;
        for (col, cd) in &self.chip.columns {
            if cd.kind != ColumnKind::Bram {
                continue;
            }
            self.col_frame[col] = self.frame_info.len();
            for minor in 0..num_main {
                self.frame_info.push(FrameAddr {
                    typ: 2,
                    major,
                    minor,
                });
            }
            major += 1;
        }

        for (col, cd) in &self.chip.columns {
            if let ColumnKind::BramCont(i) = cd.kind {
                self.col_frame[col] = self.bram_frame[col - (i as usize)] + (i as usize - 1) * 19;
            }
        }
    }
}

impl Chip {
    pub fn expand_grid<'a>(&'a self, db: &'a IntDb) -> ExpandedDevice<'a> {
        let mut egrid = GridBuilder::new(db);

        let die = egrid.add_die(self.columns.len(), self.rows.len());

        let mut expander = Expander {
            chip: self,
            die,
            egrid: &mut egrid,
            holes: vec![],
            frame_info: vec![],
            clkv_frame: 0,
            spine_frame: 0,
            term_w_frame: 0,
            term_e_frame: 0,
            col_frame: EntityVec::new(),
            bram_frame: EntityPartVec::new(),
        };

        expander.fill_gt();
        expander.fill_ppc();
        expander.fill_dcm();
        expander.fill_bram_dsp();
        expander.fill_clb();
        expander.fill_cnr();
        expander.fill_io_n();
        expander.fill_io_e();
        expander.fill_io_s();
        expander.fill_io_w();
        expander.fill_term();
        if self.has_ll {
            expander.fill_llv();
            expander.fill_llh();
        }
        expander.fill_misc_passes();
        expander.fill_main_passes();
        expander.fill_bram_passes();
        if self.kind.is_virtex2() {
            expander.fill_clk_sn_v2();
        } else if matches!(self.kind, ChipKind::Spartan3 | ChipKind::FpgaCore) {
            expander.fill_clk_sn_s3();
        } else {
            expander.fill_clk_sn_s3();
            expander.fill_clk_we_s3e();
        }
        expander.fill_hclk();
        expander.fill_hrow();
        expander.fill_clkc();
        expander.fill_clkqc();
        expander.fill_global();
        expander.fill_frame_info();

        let clkv_frame = expander.clkv_frame;
        let spine_frame = expander.spine_frame;
        let term_w_frame = expander.term_w_frame;
        let term_e_frame = expander.term_e_frame;
        let col_frame = expander.col_frame;
        let bram_frame = expander.bram_frame;
        let holes = expander.holes;

        let frame_info = expander.frame_info;
        let frame_len = 32 + self.rows.len() * if self.kind.is_virtex2() { 80 } else { 64 };

        let egrid = egrid.finish();
        ExpandedDevice {
            chip: self,
            egrid,
            frame_info,
            frame_len,
            clkv_frame,
            spine_frame,
            term_w_frame,
            term_e_frame,
            col_frame,
            bram_frame,
            holes,
        }
    }
}
