use std::collections::{BTreeMap, BTreeSet, btree_map};

use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bslots, wires},
};
use prjcombine_interconnect::{
    db::{
        BelSlotId, CellSlotId, IntDb, Mux, PermaBuf, ProgBuf, SwitchBox, SwitchBoxItem,
        TileClassId, TileWireCoord,
    },
    grid::{BelCoord, CellCoord},
};
use prjcombine_re_lattice_naming::WireName;
use prjcombine_types::bitrect::PolTileBit;

use crate::{ChipContext, RcMode};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeKind {
    Mux,
    PermaBuf,
    ProgBuf,
}

pub struct SwitchBoxData {
    pub tcid: TileClassId,
    pub bslot: BelSlotId,
    pub edges: BTreeSet<(TileWireCoord, TileWireCoord, EdgeKind)>,
    pub seen_cells: BTreeSet<CellSlotId>,
    pub missing_inputs: BTreeSet<TileWireCoord>,
}

impl SwitchBoxData {
    pub fn merge(&mut self, other: &SwitchBoxData, intdb: &IntDb) {
        for &(dst, src, kind) in &self.edges {
            if !other.edges.contains(&(dst, src, kind))
                && other.seen_cells.contains(&dst.cell)
                && other.seen_cells.contains(&src.cell)
                && !other.missing_inputs.contains(&src)
            {
                println!(
                    "MERGE FAILED at {tile}.{bel}: {dst} {src} {kind:?} in L only",
                    tile = intdb[self.tcid].name,
                    bel = intdb[self.bslot].name,
                    dst = dst.to_string(intdb, &intdb[self.tcid]),
                    src = src.to_string(intdb, &intdb[self.tcid]),
                );
            }
        }
        for &(dst, src, kind) in &other.edges {
            if !self.edges.contains(&(dst, src, kind))
                && self.seen_cells.contains(&dst.cell)
                && self.seen_cells.contains(&src.cell)
                && !self.missing_inputs.contains(&src)
            {
                println!(
                    "MERGE FAILED at {tile}.{bel}: {dst} {src} {kind:?} in R only",
                    tile = intdb[self.tcid].name,
                    bel = intdb[self.bslot].name,
                    dst = dst.to_string(intdb, &intdb[self.tcid]),
                    src = src.to_string(intdb, &intdb[self.tcid]),
                );
            }
            self.edges.insert((dst, src, kind));
        }
        self.seen_cells.extend(other.seen_cells.iter().copied());
        self.missing_inputs
            .retain(|&src| other.missing_inputs.contains(&src));
    }

    pub fn finish(&self) -> SwitchBox {
        let mut items = vec![];
        let mut muxes = BTreeMap::new();
        for &(dst, src, kind) in &self.edges {
            match kind {
                EdgeKind::Mux => {
                    muxes
                        .entry(dst)
                        .or_insert(Mux {
                            dst,
                            bits: Default::default(),
                            src: Default::default(),
                            bits_off: Default::default(),
                        })
                        .src
                        .insert(src.pos(), Default::default());
                }
                EdgeKind::PermaBuf => {
                    items.push(SwitchBoxItem::PermaBuf(PermaBuf {
                        dst,
                        src: src.pos(),
                    }));
                }
                EdgeKind::ProgBuf => {
                    items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                        dst,
                        src: src.pos(),
                        bit: PolTileBit::DUMMY,
                    }));
                }
            }
        }
        for mux in muxes.into_values() {
            items.push(SwitchBoxItem::Mux(mux));
        }
        items.sort();
        SwitchBox { items }
    }
}

pub struct SwitchBoxExtractor<'dev, 'ctx> {
    pub ctx: &'ctx mut ChipContext<'dev>,
    pub bcrd: BelCoord,
    pub cell: CellCoord,
    pub inputs: BTreeMap<WireName, TileWireCoord>,
    pub outputs: BTreeMap<WireName, (TileWireCoord, EdgeKind)>,
    pub edges: BTreeSet<(TileWireCoord, TileWireCoord, EdgeKind)>,
    pub missing_inputs: BTreeSet<TileWireCoord>,
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

