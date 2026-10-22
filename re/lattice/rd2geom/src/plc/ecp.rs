use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bcls, bslots, tcls, wires},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_plc_ecp(&mut self) {
        for tcid in [tcls::PLC, tcls::FPLC] {
            if tcid == tcls::FPLC && self.chip.kind == ChipKind::Scm {
                continue;
            }
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
                        let wire_int = self.edev.get_bel_input(slices[i], pin);
                        let wire_slice = self.rc_wire(cell, &wire);
                        self.add_bel_wire(slices[i], wn, wire_slice);
                        self.claim_pip_int_in(wire_slice, wire_int.wire);
                    }

                    let io_cell = if self.chip.kind == ChipKind::MachXo && i < 3 {
                        if cell.row == self.chip.row_s() + 1 {
                            Some(cell.delta(0, -1))
                        } else if cell.row == self.chip.row_n() - 1 {
                            Some(cell.delta(0, 1))
                        } else if cell.col == self.chip.col_w() + 1 {
                            Some(cell.delta(-1, 0))
                        } else if cell.col == self.chip.col_e() - 1 {
                            Some(cell.delta(1, 0))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // plain outputs
                    for (pin, wn, wire) in [
                        (bcls::SLICE::F[0], "F0", format!("F{i0}")),
                        (bcls::SLICE::F[1], "F1", format!("F{i1}")),
                        (bcls::SLICE::Q[0], "Q0", format!("Q{i0}")),
                        (bcls::SLICE::Q[1], "Q1", format!("Q{i1}")),
                    ] {
                        let wire_int = self.edev.get_bel_output(slices[i], pin)[0];
                        let wire_slice = self.rc_wire(
                            cell,
                            &if self.chip.kind == ChipKind::MachXo {
                                format!("J{wire}_SLICE")
                            } else {
                                format!("{wire}_SLICE")
                            },
                        );
                        self.add_bel_wire(slices[i], wn, wire_slice);
                        self.claim_pip_int_out(wire_int, wire_slice);

                        if let Some(io_cell) = io_cell {
                            let wire_io = self.rc_wire(io_cell, &format!("JPLC{wire}"));
                            self.add_bel_wire(slices[i], format!("{wn}_IO"), wire_io);
                            self.claim_pip(wire_io, wire_slice);
                        }
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

                    let (ia, ib) = [(0, 2), (1, 5), (4, 6), (3, 3)][i];
                    let fxa_int = cell.wire(wires::OUT_OFX[ia]);
                    self.claim_pip_int_in(fxa, fxa_int);
                    if i == 3 {
                        if let Some(&fxb_int) = self
                            .naming
                            .interconnect
                            .get(&cell.wire(wires::OUT_OFX_W[3]))
                        {
                            self.claim_pip(fxb, fxb_int);
                        }
                    } else {
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
                    let f0 = self.edev.get_bel_output(slices[i], bcls::SLICE::F[0])[0];
                    let f1 = self.edev.get_bel_output(slices[i], bcls::SLICE::F[1])[0];
                    self.claim_pip(di0_slice, di0);
                    self.claim_pip(di1_slice, di1);
                    self.claim_pip_int_in(di0, f0);
                    self.claim_pip_int_in(di0, ofx0_int);
                    self.claim_pip_int_in(di1, f1);
                    self.claim_pip_int_in(di1, ofx1_int);
                }

                for (i, pin, wire) in [
                    (0, "FCI", "FCI"),
                    (0, "FCI_SLICE", "FCI_SLICE"),
                    (1, "FCI_SLICE", "FCIB_SLICE"),
                    (2, "FCI_SLICE", "FCIC_SLICE"),
                    (3, "FCI_SLICE", "FCID_SLICE"),
                    (0, "FCO_SLICE", "FCOA_SLICE"),
                    (1, "FCO_SLICE", "FCOB_SLICE"),
                    (2, "FCO_SLICE", "FCOC_SLICE"),
                    (3, "FCO_SLICE", "FCO_SLICE"),
                    (3, "FCO", "FCO"),
                ] {
                    self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                }
                for (wt, wf) in [
                    ("FCI_SLICE", "FCI"),
                    ("FCIB_SLICE", "FCOA_SLICE"),
                    ("FCIC_SLICE", "FCOB_SLICE"),
                    ("FCID_SLICE", "FCOC_SLICE"),
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

                if self.chip.kind == ChipKind::Scm {
                    for (i, pin, wire) in
                        [(0, "FCO", "FCOA"), (1, "FCO", "FCOB"), (2, "FCO", "FCOC")]
                    {
                        self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                    }
                    for (wt, wf) in [
                        ("FCOA", "FCOA_SLICE"),
                        ("FCOB", "FCOB_SLICE"),
                        ("FCOC", "FCOC_SLICE"),
                    ] {
                        let wt = self.rc_wire(cell, wt);
                        let wf = self.rc_wire(cell, wf);
                        self.claim_pip(wt, wf);
                    }
                    for (i, wf) in ["FCOA", "FCOB", "FCOC", "FCO"].into_iter().enumerate() {
                        let ofx1_int = self.edev.get_bel_output(slices[i], bcls::SLICE::OFX[1])[0];
                        let wf = self.rc_wire(cell, wf);
                        self.claim_pip_int_out(ofx1_int, wf);
                    }
                }

                if tcid == tcls::PLC {
                    for (i, wt) in [
                        (0, "CLK20_SLICE"),
                        (1, "CLK21_SLICE"),
                        (2, "CLK22_SLICE"),
                        (3, "CLK23_SLICE"),
                    ] {
                        let wt = self.rc_wire(cell, wt);
                        self.add_bel_wire(slices[i], "CLK2", wt);
                        let wf = self
                            .edev
                            .get_bel_input(slices[i ^ 1], bcls::SLICE::CLK)
                            .wire;
                        self.claim_pip_int_in(wt, wf);
                    }
                    if self.chip.kind == ChipKind::Scm {
                        for (i, pin, wire) in [
                            (0, "DP64_SLICE", "DPI64A_SLICE"),
                            (1, "DP64_SLICE", "DPI64B_SLICE"),
                            (2, "DP64_SLICE", "DPI64C_SLICE"),
                            (3, "DP64_SLICE", "DPI64D_SLICE"),
                            (0, "DP_SLICE", "DPOA_SLICE"),
                            (1, "DP_SLICE", "DPOB_SLICE"),
                            (2, "DP_SLICE", "DPOC_SLICE"),
                            (3, "DP_SLICE", "DPOD_SLICE"),
                            (0, "DP", "DPI01"),
                            (2, "DP", "DPO23"),
                        ] {
                            self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                        }
                        for (wt, wf) in [
                            ("DPI64B_SLICE", "DPI01"),
                            ("DPI64D_SLICE", "DPO23"),
                            ("DPI01", "DPOA_SLICE"),
                            ("DPO23", "DPOC_SLICE"),
                        ] {
                            let wt = self.rc_wire(cell, wt);
                            let wf = self.rc_wire(cell, wf);
                            self.claim_pip(wt, wf);
                        }
                    } else {
                        for (i, pin, wire) in [
                            (0, "DP64_SLICE", "DP64A_SLICE"),
                            (1, "DP64_SLICE", "DP64B_SLICE"),
                            (2, "DP64_SLICE", "DP64C_SLICE"),
                            (3, "DP64_SLICE", "DP64D_SLICE"),
                            (0, "DP_SLICE", "DPA_SLICE"),
                            (1, "DP_SLICE", "DPB_SLICE"),
                            (2, "DP_SLICE", "DPC_SLICE"),
                            (3, "DP_SLICE", "DPD_SLICE"),
                            (0, "DP", "DP01"),
                            (2, "DP", "DP23"),
                        ] {
                            self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
                        }
                        for (wt, wf) in [
                            ("DP64B_SLICE", "DP01"),
                            ("DP64D_SLICE", "DP23"),
                            ("DP01", "DPA_SLICE"),
                            ("DP23", "DPC_SLICE"),
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
