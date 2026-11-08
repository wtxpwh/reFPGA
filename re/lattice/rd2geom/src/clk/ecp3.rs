use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, Ecp3Kind, RowKind},
    defs::{self, bcls, bslots, ecp3::tcls::CLK_ROOT, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{ColId, ColIdExt, DieIdExt, RowIdExt, RowRangeExt},
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{DirH, DirHV, DirV},
};
use prjcombine_re_lattice_naming::WireName;

use crate::{ChipContext, chip::ChipExt};

impl ChipContext<'_> {
    pub(super) fn process_hsdclk_root_ecp3(&mut self) {
        let mut vsrx = BTreeMap::new();
        let root_cols = Vec::from_iter(
            self.chip
                .columns
                .iter()
                .filter(|&(col, cd)| cd.sclk_break && col != self.chip.col_clk)
                .map(|(col, _)| col),
        );
        let root_rows = Vec::from_iter(
            self.chip
                .rows
                .iter()
                .filter(|&(_, rd)| rd.sclk_break || rd.kind == RowKind::Io)
                .map(|(row, _)| row),
        );
        for &row in &root_rows {
            for &col in &root_cols {
                let mut cell = Chip::DIE.cell(col, row);
                if !self.edev.has_bel(cell.bel(bslots::INT)) {
                    cell.row += 9;
                }
                let bcrd = cell.bel(bslots::SCLK_H_SPLITTER);
                let mut ex = self.extract_switchbox(bcrd);
                for i in 0..8 {
                    ex.progbuf_output_int(wires::SCLK_H[i].cell(0))
                        .progbuf_output_int(wires::SCLK_H[i].cell(1));
                    let wname = ex.ctx.naming.interconnect[&cell.wire(wires::SCLK_H[i])];
                    let wname_out = ex.ctx.pips_bwd[&wname]
                        .iter()
                        .copied()
                        .find(|&wn| {
                            ex.ctx.naming.strings[wn.suffix].starts_with("VSRX")
                                && ex.ctx.chip.xlat_rc_wire(wn).col == col - 1
                        })
                        .unwrap();
                    let tw_root = wires::SCLK_ROOT[i].cell(1);
                    if row == ex.ctx.chip.row_clk {
                        ex.input_bind_no_claim(wname_out, tw_root);
                    } else {
                        ex.input_bind(wname_out, tw_root);
                    }
                    match vsrx.entry((col, row, i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(wname_out);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), wname_out);
                        }
                    }
                }
            }
        }
        for (idx, &row) in root_rows.iter().enumerate() {
            for &col in &root_cols {
                let h = col.side_of(self.chip.col_clk);
                for i in 0..8 {
                    let wire = vsrx[&(col, row, i)];
                    if row < self.chip.row_clk {
                        let row_n = root_rows[idx + 1];
                        if row_n == self.chip.row_clk {
                            let root = self.claim_single_in(wire);
                            assert_eq!(
                                self.int_wires[&root][0],
                                Chip::DIE
                                    .cell(self.chip.col_edge(h), self.chip.row_s())
                                    .wire(wires::SCLK_ROOT[i])
                            );
                        } else {
                            let wire_n = vsrx[&(col, row_n, i)];
                            self.claim_pip(wire, wire_n);
                        }
                    } else if row == self.chip.row_clk {
                        let root = self.claim_single_in(wire);
                        assert_eq!(
                            self.int_wires[&root][0],
                            Chip::DIE
                                .cell(self.chip.col_edge(h), self.chip.row_n())
                                .wire(wires::SCLK_ROOT[i])
                        );
                    } else {
                        let row_s = root_rows[idx - 1];
                        let wire_s = vsrx[&(col, row_s, i)];
                        if row_s == self.chip.row_clk {
                            assert_eq!(wire, wire_s);
                        } else {
                            self.claim_pip(wire, wire_s);
                        }
                    }
                }
            }
        }
    }

    pub(super) fn process_clk_ecp3(&mut self) {
        let chip = self.chip;
        let bcrd = chip.bel_clk_root();
        let tcrd = self.edev.bel_tile(bcrd);
        let cell = bcrd.cell.delta(-1, 0);

        let mut ex = self.extract_switchbox_names(
            bcrd,
            [
                "LLGND", "LRGND", "ULGND", "URGND", "LLVCC", "LRVCC", "ULVCC", "URVCC",
            ],
        );
        ex.cell(cell)
            .input_xlat_rc("JPCLKCIBLLQ0")
            .input_xlat_rc("JPCLKCIBLLQ1")
            .input_xlat_rc("JPCLKCIBLRQ0")
            .input_xlat_rc("JPCLKCIBLRQ1")
            .input_xlat_rc("JPCLKCIBLRQ2")
            .input_xlat_rc("JPCLKCIBULQ0")
            .input_xlat_rc("JPCLKCIBULQ1")
            .input_xlat_rc("JPCLKCIBURQ0")
            .input_xlat_rc("JPCLKCIBURQ1")
            .input_xlat_rc("JPCLKCIBURQ2")
            .input_xlat_rc("JPCLKCIBMID0")
            .input_xlat_rc("JPCLKCIBMID1")
            .input_xlat_rc("JPCLKCIBMID2")
            .input_xlat_rc("JPCLKCIBMID3")
            .input_xlat_rc("JPCLKCIBMID4")
            .input_xlat_rc("JPCLKCIBMID5")
            .input_xlat_rc("JPCLKCIBMID6")
            .input_xlat_rc("JPCLKCIBMID7")
            .input_xlat_rc("JSCLKCIBL0")
            .input_xlat_rc("JSCLKCIBL1")
            .input_xlat_rc("JSCLKCIBL2")
            .input_xlat_rc("JSCLKCIBL3")
            .input_xlat_rc("JSCLKCIBR0")
            .input_xlat_rc("JSCLKCIBR1")
            .input_xlat_rc("JSCLKCIBR2")
            .input_xlat_rc("JSCLKCIBR3")
            .input_xlat_rc("JSCLKCIBB0")
            .input_xlat_rc("JSCLKCIBB1")
            .input_xlat_rc("JSCLKCIBB2")
            .input_xlat_rc("JSCLKCIBB3")
            .input_xlat_rc("JSCLKCIBT0")
            .input_xlat_rc("JSCLKCIBT1")
            .input_xlat_rc("JSCLKCIBT2")
            .input_xlat_rc("JSCLKCIBT3")
            .input_xlat_rc("JLPIO0")
            .input_xlat_rc("JLPIO1")
            .input_xlat_rc("JRPIO0")
            .input_xlat_rc("JRPIO1")
            .input_xlat_rc("JTPIO0")
            .input_xlat_rc("JTPIO1")
            .input_xlat_rc("JLUDLL0")
            .input_xlat_rc("JLUDLL1")
            .input_xlat_rc("JRUDLL0")
            .input_xlat_rc("JRUDLL1")
            .input_xlat_rc("JLCDIV1")
            .input_xlat_rc("JLCDIV2")
            .input_xlat_rc("JLCDIV4")
            .input_xlat_rc("JLCDIV8")
            .input_xlat_rc("JRCDIV1")
            .input_xlat_rc("JRCDIV2")
            .input_xlat_rc("JRCDIV4")
            .input_xlat_rc("JRCDIV8");

        for quad in DirHV::DIRS {
            let num = match quad.v {
                DirV::S => 3,
                DirV::N => 2,
            };
            for idx in 0..num {
                if chip.try_bel_pll(quad, idx).is_some() {
                    let corner = match quad {
                        DirHV::SW => "LL",
                        DirHV::SE => "RL",
                        DirHV::NW => "LU",
                        DirHV::NE => "RU",
                    };
                    for i in 0..4 {
                        ex.input_xlat_rc(format!("J{corner}PLL{ii}", ii = idx * 4 + i));
                    }
                }
            }
        }
        for (idx, abcd) in ['A', 'B', 'C', 'D'].into_iter().enumerate() {
            let cslot = CLK_ROOT::SERDES[idx];
            if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                ex.input_xlat_rc_nudge(format!("JPCS{abcd}_TX_F_CLK"), cslot)
                    .input_xlat_rc_nudge(format!("JPCS{abcd}_TX_H_CLK"), cslot);
            }
        }

        for ci in 0..4 {
            for i in 0..8 {
                ex.output_int(wires::SCLK_ROOT[i].cell(ci));
            }
        }

        for (ci, bt, we) in [(0, 'B', 'W'), (1, 'B', 'E'), (2, 'T', 'W'), (3, 'T', 'E')] {
            for i in 0..6 {
                ex.output_bind_rc(
                    format!("{bt}PC{we}CLKI{i}"),
                    wires::IMUX_DCC_CLKI[i].cell(ci),
                );
            }
            for i in 0..2 {
                let ii = i + 6;
                ex.output_bind_rc(
                    format!("{bt}PC{we}0{ii}0A"),
                    wires::IMUX_DCS_CLKA[i].cell(ci),
                )
                .output_bind_rc(
                    format!("{bt}PC{we}0{ii}0B"),
                    wires::IMUX_DCS_CLKB[i].cell(ci),
                );
            }
        }

        if chip.kind == ChipKind::Ecp3(Ecp3Kind::Ecp3A) {
            ex.input_xlat_rc("JLDLLECLK1")
                .input_xlat_rc("JLDLLECLK2")
                .permabuf_output_xlat_rc("JBRGECLK1")
                .permabuf_output_xlat_rc("JBRGECLK2");
        } else {
            ex.permabuf(
                CLK_ROOT::CENTER_M[1].wire(wires::BRGECLK[0]),
                CLK_ROOT::PLL_NW[0].wire(wires::PLL_TO_ECLK[0]),
            )
            .permabuf(
                CLK_ROOT::CENTER_M[1].wire(wires::BRGECLK[1]),
                CLK_ROOT::PLL_NW[0].wire(wires::PLL_TO_ECLK[1]),
            );
        }

        core::mem::drop(ex);

        for (hv, ll, dcc, dcs) in [
            (DirHV::SW, "LL", bslots::DCC_SW, bslots::DCS_SW),
            (DirHV::SE, "LR", bslots::DCC_SE, bslots::DCS_SE),
            (DirHV::NW, "UL", bslots::DCC_NW, bslots::DCS_NW),
            (DirHV::NE, "UR", bslots::DCC_NE, bslots::DCS_NE),
        ] {
            for i in 0..6 {
                self.extract_bel_cell(bcrd.bel(dcc[i]), cell)
                    .name(format!("{ll}DCC{i}"))
                    .input_xlat_rc(bcls::DCC::CLKI, format!("{ll}CLKI{i}_DCC"))
                    .input_xlat_rc(bcls::DCC::CE, format!("J{ll}CE{i}_DCC"))
                    .output_xlat_rc(bcls::DCC::CLKO, format!("{ll}CLKO{i}_DCC"))
                    .claim_pip("CLKO", "CLKI");
            }
            for i in 0..2 {
                self.extract_bel_cell(bcrd.bel(dcs[i]), cell)
                    .name(format!("{ll}DCS{i}"))
                    .input_xlat_rc(bcls::DCS::CLK[0], format!("{ll}CLK{i}A_DCS"))
                    .input_xlat_rc(bcls::DCS::CLK[1], format!("{ll}CLK{i}B_DCS"))
                    .input_xlat_rc(bcls::DCS::SEL[0], format!("J{ll}SEL{i}_DCS"))
                    .output_xlat_rc(bcls::DCS::DCSOUT, format!("{ll}DCSOUT{i}_DCS"))
                    .claim_pip("DCSOUT", "CLK0")
                    .claim_pip("DCSOUT", "CLK1");
            }

            for (w, name) in [(wires::TIE0, "GND"), (wires::TIE1, "VCC")] {
                let col = chip.col_edge(hv.h);
                let row = chip.row_edge(hv.v);
                let wire = self.naming.interconnect[&Chip::DIE.cell(col, row).wire(w)];
                let wire = self.find_single_in(wire);
                self.add_bel_wire(bcrd, format!("{name}_{hv}"), wire);
                let rows = chip
                    .rows
                    .ids()
                    .with_row(DirV::S, chip.row_serdes(DirV::S))
                    .split_at(chip.row_clk)[hv.v];
                for row in rows {
                    let wire_int = self.naming.interconnect[&Chip::DIE.cell(col, row).wire(w)];
                    self.claim_pip(wire_int, wire);
                }
                for ll in ["LL", "UL", "LR", "UR"] {
                    let wire_in = self.rc_wire(cell, &format!("{ll}{name}I"));
                    self.claim_pip(wire, wire_in);
                }
                let wire_in = self.rc_wire(cell, &format!("{ll}{name}I"));
                self.add_bel_wire(bcrd, format!("{name}_{hv}_IN"), wire_in);
                let wire_in_in = self.rc_wire(
                    match hv.h {
                        DirH::W => cell,
                        DirH::E => cell.delta(1, 0),
                    },
                    &format!("{name}{ll}_{name}"),
                );
                self.add_bel_wire(bcrd, format!("{name}_{hv}_IN_IN"), wire_in_in);
                self.claim_pip(wire_in, wire_in_in);
            }
        }
    }

    pub(super) fn process_pclk_ecp3(&mut self) {
        for (tcid, clks) in [
            (tcls::RPCLK_DRIVER, [0, 4].as_slice()),
            (tcls::RPCLK_DRIVER_W, [0, 4, 2, 6, 3, 7].as_slice()),
            (tcls::RPCLK_DRIVER_E, [0, 4, 1, 5].as_slice()),
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                {
                    let mut ex = self.extract_switchbox(tcrd.bel(bslots::PCLK_DRIVER));
                    for &i in clks {
                        ex.progbuf_output_int(wires::RPCLK[i].cell(0))
                            .input_int_no_resolve(wires::RPCLK_COL[i].cell(0));
                    }
                }
                if matches!(self.chip.kind, ChipKind::Ecp3(_))
                    && tcrd.row == self.chip.row_serdes(DirV::S) - 1
                {
                    for &i in clks {
                        self.claim_pip(
                            self.naming.interconnect[&tcrd.wire(wires::RPCLK_COL[i])],
                            self.naming.interconnect[&tcrd.delta(0, 1).wire(wires::RPCLK_COL[i])],
                        );
                    }
                }
            }
        }

        for (tcid, clks) in [
            (tcls::PCLK_COL_DRIVER_W, [3, 7, 2, 6].as_slice()),
            (tcls::PCLK_COL_DRIVER_IO_N_W, [3, 7, 2, 6].as_slice()),
            (tcls::PCLK_COL_DRIVER_E, [1, 5].as_slice()),
            (tcls::PCLK_COL_DRIVER_IO_N_E, [1, 5].as_slice()),
        ] {
            if !self.edev.db.tile_classes.contains_id(tcid) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::PCLK_COL_DRIVER);
                let mut ex = self.extract_switchbox(bcrd);
                for &idx in clks {
                    ex.input_int(wires::RPCLK_ROW[idx].cell(0))
                        .progbuf_output_int(wires::RPCLK_COL[idx].cell(0));
                    if ex.ctx.chip.rows_pclk.range(bcrd.row).start() != bcrd.row {
                        ex.progbuf_output_int(wires::RPCLK_COL[idx].cell(1));
                    }
                }
            }
        }

        for (rows, &row_drv) in &self.chip.rows_pclk {
            for cell in self.edev.row(Chip::DIE, row_drv) {
                let idx = self.chip.col_sclk_idx(cell.col);
                let pclk_idx = [idx, idx + 4];
                let cell_nom = if cell.row == self.chip.row_s() {
                    cell.with_row(self.chip.row_clk)
                } else {
                    cell
                };
                let (r, c) = self.rc(cell_nom);

                for i in 0..2 {
                    let pclk_i = pclk_idx[i];

                    // what in fuck's name.
                    let (name_s, name_n) = if matches!(self.chip.kind, ChipKind::Ecp3(_))
                        && self.chip.rows.len() == 53
                        && cell.col.to_idx() == 36
                        && cell.row.to_idx() == 18
                    {
                        if i == 0 {
                            (Some(format!("DCC_R{r}C{c}_0")), format!("DCC_R{r}C{c}_1"))
                        } else {
                            (Some(format!("DCC_R{r}C{c}C")), format!("DCC_R{r}C{c}D"))
                        }
                    } else if matches!(self.chip.kind, ChipKind::Ecp3(_))
                        && self.chip.rows.len() == 53
                        && cell.col.to_idx() == 45
                        && cell.row.to_idx() == 18
                    {
                        if i == 0 {
                            (Some(format!("DCC_R{r}C{c}A")), format!("DCC_R{r}C{c}B"))
                        } else {
                            (Some(format!("DCC_R{r}C{c}C")), format!("DCC_R{r}C{c}D"))
                        }
                    } else if matches!(self.chip.kind, ChipKind::MachXo2(_))
                        && self.chip.rows.len() == 26
                        && row_drv.to_idx() == 6
                    {
                        // ??? sigh.
                        if i == 0 {
                            (Some(format!("DCC_R{r}C{c}A")), format!("DCC_R{r}C{c}B"))
                        } else {
                            (Some(format!("DCC_R{r}C{c}C")), format!("DCC_R{r}C{c}D"))
                        }
                    } else if cell.row == self.chip.row_s() {
                        (None, format!("DCC_R{r}C{c}_{i}"))
                    } else {
                        (
                            Some(format!("DCC_R{r}C{c}_{i}B")),
                            format!("DCC_R{r}C{c}_{i}T"),
                        )
                    };
                    self.extract_bel_cell(cell.bel(bslots::PCLK_DCC_N[i]), cell_nom)
                        .name(name_n)
                        .input_hint_rc(
                            bcls::DCC::CLKI,
                            format!("CLKI{pclk_i}T_DCC"),
                            wires::RPCLK_ROW[i * 4].cell(0),
                        )
                        .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}T_DCC"))
                        .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{pclk_i}T_DCC"))
                        .claim_pip("CLKO", "CLKI");

                    if let Some(name_s) = name_s {
                        let mut bel =
                            self.extract_bel_cell(cell.bel(bslots::PCLK_DCC_S[i]), cell_nom);
                        bel.name(name_s)
                            .input_hint_rc(
                                bcls::DCC::CLKI,
                                format!("CLKI{pclk_i}B_DCC"),
                                wires::RPCLK_ROW[i * 4].cell(0),
                            )
                            .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}B_DCC"));
                        let out = bel.ctx.rc_wire(cell_nom, &format!("CLKO{pclk_i}B_DCC"));
                        if rows.start() != cell.row {
                            bel.output_xlat(bcls::DCC::CLKO, out);
                        } else {
                            bel.output(bcls::DCC::CLKO, wires::RPCLK_COL[i * 4].cell(1))
                                .add_wire("CLKO", out);
                        }
                        bel.claim_pip("CLKO", "CLKI");
                    }
                }
            }
        }

        let mut vprx_wires: BTreeMap<(DirV, ColId, usize), WireName> = BTreeMap::new();
        for &tcrd in &self.edev.tile_index[tcls::PCLK_ROW_DRIVER] {
            let cslot_row = defs::ecp3::tcls::PCLK_ROW_DRIVER::ROW;
            let cslot_w = defs::ecp3::tcls::PCLK_ROW_DRIVER::W;
            let cslot_e = defs::ecp3::tcls::PCLK_ROW_DRIVER::E;
            let has_w = self.edev[tcrd].cells.contains_id(cslot_w);
            let has_e = self.edev[tcrd].cells.contains_id(cslot_e);
            let bcrd = tcrd.bel(bslots::PCLK_ROW_DRIVER);
            let col_src = if has_e { tcrd.col } else { tcrd.col - 1 };
            let cell_src = tcrd.with_col(col_src);
            let v = tcrd.row.side_of(self.chip.row_clk);
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..8 {
                if matches!(ex.ctx.chip.kind, ChipKind::Ecp3(_)) {
                    let hpsx = ex.ctx.naming.interconnect[&cell_src.wire(wires::PCLK_ROW[i])];
                    let vprx = ex.ctx.find_single_in(hpsx);
                    match vprx_wires.entry((v, tcrd.col, i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(vprx);
                            ex.input_bind(vprx, cslot_row.wire(wires::PCLK_ROOT[i]));
                            ex.ctx
                                .claim_pip_int_in(vprx, tcrd.wire(wires::PCLK_ROOT[i]));
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), vprx);
                            ex.input_xlat(vprx);
                        }
                    }
                } else {
                    ex.input_int(cslot_row.wire(wires::PCLK_ROOT[i]));
                }
                if has_w {
                    ex.progbuf_output_int(cslot_w.wire(wires::PCLK_ROW[i]));
                }
                if has_e {
                    ex.progbuf_output_int(cslot_e.wire(wires::PCLK_ROW[i]));
                }
            }
        }
    }
}
