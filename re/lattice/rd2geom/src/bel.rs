use prjcombine_entity::EntityBundleIndices;
use prjcombine_interconnect::{
    db::{Bel, BelClassId, IntDb},
    grid::{BelCoord, CellCoord},
};

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
}

impl<'dev, 'ctx> Drop for BelExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        self.ctx
            .insert_bel(self.bcrd, std::mem::take(&mut self.bel));
    }
}
