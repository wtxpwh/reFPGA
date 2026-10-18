use std::collections::{BTreeMap, BTreeSet, btree_map};

use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityBundleItemIndex;
use prjcombine_interconnect::{
    db::{Bel, BelInfo, BelInput, Mux, SwitchBox, SwitchBoxItem, WireSlotIdExt},
    dir::DirH,
};

use crate::ChipContext;

impl ChipContext<'_> {
    fn process_ebr_scm(&mut self) {
        let mut pins_out = vec![
            (bcls::EBR::EF, wires::OUT_EBR_EF),
            (bcls::EBR::FF, wires::OUT_EBR_FF),
            (bcls::EBR::AE, wires::OUT_EBR_AE),
            (bcls::EBR::AF, wires::OUT_EBR_AF),
        ];
        for i in 0..18 {
            pins_out.extend([
                (bcls::EBR::DOA[i], wires::OUT_EBR_DOA[i]),
                (bcls::EBR::DOB[i], wires::OUT_EBR_DOB[i]),
            ]);
        }
        let mut pins_in = vec![
            (bcls::EBR::CLKA, wires::IMUX_EBR_CLKA),
            (bcls::EBR::CLKB, wires::IMUX_EBR_CLKB),
            (bcls::EBR::RSTA, wires::IMUX_EBR_RSTA),
            (bcls::EBR::RSTB, wires::IMUX_EBR_RSTB),
            (bcls::EBR::CEA, wires::IMUX_EBR_CEA),
            (bcls::EBR::CEB, wires::IMUX_EBR_CEB),
            (bcls::EBR::WEA, wires::IMUX_EBR_WEA),
            (bcls::EBR::WEB, wires::IMUX_EBR_WEB),
        ];
        for i in 0..18 {
            pins_in.extend([
                (bcls::EBR::DIA[i], wires::IMUX_EBR_DIA[i]),
                (bcls::EBR::DIB[i], wires::IMUX_EBR_DIB[i]),
            ]);
        }
        for i in 3..14 {
            pins_in.extend([
                (bcls::EBR::ADA[i], wires::IMUX_EBR_ADA[i - 3]),
                (bcls::EBR::ADB[i], wires::IMUX_EBR_ADB[i - 3]),
            ]);
        }
        for tcid in [tcls::EBR_W, tcls::EBR_E] {
            let tcls = &self.intdb[tcid];
            let mut ebr_from_int = BTreeMap::new();
            let mut ebr_to_int = BTreeMap::new();
            let mut x10_from_x10 = BTreeMap::new();
            let mut x10_to_ebr = BTreeMap::new();
            let mut x10_from_ebr = BTreeMap::new();
            for dir in [DirH::W, DirH::E] {
                let num = match (tcid, dir) {
                    (tcls::EBR_W, DirH::W) => 48,
                    (tcls::EBR_W, DirH::E) => 64,
                    (tcls::EBR_E, DirH::W) => 64,
                    (tcls::EBR_E, DirH::E) => 48,
                    _ => unreachable!(),
                };
                for i in 0..num {
                    let iwire_i = match dir {
                        DirH::W => wires::EBR_W1[i],
                        DirH::E => wires::EBR_E1[i],
                    }
                    .cell(0);
                    let iwire_o = match dir {
                        DirH::W => wires::EBR_W0[i],
                        DirH::E => wires::EBR_E0[i],
                    }
                    .cell(0);
                    x10_from_x10.insert(iwire_o, iwire_i);
                }
            }

            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::EBR[0]);
                let bcrd_int = tcrd.bel(bslots::BEL_INT);
                let cell_w = bcrd.cell;
                let cell_e = bcrd.cell.delta(1, 0);
                let (r, c) = self.rc(cell_w);
                self.name_bel(bcrd, [format!("EBR_R{r}C{c}")]);
                self.name_bel_null(bcrd_int);

                let mut bel = Bel::default();

                let mut x10_out_wires = BTreeMap::new();
                let mut cur_x10_out = BTreeMap::new();
                let mut x10_in_wires = BTreeMap::new();
                let mut cur_x10_in = BTreeMap::new();

                for (&iwire_o, &iwire_i) in &x10_from_x10 {
                    let wire_i = self.ebr_wires.get(&cell_w.wire(iwire_i.wire)).copied();
                    let wire_o = self.ebr_wires.get(&cell_w.wire(iwire_o.wire)).copied();

                    if let Some(wire_i) = wire_i {
                        self.add_bel_wire(bcrd_int, iwire_i.to_string(self.intdb, tcls), wire_i);
                        x10_in_wires.insert(wire_i, iwire_i);
                        cur_x10_in.insert(iwire_i, vec![]);
                    }
                    if let Some(wire_o) = wire_o {
                        self.add_bel_wire(bcrd_int, iwire_o.to_string(self.intdb, tcls), wire_o);
                        x10_out_wires.insert(wire_o, iwire_o);
                        cur_x10_out.insert(iwire_o, vec![]);
                        if let Some(wire_i) = wire_i {
                            self.claim_pip(wire_o, wire_i);
                        }
                    }
                }

                for &(pin, wslot) in &pins_out {
                    let (pname_root, pin_idx) = self.intdb[bcls::EBR].outputs.key(pin);
                    let pname = match pin_idx {
                        EntityBundleItemIndex::Single => pname_root.to_string(),
                        EntityBundleItemIndex::Array { index, .. } => {
                            format!("{pname_root}{index}")
                        }
                    };
                    let wire_eint = self.rc_wire(cell_e, &format!("JCIB_{pname}"));
                    let wires = self.xlat_output(bcrd_int, wire_eint);
                    assert_eq!(wires.len(), 1);
                    let int_wire = wires.into_iter().next().unwrap();
                    self.add_bel_wire(bcrd_int, int_wire.to_string(self.intdb, tcls), wire_eint);

                    for wn in self.pips_bwd.get(&wire_eint).cloned().into_iter().flatten() {
                        if let Some(&x10_iwire) = x10_in_wires.get(&wn) {
                            cur_x10_in.get_mut(&x10_iwire).unwrap().push(int_wire);
                            self.claim_pip(wire_eint, wn);
                        }
                    }

                    let out_wire = wslot.cell(0);
                    bel.outputs.insert(pin, BTreeSet::from_iter([out_wire]));
                    match ebr_to_int.entry(out_wire) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(int_wire);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), int_wire);
                        }
                    }

                    let wire = self.rc_wire(cell_w, &format!("J{pname}_EBR"));
                    self.add_bel_wire(bcrd, &pname, wire);
                    let wire_out = self.rc_wire(cell_e, &format!("J{pname}"));
                    self.add_bel_wire(bcrd, format!("{pname}_OUT"), wire_out);
                    self.claim_pip(wire_out, wire);
                    self.claim_pip(wire_eint, wire_out);

                    for wn in self.pips_fwd[&wire_out].clone() {
                        if let Some(&x10_iwire) = x10_out_wires.get(&wn) {
                            cur_x10_out.get_mut(&x10_iwire).unwrap().push(out_wire);
                            self.claim_pip(wn, wire_out);
                        }
                    }
                }

                for i in 0..8 {
                    let wire_eint = self.rc_wire(cell_e, &format!("JCIB_EXTRA{i}"));
                    let wires = self.xlat_output(bcrd_int, wire_eint);
                    assert_eq!(wires.len(), 1);
                    let int_wire = wires.into_iter().next().unwrap();
                    self.add_bel_wire(bcrd_int, int_wire.to_string(self.intdb, tcls), wire_eint);

                    for wn in self.pips_bwd.get(&wire_eint).cloned().into_iter().flatten() {
                        if let Some(&x10_iwire) = x10_in_wires.get(&wn) {
                            cur_x10_in.get_mut(&x10_iwire).unwrap().push(int_wire);
                            self.claim_pip(wire_eint, wn);
                        }
                    }
                }

                for &(pin, wslot) in &pins_in {
                    let (pname_root, pin_idx) = self.intdb[bcls::EBR].inputs.key(pin);
                    let pname = match pin_idx {
                        EntityBundleItemIndex::Single => pname_root.to_string(),
                        EntityBundleItemIndex::Array { index, .. } => {
                            format!("{pname_root}{index}")
                        }
                    };
                    let wire_eint = self.rc_wire(cell_e, &format!("JCIB_{pname}"));
                    let bpin = self.xlat_input(bcrd_int, wire_eint);
                    let int_wire = bpin.wire();
                    self.add_bel_wire(bcrd_int, int_wire.to_string(self.intdb, tcls), wire_eint);

                    for wn in self.pips_fwd[&wire_eint].clone() {
                        if let Some(&x10_iwire) = x10_out_wires.get(&wn) {
                            cur_x10_out.get_mut(&x10_iwire).unwrap().push(int_wire);
                            self.claim_pip(wn, wire_eint);
                        }
                    }

                    let imux_wire = wslot.cell(0);

                    bel.inputs.insert(pin, BelInput::Fixed(imux_wire.pos()));
                    match ebr_from_int.entry(imux_wire) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(int_wire);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), int_wire);
                        }
                    }

                    let wire = self.rc_wire(cell_w, &format!("J{pname}_EBR"));
                    self.add_bel_wire(bcrd, &pname, wire);
                    let wire_in = self.rc_wire(cell_e, &format!("J{pname}"));
                    self.add_bel_wire(bcrd, format!("{pname}_IN"), wire_in);
                    self.claim_pip(wire, wire_in);
                    self.claim_pip(wire_in, wire_eint);

                    for wn in self.pips_bwd[&wire_in].clone() {
                        if let Some(&x10_iwire) = x10_in_wires.get(&wn) {
                            cur_x10_in.get_mut(&x10_iwire).unwrap().push(imux_wire);
                            self.claim_pip(wire_in, wn);
                        }
                    }
                }

                for pin in [
                    bcls::EBR::ADA[0],
                    bcls::EBR::ADA[1],
                    bcls::EBR::ADA[2],
                    bcls::EBR::ADB[0],
                    bcls::EBR::ADB[1],
                    bcls::EBR::ADB[2],
                    bcls::EBR::CSA[0],
                    bcls::EBR::CSA[1],
                    bcls::EBR::CSA[2],
                    bcls::EBR::CSB[0],
                    bcls::EBR::CSB[1],
                    bcls::EBR::CSB[2],
                ] {
                    let (pname_root, pin_idx) = self.intdb[bcls::EBR].inputs.key(pin);
                    let pname = match pin_idx {
                        EntityBundleItemIndex::Single => pname_root.to_string(),
                        EntityBundleItemIndex::Array { index, .. } => {
                            format!("{pname_root}{index}")
                        }
                    };
                    let wire = self.rc_wire(cell_w, &format!("J{pname}_EBR"));
                    self.add_bel_wire(bcrd, &pname, wire);
                    let wire_in = self.rc_wire(cell_e, &format!("J{pname}"));
                    self.add_bel_wire(bcrd, format!("{pname}_IN"), wire_in);
                    let wire_cib = self.rc_wire(cell_e, &format!("JCIB_{pname}"));
                    self.add_bel_wire(bcrd, format!("{pname}_CIB"), wire_cib);
                    self.claim_pip(wire, wire_in);
                    self.claim_pip(wire_in, wire_cib);
                    bel.inputs.insert(pin, self.xlat_input(bcrd, wire_cib));
                }

                for (iwire, wires) in cur_x10_in {
                    match x10_to_ebr.entry(iwire) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(wires);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), wires);
                        }
                    }
                }
                for (iwire, wires) in cur_x10_out {
                    match x10_from_ebr.entry(iwire) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(wires);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), wires);
                        }
                    }
                }

                self.insert_bel(bcrd, bel);
            }
            let mut sb = SwitchBox::default();
            let mut muxes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
            for (wf, wt) in ebr_to_int {
                muxes.entry(wt).or_default().insert(wf.pos());
            }
            for (wt, wf) in ebr_from_int {
                muxes.entry(wt).or_default().insert(wf.pos());
            }
            for (wt, wf) in x10_from_x10 {
                muxes.entry(wt).or_default().insert(wf.pos());
            }
            for (wt, wfs) in x10_from_ebr {
                for wf in wfs {
                    muxes.entry(wt).or_default().insert(wf.pos());
                }
            }
            for (wf, wts) in x10_to_ebr {
                for wt in wts {
                    muxes.entry(wt).or_default().insert(wf.pos());
                }
            }
            for (dst, src) in muxes {
                let mux = Mux {
                    dst,
                    bits: vec![],
                    src: src.into_iter().map(|k| (k, Default::default())).collect(),
                    bits_off: None,
                };
                sb.items.push(SwitchBoxItem::Mux(mux));
            }
            self.bels
                .insert((tcid, bslots::BEL_INT), BelInfo::SwitchBox(sb));
        }
    }

    fn process_ebr_ecp(&mut self) {
        let tiles = if matches!(self.chip.kind, ChipKind::MachXo2(_)) {
            [tcls::EBR, tcls::EBR_N].as_slice()
        } else {
            [tcls::EBR].as_slice()
        };
        for &tcid in tiles {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::EBR[0]);
                let cell = if self.chip.kind == ChipKind::MachXo {
                    tcrd.cell.delta(0, 3)
                } else {
                    tcrd.cell
                };
                self.extract_bel_cell(bcrd, cell)
                    .name_rc("EBR")
                    .suffix("EBR");
            }
        }
    }

    fn process_ebr_ecp4(&mut self) {
        let tcid = tcls::EBR;
        for &tcrd in &self.edev.tile_index[tcid] {
            for i in 0..4 {
                let bcrd = tcrd.bel(bslots::EBR[i]);
                let cell = tcrd.delta(2 * (i as i32), 0);
                self.extract_bel_cell(bcrd, cell)
                    .name_rc("EBR")
                    .suffix("EBR");
            }
        }
    }

    pub fn process_ebr(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => self.process_ebr_scm(),
            ChipKind::Ecp
            | ChipKind::Xp
            | ChipKind::MachXo
            | ChipKind::Ecp2
            | ChipKind::Ecp2M
            | ChipKind::Xp2
            | ChipKind::Ecp3(_)
            | ChipKind::MachXo2(_) => self.process_ebr_ecp(),
            ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink => self.process_ebr_ecp4(),
        }
    }
}
