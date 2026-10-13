use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, MachXo2Kind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{LegacyBel, WireSlotIdExt},
    dir::{Dir, DirHV, DirV},
    grid::{ColId, DieIdExt},
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_hsdclk_root_machxo2(&mut self) {
        for (row, rd) in &self.chip.rows {
            if !matches!(rd.kind, RowKind::Io | RowKind::Ebr) {
                continue;
            }
            let bcrd = Chip::DIE
                .cell(self.chip.col_clk, row)
                .bel(bslots::HSDCLK_SPLITTER);
            for i in 0..8 {
                for ci_base in [0, 4] {
                    let wire = wires::RSCLK_H[i / 4 * 4].cell(ci_base + i % 4);
                    let wire = self.edev.bel_wire(bcrd, wire);
                    let wname = self.naming.interconnect[&wire];
                    let wname_root =
                        self.naming.interconnect[&self.chip.clk_root().wire(wires::SCLK_ROOT[i])];
                    self.claim_pip(wname, wname_root);
                }
            }
        }
    }

    pub(super) fn process_pclk_machxo2(&mut self) {
        for (row, rd) in &self.chip.rows {
            if !rd.pclk_drive {
                continue;
            }
            let mut row_n = row;
            let mut row_s = row;
            while row_n != self.chip.row_n() && !self.chip.rows[row_n + 1].pclk_break {
                row_n += 1;
            }
            row_n += 1;
            if row != self.chip.row_s() {
                row_s -= 1;
                while row_s != self.chip.row_s() && !self.chip.rows[row_s].pclk_break {
                    row_s -= 1;
                }
                if self.chip.rows[row_s].pclk_drive {
                    row_s = row;
                }
            }
            let mut hpsx_wires: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_n: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_s: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut col_hpsx = self.chip.col_w();
            for cell in self.edev.row(Chip::DIE, row) {
                let cell_nom = if cell.row == self.chip.row_s() {
                    cell.with_row(self.chip.row_clk)
                } else {
                    cell
                };
                if self.chip.columns[cell.col].sdclk_break {
                    col_hpsx = cell.col;
                }
                let idx = self.chip.col_sclk_idx(cell.col);
                let mut pclk_idx = vec![idx, idx + 4];
                if self.edev.has_bel(cell.bel(bslots::RPCLK_DRIVER)) {
                    if cell.col == self.chip.col_w() {
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

                    if row_s != cell.row {
                        let wire_s =
                            self.naming.interconnect[&cell.delta(0, -1).wire(wires::RPCLK[i])];
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
                    }
                }

                let (r, c) = self.rc(cell_nom);
                for i in 0..2 {
                    let pclk_i = pclk_idx[i];

                    let (name_s, name_n) = if cell.row == self.chip.row_s() {
                        (None, format!("DCC_R{r}C{c}_{i}"))
                    } else if self.chip.rows.len() == 26 && row.to_idx() == 6 {
                        // ??? sigh.
                        if i == 0 {
                            (Some(format!("DCC_R{r}C{c}A")), format!("DCC_R{r}C{c}B"))
                        } else {
                            (Some(format!("DCC_R{r}C{c}C")), format!("DCC_R{r}C{c}D"))
                        }
                    } else {
                        (
                            Some(format!("DCC_R{r}C{c}_{i}B")),
                            format!("DCC_R{r}C{c}_{i}T"),
                        )
                    };

                    let in_n = self.rc_wire(cell_nom, &format!("CLKI{pclk_i}T_DCC"));
                    let hpsx = self.find_single_in(in_n);
                    self.bind_wire(
                        hpsx,
                        self.edev
                            .resolve_wire(cell.wire(wires::PCLK_ROOT[pclk_i]))
                            .unwrap(),
                    );

                    self.extract_bel_cell(cell.bel(bslots::RPCLK_DCC_N[i]), cell_nom)
                        .name(name_n)
                        .input_xlat(bcls::DCC::CLKI, in_n)
                        .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}T_DCC"))
                        .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{pclk_i}T_DCC"))
                        .claim_pip("CLKO", "CLKI")
                        .add_wire_no_claim("HPSX", hpsx);

                    if let Some(name_s) = name_s {
                        let mut bel =
                            self.extract_bel_cell(cell.bel(bslots::RPCLK_DCC_S[i]), cell_nom);
                        bel.name(name_s)
                            .input_xlat_rc(bcls::DCC::CLKI, format!("CLKI{pclk_i}B_DCC"))
                            .input_xlat_rc(bcls::DCC::CE, format!("JCE{pclk_i}B_DCC"));

                        let out = bel.ctx.rc_wire(cell_nom, &format!("CLKO{pclk_i}B_DCC"));
                        if row_s != cell.row {
                            bel.output_xlat(bcls::DCC::CLKO, out);
                        } else {
                            bel.output(bcls::DCC::CLKO, wires::RPCLK[pclk_i].cell(1))
                                .add_wire("CLKO", out);
                        }
                        bel.claim_pip("CLKO", "CLKI")
                            .add_wire_no_claim("HPSX", hpsx);
                    }

                    match hpsx_wires.entry((col_hpsx, pclk_i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(hpsx);
                            self.claim_node(hpsx);
                            let vprx = self.naming.interconnect
                                [&self.chip.clk_root().wire(wires::PCLK_ROOT[pclk_i])];
                            self.claim_pip(hpsx, vprx);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), hpsx);
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
                let pclk_idx = if cell.col == self.chip.col_w() {
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
                        .permabuf_output_xlat_no_claim(vptx_wires_n[&(cell.col, i)]);
                    if cell.row != ex.ctx.chip.row_s() {
                        if row_s != cell.row {
                            ex.permabuf_output_xlat_no_claim(vptx_wires_s[&(cell.col, i)]);
                        } else {
                            ex.permabuf(wires::RPCLK[i].cell(1), wires::PCLK_ROOT[i].cell(0));
                        }
                    }
                }
            }
        }
    }

    pub(super) fn process_dlldel_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        if is_smol {
            return;
        }
        for (edge, idx) in [
            (Dir::W, 0),
            (Dir::W, 1),
            (Dir::W, 2),
            (Dir::E, 0),
            (Dir::S, 0),
            (Dir::S, 1),
            (Dir::N, 0),
            (Dir::N, 1),
        ] {
            let cell = match edge {
                Dir::H(edge) => Chip::DIE.cell(self.chip.col_edge(edge), self.chip.row_clk),
                Dir::V(edge) => Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_edge(edge)),
            };
            let ci = if edge == Dir::E && self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachXo2)
            {
                2
            } else {
                0
            };
            {
                self.extract_switchbox(cell.bel(bslots::CLK_INT))
                    .output_bind_rc_io(format!("JINCK{idx}"), wires::INCK[idx].cell(ci))
                    .input_io_rc_io(format!("JPADDI{idx}"), wires::IO_GCLK[idx].cell(ci))
                    .input_bind_rc_io(format!("DLLDEL{idx}"), wires::OUT_DLLDEL_CLKO[idx].cell(ci));
            }
            let bcrd = cell.bel(bslots::DLLDEL[idx]);
            let lrbt = match edge {
                Dir::W => 'L',
                Dir::E => 'R',
                Dir::S => 'B',
                Dir::N => 'T',
            };
            let mut bel = self.extract_bel(bcrd);
            bel.name(format!("{lrbt}DLLDEL{idx}"))
                .input_io_rc_io(
                    bcls::DLLDEL::CLKI,
                    format!("JCLKI{idx}_DLLDEL"),
                    wires::IO_GCLK[idx].cell(ci),
                )
                .output_xlat_rc_io(bcls::DLLDEL::CLKO, format!("CLKO{idx}_DLLDEL"));

            let cell_dqsdll = bel.ctx.chip.special_loc[&SpecialLocKey::DqsDll(match edge {
                Dir::W => Dir::S,
                Dir::E => Dir::N,
                Dir::S => Dir::S,
                Dir::N => Dir::N,
            })];
            bel.add_wire_rc_io("DQSDEL", format!("JDQSDEL{idx}_DLLDEL"));
            let wire_dqsdll = bel.ctx.rc_wire(cell_dqsdll, "JDQSDEL_DQSDLL");
            bel.claim_pip_from("DQSDEL", wire_dqsdll);
        }
    }

    pub(super) fn process_clk_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        let has_bank4 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(4));
        let has_2ebr = self
            .chip
            .rows
            .values()
            .filter(|rd| rd.kind == RowKind::Ebr)
            .count()
            >= 2;

        let bcrd = self.chip.bel_clk_root();
        let cell = bcrd.cell.delta(-1, 0);
        self.name_bel_null(bcrd);
        let mut bel = LegacyBel::default();

        let mut sclk_in = BTreeMap::new();
        for (edge, lrbt) in [(Dir::W, 'L'), (Dir::E, 'R'), (Dir::S, 'B'), (Dir::N, 'T')] {
            for i in 0..2 {
                let wire = self.rc_wire(cell, &format!("JSNETCIB{lrbt}{i}"));
                self.add_bel_wire(bcrd, format!("SCLK_IN_{edge}{i}"), wire);
                bel.pins
                    .insert(format!("SCLK_IN_{edge}{i}"), self.xlat_int_wire(bcrd, wire));
                sclk_in.insert((Some(edge), i), wire);
            }
        }
        for i in 0..8 {
            let wire = self.rc_wire(cell, &format!("JSNETCIBMID{i}"));
            self.add_bel_wire(bcrd, format!("SCLK_IN_M{i}"), wire);
            bel.pins
                .insert(format!("SCLK_IN_M{i}"), self.xlat_int_wire(bcrd, wire));
            sclk_in.insert((None, i), wire);
        }

        let mut io_in = BTreeMap::new();
        for (edge, i, bi) in [
            (Dir::W, 0, "30"),
            (Dir::W, 1, if has_bank4 { "40" } else { "31" }),
            (Dir::W, 2, if has_bank4 { "50" } else { "32" }),
            (Dir::E, 0, "10"),
            (Dir::S, 0, "20"),
            (Dir::S, 1, "21"),
            (Dir::N, 0, "00"),
            (Dir::N, 1, "01"),
        ] {
            let wire = self.rc_wire(cell, &format!("JPCLKT{bi}"));
            self.add_bel_wire(bcrd, format!("IO_IN_{edge}{i}"), wire);
            io_in.insert((edge, i), wire);

            if is_smol {
                let io = self.chip.special_io[&SpecialIoKey::Clock(edge, i)];
                let io_idx = bslots::IO.index_of(io.slot).unwrap();
                let wire_io = self.rc_io_wire(
                    io.cell,
                    &format!("JDI{abcd}", abcd = ['A', 'B', 'C', 'D'][io_idx]),
                );
                self.claim_pip(wire, wire_io);
            } else {
                let cell_io = match edge {
                    Dir::H(edge) => cell.with_col(self.chip.col_edge(edge)),
                    Dir::V(_) => self.chip.bel_eclksync(edge, 0).cell,
                };
                let wire_io = self.rc_io_wire(cell_io, &format!("JINCK{i}"));
                self.claim_pip(wire, wire_io);
            }
        }

        let mut pclk_in = BTreeMap::new();
        for (key, name, cond) in [
            ((Some(Dir::W), 0), "PCLKCIBLLQ0", true),
            ((Some(Dir::W), 1), "PCLKCIBLLQ1", true),
            ((Some(Dir::W), 2), "PCLKCIBULQ0", has_2ebr),
            ((Some(Dir::W), 3), "PCLKCIBULQ1", has_2ebr),
            ((Some(Dir::E), 0), "PCLKCIBLRQ0", true),
            ((Some(Dir::E), 1), "PCLKCIBLRQ1", true),
            ((Some(Dir::E), 2), "PCLKCIBURQ0", has_2ebr),
            ((Some(Dir::E), 3), "PCLKCIBURQ1", has_2ebr),
            ((Some(Dir::S), 0), "PCLKCIBVIQB0", true),
            ((Some(Dir::S), 1), "PCLKCIBVIQB1", true),
            ((Some(Dir::N), 0), "PCLKCIBVIQT0", true),
            ((Some(Dir::N), 1), "PCLKCIBVIQT1", true),
            ((None, 0), "PCLKCIBMID0", has_2ebr),
            ((None, 1), "PCLKCIBMID1", has_2ebr),
            ((None, 2), "PCLKCIBMID2", !is_smol),
            ((None, 3), "PCLKCIBMID3", !is_smol),
        ] {
            if !cond {
                continue;
            }
            let wire = self.rc_wire(cell, name);
            let wire_in = self.rc_wire(cell, &format!("J{name}"));
            let i = key.1;
            let pin = if let Some(edge) = key.0 {
                format!("PCLK_IN_{edge}{i}")
            } else {
                format!("PCLK_IN_M{i}")
            };
            self.add_bel_wire(bcrd, &pin, wire);
            self.add_bel_wire(bcrd, format!("{pin}_IN"), wire_in);
            self.claim_pip(wire, wire_in);
            bel.pins.insert(pin, self.xlat_int_wire(bcrd, wire_in));
            pclk_in.insert(key, wire);
        }

        let mut pll_cdiv_in = vec![];
        if !is_smol {
            for edge in [DirV::S, DirV::N] {
                let bt = match edge {
                    DirV::S => 'B',
                    DirV::N => 'T',
                };
                for idx in 0..2 {
                    for pin in ["CDIV1", "CDIVX"] {
                        let wire = self.rc_wire(cell, &format!("J{bt}{pin}{idx}"));
                        let cell_cdiv = self.chip.bel_eclksync(Dir::V(edge), idx).cell;
                        let wire_cdiv = self.rc_wire(cell_cdiv, &format!("J{pin}{idx}_CLKDIV"));
                        self.add_bel_wire(bcrd, format!("CLKDIV_IN_{edge}{idx}_{pin}"), wire);
                        self.claim_pip(wire, wire_cdiv);
                        pll_cdiv_in.push(wire);
                    }
                }
            }
            for (lr, quad) in [('L', DirHV::NW), ('R', DirHV::NE)] {
                let Some(bel_pll) = self.chip.try_bel_pll(quad, 0) else {
                    continue;
                };
                let cell_pll = bel_pll.cell;
                for (i, pin) in ["CLKOP", "CLKOS", "CLKOS2", "CLKOS3"]
                    .into_iter()
                    .enumerate()
                {
                    let wire = self.rc_wire(cell, &format!("J{lr}PLLCLK{i}"));
                    let wire_pll = self.rc_wire(cell_pll, &format!("J{pin}_PLL"));
                    self.add_bel_wire(bcrd, format!("PLL_IN_{quad}_{pin}"), wire);
                    self.claim_pip(wire, wire_pll);
                    pll_cdiv_in.push(wire);
                }
            }
        }

        for i in 0..8 {
            let wire_out =
                self.naming.interconnect[&self.chip.clk_root().wire(wires::PCLK_ROOT[i])];

            let bcrd_dcc = bcrd.bel(bslots::DCC[i]);
            self.name_bel(bcrd_dcc, [format!("DCC{i}")]);
            let mut bel_dcc = LegacyBel::default();

            let ce = self.rc_wire(cell, &format!("JCE{i}_DCC"));
            self.add_bel_wire(bcrd_dcc, "CE", ce);
            bel_dcc
                .pins
                .insert("CE".into(), self.xlat_int_wire(bcrd_dcc, ce));

            let clko = self.rc_wire(cell, &format!("CLKO{i}_DCC"));
            self.add_bel_wire(bcrd_dcc, "CLKO", clko);
            self.claim_pip(wire_out, clko);

            let clki = self.rc_wire(cell, &format!("CLKI{i}_DCC"));
            self.add_bel_wire(bcrd_dcc, "CLKI", clki);
            self.claim_pip(clko, clki);

            self.insert_legacy_bel(bcrd_dcc, bel_dcc);

            if i < 6 {
                let clki_in = self.rc_wire(cell, &format!("VPRXCLKI{i}"));
                self.add_bel_wire(bcrd_dcc, "CLKI_IN", clki_in);
                self.claim_pip(clki, clki_in);

                for &wire in io_in.values() {
                    self.claim_pip(clki_in, wire);
                }

                if is_smol {
                    for &wire in pclk_in.values() {
                        self.claim_pip(clki_in, wire);
                    }
                } else {
                    for &wire in &pll_cdiv_in {
                        self.claim_pip(clki_in, wire);
                    }
                    for key in [
                        [(Some(Dir::W), 2), (Some(Dir::N), 0)],
                        [(Some(Dir::W), 0), (None, 0)],
                        [(Some(Dir::E), 2), (Some(Dir::N), 1)],
                        [(Some(Dir::E), 0), (None, 1)],
                        [(Some(Dir::W), 3), (Some(Dir::S), 0)],
                        [(Some(Dir::W), 1), (None, 2)],
                    ][i]
                    {
                        if let Some(&wire) = pclk_in.get(&key) {
                            self.claim_pip(clki_in, wire);
                        }
                    }
                }
            } else {
                let bcrd_dcm = bcrd.bel(bslots::DCM[i - 6]);
                self.name_bel(bcrd_dcm, [format!("DCM{i}")]);
                let mut bel_dcm = LegacyBel::default();

                let dcmout = self.rc_wire(cell, &format!("DCMOUT{i}_DCM"));
                self.add_bel_wire(bcrd_dcm, "DCMOUT", dcmout);
                self.claim_pip(clki, dcmout);

                let sel = self.rc_wire(cell, &format!("JSEL{i}_DCM"));
                self.add_bel_wire(bcrd_dcm, "SEL", sel);
                bel_dcm
                    .pins
                    .insert("SEL".into(), self.xlat_int_wire(bcrd_dcm, sel));

                let clk0 = self.rc_wire(cell, &format!("CLK0_{i}_DCM"));
                self.add_bel_wire(bcrd_dcm, "CLK0", clk0);
                self.claim_pip(dcmout, clk0);

                let clk1 = self.rc_wire(cell, &format!("CLK1_{i}_DCM"));
                self.add_bel_wire(bcrd_dcm, "CLK1", clk1);
                self.claim_pip(dcmout, clk1);

                let clk0_in = self.rc_wire(cell, &format!("VPRXCLKI{i}0"));
                self.add_bel_wire(bcrd_dcm, "CLK0_IN", clk0_in);
                self.claim_pip(clk0, clk0_in);

                let clk1_in = self.rc_wire(cell, &format!("VPRXCLKI{i}1"));
                self.add_bel_wire(bcrd_dcm, "CLK1_IN", clk1_in);
                self.claim_pip(clk1, clk1_in);

                self.insert_legacy_bel(bcrd_dcm, bel_dcm);

                for &wire in io_in.values() {
                    self.claim_pip(clk0_in, wire);
                    self.claim_pip(clk1_in, wire);
                }

                if is_smol {
                    for &wire in pclk_in.values() {
                        self.claim_pip(clk0_in, wire);
                        self.claim_pip(clk1_in, wire);
                    }
                } else {
                    for &wire in &pll_cdiv_in {
                        self.claim_pip(clk0_in, wire);
                        self.claim_pip(clk1_in, wire);
                    }
                    for (wire_out, key) in [
                        [
                            (clk0_in, (None, 2)),
                            (clk0_in, (Some(Dir::E), 3)),
                            (clk0_in, (Some(Dir::S), 1)),
                            (clk1_in, (Some(Dir::E), 1)),
                            (clk1_in, (None, 3)),
                        ],
                        [
                            (clk0_in, (Some(Dir::E), 1)),
                            (clk0_in, (None, 3)),
                            (clk1_in, (None, 2)),
                            (clk1_in, (Some(Dir::E), 3)),
                            (clk1_in, (Some(Dir::S), 1)),
                        ],
                    ][i - 6]
                    {
                        if let Some(&wire) = pclk_in.get(&key) {
                            self.claim_pip(wire_out, wire);
                        }
                    }
                }
            }
        }

        for i in 0..8 {
            let wire_out =
                self.naming.interconnect[&self.chip.clk_root().wire(wires::SCLK_ROOT[i])];

            let (edge, idx) = [
                (Dir::W, 0),
                (Dir::W, 1),
                (Dir::E, 0),
                (Dir::W, 2),
                (Dir::N, 0),
                (Dir::N, 1),
                (Dir::S, 0),
                (Dir::S, 1),
            ][i];
            self.claim_pip(wire_out, io_in[&(edge, idx)]);

            let sources = [
                [
                    (Some(Dir::W), 1),
                    (Some(Dir::S), 1),
                    (Some(Dir::E), 0),
                    (Some(Dir::N), 0),
                    (None, 1),
                    (None, 2),
                    (None, 7),
                ],
                [
                    (Some(Dir::W), 0),
                    (Some(Dir::S), 0),
                    (Some(Dir::E), 1),
                    (Some(Dir::N), 1),
                    (None, 0),
                    (None, 5),
                    (None, 6),
                ],
                [
                    (Some(Dir::W), 1),
                    (Some(Dir::E), 0),
                    (Some(Dir::N), 0),
                    (None, 1),
                    (None, 2),
                    (None, 4),
                    (None, 7),
                ],
                [
                    (Some(Dir::S), 0),
                    (Some(Dir::E), 1),
                    (Some(Dir::N), 1),
                    (None, 0),
                    (None, 3),
                    (None, 5),
                    (None, 6),
                ],
                [
                    (Some(Dir::W), 1),
                    (Some(Dir::S), 1),
                    (Some(Dir::E), 0),
                    (None, 1),
                    (None, 2),
                    (None, 4),
                    (None, 7),
                ],
                [
                    (Some(Dir::W), 0),
                    (Some(Dir::S), 0),
                    (Some(Dir::N), 1),
                    (None, 0),
                    (None, 3),
                    (None, 5),
                    (None, 6),
                ],
                [
                    (Some(Dir::W), 1),
                    (Some(Dir::S), 1),
                    (Some(Dir::E), 0),
                    (Some(Dir::N), 0),
                    (None, 1),
                    (None, 2),
                    (None, 4),
                ],
                [
                    (Some(Dir::W), 0),
                    (Some(Dir::S), 0),
                    (Some(Dir::E), 1),
                    (Some(Dir::N), 1),
                    (None, 3),
                    (None, 5),
                    (None, 6),
                ],
            ];
            for (e, si) in sources[i] {
                self.claim_pip(wire_out, sclk_in[&(e, si)]);
            }
        }

        self.insert_legacy_bel(bcrd, bel);

        let bcrd_centest = bcrd.bel(bslots::CLKTEST);
        self.name_bel(bcrd_centest, ["CENTEST"]);
        self.insert_simple_legacy_bel(bcrd_centest, cell, "CENTEST");

        if !is_smol {
            for i in 0..2 {
                let bcrd = bcrd.bel(bslots::ECLKBRIDGECS[i]);
                self.name_bel(bcrd, [format!("ECLKBRIDGECS{i}")]);
                let mut bel = LegacyBel::default();

                let sel = self.rc_wire(cell, &format!("JSEL{i}_ECLKBRIDGECS"));
                self.add_bel_wire(bcrd, "SEL", sel);
                bel.pins.insert("SEL".into(), self.xlat_int_wire(bcrd, sel));

                for j in 0..2 {
                    let bt = ['B', 'T'][i];
                    let clk_int_in = self.rc_wire(cell, &format!("JECLKCIB{bt}{j}"));
                    self.add_bel_wire(bcrd, format!("CLK{j}_INT_IN"), clk_int_in);
                    bel.pins
                        .insert(format!("CLK{j}"), self.xlat_int_wire(bcrd, clk_int_in));

                    let clk_int = self.rc_wire(cell, &format!("ECLKCIB{bt}{j}"));
                    self.add_bel_wire(bcrd, format!("CLK{j}_INT"), clk_int);
                    self.claim_pip(clk_int, clk_int_in);

                    let clk_in = self.rc_wire(cell, &format!("EBRG{i}CLK{j}"));
                    self.add_bel_wire(bcrd, format!("CLK{j}_IN"), clk_in);
                    self.claim_pip(clk_in, clk_int);
                    self.claim_pip(clk_in, io_in[&([Dir::S, Dir::N][i], j as u8)]);

                    let wire_pll = self.rc_wire(cell, &format!("JLPLLCLK{j}"));
                    self.claim_pip(clk_in, wire_pll);
                    if self.chip.try_bel_pll(DirHV::NE, 0).is_some() {
                        let wire_pll = self.rc_wire(cell, &format!("JRPLLCLK{jj}", jj = j ^ 1));
                        self.claim_pip(clk_in, wire_pll);
                    }

                    let clk = self.rc_wire(cell, &format!("CLK{j}_{i}_ECLKBRIDGECS"));
                    self.add_bel_wire(bcrd, format!("CLK{j}"), clk);
                    self.claim_pip(clk, clk_in);
                }

                let ecsout = self.rc_wire(cell, &format!("JECSOUT{i}_ECLKBRIDGECS"));
                self.add_bel_wire(bcrd, "ECSOUT", ecsout);

                self.insert_legacy_bel(bcrd, bel);
            }
        }
    }
}
