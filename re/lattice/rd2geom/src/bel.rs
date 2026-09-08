use std::collections::{BTreeMap, BTreeSet};

use prjcombine_entity::EntityBundleIndices;
use prjcombine_interconnect::{
    db::{
        Bel, BelClassId, BelInput, BelInputId, BelOutputId, IntDb, Mux, SwitchBox, SwitchBoxItem,
        TileWireCoord,
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

    pub fn input(&mut self, pin: BelInputId, wire: TileWireCoord) -> &mut Self {
        self.bel.inputs.insert(pin, BelInput::Fixed(wire.pos()));
        self
    }

    pub fn output(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs.insert(pin, BTreeSet::from_iter([wire]));
        self
    }

    pub fn output_extra(&mut self, pin: BelOutputId, wire: TileWireCoord) -> &mut Self {
        self.bel.outputs[pin].insert(wire);
        self
    }
}

impl<'dev, 'ctx> Drop for BelExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        self.ctx
            .insert_bel(self.bcrd, std::mem::take(&mut self.bel));
    }
}

pub struct SwitchBoxExtractor<'dev, 'ctx> {
    pub ctx: &'ctx mut ChipContext<'dev>,
    pub bcrd: BelCoord,
    pub inputs: BTreeMap<WireName, TileWireCoord>,
    pub outputs: BTreeMap<WireName, TileWireCoord>,
}

impl<'dev, 'ctx> SwitchBoxExtractor<'dev, 'ctx> {
    pub fn input(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.inputs.insert(name, wire);
        self
    }

    pub fn input_xlat(&mut self, name: WireName) -> &mut Self {
        let wire = self.ctx.xlat_input(self.bcrd, name).wire();
        self.input(name, wire)
    }

    pub fn output(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs.insert(name, wire);
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
}

impl<'dev, 'ctx> Drop for SwitchBoxExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        let mut sb = SwitchBox::default();
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
            }
            sb.items.push(SwitchBoxItem::Mux(mux));
        }
        self.ctx.insert_switchbox(self.bcrd, sb);
    }
}