    pub fn input_bind_rc_corner(
        &mut self,
        name: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let name = self.ctx.rc_corner_wire(self.cell, name.as_ref());
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
            self.ctx.bind_wire(
                name,
                self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap(),
            );
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

    pub fn input_xlat_rc_corner(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.input_xlat_rc_mode(name, RcMode::Corner)
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

    pub fn input_xlat_nudge_rebind(&mut self, name: WireName, cslot: CellSlotId) -> &mut Self {
        if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            let tw = self
                .ctx
                .wire_to_tw(self.bcrd, wires[0], Some(cslot))
                .unwrap();
            self.input_raw_no_claim(name, tw)
        } else {
            let wire = self.ctx.xlat_input_nudge(self.bcrd, name, cslot).wire();
            self.ctx
                .bind_wire(name, self.ctx.edev.bel_wire(self.bcrd, wire));
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

    pub fn input_xlat_rc_cell_nudge_rebind(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        cslot: CellSlotId,
    ) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.input_xlat_nudge_rebind(name, cslot)
    }

    pub fn input_xlat_rc_nudge_rebind(
        &mut self,
        name: impl AsRef<str>,
        cslot: CellSlotId,
    ) -> &mut Self {
        self.input_xlat_rc_cell_nudge_rebind(self.cell, name, cslot)
    }

    pub fn input_io(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let rwire = self.ctx.edev.bel_wire(self.bcrd, wire);
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

    pub fn input_io_rc_corner_cell(
        &mut self,
        cell: CellCoord,
        name: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        let name = self.ctx.rc_corner_wire(cell, name.as_ref());
        self.input_io(name, wire)
    }

    pub fn input_io_rc_corner(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        self.input_io_rc_corner_cell(self.cell, name, wire)
    }

    pub fn input_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        let name = self.ctx.naming.interconnect[&gwire];
        self.input_raw_no_claim(name, wire)
    }

    pub fn input_int_no_resolve(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.bel_wire(self.bcrd, wire);
        let name = self.ctx.naming.interconnect[&gwire];
        self.input_raw_no_claim(name, wire)
    }

    pub fn input_io_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.bel_wire(self.bcrd, wire);
        let name = self.ctx.io_int_names[&gwire];
        self.input_raw_no_claim(name, wire)
    }

    pub fn bind_wire(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        let fw = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        self.ctx.bind_wire(name, fw);
        self
    }

    pub fn output_raw_no_claim(
        &mut self,
        name: WireName,
        wire: TileWireCoord,
        kind: EdgeKind,
    ) -> &mut Self {
        self.ctx.add_sb_wire_no_claim(self.bcrd, wire, name);
        self.outputs.insert(name, (wire, kind));
        self
    }

    pub fn output_raw(&mut self, name: WireName, wire: TileWireCoord, kind: EdgeKind) -> &mut Self {
        self.ctx.add_sb_wire(self.bcrd, wire, name);
        self.outputs.insert(name, (wire, kind));
        self
    }

    pub fn output_bind(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.bind_wire(name, wire);
        self.output_raw(name, wire, EdgeKind::Mux);
        self.inputs.insert(name, wire);
        self
    }

    pub fn progbuf_output_bind(&mut self, name: WireName, wire: TileWireCoord) -> &mut Self {
        self.bind_wire(name, wire);
        self.output_raw(name, wire, EdgeKind::ProgBuf);
        self.inputs.insert(name, wire);
        self
    }

    pub fn output_bind_rc_mode(
        &mut self,
        name: impl AsRef<str>,
        mode: RcMode,
        wire: TileWireCoord,
    ) -> &mut Self {
        let name = self.ctx.rc_wire_mode(self.cell, name.as_ref(), mode);
        self.output_bind(name, wire)
    }

    pub fn output_bind_rc(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        self.output_bind_rc_mode(name, RcMode::Base, wire)
    }

    pub fn output_bind_rc_io(&mut self, name: impl AsRef<str>, wire: TileWireCoord) -> &mut Self {
        self.output_bind_rc_mode(name, RcMode::Io, wire)
    }

    pub fn output_bind_rc_corner(
        &mut self,
        name: impl AsRef<str>,
        wire: TileWireCoord,
    ) -> &mut Self {
        self.output_bind_rc_mode(name, RcMode::Corner, wire)
    }

    pub fn output_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        let name = self.ctx.naming.interconnect[&gwire];
        self.output_raw_no_claim(name, wire, EdgeKind::Mux)
    }

    pub fn progbuf_output_int(&mut self, wire: TileWireCoord) -> &mut Self {
        let gwire = self.ctx.edev.resolve_bel_wire(self.bcrd, wire).unwrap();
        let name = self.ctx.naming.interconnect[&gwire];
        self.output_raw_no_claim(name, wire, EdgeKind::ProgBuf)
    }

