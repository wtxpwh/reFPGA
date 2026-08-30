use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bcls, bslots, tcls},
};
use prjcombine_interconnect::db::Bel;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_dsp_ecp(&mut self) {
        let tcid = tcls::DSP;
        let is_ecp2 = self.chip.kind != ChipKind::Ecp;
        for &tcrd in &self.edev.tile_index[tcid] {
            let bcrd = tcrd.bel(bslots::DSP[0]);
            let (r, c) = self.rc(tcrd.cell);
            self.name_bel(
                bcrd,
                [
                    format!("MULT9_R{r}C{c}"),
                    format!("MULT18_R{r}C{c}", c = c + 1),
                    format!("MULT9_R{r}C{c}", c = c + 2),
                    format!("MAC52_R{r}C{c}", c = c + 3),
                    format!("MULT9_R{r}C{c}", c = c + 4),
                    format!("MULT18_R{r}C{c}", c = c + 5),
                    format!("MULT9_R{r}C{c}", c = c + 6),
                    format!("MULT36_R{r}C{c}", c = c + 7),
                ],
            );
            let cell_mult9 = [
                tcrd.cell,
                tcrd.cell.delta(2, 0),
                tcrd.cell.delta(4, 0),
                tcrd.cell.delta(6, 0),
            ];
            let cell_mult18 = [tcrd.cell.delta(1, 0), tcrd.cell.delta(5, 0)];
            let cell_mac52 = tcrd.cell.delta(3, 0);
            let cell_mult36 = tcrd.cell.delta(7, 0);
            let mut bel = Bel::default();

            for pins in [bcls::DSP::CLK, bcls::DSP::RST, bcls::DSP::CE] {
                for (i, pin) in pins.into_iter().enumerate() {
                    let (pname_root, _) = self.intdb[bcls::DSP].inputs.key(pin);
                    let pname = format!("{pname_root}{i}");
                    let wire = self.rc_wire(cell_mac52, &format!("J{pname}_MAC52"));
                    self.add_bel_wire(bcrd, format!("{pname}_MAC52"), wire);
                    let bpin = self.xlat_input(bcrd, wire);

                    let wire = self.rc_wire(cell_mult36, &format!("J{pname}_MULT36"));
                    self.add_bel_wire(bcrd, format!("{pname}_MULT36"), wire);
                    assert_eq!(bpin, self.xlat_input(bcrd, wire));

                    for (j, cell) in cell_mult9.into_iter().enumerate() {
                        let wire = self.rc_wire(cell, &format!("J{pname}_MULT9"));
                        self.add_bel_wire(bcrd, format!("{pname}_MULT9_{j}"), wire);
                        assert_eq!(bpin, self.xlat_input(bcrd, wire));
                    }

                    for (j, cell) in cell_mult18.into_iter().enumerate() {
                        let wire = self.rc_wire(cell, &format!("J{pname}_MULT18"));
                        self.add_bel_wire(bcrd, format!("{pname}_MULT18_{j}"), wire);
                        assert_eq!(bpin, self.xlat_input(bcrd, wire));
                    }

                    bel.inputs.insert(pin, bpin);
                }
            }

            for (ab, pins) in [
                (
                    'A',
                    [bcls::DSP::A0, bcls::DSP::A1, bcls::DSP::A2, bcls::DSP::A3],
                ),
                (
                    'B',
                    [bcls::DSP::B0, bcls::DSP::B1, bcls::DSP::B2, bcls::DSP::B3],
                ),
            ] {
                for (i, pins) in pins.into_iter().enumerate() {
                    for j in 0..18 {
                        let pin = pins[j];
                        let wire = self.rc_wire(cell_mult36, &format!("JMU{ab}{i}{j}_MULT36"));
                        self.add_bel_wire(bcrd, format!("MU{ab}{i}{j}_MULT36"), wire);
                        let bpin = self.xlat_input(bcrd, wire);

                        if i < 2 || j < 9 {
                            let wire = self.rc_wire(cell_mac52, &format!("JMU{ab}{i}{j}_MAC52"));
                            self.add_bel_wire(bcrd, format!("MU{ab}{i}{j}_MAC52"), wire);
                            assert_eq!(bpin, self.xlat_input(bcrd, wire));
                        }

                        if j < 9 {
                            let wire = self.rc_wire(cell_mult9[i], &format!("JMU{ab}0{j}_MULT9"));
                            self.add_bel_wire(bcrd, format!("MU{ab}{i}{j}_MULT9"), wire);
                            assert_eq!(bpin, self.xlat_input(bcrd, wire));
                        }

                        if i == 0 || i == 2 || j < 9 {
                            let wire = self.rc_wire(
                                cell_mult18[i / 2],
                                &format!("JMU{ab}{ii}{j}_MULT18", ii = i % 2),
                            );
                            self.add_bel_wire(bcrd, format!("MU{ab}{i}{j}_MULT18"), wire);
                            assert_eq!(bpin, self.xlat_input(bcrd, wire));
                        }

                        bel.inputs.insert(pin, bpin);
                    }
                }
            }

            for (i, pins) in [bcls::DSP::P0, bcls::DSP::P1, bcls::DSP::P2, bcls::DSP::P3]
                .into_iter()
                .enumerate()
            {
                for j in 0..36 {
                    let pin = pins[j];
                    let wire = self.rc_wire(cell_mult36, &format!("JMUP{i}{j}_MULT36"));
                    self.add_bel_wire(bcrd, format!("MUP{i}{j}_MULT36"), wire);
                    let bpin = self.xlat_output(bcrd, wire);

                    if i < 2 {
                        let wire = self.rc_wire(cell_mac52, &format!("JMUP{i}{j}_MAC52"));
                        self.add_bel_wire(bcrd, format!("MUP{i}{j}_MAC52"), wire);
                        assert_eq!(bpin, self.xlat_output(bcrd, wire));
                    }

                    if j < 18 {
                        let wire = self.rc_wire(cell_mult9[i], &format!("JMUP0{j}_MULT9"));
                        self.add_bel_wire(bcrd, format!("MUP{i}{j}_MULT9"), wire);
                        assert_eq!(bpin, self.xlat_output(bcrd, wire));
                    }

                    if i == 0 || i == 2 {
                        let wire = self.rc_wire(cell_mult18[i / 2], &format!("JMUP0{j}_MULT18"));
                        self.add_bel_wire(bcrd, format!("MUP{i}{j}_MULT18"), wire);
                        assert_eq!(bpin, self.xlat_output(bcrd, wire));
                    }

                    bel.outputs.insert(pin, bpin);
                }
            }

            for (cell, kind, name, pkind) in [
                (cell_mult18[0], "MULT18", "MULT18_0", "MULT36"),
                (cell_mac52, "MAC52", "MAC52", "MULT18"),
                (cell_mult18[1], "MULT18", "MULT18_1", "MAC52"),
                (cell_mult36, "MULT36", "MULT36", "MULT18"),
            ] {
                for (ab, pins) in [('A', bcls::DSP::SROA), ('B', bcls::DSP::SROB)] {
                    for j in 0..18 {
                        let sri = self.rc_wire(cell, &format!("JSRI{ab}{j}_{kind}"));
                        let sro = self.rc_wire(cell, &format!("JSRO{ab}{j}_{kind}"));
                        self.add_bel_wire(bcrd, format!("JSRI{ab}{j}_{name}"), sri);
                        self.add_bel_wire(bcrd, format!("JSRO{ab}{j}_{name}"), sro);
                        self.claim_pip(sro, sri);
                        if cell.col != self.chip.col_w() + 2 {
                            let cell_src = if is_ecp2 && cell == cell_mult18[0] {
                                cell.delta(-3, 0)
                            } else {
                                cell.delta(-2, 0)
                            };
                            let psro = self.rc_wire(cell_src, &format!("JSRO{ab}{j}_{pkind}"));
                            self.claim_pip(sri, psro);
                        }
                        if kind == "MULT36" {
                            let bpin = self.xlat_output(bcrd, sro);
                            if is_ecp2 {
                                let pi = if ab == 'A' { j } else { 18 + j };
                                assert_eq!(bel.outputs[bcls::DSP::P3[pi]], bpin);
                            } else {
                                bel.outputs.insert(pins[j], bpin);
                            }
                        }
                    }
                }
            }

            for i in 0..4 {
                let cell = cell_mult9[i];
                for (ab, pins) in [('A', bcls::DSP::SROA), ('B', bcls::DSP::SROB)] {
                    for j in 0..9 {
                        let sri = self.rc_wire(cell, &format!("JSRI{ab}{j}_MULT9"));
                        let sro = self.rc_wire(cell, &format!("JSRO{ab}{j}_MULT9"));
                        self.add_bel_wire(bcrd, format!("JSRI{ab}{j}_MULT9_{i}"), sri);
                        self.add_bel_wire(bcrd, format!("JSRO{ab}{j}_MULT9_{i}"), sro);
                        self.claim_pip(sro, sri);
                        if cell.col != self.chip.col_w() + 1 {
                            let cell_src = if is_ecp2 && i == 0 {
                                cell.delta(-3, 0)
                            } else {
                                cell.delta(-2, 0)
                            };
                            let psro = self.rc_wire(cell_src, &format!("JSRO{ab}{j}_MULT9"));
                            self.claim_pip(sri, psro);
                        }
                        if i == 3 {
                            let bpin = self.xlat_output(bcrd, sro);
                            let pin = if is_ecp2 {
                                bcls::DSP::P3[if ab == 'A' { j } else { 18 + j }]
                            } else {
                                pins[j]
                            };
                            assert_eq!(bel.outputs[pin], bpin);
                        }
                    }
                }
            }

            let pinsets = if is_ecp2 {
                [
                    bcls::DSP::SIGNEDA,
                    bcls::DSP::SIGNEDB,
                    bcls::DSP::SOURCEA,
                    bcls::DSP::SOURCEB,
                ]
                .as_slice()
            } else {
                [bcls::DSP::SIGNEDAB].as_slice()
            };
            for pins in pinsets {
                for i in 0..4 {
                    let pin = pins[i];
                    let (pname, _) = self.intdb[bcls::DSP].inputs.key(pin);
                    let wire = self.rc_wire(cell_mult36, &format!("J{pname}{i}_MULT36"));
                    self.add_bel_wire(bcrd, format!("{pname}{i}_MULT36"), wire);
                    let bpin = self.xlat_input(bcrd, wire);

                    let wire = self.rc_wire(cell_mac52, &format!("J{pname}{i}_MAC52"));
                    self.add_bel_wire(bcrd, format!("{pname}{i}_MAC52"), wire);
                    assert_eq!(bpin, self.xlat_input(bcrd, wire));

                    let wire = self.rc_wire(cell_mult9[i], &format!("J{pname}0_MULT9"));
                    self.add_bel_wire(bcrd, format!("{pname}{i}_MULT9"), wire);
                    assert_eq!(bpin, self.xlat_input(bcrd, wire));

                    let wire = self.rc_wire(
                        cell_mult18[i / 2],
                        &format!("J{pname}{ii}_MULT18", ii = i % 2),
                    );
                    self.add_bel_wire(bcrd, format!("{pname}{i}_MULT18"), wire);
                    assert_eq!(bpin, self.xlat_input(bcrd, wire));

                    bel.inputs.insert(pin, bpin);
                }
            }

            let wire = self.rc_wire(cell_mac52, "JACCUMSLOAD1_MAC52");
            self.add_bel_wire(bcrd, "ACCUMSLOAD1_MAC52", wire);
            let bpin = self.xlat_input(bcrd, wire);
            bel.inputs.insert(bcls::DSP::ACCUMSLOAD1, bpin);

            let wire = self.rc_wire(cell_mult36, "JACCUMSLOAD3_MULT36");
            self.add_bel_wire(bcrd, "ACCUMSLOAD3_MULT36", wire);
            let bpin = self.xlat_input(bcrd, wire);
            bel.inputs.insert(bcls::DSP::ACCUMSLOAD3, bpin);

            for (i, pin) in [(1, bcls::DSP::ADDNSUB1), (3, bcls::DSP::ADDNSUB3)] {
                let wire = self.rc_wire(cell_mult36, &format!("JADDNSUB{i}_MULT36"));
                self.add_bel_wire(bcrd, format!("ADDNSUB{i}_MULT36"), wire);
                let bpin = self.xlat_input(bcrd, wire);

                let wire = self.rc_wire(cell_mac52, &format!("JADDNSUB{i}_MAC52"));
                self.add_bel_wire(bcrd, format!("ADDNSUB{i}_MAC52"), wire);
                assert_eq!(bpin, self.xlat_input(bcrd, wire));

                let wire = self.rc_wire(cell_mult18[i / 2], "JADDNSUB1_MULT18");
                self.add_bel_wire(bcrd, format!("ADDNSUB{i}_MULT18"), wire);
                assert_eq!(bpin, self.xlat_input(bcrd, wire));

                bel.inputs.insert(pin, bpin);
            }

            if is_ecp2 {
                for i in 0..16 {
                    let wire = self.rc_wire(cell_mac52, &format!("JLD{i}_MAC52"));
                    self.add_bel_wire(bcrd, format!("LD{i}_MAC52"), wire);
                    let bpin = self.xlat_input(bcrd, wire);
                    bel.inputs.insert(bcls::DSP::LD1[i], bpin);

                    let wire = self.rc_wire(cell_mult36, &format!("JLD{i}_MULT36"));
                    self.add_bel_wire(bcrd, format!("LD{i}_MULT36"), wire);
                    let bpin = self.xlat_input(bcrd, wire);
                    bel.inputs.insert(bcls::DSP::LD3[i], bpin);
                }
            }

            self.insert_bel(bcrd, bel);
        }
    }
}
