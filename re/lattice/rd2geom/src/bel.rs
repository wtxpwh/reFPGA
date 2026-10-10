use std::collections::{BTreeMap, BTreeSet};

use prjcombine_entity::{EntityBundleIndices, EntityBundleItemIndex};
use prjcombine_interconnect::{
    db::{
        Bel, BelClassId, BelInput, BelInputId, BelOutputId, CellSlotId, IntDb, Mux, PermaBuf,
        SwitchBox, SwitchBoxItem, TileWireCoord,
    },
    grid::{BelCoord, CellCoord},
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

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
                        self.bel.inputs.insert(id, bpin);
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
                        if let Some(&wire) = wires.get(iname)
                            && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true, None)
                        {
                            self.ctx.add_bel_wire(self.bcrd, iname, wire);
                            self.bel.inputs.insert(id, bpin);
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
                        self.bel.outputs.insert(id, bpin);
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
                            self.bel.outputs.insert(id, bpin);
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

    pub fn input_raw(&mut self, pin: BelInputId, wire: TileWireCoord) -> &mut Self {
        self.bel.inputs.insert(pin, BelInput::Fixed(wire.pos()));
        self
    }

    pub fn input_xlat_no_name(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_input(self.bcrd, wire);
        self.bel.inputs.insert(pin, bpin);
        self
    }

    pub fn input_xlat(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, wire);
        self.input_xlat_no_name(pin, wire)
    }

    pub fn input_xlat_rc(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        let wire = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.input_xlat(pin, wire)
    }

    pub fn input_xlat_rc_io(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        let wire = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.input_xlat(pin, wire)
    }

    pub fn output(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs.insert(pin, BTreeSet::from_iter([wire]));
        self
    }

    pub fn output_xlat_no_name(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output(self.bcrd, wire);
        self.bel.outputs.insert(pin, bpin);
        self
    }

    pub fn output_xlat_quiet_no_name(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output_quiet(self.bcrd, wire);
        self.bel.outputs.insert(pin, bpin);
        self
    }

    pub fn output_xlat(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].outputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, wire);
        self.output_xlat_no_name(pin, wire)
    }

    pub fn output_xlat_rc(&mut self, pin: BelOutputId, suffix: impl AsRef<str>) -> &mut Self {
        let wire = self.ctx.rc_wire(self.cell, suffix.as_ref());
        self.output_xlat(pin, wire)
    }

    pub fn output_xlat_rc_io(&mut self, pin: BelOutputId, suffix: impl AsRef<str>) -> &mut Self {
        let wire = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.output_xlat(pin, wire)
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

    pub fn input_xlat_rc_cell(&mut self, cell: CellCoord, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.input_xlat(name)
    }

    pub fn input_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_cell(self.cell, name)
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
        let mut sb = SwitchBox {
            items: core::mem::take(&mut self.extra),
        };
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
            sb.items.push(SwitchBoxItem::Mux(mux));
        }
        sb.items.sort();
        self.ctx.insert_switchbox(self.bcrd, sb);
    }
}
