use std::collections::{BTreeMap, BTreeSet};

use prjcombine_entity::{EntityBundleIndices, EntityBundleItemIndex};
use prjcombine_interconnect::{
    db::{
        Bel, BelClassId, BelInput, BelInputId, BelOutputId, IntDb, Mux, PermaBuf, SwitchBox,
        SwitchBoxItem, TileWireCoord,
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
                        && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.bel.inputs.insert(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        let vi = input.indexing.phys_to_virt(pi);
                        let iname = &format!("{name}{vi}");
                        if let Some(&wire) = wires.get(iname)
                            && let Some(bpin) = self.ctx.try_xlat_input(self.bcrd, wire, true)
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
                        && let Some(bpin) = self.ctx.try_xlat_output(self.bcrd, wire, true)
                    {
                        self.ctx.add_bel_wire(self.bcrd, name, wire);
                        self.bel.outputs.insert(id, bpin);
                    }
                }
                EntityBundleIndices::Array(range) => {
                    for (pi, id) in range.into_iter().enumerate() {
                        let vi = output.indexing.phys_to_virt(pi);
                        let iname = &format!("{name}{vi}");
                        if let Some(&wire) = wires.get(iname)
                            && let Some(bpin) = self.ctx.try_xlat_output(self.bcrd, wire, true)
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

    pub fn input(&mut self, pin: BelInputId, wire: TileWireCoord) -> &mut Self {
        self.bel.inputs.insert(pin, BelInput::Fixed(wire.pos()));
        self
    }

    pub fn xlat_input(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_input(self.bcrd, wire);
        self.bel.inputs.insert(pin, bpin);
        self
    }

    pub fn xlat_name_input(&mut self, pin: BelInputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].inputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, wire);
        self.xlat_input(pin, wire)
    }

    pub fn xlat_name_input_rc_io(&mut self, pin: BelInputId, suffix: impl AsRef<str>) -> &mut Self {
        let wire = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.xlat_name_input(pin, wire)
    }

    pub fn output(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs.insert(pin, BTreeSet::from_iter([wire]));
        self
    }

    pub fn xlat_output(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output(self.bcrd, wire);
        self.bel.outputs.insert(pin, bpin);
        self
    }

    pub fn xlat_output_filter(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let bpin = self.ctx.xlat_output_filter(self.bcrd, wire);
        self.bel.outputs.insert(pin, bpin);
        self
    }

    pub fn xlat_name_output(&mut self, pin: BelOutputId, wire: WireName) -> &mut Self {
        let pname = match self.ctx.intdb[self.bcid].outputs.key(pin) {
            (pname, EntityBundleItemIndex::Single) => pname.to_string(),
            (pname, EntityBundleItemIndex::Array { index, .. }) => format!("{pname}{index}"),
        };
        self.add_wire(pname, wire);
        self.xlat_output(pin, wire)
    }

    pub fn xlat_name_output_rc_io(
        &mut self,
        pin: BelOutputId,
        suffix: impl AsRef<str>,
    ) -> &mut Self {
        let wire = self.ctx.rc_io_wire(self.cell, suffix.as_ref());
        self.xlat_name_output(pin, wire)
    }

    pub fn output_extra(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs[pin].insert(wire);
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
    pub inputs: BTreeMap<WireName, TileWireCoord>,
    pub outputs: BTreeMap<WireName, TileWireCoord>,
    pub edges: Vec<(TileWireCoord, TileWireCoord)>,
    pub extra: Vec<SwitchBoxItem>,
}

impl<'dev, 'ctx> SwitchBoxExtractor<'dev, 'ctx> {
    pub fn input(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_no_claim(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_xlat(&mut self, name: WireName) -> &mut Self {
        let wire = self.ctx.xlat_input(self.bcrd, name).wire();
        self.input(name, wire)
    }

    pub fn input_io(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let rwire = self
            .ctx
            .edev
            .tile_wire(self.ctx.edev.bel_tile(self.bcrd), wire);
        let name_io = self.ctx.get_paddi_wire(rwire);
        self.ctx.claim_pip(name, name_io);
        self.input(name, wire)
    }

    pub fn output(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
        self.inputs.insert(name, wire);
        self
    }

    pub fn output_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self
            .ctx
            .edev
            .resolve_tile_wire(self.ctx.edev.bel_tile(self.bcrd), wire)
            .unwrap();
        let name = self.ctx.naming.interconnect[&gwire];
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
        self
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
