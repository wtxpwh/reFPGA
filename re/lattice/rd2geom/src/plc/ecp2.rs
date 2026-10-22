use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bcls, bslots, tcls, wires},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_plc_ecp2(&mut self) {
        let is_ecp3 = matches!(self.chip.kind, ChipKind::Ecp3(_));
        for tcid in [tcls::PLC, tcls::FPLC] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let cell = tcrd.cell;
                let slices = [
                    cell.bel(bslots::SLICE[0]),
                    cell.bel(bslots::SLICE[1]),
                    cell.bel(bslots::SLICE[2]),
                    cell.bel(bslots::SLICE[3]),
                ];
                let (r, c) = self.rc(cell);
                self.name_bel(slices[0], [format!("R{r}C{c}A")]);
                self.name_bel(slices[1], [format!("R{r}C{c}B")]);
                self.name_bel(slices[2], [format!("R{r}C{c}C")]);
                self.name_bel(slices[3], [format!("R{r}C{c}D")]);
                for i in 0..4 {
                    let abcd = ['A', 'B', 'C', 'D'][i];
                    let i0 = i * 2;
                    let i1 = i * 2 + 1;

                    // plain inputs
                    for (pin, wn, wire) in [
                        (bcls::SLICE::A[0], "A0", format!("A{i0}_SLICE")),
                        (bcls::SLICE::A[1], "A1", format!("A{i1}_SLICE")),
                        (bcls::SLICE::B[0], "B0", format!("B{i0}_SLICE")),
                        (bcls::SLICE::B[1], "B1", format!("B{i1}_SLICE")),
                        (bcls::SLICE::C[0], "C0", format!("C{i0}_SLICE")),
                        (bcls::SLICE::C[1], "C1", format!("C{i1}_SLICE")),
                        (bcls::SLICE::D[0], "D0", format!("D{i0}_SLICE")),
                        (bcls::SLICE::D[1], "D1", format!("D{i1}_SLICE")),
                        (bcls::SLICE::M[0], "M0", format!("M{i0}_SLICE")),
                        (bcls::SLICE::M[1], "M1", format!("M{i1}_SLICE")),
                        (bcls::SLICE::CLK, "CLK", format!("CLK{i}_SLICE")),
                        (bcls::SLICE::LSR, "LSR", format!("LSR{i}_SLICE")),
                        (bcls::SLICE::CE, "CE", format!("CE{i}_SLICE")),
                    ] {
                        if matches!(pin, bcls::SLICE::CLK | bcls::SLICE::LSR | bcls::SLICE::CE)
                            && i == 3
                        {
                            continue;
                        }
                        let wire_int = self.edev.get_bel_input(slices[i], pin).wire;
                        let wire_slice = self.rc_wire(cell, &wire);
                        self.add_bel_wire(slices[i], wn, wire_slice);
                        self.claim_pip_int_in(wire_slice, wire_int);
                    }

                    // plain outputs
                    for (pin, wn, wire) in [
                        (bcls::SLICE::F[0], "F0", format!("F{i0}")),
                        (bcls::SLICE::F[1], "F1", format!("F{i1}")),
                        (bcls::SLICE::Q[0], "Q0", format!("Q{i0}")),
                        (bcls::SLICE::Q[1], "Q1", format!("Q{i1}")),
                    ] {
                        if wn.starts_with('Q') && i == 3 {
                            continue;
                        }
                        let wire_int = self.edev.get_bel_output(slices[i], pin)[0];
                        let wire_slice = self.rc_wire(cell, &format!("{wire}_SLICE"));
                        self.add_bel_wire(slices[i], wn, wire_slice);
                        self.claim_pip_int_out(wire_int, wire_slice);
                    }

                    // F5, FX
                    let ofx0_int = self.edev.get_bel_output(slices[i], bcls::SLICE::OFX[0])[0];
                    let ofx1_int = self.edev.get_bel_output(slices[i], bcls::SLICE::OFX[1])[0];
                    let ofx0_slice = self.rc_wire(cell, &format!("F5{abcd}_SLICE"));
                    let ofx1_slice = self.rc_wire(cell, &format!("FX{abcd}_SLICE"));
                    self.add_bel_wire(slices[i], "OFX0", ofx0_slice);
                    self.add_bel_wire(slices[i], "OFX1", ofx1_slice);
                    self.claim_pip_int_out(ofx0_int, ofx0_slice);
                    self.claim_pip_int_out(ofx1_int, ofx1_slice);
                    let out_idx = if is_ecp3 { 1 } else { 2 };
                    if i == out_idx && self.edev.has_bel(cell.delta(-1, 0).bel(bslots::INT)) {
                        let fx_out = self.rc_wire(cell, "HL7W0001");
                        self.claim_pip_int_in(fx_out, ofx1_int);
                    }

                    // FXA, FXB
                    let fxa = self.rc_wire(cell, &format!("FXA{abcd}"));
                    let fxb = self.rc_wire(cell, &format!("FXB{abcd}"));
                    let fxa_slice = self.rc_wire(cell, &format!("FXA{abcd}_SLICE"));
                    let fxb_slice = self.rc_wire(cell, &format!("FXB{abcd}_SLICE"));
                    self.add_bel_wire(slices[i], "FXA", fxa);
                    self.add_bel_wire(slices[i], "FXB", fxb);
                    self.add_bel_wire(slices[i], "FXA_SLICE", fxa_slice);
                    self.add_bel_wire(slices[i], "FXB_SLICE", fxb_slice);
                    self.claim_pip(fxa_slice, fxa);
                    self.claim_pip(fxb_slice, fxb);

                    if is_ecp3 {
                        let (ia, ib) = [(2, 0), (5, 1), (6, 4), (3, 3)][i];
                        if i == 3 {
                            let cell_src = cell.delta(1, 0);
                            if self.edev.has_bel(cell_src.bel(bslots::INT)) {
                                let fx_out = self.rc_wire(cell_src, "HL7W0001");
                                self.claim_pip(fxa, fx_out);
                            }
                        } else {
                            let fxa_int = cell.wire(wires::OUT_OFX[ia]);
                            self.claim_pip_int_in(fxa, fxa_int);
                        }
                        let fxb_int = cell.wire(wires::OUT_OFX[ib]);
                        self.claim_pip_int_in(fxb, fxb_int);
                    } else {
                        let (ia, ib) = [(5, 5), (2, 0), (7, 3), (6, 4)][i];
                        if i == 0 {
                            let cell_src = cell.delta(1, 0);
                            if self.edev.has_bel(cell_src.bel(bslots::INT)) {
                                let fx_out = self.rc_wire(cell_src, "HL7W0001");
                                self.claim_pip(fxa, fx_out);
                            }
                        } else {
                            let fxa_int = cell.wire(wires::OUT_OFX[ia]);
                            self.claim_pip_int_in(fxa, fxa_int);
                        }
                        let fxb_int = cell.wire(wires::OUT_OFX[ib]);
                        self.claim_pip_int_in(fxb, fxb_int);
                    }

                    // DI
                    let di0 = self.rc_wire(cell, &format!("DI{ii}", ii = 2 * i));
                    let di0_slice = self.rc_wire(cell, &format!("DI{ii}_SLICE", ii = 2 * i));
                    let di1 = self.rc_wire(cell, &format!("DI{ii}", ii = 2 * i + 1));
                    let di1_slice = self.rc_wire(cell, &format!("DI{ii}_SLICE", ii = 2 * i + 1));
                    self.add_bel_wire(slices[i], "DI0", di0);
                    self.add_bel_wire(slices[i], "DI1", di1);
                    self.add_bel_wire(slices[i], "DI0_SLICE", di0_slice);
                    self.add_bel_wire(slices[i], "DI1_SLICE", di1_slice);
                    self.claim_pip(di0_slice, di0);
                    self.claim_pip(di1_slice, di1);
                    if i != 3 {
                        let f0 = self.edev.get_bel_output(slices[i], bcls::SLICE::F[0])[0];
                        let f1 = self.edev.get_bel_output(slices[i], bcls::SLICE::F[1])[0];
                        self.claim_pip_int_in(di0, f0);
                        self.claim_pip_int_in(di0, ofx0_int);
                        self.claim_pip_int_in(di1, f1);
                        self.claim_pip_int_in(di1, ofx1_int);
                    }

                    if tcid == tcls::PLC {
                        for pin in [
                            "WRE", "WREO", "WCK", "WCKO", "WAD0", "WAD1", "WAD2", "WAD3", "WADO0",
                            "WADO1", "WADO2", "WADO3", "WD0", "WD1", "WDO0", "WDO1", "WDO2",
                            "WDO3",
                        ] {
                            let wire = self.rc_wire(cell, &format!("{pin}{abcd}_SLICE"));
                            self.add_bel_wire(slices[i], pin, wire);
                        }
                    }
                }

                for (i, pin, wire) in [
                    (0, "FCI", "FCI"),
                    (0, "FCI_SLICE", "FCI_SLICE"),
                    (1, "FCI_SLICE", "FCIB_SLICE"),
                    (2, "FCI_SLICE", "FCIC_SLICE"),
                    (0, "FCO_SLICE", "FCOA_SLICE"),
                    (1, "FCO_SLICE", "FCOB_SLICE"),
                    (2, "FCO_SLICE", "FCO_SLICE"),
                    (2, "FCO", "FCO"),
                    // dummy stuff
                    (3, "FCI_SLICE", "FCID_SLICE"),
                    (3, "FCO_SLICE", "FCOD_SLICE"),
                    (3, "CLK", "CLK3_SLICE"),
                    (3, "LSR", "LSR3_SLICE"),
                    (3, "CE", "CE3_SLICE"),
                    (3, "Q0", "Q6_SLICE"),
                    (3, "Q1", "Q7_SLICE"),
                ] {
                    self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                }
                for (wt, wf) in [
                    ("FCI_SLICE", "FCI"),
                    ("FCIB_SLICE", "FCOA_SLICE"),
                    ("FCIC_SLICE", "FCOB_SLICE"),
                    ("FCO", "FCO_SLICE"),
                ] {
                    let wt = self.rc_wire(cell, wt);
                    let wf = self.rc_wire(cell, wf);
                    self.claim_pip(wt, wf);
                }
                let fci_in = self.naming.strings.get("FCI_IN").unwrap();
                if let Some(naming) = self.naming.bels.get(&cell.bel(bslots::INT))
                    && let Some(&wf) = naming.wires.get(&fci_in)
                {
                    let wt = self.rc_wire(cell, "FCI");
                    self.claim_pip(wt, wf);
                }
                if let Some(naming) = self.naming.bels.get(&cell.delta(1, 0).bel(bslots::INT))
                    && let Some(&wt) = naming.wires.get(&fci_in)
                {
                    let wf = self.rc_wire(cell, "FCO");
                    self.claim_pip(wt, wf);
                }

                if tcid == tcls::PLC {
                    for (i, pin, wire) in [
                        (1, "WCK_OUT", "WCK"),
                        (1, "WRE_OUT", "WRE"),
                        (1, "WD0_OUT", "WD0"),
                        (1, "WD1_OUT", "WD1"),
                        (1, "WD2_OUT", "WD2"),
                        (1, "WD3_OUT", "WD3"),
                        (1, "WAD0_OUT", "WAD0"),
                        (1, "WAD1_OUT", "WAD1"),
                        (1, "WAD2_OUT", "WAD2"),
                        (1, "WAD3_OUT", "WAD3"),
                    ] {
                        self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                    }
                    if is_ecp3 {
                        for (wt, wf) in [
                            ("WCK", "WCKOC_SLICE"),
                            ("WCKA_SLICE", "WCK"),
                            ("WCKB_SLICE", "WCK"),
                            ("WRE", "WREOC_SLICE"),
                            ("WREA_SLICE", "WRE"),
                            ("WREB_SLICE", "WRE"),
                            ("WD0", "WDO0C_SLICE"),
                            ("WD1", "WDO1C_SLICE"),
                            ("WD2", "WDO2C_SLICE"),
                            ("WD3", "WDO3C_SLICE"),
                            ("WD0A_SLICE", "WD0"),
                            ("WD1A_SLICE", "WD1"),
                            ("WD0B_SLICE", "WD2"),
                            ("WD1B_SLICE", "WD3"),
                            ("WAD0", "WADO0C_SLICE"),
                            ("WAD1", "WADO1C_SLICE"),
                            ("WAD2", "WADO2C_SLICE"),
                            ("WAD3", "WADO3C_SLICE"),
                            ("WAD0A_SLICE", "WAD0"),
                            ("WAD1A_SLICE", "WAD1"),
                            ("WAD2A_SLICE", "WAD2"),
                            ("WAD3A_SLICE", "WAD3"),
                            ("WAD0B_SLICE", "WAD0"),
                            ("WAD1B_SLICE", "WAD1"),
                            ("WAD2B_SLICE", "WAD2"),
                            ("WAD3B_SLICE", "WAD3"),
                        ] {
                            let wt = self.rc_wire(cell, wt);
                            let wf = self.rc_wire(cell, wf);
                            self.claim_pip(wt, wf);
                        }
                    } else {
                        for (wt, wf) in [
                            ("WCK", "WCKOB_SLICE"),
                            ("WCKA_SLICE", "WCK"),
                            ("WCKC_SLICE", "WCK"),
                            ("WRE", "WREOB_SLICE"),
                            ("WREA_SLICE", "WRE"),
                            ("WREC_SLICE", "WRE"),
                            ("WD0", "WDO0B_SLICE"),
                            ("WD1", "WDO1B_SLICE"),
                            ("WD2", "WDO2B_SLICE"),
                            ("WD3", "WDO3B_SLICE"),
                            ("WD0A_SLICE", "WD0"),
                            ("WD1A_SLICE", "WD1"),
                            ("WD0C_SLICE", "WD2"),
                            ("WD1C_SLICE", "WD3"),
                            ("WAD0", "WADO0B_SLICE"),
                            ("WAD1", "WADO1B_SLICE"),
                            ("WAD2", "WADO2B_SLICE"),
                            ("WAD3", "WADO3B_SLICE"),
                            ("WAD0A_SLICE", "WAD0"),
                            ("WAD1A_SLICE", "WAD1"),
                            ("WAD2A_SLICE", "WAD2"),
                            ("WAD3A_SLICE", "WAD3"),
                            ("WAD0C_SLICE", "WAD0"),
                            ("WAD1C_SLICE", "WAD1"),
                            ("WAD2C_SLICE", "WAD2"),
                            ("WAD3C_SLICE", "WAD3"),
                        ] {
                            let wt = self.rc_wire(cell, wt);
                            let wf = self.rc_wire(cell, wf);
                            self.claim_pip(wt, wf);
                        }
                    }
                }
            }
        }
    }
}
