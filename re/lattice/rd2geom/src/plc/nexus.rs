use prjcombine_ecp::defs::{bcls, bslots, tcls};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_plc_nexus(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PLC] {
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
                    (bcls::SLICE::A[0], "A0", format!("JA0_SLICE{abcd}")),
                    (bcls::SLICE::A[1], "A1", format!("JA1_SLICE{abcd}")),
                    (bcls::SLICE::B[0], "B0", format!("JB0_SLICE{abcd}")),
                    (bcls::SLICE::B[1], "B1", format!("JB1_SLICE{abcd}")),
                    (bcls::SLICE::C[0], "CIN0_CDMUX", format!("JCIN{i0}_CDMUX")),
                    (bcls::SLICE::C[1], "CIN1_CDMUX", format!("JCIN{i1}_CDMUX")),
                    (bcls::SLICE::C[0], "C0_DRMUX", format!("JC{i0}_DRMUX")),
                    (bcls::SLICE::C[1], "C1_DRMUX", format!("JC{i1}_DRMUX")),
                    (bcls::SLICE::D[0], "D0_DRMUX", format!("JD{i0}_DRMUX")),
                    (bcls::SLICE::D[1], "D1_DRMUX", format!("JD{i1}_DRMUX")),
                    (bcls::SLICE::D[0], "D0_DIMUX", format!("JD{i0}_DIMUX")),
                    (bcls::SLICE::D[1], "D1_DIMUX", format!("JD{i1}_DIMUX")),
                    (bcls::SLICE::M[0], "M0_DIMUX", format!("JM{i0}_DIMUX")),
                    (bcls::SLICE::M[1], "M1_DIMUX", format!("JM{i1}_DIMUX")),
                    (bcls::SLICE::M[0], "SEL", format!("JSEL_SLICE{abcd}")),
                    (bcls::SLICE::CLK, "CLK", format!("JCLK_SLICE{abcd}")),
                    (bcls::SLICE::LSR, "LSR", format!("JLSR_SLICE{abcd}")),
                    (bcls::SLICE::CE, "CE", format!("JCE_SLICE{abcd}")),
                ] {
                    let wire_int = self.edev.get_bel_input(slices[i], pin).wire;
                    let wire_slice = self.rc_wire(cell, &wire);
                    self.add_bel_wire(slices[i], wn, wire_slice);
                    self.claim_pip_int_in(wire_slice, wire_int);
                }
                // plain outputs
                for (pin, wn, wire) in [
                    (bcls::SLICE::F[0], "F0", format!("JF0_SLICE{abcd}")),
                    (bcls::SLICE::F[1], "F1", format!("JF1_SLICE{abcd}")),
                    (bcls::SLICE::F[0], "OFX0", format!("JOFX0_SLICE{abcd}")),
                    (bcls::SLICE::Q[0], "Q0", format!("JQ0_SLICE{abcd}")),
                    (bcls::SLICE::Q[1], "Q1", format!("JQ1_SLICE{abcd}")),
                ] {
                    let wire_int = self.edev.get_bel_output(slices[i], pin)[0];
                    let wire_slice = self.rc_wire(cell, &wire);
                    self.add_bel_wire(slices[i], wn, wire_slice);
                    self.claim_pip_int_out(wire_int, wire_slice);
                }

                // muxes
                for (wn, wire, ins) in [
                    (
                        "DI0",
                        format!("JDI0_SLICE{abcd}"),
                        ["F0", "OFX0"].as_slice(),
                    ),
                    ("DI1", format!("JDI1_SLICE{abcd}"), ["F1"].as_slice()),
                    ("F1_DRMUX", format!("JF{i1}_DRMUX"), ["F1"].as_slice()),
                    (
                        "COUT0_CDMUX",
                        format!("JCOUT{i0}_CDMUX"),
                        ["CIN0_CDMUX"].as_slice(),
                    ),
                    (
                        "COUT1_CDMUX",
                        format!("JCOUT{i1}_CDMUX"),
                        ["CIN1_CDMUX"].as_slice(),
                    ),
                    (
                        "DL0_DRMUX",
                        format!("JDL{i0}_DRMUX"),
                        ["C1_DRMUX", "D0_DRMUX", "F1_DRMUX"].as_slice(),
                    ),
                    (
                        "DL1_DRMUX",
                        format!("JDL{i1}_DRMUX"),
                        ["C0_DRMUX", "D1_DRMUX"].as_slice(),
                    ),
                    (
                        "DI0_DIMUX",
                        format!("JDI{i0}_DIMUX"),
                        ["D0_DIMUX", "M0_DIMUX"].as_slice(),
                    ),
                    (
                        "DI1_DIMUX",
                        format!("JDI{i1}_DIMUX"),
                        ["D1_DIMUX", "M1_DIMUX"].as_slice(),
                    ),
                    ("C0", format!("JC0_SLICE{abcd}"), ["COUT0_CDMUX"].as_slice()),
                    ("C1", format!("JC1_SLICE{abcd}"), ["COUT1_CDMUX"].as_slice()),
                    ("D0", format!("JD0_SLICE{abcd}"), ["DL0_DRMUX"].as_slice()),
                    ("D1", format!("JD1_SLICE{abcd}"), ["DL1_DRMUX"].as_slice()),
                    ("M0", format!("JM0_SLICE{abcd}"), ["DI0_DIMUX"].as_slice()),
                    ("M1", format!("JM1_SLICE{abcd}"), ["DI1_DIMUX"].as_slice()),
                    ("FCI", format!("JFCI_SLICE{abcd}"), [].as_slice()),
                    ("FCO", format!("JFCO_SLICE{abcd}"), [].as_slice()),
                    ("WCK", format!("JWCK_SLICE{abcd}"), [].as_slice()),
                    ("WRE", format!("JWRE_SLICE{abcd}"), [].as_slice()),
                    ("WCKO", format!("JWCKO_SLICE{abcd}"), [].as_slice()),
                    ("WREO", format!("JWREO_SLICE{abcd}"), [].as_slice()),
                    ("WAD0", format!("JWAD0_SLICE{abcd}"), [].as_slice()),
                    ("WAD1", format!("JWAD1_SLICE{abcd}"), [].as_slice()),
                    ("WAD2", format!("JWAD2_SLICE{abcd}"), [].as_slice()),
                    ("WAD3", format!("JWAD3_SLICE{abcd}"), [].as_slice()),
                    ("WADO0", format!("JWADO0_SLICE{abcd}"), [].as_slice()),
                    ("WADO1", format!("JWADO1_SLICE{abcd}"), [].as_slice()),
                    ("WADO2", format!("JWADO2_SLICE{abcd}"), [].as_slice()),
                    ("WADO3", format!("JWADO3_SLICE{abcd}"), [].as_slice()),
                    ("WDI0", format!("JWDI0_SLICE{abcd}"), [].as_slice()),
                    ("WDI1", format!("JWDI1_SLICE{abcd}"), [].as_slice()),
                    ("WDO0", format!("JWDO0_SLICE{abcd}"), [].as_slice()),
                    ("WDO1", format!("JWDO1_SLICE{abcd}"), [].as_slice()),
                    ("WDO2", format!("JWDO2_SLICE{abcd}"), [].as_slice()),
                    ("WDO3", format!("JWDO3_SLICE{abcd}"), [].as_slice()),
                ] {
                    let wire_slice = self.rc_wire(cell, &wire);
                    self.add_bel_wire(slices[i], wn, wire_slice);
                    for &inp in ins {
                        let wire_in = self.naming.bel_wire(slices[i], inp);
                        self.claim_pip(wire_slice, wire_in);
                    }
                }
            }
            for (i, pin, wire) in [(0, "FCIN", "JFCIN"), (3, "FCOUT", "JFCOUT")] {
                self.add_bel_wire(slices[i], pin, self.rc_wire(cell, wire));
            }
            for (wt, wf) in [
                ("JFCI_SLICEA", "JFCIN"),
                ("JFCI_SLICEB", "JFCO_SLICEA"),
                ("JFCI_SLICEC", "JFCO_SLICEB"),
                ("JFCI_SLICED", "JFCO_SLICEC"),
                ("JFCOUT", "JFCO_SLICED"),
                ("JWCK_SLICEA", "JWCKO_SLICEC"),
                ("JWCK_SLICEB", "JWCKO_SLICEC"),
                ("JWRE_SLICEA", "JWREO_SLICEC"),
                ("JWRE_SLICEB", "JWREO_SLICEC"),
                ("JWDI0_SLICEA", "JWDO0_SLICEC"),
                ("JWDI1_SLICEA", "JWDO1_SLICEC"),
                ("JWDI0_SLICEB", "JWDO2_SLICEC"),
                ("JWDI1_SLICEB", "JWDO3_SLICEC"),
                ("JWAD0_SLICEA", "JWADO0_SLICEC"),
                ("JWAD1_SLICEA", "JWADO1_SLICEC"),
                ("JWAD2_SLICEA", "JWADO2_SLICEC"),
                ("JWAD3_SLICEA", "JWADO3_SLICEC"),
                ("JWAD0_SLICEB", "JWADO0_SLICEC"),
                ("JWAD1_SLICEB", "JWADO1_SLICEC"),
                ("JWAD2_SLICEB", "JWADO2_SLICEC"),
                ("JWAD3_SLICEB", "JWADO3_SLICEC"),
            ] {
                let wt = self.rc_wire(cell, wt);
                let wf = self.rc_wire(cell, wf);
                self.claim_pip(wt, wf);
            }
            let bcrd_int = cell.bel(bslots::INT);
            let fcin = self.rc_wire(cell, "JFCIN");
            let hfie0 = self.naming.bel_wire(bcrd_int, "HFIE0");
            self.claim_pip(fcin, hfie0);
            let fcout = self.rc_wire(cell, "JFCOUT");
            let hfoe0 = self.naming.bel_wire(bcrd_int, "HFOE0");
            self.claim_pip(hfoe0, fcout);
        }
    }
}
