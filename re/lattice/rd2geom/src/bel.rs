use std::collections::{BTreeSet, btree_map};

use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bslots, wires},
};
use prjcombine_entity::{EntityBundleIndices, EntityBundleItemIndex};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord},
    db::{
        Bel, BelClassId, BelInfo, BelInput, BelInputId, BelOutputId, CellSlotId, IntDb, SwitchBox,
        TileWireCoord,
    },
};
use prjcombine_re_lattice_naming::{BelNaming, WireName};

use crate::{ChipContext, RcMode};

pub struct BelExtractor<'dev, 'ctx> {
    pub ctx: &'ctx mut ChipContext<'dev>,
    pub intdb: &'dev IntDb,
    pub bcid: BelClassId,
    pub bcrd: BelCoord,
    pub cell: CellCoord,
    pub bel: Bel,
    pub discard: bool,
}

impl<'dev, 'ctx> BelExtractor<'dev, 'ctx> {
    pub fn suffix(&mut self, suffix: &'static str) -> &mut Self {
        let wires = self.ctx.sorted_wires[&(self.cell, suffix)].clone();
        for (idx, name, input) in self.intdb[self.bcid].inputs.bundles() {
            match idx {
                EntityBundleIndices::Single(id) => {
                    if self.bel.inputs.contains_id(id) {
                        continue;
                    }
                    if let Some(&wire) = wires.get(name)
                        && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true, None)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.input_raw(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        if self.bel.inputs.contains_id(id) {
                            continue;
                        }
                        let vi = input.indexing.phys_to_virt(pi);
                        let iname = &if name.ends_with(|c: char| c.is_ascii_digit()) {
                            format!("{name}_{vi}")
                        } else {
                            format!("{name}{vi}")
                        };
                        let mut wire = wires.get(&format!("{name}_{vi}"));
                        if range.len() == 1 && wire.is_none() {
                            wire = wires.get(name);
                        }
                        if wire.is_none() {
                            wire = wires.get(&format!("{name}{vi}"));
                        }
                        if let Some(&wire) = wire
                            && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true, None)
                        {
                            self.ctx.add_bel_wire(self.bcrd, iname, wire);
                            self.input_raw(id, bpin);
                        }
                    }
                }
            }
        }
        for (idx, name, output) in self.intdb[self.bcid].outputs.bundles() {
            match idx {
                EntityBundleIndices::Single(id) => {
                    if self.bel.outputs.contains_id(id) {
                        continue;
                    }
                    if let Some(&wire) = wires.get(name)
                        && let Some(bpin) =
                            self.ctx.try_xlat_output(self.bcrd, wire, false, true, None)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.output_raw(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        if self.bel.outputs.contains_id(id) {
                            continue;
                        }
                        let vi = output.indexing.phys_to_virt(pi);
                        let iname = &if name.ends_with(|c: char| c.is_ascii_digit()) {
                            format!("{name}_{vi}")
                        } else {
                            format!("{name}{vi}")
                        };
                        let mut wire = wires.get(&format!("{name}_{vi}"));
                        if range.len() == 1 && wire.is_none() {
                            wire = wires.get(name);
                        }
                        if wire.is_none() {
                            wire = wires.get(&format!("{name}{vi}"));
                        }
                        if let Some(&wire) = wire
                            && let Some(bpin) =
                                self.ctx.try_xlat_output(self.bcrd, wire, false, true, None)
                        {
                            self.ctx.add_bel_wire(self.bcrd, iname, wire);
                            self.output_raw(id, bpin);
                        }
                    }
                }
            }
        }
        self
    }

    pub fn names<T>(&mut self, names: impl IntoIterator<Item = T>) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.ctx.name_bel(self.bcrd, names);
        self
    }

    pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.names([name])
    }

    pub fn name_null(&mut self) -> &mut Self {
        self.names::<&str>([])
    }

    pub fn name_rc_suffix(
        &mut self,
        prefix: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let prefix = prefix.as_ref();
        let suffix = suffix.as_ref();
        let (r, c) = self.ctx.rc(self.cell);
        self.name(format!("{prefix}_R{r}C{c}{suffix}"))
    }

    pub fn name_rc(&mut self, prefix: impl AsRef<str>) -> &mut Self {
        self.name_rc_suffix(prefix, "")
    }

    fn input_pname(&self, pin: BelInputId) -> String {
        match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        }
    }

    pub fn input_raw(&mut self, pin: BelInputId, wire: BelInput) -> &mut Self {
        if self.bel.inputs.contains_id(pin) {
            assert_eq!(self.bel.inputs[pin], wire);
        } else {
            self.bel.inputs.insert(pin, wire);
        }
        self
    }

    pub fn input(&mut self, pin: BelInputId, wire: TileWireCoord) -> &mut Self {
        self.input_raw(pin, BelInput::Fixed(wire.pos()))
    }

    pub fn input_xlat_no_name(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_input(self.bcrd, wire);
        if !self.ctx.io_int_wires.contains_key(&wire) {
            self.ctx.bind_wire(
                wire,
                self.ctx
                    .edev
                    .resolve_bel_wire(self.bcrd, bpin.wire())
                    .unwrap(),
            );
        }
        self.input_raw(pin, bpin)
    }

    pub fn input_xlat_no_name_nudge(
        &mut self,
        pin: BelInputId,
        wire: WireName,
        cslot: CellSlotId,
    ) -> &mut Self {
        let bpin = self.ctx.xlat_input_nudge(self.bcrd, wire, cslot);
        if !self.ctx.io_int_wires.contains_key(&wire) {
            self.ctx.bind_wire(
                wire,
                self.ctx
                    .edev
                    .resolve_bel_wire(self.bcrd, bpin.wire())
                    .unwrap(),
            );
        }
        self.input_raw(pin, bpin)
    }

    pub fn input_xlat(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        self.input_xlat_rename(pin, self.input_pname(pin), wire)
    }

    pub fn input_xlat_nudge(
        &mut self,
        pin: BelInputId,
        wire: WireName,
        cslot: CellSlotId,
    ) -> &mut Self {
        self.input_xlat_rename_nudge(pin, self.input_pname(pin), wire, cslot)
    }

    pub fn input_xlat_rc_mode(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        mode: RcMode,
    ) -> &mut Self {
        let wire = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.input_xlat(pin, wire)
    }

    pub fn input_xlat_rc(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(pin, suffix, RcMode::Base)
    }

    pub fn input_xlat_rc_io(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(pin, suffix, RcMode::Io)
    }

    pub fn input_xlat_rc_corner(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(pin, suffix, RcMode::Corner)
    }

    pub fn input_xlat_rc_nudge(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        cslot: CellSlotId,
    ) -> &mut Self {
        let wire = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.input_xlat_nudge(pin, wire, cslot)
    }

    pub fn input_xlat_rename(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        wire: WireName,
    ) -> &mut Self {
        if let Some(wires) = self
            .ctx
            .int_wires
            .get(&wire)
            .or(self.ctx.io_int_wires.get(&wire))
        {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.add_wire_no_claim(pname, wire);
            self.input(pin, tw)
        } else {
            self.add_wire(pname, wire);
            self.input_xlat_no_name(pin, wire)
        }
    }

    pub fn input_xlat_rename_nudge(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        wire: WireName,
        cslot: CellSlotId,
    ) -> &mut Self {
        if let Some(wires) = self
            .ctx
            .int_wires
            .get(&wire)
            .or(self.ctx.io_int_wires.get(&wire))
        {
            let tw = self
                .ctx
                .wire_to_tw(self.bcrd, wires[0], Some(cslot))
                .unwrap();
            self.add_wire_no_claim(pname, wire);
            self.input(pin, tw)
        } else {
            self.add_wire(pname, wire);
            self.input_xlat_no_name_nudge(pin, wire, cslot)
        }
    }

    pub fn input_xlat_rename_rc_mode(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        suffix: impl AsRef<str>,
        mode: RcMode,
    ) -> &mut Self {
        let wire = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.input_xlat_rename(pin, pname, wire)
    }

    pub fn input_xlat_rename_rc(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        self.input_xlat_rename_rc_mode(pin, pname, suffix, RcMode::Base)
    }

    pub fn input_xlat_rename_rc_io(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        self.input_xlat_rename_rc_mode(pin, pname, suffix, RcMode::Io)
    }

    pub fn input_hint(&mut self, pin: BelInputId, name: WireName, tw: TileWireCoord) -> &mut Self {
        let atw = self.ctx.xlat_input(self.bcrd, name).wire();
        assert_eq!(
            self.ctx.edev.resolve_bel_wire(self.bcrd, tw).unwrap(),
            self.ctx.edev.resolve_bel_wire(self.bcrd, atw).unwrap(),
        );
        self.add_wire(self.input_pname(pin), name);
        self.input(pin, tw)
    }

    pub fn input_hint_rc(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let wname = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.input_hint(pin, wname, wire)
    }

    pub fn input_io(&mut self, pin: BelInputId, name: WireName, wire: TileWireCoord) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, name);
        let rwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        if matches!(self.ctx.chip.kind, ChipKind::Nexus(_)) {
            let idx = wires::OUT_IO_INCK.index_of(rwire.slot).unwrap();
            let bcrd = rwire.bel(bslots::IO[idx]);
            let name_io = self.ctx.get_inck_wire_nexus(bcrd);
            self.ctx.claim_pip(name, name_io);
            let name_io = self.ctx.get_paddi_wire_nexus(bcrd);
            self.ctx.claim_pip(name, name_io);
        } else {
            let name_io = self.ctx.get_paddi_wire(rwire);
            self.ctx.claim_pip(name, name_io);
        }
        self.input_raw(pin, BelInput::Fixed(wire.pos()))
    }

    pub fn input_io_rc_mode(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        mode: RcMode,
        wire: TileWireCoord,
    ) -> &mut Self {
        let wname = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.input_io(pin, wname, wire)
    }

    pub fn input_io_rc(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.input_io_rc_mode(pin, suffix, RcMode::Base, wire)
    }

    pub fn input_io_rc_io(
        &mut self,
        pin: BelInputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.input_io_rc_mode(pin, suffix, RcMode::Io, wire)
    }

    pub fn output_raw(&mut self, pin: BelOutputId, wires: BTreeSet<TileWireCoord>) -> &mut Self {
        if self.bel.outputs.contains_id(pin) {
            assert_eq!(self.bel.outputs[pin], wires);
        } else {
            self.bel.outputs.insert(pin, wires);
        }
        self
    }

    fn output_pname(&self, pin: BelOutputId) -> String {
        match self.ctx.intdb[self.bcid].outputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        }
    }

    pub fn output(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.output_raw(pin, BTreeSet::from_iter([wire]))
    }

    pub fn output_bind(
        &mut self,
        pin: BelOutputId,
        wname: WireName,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.add_wire(self.output_pname(pin), wname);
        self.ctx.bind_wire(
            wname,
            self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap(),
        );
        self.output_raw(pin, BTreeSet::from_iter([wire]))
    }

    pub fn output_bind_rc_mode(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        mode: RcMode,
        wire: TileWireCoord,
    ) -> &mut Self {
        let wname = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.output_bind(pin, wname, wire)
    }

    pub fn output_bind_rc(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.output_bind_rc_mode(pin, suffix, RcMode::Base, wire)
    }

    pub fn output_bind_rc_io(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.output_bind_rc_mode(pin, suffix, RcMode::Io, wire)
    }

    pub fn output_bind_step(
        &mut self,
        pin: BelOutputId,
        wname: WireName,
        wire: TileWireCoord,
    ) -> &mut Self {
        let pname = self.output_pname(pin);
        let wname_out = self.ctx.claim_single_out(wname);
        self.add_wire(&pname, wname);
        self.add_wire(format!("{pname}_OUT"), wname_out);
        let rwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        self.ctx.bind_wire(wname, rwire);
        self.ctx.bind_wire(wname_out, rwire);
        self.output_raw(pin, BTreeSet::from_iter([wire]))
    }

    pub fn output_bind_step_rc_mode(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        mode: RcMode,
        wire: TileWireCoord,
    ) -> &mut Self {
        let wname = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.output_bind_step(pin, wname, wire)
    }

    pub fn output_xlat_no_name(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output(self.bcrd, wire);
        self.output_raw(pin, bpin)
    }

    pub fn output_xlat_quiet_no_name(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output_quiet(self.bcrd, wire);
        self.output_raw(pin, bpin)
    }

    pub fn output_xlat_rename(
        &mut self,
        pin: BelOutputId,
        pname: impl AsRef<str>,
        wire: WireName,
    ) -> &mut Self {
        if let Some(&(bcrd, _tw, ref out)) = self.ctx.hybrid_out.get(&wire) {
            assert_eq!(bcrd, self.bcrd);
            let out = out.clone();
            self.add_wire(pname, wire);
            self.output_raw(pin, out)
        } else if let Some(wires) = self.ctx.io_int_wires.get(&wire) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.add_wire_no_claim(pname, wire);
            self.output(pin, tw)
        } else {
            self.add_wire(pname, wire);
            self.output_xlat_no_name(pin, wire)
        }
    }

    pub fn output_xlat(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].outputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.output_xlat_rename(pin, pname, wire)
    }

    pub fn output_xlat_quiet(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].outputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };

        if let Some(wires) = self.ctx.io_int_wires.get(&wire) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.add_wire_no_claim(pname, wire);
            self.output(pin, tw)
        } else {
            self.add_wire(pname, wire);
            self.output_xlat_quiet_no_name(pin, wire)
        }
    }

    pub fn output_xlat_rc_mode(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        mode: RcMode,
    ) -> &mut Self {
        let wire = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.output_xlat(pin, wire)
    }

    pub fn output_xlat_rc(&mut self, pin: BelOutputId, suffix: impl AsRef<str>) -> &mut Self {
        self.output_xlat_rc_mode(pin, suffix, RcMode::Base)
    }

    pub fn output_xlat_rc_io(&mut self, pin: BelOutputId, suffix: impl AsRef<str>) -> &mut Self {
        self.output_xlat_rc_mode(pin, suffix, RcMode::Io)
    }

    pub fn output_xlat_rc_corner(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        self.output_xlat_rc_mode(pin, suffix, RcMode::Corner)
    }

    pub fn output_xlat_rename_rc_io(
        &mut self,
        pin: BelOutputId,
        name: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let wire = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.output_xlat_rename(pin, name, wire)
    }

    pub fn output_xlat_rename_rc_corner(
        &mut self,
        pin: BelOutputId,
        name: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let wire = self.ctx.rc_corner_wire(self.cell, suffix.as_ref());
        self.output_xlat_rename(pin, name, wire)
    }

    pub fn output_xlat_rc_corner_quiet(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let wire = self.ctx.rc_corner_wire(self.cell, suffix.as_ref());
        self.output_xlat_quiet(pin, wire)
    }

    pub fn add_wire(&mut self, pin: impl AsRef<str>, wire: WireName) -> &mut Self {
        self.ctx.add_bel_wire(self.bcrd, pin, wire);
        self
    }

    pub fn add_wire_rc(&mut self, pin: impl AsRef<str>, suffix: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.add_wire(pin, name)
    }

    pub fn add_wire_rc_io(&mut self, pin: impl AsRef<str>, suffix: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.add_wire(pin, name)
    }

    pub fn add_wire_rc_mode(
        &mut self,
        pin: impl AsRef<str>,
        suffix: impl AsRef<str>,
        mode: RcMode,
    ) -> &mut Self {
        let name = self.ctx.rc_wire_mode(self.cell, suffix.as_ref(), mode);
        self.add_wire(pin, name)
    }

    pub fn add_wire_rc_corner(
        &mut self,
        pin: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let name = self.ctx.rc_corner_wire(self.cell, suffix.as_ref());
        self.add_wire(pin, name)
    }

    pub fn add_wire_rc_get(&mut self, pin: impl AsRef<str>, suffix: impl AsRef<str>) -> WireName {
        let name = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.add_wire(pin, name);
        name
    }

    pub fn add_wire_rc_io_get(
        &mut self,
        pin: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> WireName {
        let name = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.add_wire(pin, name);
        name
    }

    pub fn add_wire_no_claim(&mut self, pin: impl AsRef<str>, wire: WireName) -> &mut Self {
        self.ctx.add_bel_wire_no_claim(self.bcrd, pin, wire);
        self
    }

    pub fn bind_wire(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let fw = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        self.ctx.bind_wire(name, fw);
        self
    }

    pub fn claim_pip(&mut self, dst: impl AsRef<str>, src: impl AsRef<str>) -> &mut Self {
        let dst = self.ctx.naming.bel_wire(self.bcrd, dst.as_ref());
        let src = self.ctx.naming.bel_wire(self.bcrd, src.as_ref());
        self.ctx.claim_pip(dst, src);
        self
    }

    pub fn claim_pip_from(&mut self, dst: impl AsRef<str>, src: WireName) -> &mut Self {
        let dst = self.ctx.naming.bel_wire(self.bcrd, dst.as_ref());
        self.ctx.claim_pip(dst, src);
        self
    }

    pub fn claim_pip_to(&mut self, dst: WireName, src: impl AsRef<str>) -> &mut Self {
        let src = self.ctx.naming.bel_wire(self.bcrd, src.as_ref());
        self.ctx.claim_pip(dst, src);
        self
    }

    pub fn discard(mut self) {
        self.discard = true;
    }

    // SC SERDES_CORNER special
    pub fn input_sc_xlat(&mut self, pin: BelInputId, wires: &[WireName]) -> &mut Self {
        let wname = self.ctx.find_single_in(wires[0]);
        for &w in wires {
            self.ctx.claim_pip(w, wname);
        }
        self.add_wire(self.input_pname(pin), wname);
        let tw = self.ctx.xlat_sc_wire(wname, self.bcrd);
        self.input(pin, tw)
    }

    pub fn output_sc_xlat(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let wname = self.ctx.pips_fwd[&wire]
            .iter()
            .copied()
            .find(|&w| !self.ctx.int_wires.contains_key(&w))
            .unwrap();
        self.ctx.claim_pip(wname, wire);
        self.add_wire(self.output_pname(pin), wname);
        let tw = self.ctx.xlat_sc_wire(wname, self.bcrd);
        self.output(pin, tw)
    }
}

