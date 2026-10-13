use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind},
    defs::{bcls, bslots, ecp3::tcls::CLK_ROOT, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{DirH, DirHV, DirV},
    grid::{ColId, DieIdExt},
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
                .filter(|&(col, cd)| cd.sdclk_break && col != self.chip.col_clk)
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
                let bcrd = cell.bel(bslots::HSDCLK_SPLITTER);
                for i in 0..8 {
                    for ci_base in [0, 4] {
                        let wire = wires::RSCLK_H[i / 4 * 4].cell(ci_base + i % 4);
                        let wire = self.edev.bel_wire(bcrd, wire);
                        let wname = self.naming.interconnect[&wire];
                        let wname_out = self.pips_bwd[&wname]
                            .iter()
                            .copied()
                            .find(|&wn| {
                                self.naming.strings[wn.suffix].starts_with("VSRX")
                                    && self.chip.xlat_rc_wire(wn).col == col - 1
                            })
                            .unwrap();
                        match vsrx.entry((col, row, i)) {
                            btree_map::Entry::Vacant(e) => {
                                e.insert(wname_out);
                                if row == self.chip.row_clk {
                                    self.add_bel_wire_no_claim(bcrd, format!("OUT_{i}"), wname_out);
                                } else {
                                    self.add_bel_wire(bcrd, format!("OUT_{i}"), wname_out);
                                }
                            }
                            btree_map::Entry::Occupied(e) => {
                                assert_eq!(*e.get(), wname_out);
                            }
                        }
                        self.claim_pip(wname, wname_out);
                    }
                }
            }
        }
        for (idx, &row) in root_rows.iter().enumerate() {
            for &col in &root_cols {
                let h = if col < self.chip.col_clk {
                    DirH::W
                } else {
                    DirH::E
                };
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
                } else {
                    let cslot = match quad {
                        DirHV::SW => CLK_ROOT::PLL_SW[idx],
                        DirHV::SE => CLK_ROOT::PLL_SE[idx],
                        DirHV::NW => CLK_ROOT::PLL_NW[idx],
                        DirHV::NE => CLK_ROOT::PLL_NE[idx],
                    };
                    for ci in 0..4 {
                        for i in 0..6 {
                            for w in [
                                wires::OUT_PLL_CLKOP[0],
                                wires::OUT_PLL_CLKOS[0],
                                wires::OUT_PLL_CLKOK,
                                wires::OUT_PLL_CLKOK2,
                            ] {
                                ex.edge(wires::IMUX_DCC_CLKI[i].cell(ci), cslot.wire(w));
                            }
                        }
                        for i in 0..2 {
                            for w in [
                                wires::OUT_PLL_CLKOS[0],
                                wires::OUT_PLL_CLKOK,
                                wires::OUT_PLL_CLKOK2,
                            ] {
                                ex.edge(wires::IMUX_DCS_CLKA[i].cell(ci), cslot.wire(w));
                                ex.edge(wires::IMUX_DCS_CLKB[i].cell(ci), cslot.wire(w));
                            }
                            let out = match quad.h {
                                DirH::W => wires::IMUX_DCS_CLKA,
                                DirH::E => wires::IMUX_DCS_CLKB,
                            };
                            ex.edge(out[i].cell(ci), cslot.wire(wires::OUT_PLL_CLKOP[0]));
                        }
                    }
                }
            }
        }
        for (idx, abcd) in ['A', 'B', 'C', 'D'].into_iter().enumerate() {
            let cslot = CLK_ROOT::SERDES[idx];
            if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                ex.input_xlat_rc_nudge(format!("JPCS{abcd}_TX_F_CLK"), cslot)
                    .input_xlat_rc_nudge(format!("JPCS{abcd}_TX_H_CLK"), cslot);
            } else {
                for ci in 0..4 {
                    for i in 0..6 {
                        let src =
                            [wires::OUT_SERDES_FF_TX_H_CLK, wires::OUT_SERDES_FF_TX_F_CLK][i % 2];
                        ex.edge(wires::IMUX_DCC_CLKI[i].cell(ci), cslot.wire(src));
                    }
                    for i in 0..2 {
                        ex.edge(
                            wires::IMUX_DCS_CLKA[i].cell(ci),
                            cslot.wire(wires::OUT_SERDES_FF_TX_H_CLK),
                        );
                        ex.edge(
                            wires::IMUX_DCS_CLKB[i].cell(ci),
                            cslot.wire(wires::OUT_SERDES_FF_TX_F_CLK),
                        );
                    }
                }
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

        if chip.kind == ChipKind::Ecp3A {
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
                    .input_xlat_rc(bcls::DCS::SEL, format!("J{ll}SEL{i}_DCS"))
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
                let (row_s, row_n) = match hv.v {
                    DirV::S => (chip.row_s() + 9, chip.row_clk),
                    DirV::N => (chip.row_clk, chip.row_n() + 1),
                };
                for row in row_s.range(row_n) {
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
        let mut is_bot = true;
        let mut vprx_wires: BTreeMap<(DirV, ColId, usize), WireName> = BTreeMap::new();
        let mut vprx_source = BTreeMap::new();
        let mut prev = self.chip.col_w();
        for (col, cd) in &self.chip.columns {
            if cd.sdclk_break {
                if col < self.chip.col_clk {
                    vprx_source.insert(prev, col);
                } else if col == self.chip.col_clk {
                    vprx_source.insert(prev, prev);
                } else {
                    if prev == self.chip.col_clk {
                        vprx_source.insert(prev, col);
                    }
                    vprx_source.insert(col, col);
                }
                prev = col;
            }
        }
        for (row, rd) in &self.chip.rows {
            if !rd.pclk_drive {
                continue;
            }
            let mut row_n = row;
            let mut row_s = row - 1;
            while row_n != self.chip.row_n() && !self.chip.rows[row_n + 1].pclk_break {
                row_n += 1;
            }
            row_n += 1;
            while !self.chip.rows[row_s].pclk_break {
                row_s -= 1;
            }
            let mut hpsx_wires: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_n: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_s: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_ss: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut col_hpsx = self.chip.col_w();
            for cell in self.edev.row(Chip::DIE, row) {
                if self.chip.columns[cell.col].sdclk_break {
                    col_hpsx = cell.col;
                }
                let idx = self.chip.col_sclk_idx(cell.col);
                let mut pclk_idx = vec![idx, idx + 4];
                if self.edev.has_bel(cell.bel(bslots::RPCLK_DRIVER)) {
                    if cell.col == self.chip.col_w() || cell.col == self.chip.col_clk {
                        pclk_idx.extend([
                            (idx + 3) % 4,
                            (idx + 3) % 4 + 4,
                            (idx + 2) % 4,
                            (idx + 2) % 4 + 4,
                        ]);
                    } else {
                        pclk_idx.extend([(idx + 1) % 4, (idx + 1) % 4 + 4]);
                    }
                }
                for &i in &pclk_idx {
                    let wire_n = self.naming.interconnect[&cell.wire(wires::RPCLK[i])];
                    let wire_n = self.find_single_in(wire_n);
                    self.add_bel_wire(cell.bel(bslots::INT), format!("PCLK{i}_N"), wire_n);
                    vptx_wires_n.insert((cell.col, i), wire_n);
                    for row in cell.row.range(row_n) {
                        let wire =
                            self.naming.interconnect[&cell.with_row(row).wire(wires::RPCLK[i])];
                        self.claim_pip(wire, wire_n);
                    }
                    self.bind_wire(
                        wire_n,
                        self.edev.resolve_wire(cell.wire(wires::RPCLK[i])).unwrap(),
                    );

                    let wire_s = self.naming.interconnect[&cell.delta(0, -1).wire(wires::RPCLK[i])];
                    let wire_s = self.find_single_in(wire_s);
                    self.add_bel_wire(cell.bel(bslots::INT), format!("PCLK{i}_S"), wire_s);
                    vptx_wires_s.insert((cell.col, i), wire_s);
                    for row in row_s.range(cell.row) {
                        let wire =
                            self.naming.interconnect[&cell.with_row(row).wire(wires::RPCLK[i])];
                        self.claim_pip(wire, wire_s);
                    }
                    self.bind_wire(
                        wire_s,
                        self.edev
                            .resolve_wire(cell.delta(0, -1).wire(wires::RPCLK[i]))
                            .unwrap(),
                    );

                    if is_bot
                        && let Some(&wire_ss) = self
                            .naming
                            .interconnect
                            .get(&cell.with_row(self.chip.row_s()).wire(wires::RPCLK[i]))
                    {
                        let wire_ss = self.find_single_in(wire_ss);
                        vptx_wires_ss.insert((cell.col, i), wire_ss);
                        self.add_bel_wire(cell.bel(bslots::INT), format!("PCLK{i}_SS"), wire_ss);
                        self.claim_pip(wire_ss, wire_s);
                        for row in self.chip.row_s().range(row_s) {
                            let wire =
                                self.naming.interconnect[&cell.with_row(row).wire(wires::RPCLK[i])];
                            self.claim_pip(wire, wire_ss);
                        }
                    }
                }

                let (r, c) = self.rc(cell);
                for i in 0..2 {
                    let pclk_i = pclk_idx[i];

                    // what in fuck's name.
                    let (name_s, name_n) = if self.chip.rows.len() == 53
                        && cell.col.to_idx() == 36
                        && cell.row.to_idx() == 18
                    {
                        if i == 0 {
                            (format!("DCC_R{r}C{c}_0"), format!("DCC_R{r}C{c}_1"))
                        } else {
                            (format!("DCC_R{r}C{c}C"), format!("DCC_R{r}C{c}D"))
                        }
                    } else if self.chip.rows.len() == 53
                        && cell.col.to_idx() == 45
                        && cell.row.to_idx() == 18
                    {
                        if i == 0 {
                            (format!("DCC_R{r}C{c}A"), format!("DCC_R{r}C{c}B"))
                        } else {
                            (format!("DCC_R{r}C{c}C"), format!("DCC_R{r}C{c}D"))
                        }
                    } else {
                        (format!("DCC_R{r}C{c}_{i}B"), format!("DCC_R{r}C{c}_{i}T"))
                    };

                    let in_s = self.rc_wire(cell, &format!("CLKI{pclk_i}B_DCC"));
                    let in_n = self.rc_wire(cell, &format!("CLKI{pclk_i}T_DCC"));
                    let hpsx = self.find_single_in(in_s);
                    self.bind_wire(
                        hpsx,
                        self.edev
                            .resolve_wire(cell.wire(wires::PCLK_ROOT[pclk_i]))
                            .unwrap(),
                    );
                    let vprx = self.find_single_in(hpsx);

                    self.extract_bel(cell.bel(bslots::RPCLK_DCC_S[i]))
                        .name(name_s)
                        .input_xlat(bcls::DCC::CLKI, in_s)
                        .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}B_DCC"))
                        .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{pclk_i}B_DCC"))
                        .claim_pip("CLKO", "CLKI")
                        .add_wire_no_claim("HPSX", hpsx)
                        .add_wire_no_claim("VPRX", vprx);

                    self.extract_bel(cell.bel(bslots::RPCLK_DCC_N[i]))
                        .name(name_n)
                        .input_xlat(bcls::DCC::CLKI, in_n)
                        .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}T_DCC"))
                        .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{pclk_i}T_DCC"))
                        .claim_pip("CLKO", "CLKI")
                        .add_wire_no_claim("HPSX", hpsx)
                        .add_wire_no_claim("VPRX", vprx);

                    match hpsx_wires.entry((col_hpsx, pclk_i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(hpsx);
                            self.claim_node(hpsx);
                            self.claim_pip(hpsx, vprx);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), hpsx);
                        }
                    }

                    let hv = DirHV {
                        h: if cell.col < self.chip.col_clk {
                            DirH::W
                        } else {
                            DirH::E
                        },
                        v: if cell.row < self.chip.row_clk {
                            DirV::S
                        } else {
                            DirV::N
                        },
                    };
                    match vprx_wires.entry((hv.v, vprx_source[&col_hpsx], pclk_i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(vprx);
                            self.claim_node(vprx);
                            let root = self.naming.interconnect[&Chip::DIE
                                .cell(self.chip.col_edge(hv.h), self.chip.row_edge(hv.v))
                                .wire(wires::PCLK_ROOT[pclk_i])];
                            self.claim_pip(vprx, root);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), vprx);
                        }
                    }
                }
            }

            let mut col_hpsx = self.chip.col_w();
            for cell in self.edev.row(Chip::DIE, row) {
                if self.chip.columns[cell.col].sdclk_break {
                    col_hpsx = cell.col;
                }
                let idx = self.chip.col_sclk_idx(cell.col);
                let bcrd = cell.bel(bslots::RPCLK_DRIVER);
                if !self.edev.has_bel(bcrd) {
                    continue;
                }
                let pclk_idx = if cell.col == self.chip.col_w() || cell.col == self.chip.col_clk {
                    vec![
                        (idx + 3) % 4,
                        (idx + 3) % 4 + 4,
                        (idx + 2) % 4,
                        (idx + 2) % 4 + 4,
                    ]
                } else {
                    vec![(idx + 1) % 4, (idx + 1) % 4 + 4]
                };

                let mut ex = self.extract_switchbox(bcrd);
                for i in pclk_idx {
                    ex.input_xlat(hpsx_wires[&(col_hpsx, i)])
                        .permabuf_output_xlat_no_claim(vptx_wires_s[&(cell.col, i)])
                        .permabuf_output_xlat_no_claim(vptx_wires_n[&(cell.col, i)]);
                }
            }

            is_bot = false;
        }
    }
}
