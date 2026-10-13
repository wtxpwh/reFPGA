use std::collections::{BTreeMap, BTreeSet, btree_map};

use prjcombine_entity::{EntityBundleIndices, EntityBundleItemIndex};
use prjcombine_interconnect::{
    db::{
        Bel, BelClassId, BelInfo, BelInput, BelInputId, BelOutputId, CellSlotId, IntDb, LegacyBel,
        Mux, PermaBuf, SwitchBox, SwitchBoxItem, TileWireCoord,
    },
    grid::{BelCoord, CellCoord},
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
                    if let Some(&wire) = wires.get(name)
                        && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true, None)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.input_raw(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        let vi = input.indexing.phys_to_virt(pi);
                        let iname = &if name.ends_with(|c: char| c.is_ascii_digit()) {
                            format!("{name}_{vi}")
                        } else {
                            format!("{name}{vi}")
                        };
                        let mut wire = wires.get(iname);
                        if range.len() == 1 && wire.is_none() {
                            wire = wires.get(name);
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
                    if let Some(&wire) = wires.get(name)
                        && let Some(bpin) = self.ctx.try_xlat_output(self.bcrd, wire, false, true)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.output_raw(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        let vi = output.indexing.phys_to_virt(pi);
                        let iname = &if name.ends_with(|c: char| c.is_ascii_digit()) {
                            format!("{name}_{vi}")
                        } else {
                            format!("{name}{vi}")
                        };
                        if let Some(&wire) = wires.get(iname)
                            && let Some(bpin) =
                                self.ctx.try_xlat_output(self.bcrd, wire, false, true)
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

    pub fn name_rc(&mut self, prefix: impl AsRef<str>) -> &mut Self {
        let prefix = prefix.as_ref();
        let (r, c) = self.ctx.rc(self.cell);
        self.name(format!("{prefix}_R{r}C{c}"))
    }

    pub fn input_raw(&mut self, pin: BelInputId, wire: BelInput) -> &mut Self {
        if self.bel.inputs.contains_id(pin) {
            assert_eq!(self.bel.inputs[pin], wire);
        } else {
            self.bel.inputs.insert(pin, wire);
        }
        self
    }

    pub fn input_xlat_no_name(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_input(self.bcrd, wire);
        self.input_raw(pin, bpin)
    }

    pub fn input_xlat(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, wire);
        self.input_xlat_no_name(pin, wire)
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

    pub fn input_xlat_rename(
        &mut self,
        pin: BelInputId,
        pname: impl AsRef<str>,
        wire: WireName,
    ) -> &mut Self {
        self.add_wire(pname, wire);
        self.input_xlat_no_name(pin, wire)
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

    pub fn input_io(&mut self, pin: BelInputId, name: WireName, wire: TileWireCoord) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, name);
        let rwire = self.ctx.edev.bel_wire(self.bcrd, wire);
        let name_io = self.ctx.get_paddi_wire(rwire);
        self.ctx.claim_pip(name, name_io);
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

    pub fn output_bind_rc_io(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let wname = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.output_bind(pin, wname, wire)
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

    pub fn output_extra(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs[pin].insert(wire);
        self
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

    pub fn discard(mut self) {
        self.discard = true;
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

pub struct SwitchBoxExtractor<'dev, 'ctx> {
    pub ctx: &'ctx mut ChipContext<'dev>,
    pub bcrd: BelCoord,
    pub cell: CellCoord,
    pub inputs: BTreeMap<WireName, TileWireCoord>,
    pub outputs: BTreeMap<WireName, TileWireCoord>,
    pub outputs_permabuf: BTreeMap<WireName, TileWireCoord>,
    pub edges: Vec<(TileWireCoord, TileWireCoord)>,
    pub extra: Vec<SwitchBoxItem>,
}

impl<'dev, 'ctx> SwitchBoxExtractor<'dev, 'ctx> {
    pub fn cell(&mut self, cell: CellCoord) -> &mut Self {
        self.cell = cell;
        self
    }

    pub fn input_raw(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_raw_no_claim(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_bind(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.bind_wire(name, wire);
        self.input_raw(name, wire)
    }

    pub fn input_bind_rc(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        let name = self.ctx.rc_wire(self.cell, name.as_ref());
        self.input_bind(name, wire)
    }

    pub fn input_bind_rc_io(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        let name = self.ctx.rc_io_wire(self.cell, name.as_ref());
        self.input_bind(name, wire)
    }

    pub fn input_bind_no_claim(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.bind_wire(name, wire);
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_xlat(&mut self, name: WireName) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.input_raw_no_claim(name, tw)
        } else {
            let wire = self.ctx.xlat_input(self.bcrd, name).wire();
            self.input_raw(name, wire)
        }
    }

    pub fn input_hint(&mut self, name: WireName, tw: TileWireCoord) -> &mut Self {
        let wire = self.ctx.io_int_wires[&name][0];
        assert_eq!(self.ctx.edev.resolve_bel_wire(self.bcrd, tw).unwrap(), wire);
        self.input_raw_no_claim(name, tw)
    }

    pub fn input_xlat_rc_mode_cell(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        mode: RcMode,
    ) -> &mut Self {
        let name = self.ctx.rc_wire_mode(cell, name.as_ref(), mode);
        self.input_xlat(name)
    }

    pub fn input_xlat_rc_mode(&mut self, name: impl AsRef<str>, mode: RcMode) -> &mut Self {
        self.input_xlat_rc_mode_cell(self.cell, name, mode)
    }

    pub fn input_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(name, RcMode::Base)
    }

    pub fn input_xlat_rc_io(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(name, RcMode::Io)
    }

    pub fn input_xlat_nudge(&mut self, name: WireName, cslot: CellSlotId) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self
                .ctx
                .wire_to_tw(self.bcrd, wires[0], Some(cslot))
                .unwrap();
            self.input_raw_no_claim(name, tw)
        } else {
            let wire = self.ctx.xlat_input_nudge(self.bcrd, name, cslot).wire();
            self.input_raw(name, wire)
        }
    }

    pub fn input_xlat_rc_cell_nudge(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        cslot: CellSlotId,
    ) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.input_xlat_nudge(name, cslot)
    }

    pub fn input_xlat_rc_nudge(&mut self, name: impl AsRef<str>, cslot: CellSlotId) -> &mut Self {
        self.input_xlat_rc_cell_nudge(self.cell, name, cslot)
    }

    pub fn input_io(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let rwire = self.ctx.edev.bel_wire(self.bcrd, wire);
        let name_io = self.ctx.get_paddi_wire(rwire);
        self.ctx.claim_pip(name, name_io);
        self.input_raw(name, wire)
    }

    pub fn input_io_rc_cell(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.input_io(name, wire)
    }

    pub fn input_io_rc(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        self.input_io_rc_cell(self.cell, name, wire)
    }

    pub fn input_io_rc_io_cell(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let name = self.ctx.rc_io_wire(cell, name.as_ref());
        self.input_io(name, wire)
    }

    pub fn input_io_rc_io(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        self.input_io_rc_io_cell(self.cell, name, wire)
    }

    pub fn bind_wire(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let fw = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        self.ctx.bind_wire(name, fw);
        self
    }

    pub fn output_raw_no_claim(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
        self
    }

    pub fn output_raw(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
        self
    }

    pub fn output_bind(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.bind_wire(name, wire);
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
        self.inputs.insert(name, wire);
        self
    }

    pub fn output_bind_rc(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        let name = self.ctx.rc_wire(self.cell, name.as_ref());
        self.output_bind(name, wire)
    }

    pub fn output_bind_rc_io(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        let name = self.ctx.rc_io_wire(self.cell, name.as_ref());
        self.output_bind(name, wire)
    }

    pub fn output_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        let name = self.ctx.naming.interconnect[&gwire];
        self.output_raw_no_claim(name, wire)
    }

    pub fn output_xlat(&mut self, name: WireName) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.output_raw(name, tw)
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            self.output_raw(name, wires.into_iter().next().unwrap())
        }
    }

    pub fn output_xlat_no_claim(&mut self, name: WireName) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.output_raw_no_claim(name, tw)
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            self.output_raw_no_claim(name, wires.into_iter().next().unwrap())
        }
    }

    pub fn output_xlat_rc_cell(&mut self, cell: CellCoord, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.output_xlat(name)
    }

    pub fn output_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.output_xlat_rc_cell(self.cell, name)
    }

    pub fn permabuf_output_raw(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs_permabuf.insert(name, wire);
        self
    }

    pub fn permabuf_output_raw_no_claim(
        &mut self,
        name: WireName,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.outputs_permabuf.insert(name, wire);
        self
    }

    pub fn permabuf_output_xlat(&mut self, name: WireName) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.permabuf_output_raw(name, tw)
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            self.permabuf_output_raw(name, wires.into_iter().next().unwrap())
        }
    }

    pub fn permabuf_output_xlat_no_claim(&mut self, name: WireName) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap();
            self.permabuf_output_raw_no_claim(name, tw)
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            self.permabuf_output_raw_no_claim(name, wires.into_iter().next().unwrap())
        }
    }

    pub fn permabuf_output_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(self.cell, name.as_ref());
        self.permabuf_output_xlat(name)
    }

    pub fn edge(&mut self, dst: TileWireCoord, src: TileWireCoord) -> &mut Self {
        self.edges.push((dst, src));
        self
    }

    pub fn add_wire(&mut self, pin: impl AsRef<str>, wire: WireName) -> &mut Self {
        self.ctx.add_bel_wire(self.bcrd, pin, wire);
        self
    }

    pub fn add_wire_no_claim(&mut self, pin: impl AsRef<str>, wire: WireName) -> &mut Self {
        self.ctx.add_bel_wire_no_claim(self.bcrd, pin, wire);
        self
    }

    pub fn permabuf_int_io(&mut self, dst: TileWireCoord, src: TileWireCoord) -> &mut Self {
        let wname_io = self
            .ctx
            .get_paddi_wire(self.ctx.edev.bel_wire(self.bcrd, src));
        let wire_dst = self.ctx.edev.bel_wire(self.bcrd, dst);
        self.ctx.claim_pip_int_out(wire_dst, wname_io);
        self.permabuf(dst, src)
    }

    pub fn permabuf(&mut self, dst: TileWireCoord, src: TileWireCoord) -> &mut Self {
        self.extra.push(SwitchBoxItem::PermaBuf(PermaBuf {
            dst,
            src: src.pos(),
        }));
        self
    }
}

