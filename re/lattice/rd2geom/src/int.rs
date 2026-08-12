use prjcombine_ecp::{
    chip::{ChipKind, RowKind},
    defs::{self, bcls, bslots, cslots, tcls, tslots},
};
use prjcombine_interconnect::db::Bel;

use super::ChipContext;

mod io;
mod pips;
mod wires;

impl ChipContext<'_> {
    fn process_pclk_cols(&mut self) {
        let mut ranges = vec![];
        let mut prev = self.chip.col_w();
        for (col, cd) in &self.chip.columns {
            if cd.pclk_break {
                ranges.push((prev, col));
                prev = col;
            }
        }
        ranges.push((prev, self.chip.col_e() + 1));
        for (col_w, col_e) in ranges {
            for _ in col_w.range(col_e) {
                self.pclk_cols.push((col_w, col_e));
            }
        }
    }

    fn process_cibtest(&mut self) {
        for (cell, cell_data) in self.edev.cells() {
            let Some(tile) = cell_data.tiles.get(tslots::INT) else {
                continue;
            };
            let has_cibtest = if self.chip.kind == ChipKind::MachXo {
                tile.class == tcls::INT_SIO_L_W
            } else {
                tile.class != tcls::INT_PLC
            };
            if has_cibtest {
                let (r, c) = self.rc(cell);
                self.name_bel(cell.bel(bslots::INT), [format!("CIBTEST_R{r}C{c}")]);
                if self.chip.kind == ChipKind::MachXo {
                    for (w, n) in [
                        (defs::wires::IMUX_A[0], "A0"),
                        (defs::wires::IMUX_A[1], "A1"),
                        (defs::wires::IMUX_A[2], "A2"),
                        (defs::wires::IMUX_A[3], "A3"),
                        (defs::wires::IMUX_A[6], "A6"),
                        (defs::wires::IMUX_B[0], "B0"),
                        (defs::wires::IMUX_B[1], "B1"),
                        (defs::wires::IMUX_B[2], "B2"),
                        (defs::wires::IMUX_B[3], "B3"),
                        (defs::wires::IMUX_B[6], "B6"),
                        (defs::wires::IMUX_C[0], "C0"),
                        (defs::wires::IMUX_C[1], "C1"),
                        (defs::wires::IMUX_C[2], "C2"),
                        (defs::wires::IMUX_C[3], "C3"),
                        (defs::wires::IMUX_C[6], "C6"),
                        (defs::wires::IMUX_D[0], "D0"),
                        (defs::wires::IMUX_D[1], "D1"),
                        (defs::wires::IMUX_D[2], "D2"),
                        (defs::wires::IMUX_D[3], "D3"),
                        (defs::wires::IMUX_D[6], "D6"),
                        (defs::wires::IMUX_M[0], "M0"),
                        (defs::wires::IMUX_M[1], "M1"),
                        (defs::wires::IMUX_CLK[0], "CLK0"),
                        (defs::wires::IMUX_CLK[1], "CLK1"),
                        (defs::wires::IMUX_LSR[0], "LSR0"),
                        (defs::wires::IMUX_CE[0], "CE0"),
                    ] {
                        let wt = self.rc_wire(cell, &format!("J{n}_CIBTEST"));
                        let wf = cell.wire(w);
                        self.add_bel_wire(cell.bel(bslots::INT), n, wt);
                        self.claim_pip_int_in(wt, wf);
                    }
                    for i in 0..12 {
                        let wf = self.rc_wire(cell, &format!("JTI{i}_CIBTEST"));
                        let wt = cell.wire(defs::wires::OUT_TI[i]);
                        self.add_bel_wire(cell.bel(bslots::INT), format!("TI{i}"), wf);
                        self.claim_pip_int_out(wt, wf);
                    }
                    let bcrd = self.chip.bel_cibtest_sel();
                    let mut bel = Bel::default();
                    for i in 0..2 {
                        let pin = bcls::CIBTEST_SEL::TSEL[i];
                        let wire = self.rc_wire(cell, &format!("JTSEL{i}_CIBTEST"));
                        self.add_bel_wire(cell.bel(bslots::INT), format!("TSEL{i}"), wire);
                        bel.inputs.insert(pin, self.xlat_input(bcrd, wire));
                    }
                    self.insert_bel(bcrd, bel);
                } else {
                    let num_clk = if matches!(
                        self.chip.kind,
                        ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink
                    ) {
                        2
                    } else {
                        4
                    };
                    for (w, l, n) in [
                        (defs::wires::IMUX_A.as_slice(), "A", 8),
                        (defs::wires::IMUX_B.as_slice(), "B", 8),
                        (defs::wires::IMUX_C.as_slice(), "C", 8),
                        (defs::wires::IMUX_D.as_slice(), "D", 8),
                        (defs::wires::IMUX_M.as_slice(), "M", 8),
                        (defs::wires::IMUX_CLK.as_slice(), "CLK", num_clk),
                        (defs::wires::IMUX_LSR.as_slice(), "LSR", num_clk),
                        (defs::wires::IMUX_CE.as_slice(), "CE", 4),
                    ] {
                        for i in 0..n {
                            let wt = self.rc_wire(cell, &format!("J{l}{i}_CIBTEST"));
                            let wf = if self.chip.kind == ChipKind::Xp2
                                && l == "CLK"
                                && i == 2
                                && matches!(
                                    tile.class,
                                    tcls::INT_IO_WE | tcls::INT_IO_S | tcls::INT_IO_N
                                )
                                && !((cell.col == self.chip.col_w()
                                    || cell.col == self.chip.col_e())
                                    && (cell.row == self.chip.row_s()
                                        || cell.row == self.chip.row_n()))
                            {
                                cell.wire(defs::wires::IMUX_CLK[0])
                            } else if self.chip.kind == ChipKind::Ecp4 && l == "CLK" {
                                cell.wire(defs::wires::IMUX_CLK_DELAY[i])
                            } else {
                                cell.wire(w[i])
                            };
                            self.add_bel_wire(cell.bel(bslots::INT), format!("{l}{i}"), wt);
                            self.claim_pip_int_in(wt, wf);
                        }
                    }
                    for (w, l, n) in [
                        (defs::wires::OUT_F, "F", 8),
                        (defs::wires::OUT_Q, "Q", 8),
                        (defs::wires::OUT_OFX, "OFX", 8),
                    ] {
                        if l == "OFX"
                            && matches!(self.chip.kind, ChipKind::Ecp5 | ChipKind::Crosslink)
                        {
                            continue;
                        }
                        for i in 0..n {
                            let wf = self.rc_wire(cell, &format!("J{l}{i}_CIBTEST"));
                            let wt = cell.wire(w[i]);
                            self.add_bel_wire(cell.bel(bslots::INT), format!("{l}{i}"), wf);
                            if matches!(
                                self.chip.kind,
                                ChipKind::Scm
                                    | ChipKind::Xp2
                                    | ChipKind::Ecp3
                                    | ChipKind::Ecp3A
                                    | ChipKind::MachXo2(_)
                                    | ChipKind::Ecp4
                                    | ChipKind::Ecp5
                                    | ChipKind::Crosslink
                            ) {
                                self.claim_pip_int_out(wt, wf);
                            } else {
                                self.claim_pip_int_in(wf, wt);
                            }
                        }
                    }
                }
            } else {
                self.name_bel_null(cell.bel(bslots::INT));
            }
        }
    }

    fn process_int_misc(&mut self) {
        for (cell, cell_data) in self.edev.cells() {
            if !self.edev.has_bel(cell.bel(bslots::INT)) {
                continue;
            }
            let has_cin = if matches!(self.chip.kind, ChipKind::MachXo | ChipKind::MachXo2(_)) {
                matches!(self.chip.rows[cell.row].kind, RowKind::Plc | RowKind::Fplc)
                    && cell.col != self.chip.col_e()
                    && cell.col > self.chip.col_w() + 1
            } else {
                cell_data.conns[cslots::W].target.is_some()
            };
            if has_cin {
                let wire = self.rc_wire(cell, "HFIE0000");
                self.add_bel_wire(cell.bel(bslots::INT), "FCI_IN", wire);
                if !self.chip.kind.has_out_ofx_branch() {
                    let wire = self.rc_wire(cell, "HL7W0001");
                    self.add_bel_wire(cell.bel(bslots::INT), "SLICE2_FX_OUT", wire);
                }
            }
        }
    }

    fn process_int_delay(&mut self) {
        if self.chip.kind != ChipKind::Ecp4 {
            return;
        }
        for (cell, cell_data) in self.edev.cells() {
            let Some(tile) = cell_data.tiles.get(tslots::INT) else {
                continue;
            };
            if tile.class == tcls::INT_PLC {
                continue;
            }
            let bcrd = cell.bel(bslots::INT);
            for i in 0..2 {
                let clk = self.naming.interconnect[&cell.wire(defs::wires::IMUX_CLK[i])];
                let clk_delay =
                    self.naming.interconnect[&cell.wire(defs::wires::IMUX_CLK_DELAY[i])];
                let del1 = self.rc_wire(cell, &format!("CLK{i}_DEL1"));
                let del2 = self.rc_wire(cell, &format!("CLK{i}_DEL2"));
                let del3 = self.rc_wire(cell, &format!("CLK{i}_DEL3"));
                self.add_bel_wire(bcrd, format!("CLK{i}_DEL1"), del1);
                self.add_bel_wire(bcrd, format!("CLK{i}_DEL2"), del2);
                self.add_bel_wire(bcrd, format!("CLK{i}_DEL3"), del3);
                self.claim_pip(del1, clk);
                self.claim_pip(del2, del1);
                self.claim_pip(del3, del2);
                self.claim_pip(clk_delay, clk);
                self.claim_pip(clk_delay, del1);
                self.claim_pip(clk_delay, del2);
                self.claim_pip(clk_delay, del3);
            }
        }
    }

    fn process_sclk_source(&mut self) {
        if !self.chip.kind.has_distributed_sclk() {
            return;
        }
        for (cell, cell_data) in self.edev.cells() {
            let idx = self.chip.col_sclk_idx(cell.col);
            let has_int = cell_data.tiles.contains_id(tslots::INT);
            let mut clocks = vec![(idx, 0, 0), (idx + 4, 4, 1)];
            if self.chip.kind.has_distributed_sclk_ecp3() {
                if cell.col == self.chip.col_w() {
                    let idx3 = (idx + 3) % 4;
                    let idx2 = (idx + 2) % 4;
                    clocks.extend([
                        (idx3, 1, 6),
                        (idx3 + 4, 5, 7),
                        (idx2, 2, 4),
                        (idx2 + 4, 6, 5),
                    ]);
                }
                if cell.col == self.chip.col_e() {
                    let idx1 = (idx + 1) % 4;
                    clocks.extend([(idx1, 1, 2), (idx1 + 4, 5, 3)]);
                }
            }
            if has_int {
                for (sclki, ti, vsdclki) in clocks {
                    let sclk_in = self.rc_wire(cell, &format!("HSBX0{ti}00"));
                    self.add_bel_wire(cell.bel(bslots::INT), format!("SCLK{sclki}_IN"), sclk_in);
                    let sclk = cell.wire(defs::wires::RSCLK[sclki]);
                    self.claim_pip_int_out(sclk, sclk_in);
                    let vsdclk = cell.wire(defs::wires::RSCLK_V[vsdclki]);
                    self.claim_pip_int_in(sclk_in, vsdclk);
                }
            } else if self.chip.kind.has_distributed_sclk_ecp3() {
                for (sclki, _, vsdclki) in clocks {
                    let sclk = cell.wire(defs::wires::RSCLK[sclki]);
                    if let Some(&sclk) = self.naming.interconnect.get(&sclk) {
                        let vsdclk = cell.wire(defs::wires::RSCLK_V[vsdclki]);
                        self.claim_pip_int_in(sclk, vsdclk);
                    }
                }
            }
            if matches!(self.chip.kind, ChipKind::Ecp3 | ChipKind::Ecp3A) {
                // ??!? special garbage wires
                let clocks =
                    if cell.row == self.chip.row_s() + 9 && cell.col == self.chip.col_clk - 1 {
                        vec![(0, 1), (4, 5)]
                    } else if cell.row == self.chip.row_s() + 9 && cell.col == self.chip.col_clk {
                        vec![(2, 2), (6, 6), (3, 1), (7, 5)]
                    } else {
                        vec![]
                    };
                for (sclki, ti) in clocks {
                    let sclk_in = self.rc_wire(cell, &format!("HSBX0{ti}00"));
                    self.add_bel_wire(cell.bel(bslots::INT), format!("SCLK{sclki}_IN"), sclk_in);
                    let sclk = cell.wire(defs::wires::RSCLK[sclki]);
                    self.claim_pip_int_out(sclk, sclk_in);
                }
            }
            if matches!(self.chip.kind, ChipKind::MachXo2(_))
                && matches!(self.chip.rows[cell.row].kind, RowKind::Io | RowKind::Ebr)
            {
                let keep = self.rc_wire(cell, "SOUTHKEEP");
                for i in 0..8 {
                    if matches!(i, 2..4) && cell.col != self.chip.col_e() {
                        continue;
                    }
                    if matches!(i, 4..8) && cell.col != self.chip.col_w() {
                        continue;
                    }
                    let vsdclk = cell.wire(defs::wires::RSCLK_V[i]);
                    let vsdclk_n = cell.wire(defs::wires::RSCLK_V_N[i]);
                    let idx = [2, 3, 6, 7, 10, 11, 6, 7][i];
                    let b2t = self.rc_wire(cell, &format!("VSTX{idx:02}00_B2T"));
                    let t2b = self.rc_wire(cell, &format!("VSTX{idx:02}00_T2B"));
                    self.add_bel_wire(cell.bel(bslots::INT), format!("VSDCLK{i}_B2T"), b2t);
                    self.add_bel_wire(cell.bel(bslots::INT), format!("VSDCLK{i}_T2B"), t2b);
                    self.claim_pip(t2b, keep);
                    self.claim_pip(b2t, keep);
                    if cell.row == self.chip.row_s() {
                        self.claim_pip_int_in(t2b, vsdclk);
                        self.claim_pip_int_out(vsdclk, b2t);
                    } else if cell.row == self.chip.row_n() {
                        self.claim_pip_int_in(b2t, vsdclk);
                        self.claim_pip_int_out(vsdclk, t2b);
                    } else {
                        self.claim_pip_int_in(t2b, vsdclk);
                        self.claim_pip_int_out(vsdclk, b2t);
                        self.claim_pip_int_in(b2t, vsdclk_n);
                        self.claim_pip_int_out(vsdclk_n, t2b);
                    }
                }
            }
        }
    }

    pub fn process_int(&mut self) {
        self.process_pclk_cols();
        self.process_int_wires();
        self.process_int_pips();
        self.process_cibtest();
        self.process_int_misc();
        self.process_int_delay();
        self.process_sclk_source();
        if self.chip.kind == ChipKind::Scm {
            self.process_int_io_wires();
            self.process_int_io_conn_hv();
            self.process_int_io_pips();
        }
    }
}