impl<'dev, 'ctx> Drop for BelExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        if !self.discard {
            self.ctx
                .insert_bel(self.bcrd, std::mem::take(&mut self.bel));
        }
    }
}

impl<'dev> ChipContext<'dev> {
    pub fn name_bel<T: AsRef<str>>(&mut self, bel: BelCoord, names: impl IntoIterator<Item = T>) {
        let names = Vec::from_iter(
            names
                .into_iter()
                .map(|x| self.naming.strings.get_or_insert(x.as_ref())),
        );
        assert!(
            self.naming
                .bels
                .insert(
                    bel,
                    BelNaming {
                        names,
                        wires: Default::default(),
                        int_wires: Default::default(),
                        int_inputs: Default::default(),
                    },
                )
                .is_none()
        );
    }

    pub fn name_bel_null(&mut self, bel: BelCoord) {
        self.name_bel::<&str>(bel, []);
    }

    pub fn add_bel_wire(&mut self, bel: BelCoord, pin: impl AsRef<str>, wire: WireName) {
        self.add_bel_wire_no_claim(bel, pin, wire);
        self.claim_node(wire);
    }

    pub fn add_bel_wire_no_claim(&mut self, bel: BelCoord, pin: impl AsRef<str>, wire: WireName) {
        let pin = self.naming.strings.get_or_insert(pin.as_ref());
        self.naming
            .bels
            .get_mut(&bel)
            .unwrap()
            .wires
            .insert(pin, wire);
    }

