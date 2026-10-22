use prjcombine_interconnect::{
    dir::{Dir, DirHV, DirV},
    grid::DieIdExt,
};

use crate::{
    chip::{IoGroupKind, PllPad, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bslots, ccls, tcls, wires},
    expand::Expander,
};

impl Expander<'_, '_> {
    pub(super) fn fill_io_machxo(&mut self) {
        for col in self.egrid.cols(self.die) {
            if col != self.chip.col_w() && col != self.chip.col_e() {
                let cell_s = self.die.cell(col, self.chip.row_s());
                self.egrid.fill_conn_pair(
                    cell_s,
                    cell_s.delta(0, 1),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
                let cell_n = self.die.cell(col, self.chip.row_n());
                self.egrid.fill_conn_pair(
                    cell_n,
                    cell_n.delta(0, -1),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
            }
        }
        for row in self.egrid.rows(self.die) {
            if row > self.chip.row_s() + 1 && row < self.chip.row_n() - 1 {
                let cell_w = self.die.cell(self.chip.col_w(), row);
                self.egrid.fill_conn_pair(
                    cell_w,
                    cell_w.delta(1, 0),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
                let cell_e = self.die.cell(self.chip.col_e(), row);
                self.egrid.fill_conn_pair(
                    cell_e,
                    cell_e.delta(-1, 0),
                    ccls::IO_FROM_PLC,
                    ccls::PLC_TO_IO,
                );
            }
        }
        for (&io, &(src_cell, plc_lut)) in &self.chip.io_direct_plc {
            let io_idx = bslots::IO.index_of(io.slot).unwrap();
            let dst_wire = io.wire(wires::IMUX_IO_O_PLC[io_idx]);
            let src_wire = src_cell.wire(wires::PLC_IO[plc_lut]);
            self.egrid.extra_conns.insert(dst_wire, src_wire);
        }
        let has_ebr = self.chip.special_loc.contains_key(&SpecialLocKey::Ebr(0));
        for cell in self.egrid.column(self.die, self.chip.col_w()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            let int_kind = if has_ebr {
                tcls::INT_SIO_L_W
            } else if cell.row >= self.chip.row_clk - 3 && cell.row < self.chip.row_clk + 3 {
                tcls::INT_SIO_S_W_CLK
            } else {
                tcls::INT_SIO_S_W
            };
            self.egrid.add_tile_single(cell, int_kind);
            let kind = match (has_ebr, rd.io_w) {
                (true, IoGroupKind::Double) => tcls::SIO_L_W2,
                (true, IoGroupKind::Quad | IoGroupKind::QuadReverse) => tcls::SIO_L_W4,
                (false, IoGroupKind::Double) => tcls::SIO_S_W2,
                (false, IoGroupKind::Quad) => tcls::SIO_S_W4,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.column(self.die, self.chip.col_e()) {
            let rd = &self.chip.rows[cell.row];
            if !matches!(rd.kind, RowKind::Plc | RowKind::Fplc) {
                continue;
            }
            let int_kind = if cell == self.chip.special_loc[&SpecialLocKey::Config] {
                if has_ebr {
                    tcls::INT_SIO_L_E_CFG
                } else {
                    tcls::INT_SIO_S_E_CFG
                }
            } else {
                if has_ebr {
                    tcls::INT_SIO_L_E
                } else {
                    tcls::INT_SIO_S_E
                }
            };
            self.egrid.add_tile_single(cell, int_kind);
            let kind = match (has_ebr, rd.io_e) {
                (false, IoGroupKind::Double) => tcls::SIO_S_E2,
                (false, IoGroupKind::Quad) => tcls::SIO_S_E4,
                (true, IoGroupKind::Double) => tcls::SIO_L_E2,
                (true, IoGroupKind::Quad) => tcls::SIO_L_E4,
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_s()) {
            let cd = &self.chip.columns[cell.col];
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let (int_kind, kind) = match (has_ebr, cd.io_s) {
                (false, IoGroupKind::Quad) => (tcls::INT_SIO_S_S4, tcls::SIO_S_S4),
                (false, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_S_S6, tcls::SIO_S_S6)
                }
                (true, IoGroupKind::Quad) => (tcls::INT_SIO_L_S4, tcls::SIO_L_S4),
                (true, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_L_S6, tcls::SIO_L_S6)
                }
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, kind);
        }
        for cell in self.egrid.row(self.die, self.chip.row_n()) {
            let cd = &self.chip.columns[cell.col];
            if cell.col == self.chip.col_w() || cell.col == self.chip.col_e() {
                continue;
            }
            let (int_kind, kind) = match (has_ebr, cd.io_n) {
                (false, IoGroupKind::Quad) => (tcls::INT_SIO_S_N4, tcls::SIO_S_N4),
                (false, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_S_N6, tcls::SIO_S_N6)
                }
                (true, IoGroupKind::Quad) => (tcls::INT_SIO_L_N4, tcls::SIO_L_N4),
                (true, IoGroupKind::Hex | IoGroupKind::HexReverse) => {
                    (tcls::INT_SIO_L_N6, tcls::SIO_L_N6)
                }
                _ => unreachable!(),
            };
            self.egrid.add_tile_single(cell, int_kind);
            self.egrid.add_tile_single(cell, kind);
        }
    }

    pub(super) fn fill_special_machxo(&mut self) {
        let has_ebr = self.chip.special_loc.contains_key(&SpecialLocKey::Ebr(0));
        for (&key, &cell) in &self.chip.special_loc {
            match key {
                SpecialLocKey::Pll(which) => {
                    let kind = match which.quad.v {
                        DirV::S => tcls::PLL_S,
                        DirV::N => tcls::PLL_N,
                    };
                    self.egrid.add_tile_single(cell, kind);
                    for (wire, pad) in [
                        (wires::IO_PLL_CLKI, PllPad::PllIn0),
                        (wires::IO_PLL_CLKFB, PllPad::PllFb),
                    ] {
                        self.add_io_conn(cell.wire(wire), SpecialIoKey::Pll(pad, which));
                    }
                }
                SpecialLocKey::Ebr(_) => {
                    self.egrid.add_tile_n(cell, tcls::EBR, 4);
                }
                SpecialLocKey::Config => {
                    self.egrid.add_tile_n(cell, tcls::CONFIG, 5);
                }
                SpecialLocKey::Osc => {
                    let kind = if has_ebr { tcls::OSC_X } else { tcls::OSC };
                    self.egrid.add_tile_single(cell, kind);
                }
                _ => unreachable!(),
            }
        }
        if has_ebr {
            let bel = self.chip.bel_cibtest_sel();
            self.egrid.add_tile_single(bel.cell, tcls::CIBTEST_SEL);
        }
    }

    pub(super) fn fill_clk_machxo(&mut self) {
        for &key in self.chip.special_io.keys() {
            let SpecialIoKey::Clock(edge, cidx) = key else {
                continue;
            };
            let wire = match edge {
                Dir::N => wires::IO_GCLK_N[usize::from(cidx)],
                Dir::S => wires::IO_GCLK_S[usize::from(cidx)],
                _ => unreachable!(),
            };
            self.add_io_conn(
                self.chip.clk_root().with_col(self.chip.col_w()).wire(wire),
                key,
            );
        }
        let mut tcells = self
            .die
            .cell(self.chip.col_w(), self.chip.row_clk - 3)
            .cells_n(6);
        let kind = if let Some(pll_sw) = self.chip.try_bel_pll(DirHV::SW, 0) {
            if let Some(pll_nw) = self.chip.try_bel_pll(DirHV::NW, 0) {
                tcells.extend([pll_sw.cell, pll_nw.cell]);
                tcls::CLK_ROOT_2PLL
            } else {
                tcells.extend([pll_sw.cell]);
                tcls::CLK_ROOT_1PLL
            }
        } else {
            tcls::CLK_ROOT_0PLL
        };
        self.egrid.add_tile(
            self.die.cell(self.chip.col_clk, self.chip.row_clk),
            kind,
            &tcells,
        );
    }
}
