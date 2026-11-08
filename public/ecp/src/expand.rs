use prjcombine_entity::{EntityIdArith, EntityPartVec, EntityVec};
use prjcombine_interconnect::{
    coord::{CellCoord, ColId, ColIdExt, DieId, DieIdExt, Rect, RowId, RowIdExt, WireCoord},
    db::IntDb,
    dir::DirV,
    grid::builder::GridBuilder,
};

use crate::{
    chip::{Chip, ChipKind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, ccls, rslots, tcls, wires},
    expanded::ExpandedDevice,
};

mod crosslink;
mod ecp;
mod ecp2;
mod ecp3;
mod ecp4;
mod ecp5;
mod machxo;
mod machxo2;
mod nexus;
mod scm;

struct Expander<'a, 'b> {
    chip: &'b Chip,
    die: DieId,
    egrid: &'a mut GridBuilder<'b>,
    bel_holes: Vec<Rect>,
    frame_len: usize,
    frames_num: usize,
    clk_frame: usize,
    col_frame: EntityVec<ColId, usize>,
    col_term_frame: EntityPartVec<ColId, usize>,
    row_bit: EntityVec<RowId, usize>,
    row_ebr_bit: EntityPartVec<RowId, usize>,
}

impl Expander<'_, '_> {
    fn is_in_bel_hole(&self, cell: CellCoord) -> bool {
        for hole in &self.bel_holes {
            if hole.contains(cell) {
                return true;
            }
        }
        false
    }

    fn fill_plc(&mut self) {
        for (row, rd) in &self.chip.rows {
            let tcls_plc = match rd.kind {
                RowKind::Plc => tcls::PLC,
                RowKind::Fplc => tcls::FPLC,
                _ => continue,
            };
            for cell in self.egrid.row(self.die, row) {
                if (self.chip.is_col_io(cell.col)) && self.chip.kind != ChipKind::Crosslink {
                    continue;
                }
                if self.is_in_bel_hole(cell) {
                    continue;
                }
                self.egrid.add_tile_single(cell, tcls::INT_PLC);
                self.egrid.add_tile_single(cell, tcls_plc);
            }
        }
    }

    fn add_io_conn(&mut self, wire: WireCoord, key: SpecialIoKey) {
        let io = self.chip.special_io[&key];
        let ioidx = bslots::IO.index_of(io.slot).unwrap();
        self.egrid
            .extra_conns
            .insert(wire, io.wire(wires::OUT_IO_INCK[ioidx]));
    }

    fn fill_int_io(&mut self, cell: CellCoord) {
        let tcid = match self.chip.kind {
            ChipKind::Ecp | ChipKind::Xp => {
                if cell.row == self.chip.row_s() || cell.row == self.chip.row_n() {
                    tcls::INT_IO_SN
                } else {
                    tcls::INT_IO_WE
                }
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                if cell.row == self.chip.row_n() {
                    tcls::INT_IO_N
                } else if cell.row == self.chip.row_s() {
                    tcls::INT_IO_S
                } else {
                    tcls::INT_IO_WE
                }
            }
            _ => unreachable!(),
        };
        self.egrid.add_tile_single(cell, tcid);
    }

