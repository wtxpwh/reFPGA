use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, MachXo2Kind, RowKind, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_entity::{EntityId, EntityIdArith};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV},
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
        for (rows, &row_drv) in &self.chip.rows_pclk {
            let mut hpsx_wires: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_n: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut vptx_wires_s: BTreeMap<(ColId, usize), WireName> = BTreeMap::new();
            let mut col_hpsx = self.chip.col_w();
            for cell in self.edev.row(Chip::DIE, row_drv) {
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
                    for row in cell.row.range(rows.end()) {
                        let wire =
                            self.naming.interconnect[&cell.with_row(row).wire(wires::RPCLK[i])];
                        self.claim_pip(wire, wire_n);
                    }
                    self.bind_wire(
                        wire_n,
                        self.edev.resolve_wire(cell.wire(wires::RPCLK[i])).unwrap(),
                    );

                    if rows.start() != cell.row {
                        let wire_s =
                            self.naming.interconnect[&cell.delta(0, -1).wire(wires::RPCLK[i])];
                        let wire_s = self.find_single_in(wire_s);
                        self.add_bel_wire(cell.bel(bslots::INT), format!("PCLK{i}_S"), wire_s);
                        vptx_wires_s.insert((cell.col, i), wire_s);
                        for row in rows.start().range(cell.row) {
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
                    } else if self.chip.rows.len() == 26 && row_drv.to_idx() == 6 {
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
                        if rows.start() != cell.row {
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
            for cell in self.edev.row(Chip::DIE, row_drv) {
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
                        if rows.start() != cell.row {
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
                    .output_xlat_rc_io(format!("JINCK{idx}"))
                    .input_io_rc_io(
                        format!("JPADDI{idx}"),
                        match edge {
                            Dir::W => wires::IO_GCLK_W[idx].cell(ci),
                            Dir::E => wires::IO_GCLK_E[idx].cell(ci),
                            Dir::S => wires::IO_GCLK_S[idx].cell(ci),
                            Dir::N => wires::IO_GCLK_N[idx].cell(ci),
                        },
                    )
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
                    match edge {
                        Dir::W => wires::IO_GCLK_W[idx].cell(ci),
                        Dir::E => wires::IO_GCLK_E[idx].cell(ci),
                        Dir::S => wires::IO_GCLK_S[idx].cell(ci),
                        Dir::N => wires::IO_GCLK_N[idx].cell(ci),
                    },
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

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::CLK_INT));
            ex.cell(cell)
                .input_xlat_rc("JSNETCIBL0")
                .input_xlat_rc("JSNETCIBL1")
                .input_xlat_rc("JSNETCIBR0")
                .input_xlat_rc("JSNETCIBR1")
                .input_xlat_rc("JSNETCIBB0")
                .input_xlat_rc("JSNETCIBB1")
                .input_xlat_rc("JSNETCIBT0")
                .input_xlat_rc("JSNETCIBT1")
                .input_xlat_rc("JSNETCIBMID0")
                .input_xlat_rc("JSNETCIBMID1")
                .input_xlat_rc("JSNETCIBMID2")
                .input_xlat_rc("JSNETCIBMID3")
                .input_xlat_rc("JSNETCIBMID4")
                .input_xlat_rc("JSNETCIBMID5")
                .input_xlat_rc("JSNETCIBMID6")
                .input_xlat_rc("JSNETCIBMID7")
                .output_bind_rc("VPRXCLKI0", wires::IMUX_DCC_CLKI[0].cell(0))
                .output_bind_rc("VPRXCLKI1", wires::IMUX_DCC_CLKI[1].cell(0))
                .output_bind_rc("VPRXCLKI2", wires::IMUX_DCC_CLKI[2].cell(0))
                .output_bind_rc("VPRXCLKI3", wires::IMUX_DCC_CLKI[3].cell(0))
                .output_bind_rc("VPRXCLKI4", wires::IMUX_DCC_CLKI[4].cell(0))
                .output_bind_rc("VPRXCLKI5", wires::IMUX_DCC_CLKI[5].cell(0))
                .output_bind_rc("VPRXCLKI60", wires::IMUX_DCM_CLK0[0].cell(0))
                .output_bind_rc("VPRXCLKI61", wires::IMUX_DCM_CLK1[0].cell(0))
                .output_bind_rc("VPRXCLKI70", wires::IMUX_DCM_CLK0[1].cell(0))
                .output_bind_rc("VPRXCLKI71", wires::IMUX_DCM_CLK1[1].cell(0))
                .input_xlat_rc("JPCLKCIBLLQ0")
                .input_xlat_rc("JPCLKCIBLLQ1")
                .input_xlat_rc("JPCLKCIBLRQ0")
                .input_xlat_rc("JPCLKCIBLRQ1")
                .input_xlat_rc("JPCLKCIBVIQB0")
                .input_xlat_rc("JPCLKCIBVIQB1")
                .input_xlat_rc("JPCLKCIBVIQT0")
                .input_xlat_rc("JPCLKCIBVIQT1")
                .input_xlat_rc("PCLKCIBLLQ0")
                .input_xlat_rc("PCLKCIBLLQ1")
                .input_xlat_rc("PCLKCIBLRQ0")
                .input_xlat_rc("PCLKCIBLRQ1")
                .input_xlat_rc("PCLKCIBVIQB0")
                .input_xlat_rc("PCLKCIBVIQB1")
                .input_xlat_rc("PCLKCIBVIQT0")
                .input_xlat_rc("PCLKCIBVIQT1");

            for i in 0..8 {
                ex.output_int(wires::SCLK_ROOT[i].cell(0));
            }

            if is_smol {
                ex.input_io_rc("JPCLKT30", wires::IO_GCLK_W[0].cell(0))
                    .input_io_rc("JPCLKT31", wires::IO_GCLK_W[1].cell(0))
                    .input_io_rc("JPCLKT32", wires::IO_GCLK_W[2].cell(0))
                    .input_io_rc("JPCLKT10", wires::IO_GCLK_E[0].cell(0))
                    .input_io_rc("JPCLKT20", wires::IO_GCLK_S[0].cell(0))
                    .input_io_rc("JPCLKT21", wires::IO_GCLK_S[1].cell(0))
                    .input_io_rc("JPCLKT00", wires::IO_GCLK_N[0].cell(0))
                    .input_io_rc("JPCLKT01", wires::IO_GCLK_N[1].cell(0));
            } else {
                ex.input_xlat_rc("JPCLKT30")
                    .input_xlat_rc(if has_bank4 { "JPCLKT40" } else { "JPCLKT31" })
                    .input_xlat_rc(if has_bank4 { "JPCLKT50" } else { "JPCLKT32" })
                    .input_xlat_rc("JPCLKT10")
                    .input_xlat_rc("JPCLKT20")
                    .input_xlat_rc("JPCLKT21")
                    .input_xlat_rc("JPCLKT00")
                    .input_xlat_rc("JPCLKT01")
                    .input_xlat_rc("JPCLKCIBMID2")
                    .input_xlat_rc("JPCLKCIBMID3")
                    .input_xlat_rc("PCLKCIBMID2")
                    .input_xlat_rc("PCLKCIBMID3")
                    .input_xlat_rc("JBCDIV10")
                    .input_xlat_rc("JBCDIV11")
                    .input_xlat_rc("JBCDIVX0")
                    .input_xlat_rc("JBCDIVX1")
                    .input_xlat_rc("JTCDIV10")
                    .input_xlat_rc("JTCDIV11")
                    .input_xlat_rc("JTCDIVX0")
                    .input_xlat_rc("JTCDIVX1")
                    .output_bind_rc("EBRG0CLK0", wires::IMUX_ECLKBRIDGECS_CLK0[0].cell(0))
                    .output_bind_rc("EBRG0CLK1", wires::IMUX_ECLKBRIDGECS_CLK1[0].cell(0))
                    .output_bind_rc("EBRG1CLK0", wires::IMUX_ECLKBRIDGECS_CLK0[1].cell(0))
                    .output_bind_rc("EBRG1CLK1", wires::IMUX_ECLKBRIDGECS_CLK1[1].cell(0))
                    .input_xlat_rc("JECLKCIBB0")
                    .input_xlat_rc("JECLKCIBB1")
                    .input_xlat_rc("JECLKCIBT0")
                    .input_xlat_rc("JECLKCIBT1")
                    .input_xlat_rc("ECLKCIBB0")
                    .input_xlat_rc("ECLKCIBB1")
                    .input_xlat_rc("ECLKCIBT0")
                    .input_xlat_rc("ECLKCIBT1");
                if has_2ebr {
                    ex.input_xlat_rc("JPCLKCIBULQ0")
                        .input_xlat_rc("JPCLKCIBULQ1")
                        .input_xlat_rc("JPCLKCIBURQ0")
                        .input_xlat_rc("JPCLKCIBURQ1")
                        .input_xlat_rc("JPCLKCIBMID0")
                        .input_xlat_rc("JPCLKCIBMID1")
                        .input_xlat_rc("PCLKCIBULQ0")
                        .input_xlat_rc("PCLKCIBULQ1")
                        .input_xlat_rc("PCLKCIBURQ0")
                        .input_xlat_rc("PCLKCIBURQ1")
                        .input_xlat_rc("PCLKCIBMID0")
                        .input_xlat_rc("PCLKCIBMID1");
                }
                if ex.ctx.chip.try_bel_pll(DirHV::NW, 0).is_some() {
                    ex.input_xlat_rc("JLPLLCLK0")
                        .input_xlat_rc("JLPLLCLK1")
                        .input_xlat_rc("JLPLLCLK2")
                        .input_xlat_rc("JLPLLCLK3");
                }
                if ex.ctx.chip.try_bel_pll(DirHV::NE, 0).is_some() {
                    ex.input_xlat_rc("JRPLLCLK0")
                        .input_xlat_rc("JRPLLCLK1")
                        .input_xlat_rc("JRPLLCLK2")
                        .input_xlat_rc("JRPLLCLK3");
                }
            }
        }

        for i in 0..2 {
            let ii = i + 6;
            self.extract_bel_cell(bcrd.bel(bslots::DCM[i]), cell)
                .name(format!("DCM{ii}"))
                .input_xlat_rc(bcls::DCM::SEL, format!("JSEL{ii}_DCM"))
                .input_xlat_rc(bcls::DCM::CLK[0], format!("CLK0_{ii}_DCM"))
                .input_xlat_rc(bcls::DCM::CLK[1], format!("CLK1_{ii}_DCM"))
                .output_bind_rc(
                    bcls::DCM::DCMOUT,
                    format!("DCMOUT{ii}_DCM"),
                    wires::OUT_DCM_DCMOUT[i].cell(0),
                )
                .claim_pip("DCMOUT", "CLK0")
                .claim_pip("DCMOUT", "CLK1");
        }

        for i in 0..8 {
            self.extract_bel_cell(bcrd.bel(bslots::DCC[i]), cell)
                .name(format!("DCC{i}"))
                .input_xlat_rc(bcls::DCC::CLKI, format!("CLKI{i}_DCC"))
                .input_xlat_rc(bcls::DCC::CE, format!("JCE{i}_DCC"))
                .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{i}_DCC"))
                .claim_pip("CLKO", "CLKI");
        }

        self.extract_bel_cell(bcrd.bel(bslots::CLKTEST), cell)
            .name("CENTEST")
            .suffix("CENTEST");

        if !is_smol {
            for i in 0..2 {
                let bcrd = bcrd.bel(bslots::ECLKBRIDGECS[i]);
                self.extract_bel_cell(bcrd, cell)
                    .name(format!("ECLKBRIDGECS{i}"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::SEL, format!("JSEL{i}_ECLKBRIDGECS"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::CLK[0], format!("CLK0_{i}_ECLKBRIDGECS"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::CLK[1], format!("CLK1_{i}_ECLKBRIDGECS"))
                    .output_xlat_rc(
                        bcls::ECLKBRIDGECS::ECSOUT,
                        format!("JECSOUT{i}_ECLKBRIDGECS"),
                    );
            }
        }
    }
}