    pub fn extract_bel_cell(&mut self, bcrd: BelCoord, cell: CellCoord) -> BelExtractor<'dev, '_> {
        let intdb = self.intdb;
        let bcid = intdb.bel_class(bcrd.slot);
        BelExtractor {
            ctx: self,
            intdb,
            bcid,
            bcrd,
            cell,
            bel: Bel::default(),
            discard: false,
        }
    }

    pub fn extract_bel(&mut self, bcrd: BelCoord) -> BelExtractor<'dev, '_> {
        self.extract_bel_cell(bcrd, bcrd.cell)
    }

    pub fn insert_bel_generic(&mut self, bcrd: BelCoord, bel: BelInfo) {
        let tcrd = self.edev.bel_tile(bcrd);
        match self.bels.entry((self.edev[tcrd].class, bcrd.slot)) {
            btree_map::Entry::Vacant(e) => {
                e.insert(bel);
            }
            btree_map::Entry::Occupied(e) => {
                assert_eq!(*e.get(), bel);
            }
        }
    }

    pub fn insert_bel(&mut self, bcrd: BelCoord, bel: Bel) {
        self.insert_bel_generic(bcrd, BelInfo::Bel(bel));
    }

    pub fn insert_switchbox(&mut self, bcrd: BelCoord, sb: SwitchBox) {
        self.insert_bel_generic(bcrd, BelInfo::SwitchBox(sb));
    }
}