    pub fn output_xlat(&mut self, name: WireName) -> &mut Self {
        let tw = if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap()
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            wires.into_iter().next().unwrap()
        };
        self.output_raw(name, tw, EdgeKind::Mux)
    }

    pub fn output_xlat_nudge(&mut self, name: WireName, cslot: CellSlotId) -> &mut Self {
        let tw = if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            self.ctx
                .wire_to_tw(self.bcrd, wires[0], Some(cslot))
                .unwrap()
        } else {
            let wires = self.ctx.xlat_output_nudge(self.bcrd, name, cslot);
            assert_eq!(wires.len(), 1);
            wires.into_iter().next().unwrap()
        };
        self.output_raw(name, tw, EdgeKind::Mux)
    }

    pub fn output_xlat_no_claim(&mut self, name: WireName) -> &mut Self {
        let tw = if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap()
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            wires.into_iter().next().unwrap()
        };
        self.output_raw_no_claim(name, tw, EdgeKind::Mux)
    }

    pub fn output_xlat_rc_cell(&mut self, cell: CellCoord, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(cell, name.as_ref());
        self.output_xlat(name)
    }

    pub fn output_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.output_xlat_rc_cell(self.cell, name)
    }

    pub fn output_xlat_rc_io_cell(&mut self, cell: CellCoord, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_io_wire(cell, name.as_ref());
        self.output_xlat(name)
    }

    pub fn output_xlat_rc_io(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.output_xlat_rc_io_cell(self.cell, name)
    }

    pub fn permabuf_output_xlat(&mut self, name: WireName) -> &mut Self {
        let tw = if let Some(wires) = self.ctx.io_int_wires.get(&name) {
            self.ctx.wire_to_tw(self.bcrd, wires[0], None).unwrap()
        } else {
            let wires = self.ctx.xlat_output(self.bcrd, name);
            assert_eq!(wires.len(), 1);
            wires.into_iter().next().unwrap()
        };
        self.output_raw(name, tw, EdgeKind::PermaBuf)
    }

    pub fn permabuf_output_xlat_rc(&mut self, name: impl AsRef<str>) -> &mut Self {
        let name = self.ctx.rc_wire(self.cell, name.as_ref());
        self.permabuf_output_xlat(name)
    }

    pub fn edge(&mut self, dst: TileWireCoord, src: TileWireCoord) -> &mut Self {
        self.edges.insert((dst, src, EdgeKind::Mux));
        self
    }

    pub fn edge_progbuf(&mut self, dst: TileWireCoord, src: TileWireCoord) -> &mut Self {
        self.edges.insert((dst, src, EdgeKind::ProgBuf));
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
        self.edges.insert((dst, src, EdgeKind::PermaBuf));
        self
    }

    pub fn missing_input(&mut self, src: TileWireCoord) -> &mut Self {
        self.missing_inputs.insert(src);
        self
    }

    pub fn flush_edges(&mut self) -> &mut Self {
        for (&name_out, &(wire_out, kind)) in &self.outputs {
            for name_in in self.ctx.pips_bwd[&name_out].clone() {
                let Some(&wire_in) = self.inputs.get(&name_in) else {
                    continue;
                };
                self.ctx.claim_pip(name_out, name_in);
                self.edges.insert((wire_out, wire_in, kind));
                self.ctx
                    .naming
                    .bels
                    .get_mut(&self.bcrd)
                    .unwrap()
                    .int_inputs
                    .insert((wire_out, wire_in), name_in);
            }
        }
        self.outputs.clear();
        self.inputs.clear();
        self
    }
}

impl<'dev, 'ctx> Drop for SwitchBoxExtractor<'dev, 'ctx> {
    fn drop(&mut self) {
        self.flush_edges();
        let edges = core::mem::take(&mut self.edges);
        match self.ctx.switchboxes.entry(self.bcrd) {
            btree_map::Entry::Vacant(e) => {
                let tile = &self.ctx.edev[self.ctx.edev.bel_tile(self.bcrd)];
                e.insert(SwitchBoxData {
                    tcid: tile.class,
                    bslot: self.bcrd.slot,
                    edges,
                    seen_cells: BTreeSet::from_iter(tile.cells.ids()),
                    missing_inputs: core::mem::take(&mut self.missing_inputs),
                });
            }
            btree_map::Entry::Occupied(mut e) => {
                e.get_mut().edges.extend(edges);
                e.get_mut()
                    .missing_inputs
                    .extend(core::mem::take(&mut self.missing_inputs));
            }
        }
    }
}

impl<'dev> ChipContext<'dev> {
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
            edges: BTreeSet::new(),
            missing_inputs: BTreeSet::new(),
        }
    }
}