    fn fill_region_pclk(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            let cell_pclk = if matches!(self.chip.kind, ChipKind::MachXo | ChipKind::MachXo2(..)) {
                self.chip.clk_root()
            } else if self.chip.kind == ChipKind::Crosslink
                || matches!(
                    self.chip.kind,
                    ChipKind::Nexus(kind)
                    if !kind.is_large()
                )
            {
                let col_pclk = cell.col.snap_to_side_of(self.chip.col_clk);
                self.chip.clk_root().with_col(col_pclk)
            } else {
                let col_pclk = self.chip.col_edge(cell.col.side_of(self.chip.col_clk));
                let row_pclk = self.chip.row_edge(cell.row.side_of(self.chip.row_clk));
                self.die.cell(col_pclk, row_pclk)
            };
            self.egrid[cell].region_root[rslots::PCLK] = cell_pclk;
            if matches!(
                self.chip.kind,
                ChipKind::Scm | ChipKind::Ecp5 | ChipKind::Crosslink | ChipKind::Nexus(_)
            ) {
                let row = match self.chip.kind {
                    ChipKind::Scm => {
                        if cell.row < self.chip.row_clk {
                            self.chip.row_clk - 5
                        } else {
                            self.chip.row_clk + 4
                        }
                    }
                    ChipKind::Ecp5 => {
                        self.chip.special_loc
                            [&SpecialLocKey::ClkQuarter(cell.row.side_of(self.chip.row_clk))]
                            .row
                    }
                    ChipKind::Crosslink => self.chip.row_clk,
                    ChipKind::Nexus(kind) => {
                        if !kind.is_large() {
                            self.chip.row_clk
                        } else if cell.row < self.chip.row_clk {
                            self.chip.row_clk - 9
                        } else {
                            self.chip.row_clk + 9
                        }
                    }
                    _ => unreachable!(),
                };
                self.egrid[cell].region_root[rslots::PCLK_COL] =
                    self.die.cell(self.chip.cols_pclk[cell.col], row);
            }
            if matches!(
                self.chip.kind,
                ChipKind::Scm | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2
            ) {
                self.egrid[cell].region_root[rslots::PCLK_LEAF] =
                    cell.with_col(self.chip.cols_pclk[cell.col]);
            } else if matches!(self.chip.kind, ChipKind::Ecp5 | ChipKind::Crosslink) {
                let col_pclk = self.chip.cols_pclk[cell.col];
                self.egrid[cell].region_root[rslots::PCLK_LEAF] =
                    cell.with_col(cell.col.snap_to_side_of(col_pclk));
            } else if matches!(self.chip.kind, ChipKind::Nexus(_)) {
                let col_pclk = self.chip.cols_pclk[cell.col];
                let col_src = cell.col.snap_to_side_of(if col_pclk < self.chip.col_clk {
                    col_pclk + 1
                } else {
                    col_pclk
                });
                self.egrid[cell].region_root[rslots::PCLK_LEAF] = cell.with_col(col_src);
            }
        }
    }

    fn fill_rsclk_h_regions(&mut self) {
        let mut regions = vec![];
        {
            let mut prev = self.chip.columns.ids().start();
            for (col, cd) in &self.chip.columns {
                if cd.sclk_break {
                    regions.push(prev.range_until(col));
                    prev = col;
                }
            }
            regions.push(prev.range_until(self.chip.columns.ids().end()));
        }
        for (row, rd) in &self.chip.rows {
            let is_sclk_row = if self.chip.kind.has_distributed_sclk_ecp3() {
                rd.kind == RowKind::Io || rd.sclk_break
            } else {
                matches!(rd.kind, RowKind::Io | RowKind::Ebr | RowKind::Dsp)
            };
            if !is_sclk_row {
                continue;
            }
            for &region in &regions {
                let mut cells = vec![];
                for col in region {
                    let mut cell = self.die.cell(col, row);
                    if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                        match self.chip.kind {
                            ChipKind::Ecp2M => {
                                if cell.row == self.chip.row_s() {
                                    cell.row = self.chip.row_serdes(DirV::S);
                                } else if cell.row == self.chip.row_n() {
                                    cell.row = self.chip.row_serdes(DirV::N);
                                } else {
                                    unreachable!()
                                }
                            }
                            ChipKind::Ecp3(_) => {
                                assert_eq!(cell.row, self.chip.row_s());
                                cell.row = self.chip.row_serdes(DirV::S);
                            }
                            _ => unreachable!(),
                        }
                    }
                    cells.push(cell);
                }
                let cell_root = if region.start() < self.chip.col_clk {
                    *cells.last().unwrap()
                } else {
                    *cells.first().unwrap()
                };
                for &cell in &cells {
                    self.egrid[cell].region_root[rslots::SCLK_H] = cell_root;
                }
                if cells[0].col != self.chip.col_w() {
                    let tcid = match self.chip.kind {
                        ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                            if cells[0].col == self.chip.col_clk {
                                tcls::SCLK_H_ROOT
                            } else {
                                tcls::SCLK_H_SPLITTER
                            }
                        }
                        ChipKind::Ecp3(_) => {
                            if cells[0].col == self.chip.col_clk {
                                tcls::SCLK_H_SPLITTER
                            } else {
                                tcls::SCLK_H_ROOT
                            }
                        }
                        ChipKind::MachXo2(_) => tcls::SCLK_H_ROOT,
                        _ => unreachable!(),
                    };
                    self.egrid.add_tile_we(cells[0], tcid, 1, 2);
                }
            }
        }
    }

    fn fill_conns(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            if !self.egrid.has_bel(cell.bel(bslots::INT)) {
                continue;
            }
            if self.egrid.bel_delta(cell, -1, 0, bslots::INT).is_some() {
                self.egrid
                    .fill_conn_pair(cell.delta(-1, 0), cell, ccls::PASS_E, ccls::PASS_W);
            } else {
                self.egrid.fill_conn_term(cell, ccls::TERM_W);
            }
            if self.egrid.bel_delta(cell, 1, 0, bslots::INT).is_none() {
                self.egrid.fill_conn_term(cell, ccls::TERM_E);
            }
            if self.egrid.bel_delta(cell, 0, -1, bslots::INT).is_some() {
                self.egrid
                    .fill_conn_pair(cell.delta(0, -1), cell, ccls::PASS_N, ccls::PASS_S);
            } else {
                self.egrid.fill_conn_term(cell, ccls::TERM_S);
            }
            if self.egrid.bel_delta(cell, 0, 1, bslots::INT).is_none() {
                self.egrid.fill_conn_term(cell, ccls::TERM_N);
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
            egrid: &mut egrid,
            die,
            bel_holes: vec![],
            frame_len: 0,
            frames_num: 0,
            clk_frame: 0,
            col_frame: Default::default(),
            col_term_frame: Default::default(),
            row_bit: Default::default(),
            row_ebr_bit: Default::default(),
        };

        expander.fill_region_pclk();
        match self.kind {
            ChipKind::Scm => {
                expander.fill_config_scm();
                expander.fill_pll_scm();
                expander.fill_serdes_scm();
                expander.fill_plc();
                expander.fill_ebr_scm();
                expander.fill_io_scm();
                expander.fill_clk_scm();
            }
            ChipKind::Ecp => {
                expander.fill_config_ecp();
                expander.fill_pll_ecp();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp();
                expander.fill_bs_ecp();
            }
            ChipKind::Xp => {
                expander.fill_pll_xp();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_config_xp();
                expander.fill_io_ecp();
                expander.fill_clk_ecp();
                expander.fill_bs_ecp();
            }
            ChipKind::MachXo => {
                expander.fill_plc();
                expander.fill_io_machxo();
                expander.fill_special_machxo();
                expander.fill_clk_machxo();
                expander.fill_bs_ecp();
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M => {
                expander.fill_config_ecp2();
                expander.fill_pll_ecp2();
                if self.kind == ChipKind::Ecp2M {
                    expander.fill_serdes_ecp2();
                }
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_rsclk_h_regions();
                expander.fill_clk_ecp2();
            }
            ChipKind::Xp2 => {
                expander.fill_config_xp2();
                expander.fill_pll_xp2();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp();
                expander.fill_rsclk_h_regions();
                expander.fill_clk_ecp2();
            }
            ChipKind::Ecp3(_) => {
                expander.fill_config_ecp3();
                expander.fill_pll_ecp3();
                expander.fill_serdes_ecp3();
                expander.fill_plc();
                expander.fill_ebr_ecp();
                expander.fill_dsp_ecp();
                expander.fill_io_ecp3();
                expander.fill_rsclk_h_regions();
                expander.fill_sclk_ecp3();
                expander.fill_pclk_ecp3();
                expander.fill_clk_ecp3();
            }
            ChipKind::MachXo2(_) => {
                expander.fill_config_machxo2();
                expander.fill_pll_machxo2();
                expander.fill_plc();
                expander.fill_ebr_machxo2();
                expander.fill_io_machxo2();
                expander.fill_rsclk_h_regions();
                expander.fill_sclk_ecp3();
                expander.fill_pclk_ecp3();
                expander.fill_clk_machxo2();
            }
            ChipKind::Ecp4 => {
                expander.fill_config_ecp4();
                expander.fill_pll_ecp4();
                expander.fill_plc();
                expander.fill_ebr_ecp4();
                expander.fill_io_ecp4();
                expander.fill_serdes_ecp4();
                expander.fill_clk_ecp4();
            }
            ChipKind::Ecp5 => {
                expander.fill_config_ecp5();
                expander.fill_pll_ecp5();
                expander.fill_plc();
                expander.fill_ebr_ecp4();
                expander.fill_io_ecp5();
                expander.fill_pclk_ecp5();
                expander.fill_clk_ecp5();
            }
            ChipKind::Crosslink => {
                expander.fill_config_crosslink();
                expander.fill_plc();
                expander.fill_ebr_crosslink();
                expander.fill_io_crosslink();
                expander.fill_pclk_ecp5();
                expander.fill_clk_crosslink();
            }
            ChipKind::Nexus(_) => {
                expander.fill_pll_nexus();
                expander.fill_plc();
                expander.fill_ebr_nexus();
                expander.fill_io_nexus();
                expander.fill_misc_nexus();
                expander.fill_pclk_nexus();
                expander.fill_clk_nexus();
            }
        }
        expander.fill_conns();

        let bel_holes = expander.bel_holes;
        let frame_len = expander.frame_len;
        let frames_num = expander.frames_num;
        let clk_frame = expander.clk_frame;
        let col_frame = expander.col_frame;
        let col_term_frame = expander.col_term_frame;
        let row_bit = expander.row_bit;
        let row_ebr_bit = expander.row_ebr_bit;

        let egrid = egrid.finish();
        ExpandedDevice {
            chip: self,
            egrid,
            bel_holes,
            frame_len,
            frames_num,
            clk_frame,
            col_frame,
            col_term_frame,
            row_bit,
            row_ebr_bit,
        }
    }
}