impl<'dev, 'ctx> Drop for SwitchBoxExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        let mut items = core::mem::take(&mut self.extra);
        let mut muxes = BTreeMap::new();
        for (&name_out, &wire_out) in &self.outputs {
            let mut mux = Mux {
                dst: wire_out,
                bits: vec![],
                src: Default::default(),
                bits_off: None,
            };
            for name_in in self.ctx.pips_bwd[&name_out].clone() {
                let Some(&wire_in) = self.inputs.get(&name_in) else {
                    continue;
                };
                self.ctx.claim_pip(name_out, name_in);
                mux.src.insert(wire_in.pos(), Default::default());
                self.ctx
                    .naming
                    .bels
                    .get_mut(&self.bcrd)
                    .unwrap()
                    .int_inputs
                    .insert((wire_out, wire_in), name_in);
            }
            muxes.insert(wire_out, mux);
        }
        for &(dst, src) in &self.edges {
            muxes
                .entry(dst)
                .or_insert_with(|| Mux {
                    dst,
                    bits: vec![],
                    src: Default::default(),
                    bits_off: None,
                })
                .src
                .insert(src.pos(), Default::default());
        }
        for mux in muxes.into_values() {
            items.push(SwitchBoxItem::Mux(mux));
        }
        for (&name_out, &wire_out) in &self.outputs_permabuf {
            let name_in = self.ctx.claim_single_in(name_out);
            let wire_in = self.inputs[&name_in];
            items.push(SwitchBoxItem::PermaBuf(PermaBuf {
                dst: wire_out,
                src: wire_in.pos(),
            }));
            self.ctx
                .naming
                .bels
                .get_mut(&self.bcrd)
                .unwrap()
                .int_inputs
                .insert((wire_out, wire_in), name_in);
        }
        self.ctx
            .switchboxes
            .entry(self.bcrd)
            .or_default()
            .extend(items);
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

    pub fn add_sb_wire(&mut self, bel: BelCoord, tw: TileWireCoord, wire: WireName) {
        self.add_sb_wire_no_claim(bel, tw, wire);
        self.claim_node(wire);
    }

    pub fn add_sb_wire_no_claim(&mut self, bel: BelCoord, tw: TileWireCoord, wire: WireName) {
        self.naming
            .bels
            .get_mut(&bel)
            .unwrap()
            .int_wires
            .insert(tw, wire);
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

    pub fn extract_switchbox(&mut self, bcrd: BelCoord) -> SwitchBoxExtractor<'dev, '_> {
        self.extract_switchbox_names::<&str>(bcrd, [])
    }

    pub fn extract_switchbox_names<T: AsRef<str>>(
        &mut self,
        bcrd: BelCoord,
        names: impl IntoIterator<Item = T>,
    ) -> SwitchBoxExtractor<'dev, '_> {
        if !self.switchboxes.contains_key(&bcrd) {
            self.name_bel(bcrd, names);
        }
        SwitchBoxExtractor {
            ctx: self,
            bcrd,
            cell: bcrd.cell,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            outputs_permabuf: BTreeMap::new(),
            edges: vec![],
            extra: vec![],
        }
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

    pub fn insert_legacy_bel(&mut self, bcrd: BelCoord, bel: LegacyBel) {
        self.insert_bel_generic(bcrd, BelInfo::Legacy(bel));
    }

    pub fn insert_switchbox(&mut self, bcrd: BelCoord, sb: SwitchBox) {
        self.insert_bel_generic(bcrd, BelInfo::SwitchBox(sb));
    }

    pub fn extract_simple_legacy_bel(
        &mut self,
        bcrd: BelCoord,
        cell: CellCoord,
        suffix: &'static str,
    ) -> LegacyBel {
        let wires = self.sorted_wires[&(cell, suffix)].clone();
        let mut bel = LegacyBel::default();
        for (pin, wire) in wires {
            if suffix == "DLLDEL" && pin == "A" {
                continue;
            }
            let Some(bpin) = self.try_xlat_int_wire(bcrd, wire) else {
                continue;
            };
            self.add_bel_wire(bcrd, &pin, wire);
            bel.pins.insert(pin, bpin);
        }
        bel
    }

    pub fn insert_simple_legacy_bel(
        &mut self,
        bcrd: BelCoord,
        cell: CellCoord,
        suffix: &'static str,
    ) {
        let bel = self.extract_simple_legacy_bel(bcrd, cell, suffix);
        self.insert_legacy_bel(bcrd, bel);
    }
}
