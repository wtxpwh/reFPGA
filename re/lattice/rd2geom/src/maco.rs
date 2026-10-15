use std::collections::{BTreeMap, BTreeSet, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind},
    defs::{bslots, ccls, cslots, scm::bcls, tcls, wiredata, wires},
};
use prjcombine_entity::{EntityId, EntityPartVec};
use prjcombine_interconnect::{
    db::{
        Bel, BelInfo, BelInput, ConnectorWire, Mux, PinDir, SwitchBox, SwitchBoxItem, WireSlotIdExt,
    },
    dir::{DirH, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    fn gather_ebr_wires(&mut self) {
        for (row, rd) in &self.chip.rows {
            if rd.kind != RowKind::Ebr {
                continue;
            }
            if row == self.chip.row_n() - 12 {
                continue;
            }
            for cell in self.edev.row(Chip::DIE, row) {
                if cell.col < self.chip.col_w() + 4 {
                    continue;
                }
                if cell.col >= self.chip.col_e() - 3 {
                    continue;
                }
                if cell.col.to_idx() % 2 == 1 {
                    continue;
                }
                for dir in [DirH::W, DirH::E] {
                    let ab = match dir {
                        DirH::W => 'B',
                        DirH::E => 'A',
                    };
                    let (has_0, has_1) = match dir {
                        DirH::W => (
                            cell.col < self.chip.col_e() - 13,
                            cell.col >= self.chip.col_w() + 14,
                        ),
                        DirH::E => (
                            cell.col >= self.chip.col_w() + 14,
                            cell.col < self.chip.col_e() - 13,
                        ),
                    };
                    let skip_mid = match dir {
                        DirH::W => {
                            cell.col >= self.chip.col_clk && cell.col < self.chip.col_clk + 10
                        }
                        DirH::E => {
                            cell.col >= self.chip.col_clk - 10 && cell.col < self.chip.col_clk
                        }
                    };
                    let num = match (dir, cell.col < self.chip.col_clk) {
                        (DirH::W, true) => 48,
                        (DirH::W, false) => 64,
                        (DirH::E, true) => 64,
                        (DirH::E, false) => 48,
                    };
                    if has_1 {
                        for i in 0..num {
                            let iwire_1 = cell.wire(match dir {
                                DirH::W => wires::EBR_W1[i],
                                DirH::E => wires::EBR_E1[i],
                            });
                            let wire_1 =
                                self.rc_wire(cell.delta(1, 1), &format!("HMA{ab}SI{i:02}"));
                            self.ebr_wires.insert(iwire_1, wire_1);
                            if has_0 && !(skip_mid && (24..40).contains(&i)) {
                                let wire_0 = self.pips_fwd[&wire_1]
                                    .iter()
                                    .copied()
                                    .find(|wn| self.naming.strings[wn.suffix].starts_with("HMA"))
                                    .unwrap();
                                let iwire_0 = cell.wire(match dir {
                                    DirH::W => wires::EBR_W0[i],
                                    DirH::E => wires::EBR_E0[i],
                                });
                                self.ebr_wires.insert(iwire_0, wire_0);
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_ebr_conns(&mut self) {
        for (ccrd, conn) in self.edev.connectors() {
            if ccrd.slot == cslots::EBR_W || ccrd.slot == cslots::EBR_E {
                let cell_to = ccrd.cell;
                let cell_from = conn.target.unwrap();
                let ccls = &self.intdb[conn.class];
                for (wt, &wf) in &ccls.wires {
                    let ConnectorWire::Pass(wf) = wf else {
                        unreachable!()
                    };
                    let wt = cell_to.wire(wt);
                    let wf = cell_from.wire(wf);
                    let Some(&wt) = self.ebr_wires.get(&wt) else {
                        println!(
                            "{name}: OOPS {wt}",
                            name = self.name,
                            wt = wt.to_string(self.intdb)
                        );
                        continue;
                    };
                    let Some(&wf) = self.ebr_wires.get(&wf) else {
                        println!(
                            "{name}: OOPS {wf}",
                            name = self.name,
                            wf = wf.to_string(self.intdb)
                        );
                        continue;
                    };
                    self.claim_pip(wt, wf);
                }
                if conn.class == ccls::PASS_EBR_E_W {
                    for i in 48..64 {
                        let wire_1 = self.rc_wire(cell_to.delta(1, 1), &format!("HMAASI{i:02}"));
                        self.claim_node(wire_1);
                        let wire_0 = self.claim_single_in(wire_1);
                        self.claim_node(wire_0);
                        assert!(self.naming.strings[wire_0.suffix].starts_with("HMA"));
                    }
                }
                if conn.class == ccls::PASS_EBR_W_E {
                    for i in 48..64 {
                        let wire_1 = self.rc_wire(cell_to.delta(1, 1), &format!("HMABSI{i:02}"));
                        self.claim_node(wire_1);
                        let wire_0 = self.claim_single_in(wire_1);
                        self.claim_node(wire_0);
                        assert!(self.naming.strings[wire_0.suffix].starts_with("HMA"));
                    }
                }
            }
        }
    }

    pub fn process_maco(&mut self) {
        if self.chip.kind != ChipKind::Scm {
            return;
        }
        self.gather_ebr_wires();
        self.process_ebr_conns();
        let mut maco_rows = EntityPartVec::new();
        let mut idx = 0;
        for (row, rd) in &self.chip.rows {
            if rd.kind == RowKind::Ebr && row >= self.chip.row_clk {
                maco_rows.insert(row, ('U', idx));
                idx += 1;
            }
        }
        let mut idx = 0;
        for (row, rd) in self.chip.rows.iter().rev() {
            if rd.kind == RowKind::Ebr && row < self.chip.row_clk {
                maco_rows.insert(row, ('L', idx));
                idx += 1;
            }
        }
        for (edge, tcid) in [(DirH::W, tcls::MACO_W), (DirH::E, tcls::MACO_E)] {
            let lr = match edge {
                DirH::W => 'L',
                DirH::E => 'R',
            };
            let tcls = &self.intdb[tcid];
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::MACO);
                let bcrd_int = tcrd.bel(bslots::BEL_INT);
                let cell = bcrd.delta(0, 1);
                let (ul, idx) = maco_rows[bcrd.row];
                self.name_bel(bcrd, [format!("{lr}{ul}MACO{idx}")]);
                self.name_bel_null(bcrd_int);
                let mut bel = Bel::default();
                let mut sb = SwitchBox::default();

                for i in 0..3 {
                    for (pname, pins) in [
                        (
                            "CLK",
                            [
                                bcls::MACO::CLK_0.as_slice(),
                                bcls::MACO::CLK_1.as_slice(),
                                bcls::MACO::CLK_2.as_slice(),
                            ],
                        ),
                        (
                            "LSR",
                            [
                                bcls::MACO::LSR_0.as_slice(),
                                bcls::MACO::LSR_1.as_slice(),
                                bcls::MACO::LSR_2.as_slice(),
                            ],
                        ),
                        (
                            "CE",
                            [
                                bcls::MACO::CE_0.as_slice(),
                                bcls::MACO::CE_1.as_slice(),
                                bcls::MACO::CE_2.as_slice(),
                            ],
                        ),
                    ] {
                        let pins = pins[i];
                        for (j, &pin) in pins.iter().enumerate() {
                            let wire = self.rc_io_wire(cell, &format!("J{pname}_{i}_{j}_MACO"));
                            self.add_bel_wire(bcrd, format!("{pname}_{i}_{j}"), wire);
                            bel.inputs.insert(pin, self.xlat_input(bcrd, wire));
                        }
                    }
                    let pins = [
                        bcls::MACO::CIBIN_0,
                        bcls::MACO::CIBIN_1,
                        bcls::MACO::CIBIN_2,
                    ][i];
                    for j in 0..32 {
                        let wire = self.rc_io_wire(cell, &format!("JCIBIN_{i}_{j}_MACO"));
                        self.add_bel_wire(bcrd, format!("CIBIN_{i}_{j}"), wire);
                        bel.inputs.insert(pins[j], self.xlat_input(bcrd, wire));
                    }
                    let pins = [
                        bcls::MACO::CIBOUT_0.as_slice(),
                        bcls::MACO::CIBOUT_1.as_slice(),
                        bcls::MACO::CIBOUT_2.as_slice(),
                    ][i];
                    for (j, &pin) in pins.iter().enumerate() {
                        let wire = self.rc_io_wire(cell, &format!("JCIBOUT_{i}_{j}_MACO"));
                        self.add_bel_wire(bcrd, format!("CIBOUT_{i}_{j}"), wire);
                        bel.outputs.insert(pin, self.xlat_output(bcrd, wire));
                    }
                }

                for i in 0..10 {
                    let pins = [
                        bcls::MACO::EBRO0,
                        bcls::MACO::EBRO1,
                        bcls::MACO::EBRO2,
                        bcls::MACO::EBRO3,
                        bcls::MACO::EBRO4,
                        bcls::MACO::EBRO5,
                        bcls::MACO::EBRO6,
                        bcls::MACO::EBRO7,
                        bcls::MACO::EBRO8,
                        bcls::MACO::EBRO9,
                    ][i];
                    for j in 0..64 {
                        let wire = self.rc_io_wire(cell, &format!("EBRO{i}{j:02}_MACO"));
                        self.add_bel_wire(bcrd, format!("EBRO{i}{j:02}"), wire);
                        let wire_out = self.rc_io_wire(cell, &format!("EBRO{i}{j:02}"));
                        self.add_bel_wire(bcrd, format!("EBRO{i}{j:02}_OUT"), wire_out);
                        self.claim_pip(wire_out, wire);
                        let wire_out_out = self.claim_single_out(wire_out);
                        self.add_bel_wire(bcrd, format!("EBRO{i}{j:02}_OUT_OUT"), wire_out_out);

                        let int_wire = match !edge {
                            DirH::W => wires::EBR_W1[j],
                            DirH::E => wires::EBR_E1[j],
                        }
                        .cell(15 - i);
                        bel.outputs.insert(pins[j], BTreeSet::from([int_wire]));
                        let wire_ebr = self.ebr_wires[&self.edev.bel_wire(bcrd, int_wire)];
                        self.claim_pip(wire_ebr, wire_out_out);
                    }
                    let pins = [
                        bcls::MACO::EBRI0,
                        bcls::MACO::EBRI1,
                        bcls::MACO::EBRI2,
                        bcls::MACO::EBRI3,
                        bcls::MACO::EBRI4,
                        bcls::MACO::EBRI5,
                        bcls::MACO::EBRI6,
                        bcls::MACO::EBRI7,
                        bcls::MACO::EBRI8,
                        bcls::MACO::EBRI9,
                    ][i];
                    for j in 0..48 {
                        let wire = self.rc_io_wire(cell, &format!("EBRI{i}{j:02}_MACO"));
                        self.add_bel_wire(bcrd, format!("EBRI{i}{j:02}"), wire);
                        let wire_in = self.rc_io_wire(cell, &format!("EBRI{i}{j:02}"));
                        self.add_bel_wire(bcrd, format!("EBRI{i}{j:02}_IN"), wire_in);
                        self.claim_pip(wire, wire_in);
                        let wire_ebr = self.claim_single_in(wire_in);
                        let int_wire = match edge {
                            DirH::W => wires::EBR_W0[j],
                            DirH::E => wires::EBR_E0[j],
                        }
                        .cell(6 + i);
                        bel.inputs.insert(pins[j], BelInput::Fixed(int_wire.pos()));
                        match self.ebr_wires.entry(self.edev.bel_wire(bcrd, int_wire)) {
                            btree_map::Entry::Vacant(e) => {
                                e.insert(wire_ebr);
                            }
                            btree_map::Entry::Occupied(e) => {
                                assert_eq!(*e.get(), wire_ebr);
                            }
                        }
                    }
                }

                for i in 0..4 {
                    let wire = self.rc_io_wire(cell, &format!("ECLK{i}_MACO"));
                    self.add_bel_wire(bcrd, format!("ECLK{i}"), wire);
                    let wire_in = self.rc_io_wire(cell, &format!("ECLK{i}"));
                    self.add_bel_wire(bcrd, format!("ECLK{i}_IN"), wire_in);
                    self.claim_pip(wire, wire_in);

                    let bank = match edge {
                        DirH::W => 7,
                        DirH::E => 2,
                    };
                    let bcrd_eclk = self.chip.bel_eclk_root_bank(bank);
                    for i in 0..8 {
                        let wire_eclk = self.naming.bel_wire(bcrd_eclk, &format!("ECLK{i}"));
                        self.claim_pip(wire_in, wire_eclk);
                    }
                }

                for dir in [DirV::S, DirV::N] {
                    let dir_io = match (dir, edge) {
                        (DirV::S, DirH::W) => DirH::E,
                        (DirV::S, DirH::E) => DirH::W,
                        (DirV::N, DirH::W) => DirH::W,
                        (DirV::N, DirH::E) => DirH::E,
                    };
                    for i in 0..8 {
                        let pins_o = match dir {
                            DirV::S => [
                                bcls::MACO::SIPO0,
                                bcls::MACO::SIPO1,
                                bcls::MACO::SIPO2,
                                bcls::MACO::SIPO3,
                                bcls::MACO::SIPO4,
                                bcls::MACO::SIPO5,
                                bcls::MACO::SIPO6,
                                bcls::MACO::SIPO7,
                            ][i],
                            DirV::N => [
                                bcls::MACO::NIPO0,
                                bcls::MACO::NIPO1,
                                bcls::MACO::NIPO2,
                                bcls::MACO::NIPO3,
                                bcls::MACO::NIPO4,
                                bcls::MACO::NIPO5,
                                bcls::MACO::NIPO6,
                                bcls::MACO::NIPO7,
                            ][i],
                        };
                        let pins_i = match dir {
                            DirV::S => [
                                bcls::MACO::SIPI0,
                                bcls::MACO::SIPI1,
                                bcls::MACO::SIPI2,
                                bcls::MACO::SIPI3,
                                bcls::MACO::SIPI4,
                                bcls::MACO::SIPI5,
                                bcls::MACO::SIPI6,
                                bcls::MACO::SIPI7,
                            ][i],
                            DirV::N => [
                                bcls::MACO::NIPI0,
                                bcls::MACO::NIPI1,
                                bcls::MACO::NIPI2,
                                bcls::MACO::NIPI3,
                                bcls::MACO::NIPI4,
                                bcls::MACO::NIPI5,
                                bcls::MACO::NIPI6,
                                bcls::MACO::NIPI7,
                            ][i],
                        };
                        for j in 0..32 {
                            let twire_ipo = match dir {
                                DirV::S => wiredata::OUT_MACO_IO_S[i][j],
                                DirV::N => wiredata::OUT_MACO_IO_N[i][j],
                            }
                            .cell(0);
                            let wire_ipo =
                                self.rc_io_wire(cell, &format!("{dir}IPO{i}{j:02}_MACO"));
                            self.add_bel_wire(bcrd, format!("{dir}IPO{i}{j:02}"), wire_ipo);
                            bel.outputs.insert(pins_o[j], BTreeSet::from([twire_ipo]));

                            let wire_ipo_out = self.rc_io_wire(cell, &format!("{dir}IPO{i}{j:02}"));
                            self.claim_pip(wire_ipo_out, wire_ipo);
                            self.add_bel_wire(
                                bcrd_int,
                                twire_ipo.to_string(self.intdb, tcls),
                                wire_ipo_out,
                            );

                            let twire_out = match dir_io {
                                DirH::W => wiredata::IO_W[i + 1][j],
                                DirH::E => wiredata::IO_E[i + 1][j],
                            }
                            .cell(match dir {
                                DirV::S => 4,
                                DirV::N => 5,
                            });
                            let wire_out = self
                                .io_int_names
                                .get(&self.edev.bel_wire(bcrd, twire_out))
                                .copied();
                            if let Some(wire_out) = wire_out {
                                self.add_bel_wire_no_claim(
                                    bcrd_int,
                                    twire_out.to_string(self.intdb, tcls),
                                    wire_out,
                                );
                            }

                            let twire_in = match dir_io {
                                DirH::W => wiredata::IO_W[i][j],
                                DirH::E => wiredata::IO_E[i][j],
                            }
                            .cell(match dir {
                                DirV::S => 5,
                                DirV::N => 4,
                            });
                            let wire_in = self
                                .io_int_names
                                .get(&self.edev.bel_wire(bcrd, twire_in))
                                .copied();
                            if let Some(wire_in) = wire_in {
                                self.add_bel_wire_no_claim(
                                    bcrd_int,
                                    twire_in.to_string(self.intdb, tcls),
                                    wire_in,
                                );
                            }

                            if let Some(wire_out) = wire_out {
                                self.claim_pip(wire_out, wire_ipo_out);
                                if let Some(wire_in) = wire_in {
                                    self.claim_pip(wire_out, wire_in);
                                }
                            }
                            sb.items.push(SwitchBoxItem::Mux(Mux {
                                dst: twire_out,
                                bits: vec![],
                                src: BTreeMap::from_iter(
                                    [twire_in.pos(), twire_ipo.pos()]
                                        .into_iter()
                                        .map(|k| (k, Default::default())),
                                ),
                                bits_off: None,
                            }));

                            let wire_ipi =
                                self.rc_io_wire(cell, &format!("{dir}IPI{i}{j:02}_MACO"));
                            self.add_bel_wire(bcrd, format!("{dir}IPI{i}{j:02}"), wire_ipi);
                            let wire_ipi_in = self.rc_io_wire(cell, &format!("{dir}IPI{i}{j:02}"));
                            self.add_bel_wire(bcrd, format!("{dir}IPI{i}{j:02}_IN"), wire_ipi_in);
                            self.claim_pip(wire_ipi, wire_ipi_in);
                            if let Some(wire_in) = wire_in {
                                self.claim_pip(wire_ipi_in, wire_in);
                            }
                            bel.inputs
                                .insert(pins_i[j], BelInput::Fixed(twire_in.pos()));
                        }
                    }
                }

                let mut tips = vec![];
                for i in 0..2 {
                    let tip = self.rc_io_wire(cell, &format!("TIP{i:04}_MACO"));
                    self.add_bel_wire(bcrd, format!("TIP{i:04}"), tip);
                    let tip_out = self.rc_io_wire(cell, &format!("TIP{i:04}"));
                    self.add_bel_wire(bcrd, format!("TIP{i:04}_OUT"), tip_out);
                    self.claim_pip(tip_out, tip);
                    let tip_cib = self.rc_io_wire(cell, &format!("JTIP{i:04}_CIB"));
                    self.add_bel_wire(bcrd, format!("TIP{i:04}_CIB"), tip_cib);
                    self.claim_pip(tip_cib, tip_out);
                    let bpin = self.xlat_output(bcrd, tip_cib);
                    assert_eq!(bpin.len(), 1);
                    let twire = bpin.iter().copied().next().unwrap();
                    self.add_bel_wire_no_claim(
                        bcrd_int,
                        twire.to_string(self.intdb, tcls),
                        tip_out,
                    );
                    tips.push((twire, tip_out));
                    bel.outputs.insert(bcls::MACO::TIP[i], bpin);
                }

                for dir in [DirV::S, DirV::N] {
                    let wire = match (dir, edge) {
                        (DirV::S, DirH::W) => wires::IO_T_E,
                        (DirV::S, DirH::E) => wires::IO_T_W,
                        (DirV::N, DirH::W) => wires::IO_T_W,
                        (DirV::N, DirH::E) => wires::IO_T_E,
                    };

                    let twire_out = wire.cell(match dir {
                        DirV::S => 4,
                        DirV::N => 5,
                    });
                    let wire_out = self.io_int_names[&self.edev.bel_wire(bcrd, twire_out)];
                    self.add_bel_wire_no_claim(
                        bcrd_int,
                        twire_out.to_string(self.intdb, tcls),
                        wire_out,
                    );

                    let twire_in = wire.cell(match dir {
                        DirV::S => 5,
                        DirV::N => 4,
                    });
                    let wire_in = self.io_int_names[&self.edev.bel_wire(bcrd, twire_in)];
                    self.add_bel_wire_no_claim(
                        bcrd_int,
                        twire_in.to_string(self.intdb, tcls),
                        wire_in,
                    );

                    let mut src = BTreeSet::from_iter([twire_in.pos()]);
                    self.claim_pip(wire_out, wire_in);

                    for &(twire_tip, wire_tip) in &tips {
                        src.insert(twire_tip.pos());
                        self.claim_pip(wire_out, wire_tip);
                    }

                    let wire_cib = self.rc_io_wire(cell, &format!("JVMT{dir}CIBI"));
                    let bpin = self.xlat_int_wire(bcrd, wire_cib);
                    assert_eq!(bpin.wires.len(), 1);
                    assert_eq!(bpin.dir, PinDir::Input);
                    let twire_cib = bpin.wires.into_iter().next().unwrap();
                    self.add_bel_wire(bcrd_int, twire_cib.to_string(self.intdb, tcls), wire_cib);
                    self.claim_pip(wire_out, wire_cib);
                    src.insert(twire_cib.pos());

                    sb.items.push(SwitchBoxItem::Mux(Mux {
                        dst: twire_out,
                        bits: vec![],
                        src: src.into_iter().map(|k| (k, Default::default())).collect(),
                        bits_off: None,
                    }));
                }

                self.insert_bel(bcrd, bel);
                self.insert_bel_generic(bcrd_int, BelInfo::SwitchBox(sb));
            }
        }
    }
}
