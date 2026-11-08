#![allow(clippy::too_many_arguments)]

use bimap::BiHashMap;
use prjcombine_entity::{EntityId, EntityIdArith, EntityPartVec, EntityVec, id::EntityRange};
use std::collections::{HashMap, HashSet};

use crate::{
    coord::{
        BelCoord, CellCoord, ColId, ConnectorCoord, DieId, DieIdExt, PolWireCoord, Rect, RowId,
        TileCoord, WireCoord,
    },
    db::{
        BelBidirId, BelInfo, BelInput, BelInputId, BelOutputId, BelSlotId, CellSlotId,
        ConnectorClassId, ConnectorSlotId, ConnectorWire, IntDb, IntDbIndex, RegionSlotId,
        TileClassId, TileSlotId, TileWireCoord, WireKind,
    },
};

pub mod builder;

#[derive(Clone, Debug)]
pub struct ExpandedGrid<'a> {
    pub db: &'a IntDb,
    pub db_index: IntDbIndex,
    pub die: EntityVec<DieId, ExpandedDie>,
    pub extra_conns: BiHashMap<WireCoord, WireCoord>,
    pub blackhole_wires: HashSet<WireCoord>,
    pub tile_index: EntityPartVec<TileClassId, Vec<TileCoord>>,
    pub region_root_cells: EntityPartVec<RegionSlotId, HashMap<CellCoord, HashSet<CellCoord>>>,
}

#[derive(Clone, Debug)]
pub struct ExpandedDie {
    width: usize,
    height: usize,
    tiles: Vec<Cell>,
}

impl<'a> ExpandedGrid<'a> {
    pub fn die(&self) -> EntityRange<DieId> {
        self.die.ids()
    }

    pub fn rows(&self, die: DieId) -> EntityRange<RowId> {
        EntityRange::new(0, self.die[die].height)
    }

    pub fn cols(&self, die: DieId) -> EntityRange<ColId> {
        EntityRange::new(0, self.die[die].width)
    }

    pub fn column(&self, die: DieId, col: ColId) -> Rect {
        die.rect(col.range_up(1), self.rows(die))
    }

    pub fn row(&self, die: DieId, row: RowId) -> Rect {
        die.rect(self.cols(die), row.range_up(1))
    }

    pub fn die_cells(&self, die: DieId) -> Rect {
        die.rect(self.cols(die), self.rows(die))
    }

    pub fn tile_wire(&self, tcrd: TileCoord, wire: TileWireCoord) -> WireCoord {
        WireCoord {
            cell: self.tile_cell(tcrd, wire.cell),
            slot: wire.wire,
        }
    }

    pub fn try_tile_wire(&self, tcrd: TileCoord, wire: TileWireCoord) -> Option<WireCoord> {
        Some(WireCoord {
            cell: self.try_tile_cell(tcrd, wire.cell)?,
            slot: wire.wire,
        })
    }

    pub fn resolve_tile_wire(&self, tcrd: TileCoord, wire: TileWireCoord) -> Option<WireCoord> {
        self.resolve_wire(self.try_tile_wire(tcrd, wire)?)
    }

    pub fn bel_wire(&self, bcrd: BelCoord, wire: TileWireCoord) -> WireCoord {
        self.tile_wire(self.bel_tile(bcrd), wire)
    }

    pub fn try_bel_wire(&self, bcrd: BelCoord, wire: TileWireCoord) -> Option<WireCoord> {
        self.try_tile_wire(self.bel_tile(bcrd), wire)
    }

    pub fn resolve_bel_wire(&self, bcrd: BelCoord, wire: TileWireCoord) -> Option<WireCoord> {
        self.resolve_tile_wire(self.bel_tile(bcrd), wire)
    }

    pub fn cells(&self) -> impl Iterator<Item = (CellCoord, &Cell)> + '_ {
        self.die().into_iter().flat_map(move |die| {
            self.cols(die).into_iter().flat_map(move |col| {
                self.rows(die).map(move |row| {
                    let crd = die.cell(col, row);
                    let cell = &self[crd];
                    (crd, cell)
                })
            })
        })
    }

    pub fn tiles(&self) -> impl Iterator<Item = (TileCoord, &Tile)> + '_ {
        self.cells().flat_map(|(crd, cell)| {
            cell.tiles
                .iter()
                .map(move |(tslot, tile)| (crd.tile(tslot), tile))
        })
    }

    pub fn connectors(&self) -> impl Iterator<Item = (ConnectorCoord, &Connector)> + '_ {
        self.cells().flat_map(|(crd, cell)| {
            cell.conns
                .iter()
                .map(move |(cslot, conn)| (crd.connector(cslot), conn))
        })
    }

    pub fn tile_cell(&self, tcrd: TileCoord, slot: CellSlotId) -> CellCoord {
        self[tcrd].cells[slot]
    }

    pub fn try_tile_cell(&self, tcrd: TileCoord, slot: CellSlotId) -> Option<CellCoord> {
        self[tcrd].cells.get(slot).copied()
    }

    pub fn get_tile(&self, tcrd: TileCoord) -> Option<&Tile> {
        self[tcrd.cell].tiles.get(tcrd.slot)
    }

    pub fn cell_delta(&self, mut cell: CellCoord, dx: i32, dy: i32) -> Option<CellCoord> {
        if dx < 0 {
            if cell.col.to_idx() < (-dx) as usize {
                return None;
            }
            cell.col -= (-dx) as usize;
        } else {
            cell.col += dx as usize;
            if cell.col.to_idx() >= self.cols(cell.die).len() {
                return None;
            }
        }
        if dy < 0 {
            if cell.row.to_idx() < (-dy) as usize {
                return None;
            }
            cell.row -= (-dy) as usize;
        } else {
            cell.row += dy as usize;
            if cell.row.to_idx() >= self.rows(cell.die).len() {
                return None;
            }
        }
        Some(cell)
    }

    pub fn bel_delta(
        &self,
        cell: CellCoord,
        dx: i32,
        dy: i32,
        slot: BelSlotId,
    ) -> Option<BelCoord> {
        let cell = self.cell_delta(cell, dx, dy)?;
        let bcrd = cell.bel(slot);
        if self.has_bel(bcrd) { Some(bcrd) } else { None }
    }

    pub fn has_bel(&self, bel: BelCoord) -> bool {
        if !self.db.bel_slots.contains_id(bel.slot) {
            return false;
        }
        let tslot = self.db.bel_slots[bel.slot].tile_slot;
        if let Some(tile) = self[bel.cell].tiles.get(tslot) {
            let nk = &self.db[tile.class];
            if nk.bels.contains_id(bel.slot) {
                return true;
            }
        }
        false
    }

    pub fn find_tile_by_bel(&self, bel: BelCoord) -> Option<TileCoord> {
        if self.has_bel(bel) {
            Some(self.bel_tile(bel))
        } else {
            None
        }
    }

    pub fn bel_tile(&self, bel: BelCoord) -> TileCoord {
        bel.tile(self.db.bel_slots[bel.slot].tile_slot)
    }

    pub fn get_bel_input(&self, bel: BelCoord, pin: BelInputId) -> PolWireCoord {
        let tcrd = self.bel_tile(bel);
        let tile = &self[tcrd];
        let BelInfo::Bel(ref bel) = self.db[tile.class].bels[bel.slot] else {
            unreachable!()
        };
        let inp = bel.inputs[pin];
        match inp {
            BelInput::Fixed(ptwc) => PolWireCoord {
                wire: self.tile_wire(tcrd, ptwc.tw),
                inv: ptwc.inv,
            },
            BelInput::Invertible(twc, _) => self.tile_wire(tcrd, twc).pos(),
        }
    }

    pub fn get_bel_output(&self, bel: BelCoord, pin: BelOutputId) -> Vec<WireCoord> {
        let tcrd = self.bel_tile(bel);
        let tile = &self[tcrd];
        let BelInfo::Bel(ref bel) = self.db[tile.class].bels[bel.slot] else {
            unreachable!()
        };
        let outp = &bel.outputs[pin];
        outp.iter()
            .copied()
            .map(|twc| self.tile_wire(tcrd, twc))
            .collect()
    }

    pub fn get_bel_bidir(&self, bel: BelCoord, pin: BelBidirId) -> WireCoord {
        let tcrd = self.bel_tile(bel);
        let tile = &self[tcrd];
        let BelInfo::Bel(ref bel) = self.db[tile.class].bels[bel.slot] else {
            unreachable!()
        };
        self.tile_wire(tcrd, bel.bidirs[pin])
    }
}

impl core::ops::Index<(ColId, RowId)> for ExpandedDie {
    type Output = Cell;
    fn index(&self, xy: (ColId, RowId)) -> &Cell {
        assert!(xy.0.to_idx() < self.width);
        &self.tiles[xy.1.to_idx() * self.width + xy.0.to_idx()]
    }
}

impl core::ops::IndexMut<(ColId, RowId)> for ExpandedDie {
    fn index_mut(&mut self, xy: (ColId, RowId)) -> &mut Cell {
        assert!(xy.0.to_idx() < self.width);
        &mut self.tiles[xy.1.to_idx() * self.width + xy.0.to_idx()]
    }
}

impl core::ops::Index<CellCoord> for ExpandedGrid<'_> {
    type Output = Cell;
    fn index(&self, cell: CellCoord) -> &Cell {
        &self.die[cell.die][(cell.col, cell.row)]
    }
}

impl core::ops::IndexMut<CellCoord> for ExpandedGrid<'_> {
    fn index_mut(&mut self, cell: CellCoord) -> &mut Cell {
        &mut self.die[cell.die][(cell.col, cell.row)]
    }
}

impl core::ops::Index<TileCoord> for ExpandedGrid<'_> {
    type Output = Tile;
    fn index(&self, tcrd: TileCoord) -> &Tile {
        &self[tcrd.cell].tiles[tcrd.slot]
    }
}

impl core::ops::IndexMut<TileCoord> for ExpandedGrid<'_> {
    fn index_mut(&mut self, tcrd: TileCoord) -> &mut Tile {
        &mut self[tcrd.cell].tiles[tcrd.slot]
    }
}

impl core::ops::Index<ConnectorCoord> for ExpandedGrid<'_> {
    type Output = Connector;
    fn index(&self, ccrd: ConnectorCoord) -> &Connector {
        &self[ccrd.cell].conns[ccrd.slot]
    }
}

impl core::ops::IndexMut<ConnectorCoord> for ExpandedGrid<'_> {
    fn index_mut(&mut self, ccrd: ConnectorCoord) -> &mut Connector {
        &mut self[ccrd.cell].conns[ccrd.slot]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TilePip {
    pub wire_out: WireCoord,
    pub wire_in: WireCoord,
    pub wire_out_raw: WireCoord,
    pub wire_in_raw: WireCoord,
    pub inv: bool,

    pub tile: TileCoord,
    pub tile_wire_out: TileWireCoord,
    pub tile_wire_in: TileWireCoord,
}

impl ExpandedGrid<'_> {
    pub fn resolve_wire(&self, mut wire: WireCoord) -> Option<WireCoord> {
        loop {
            let cell = &self[wire.cell];
            match self.db[wire.slot].kind {
                WireKind::Regional(rslot) => {
                    wire.cell = cell.region_root[rslot];
                    break;
                }
                WireKind::MultiBranch(slot) | WireKind::Branch(slot) => {
                    if let Some(conn) = cell.conns.get(slot) {
                        let ccls = &self.db[conn.class];
                        match ccls.wires.get(wire.slot) {
                            Some(&ConnectorWire::BlackHole) => return None,
                            Some(&ConnectorWire::Reflect(wf)) => {
                                wire.slot = wf;
                            }
                            Some(&ConnectorWire::Pass(wf)) => {
                                wire.cell = conn.target.unwrap();
                                wire.slot = wf;
                            }
                            None => break,
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        if let Some(&twire) = self.extra_conns.get_by_left(&wire) {
            wire = twire;
        }
        if self.blackhole_wires.contains(&wire) {
            None
        } else {
            Some(wire)
        }
    }

    pub fn wire_tree(&self, wire: WireCoord) -> Vec<WireCoord> {
        if self.blackhole_wires.contains(&wire) {
            return vec![];
        }
        let mut res = vec![];
        let mut queue = vec![wire];
        if let Some(&twire) = self.extra_conns.get_by_right(&wire) {
            queue.push(twire);
        }
        while let Some(wire) = queue.pop() {
            let tile = &self[wire.cell];
            res.push(wire);
            if let WireKind::Regional(rslot) = self.db[wire.slot].kind
                && tile.region_root[rslot] == wire.cell
            {
                for &cell in &self.region_root_cells[rslot][&wire.cell] {
                    if cell != wire.cell {
                        queue.push(cell.wire(wire.slot));
                    }
                }
            }
            for (slot, conn) in &tile.conns {
                let oslot = self.db.conn_slots[slot].opposite;
                for &wt in &self.db_index[conn.class].wire_ins_near[wire.slot] {
                    queue.push(wire.cell.wire(wt));
                }
                if let Some(ocrd) = conn.target {
                    let oconn = &self[ocrd].conns[oslot];
                    for &wt in &self.db_index[oconn.class].wire_ins_far[wire.slot] {
                        queue.push(ocrd.wire(wt));
                    }
                }
            }
        }
        res
    }

    pub fn wire_pips_bwd(&self, wire: WireCoord) -> Vec<TilePip> {
        let mut wires = vec![wire];
        if matches!(
            self.db[wire.slot].kind,
            WireKind::MultiRoot | WireKind::MultiBranch(_)
        ) {
            wires = self.wire_tree(wire);
        }
        let mut res = vec![];
        for w in wires {
            for &(tcrd, cid) in &self[w.cell].tile_index {
                let tcls = &self.db_index[self[tcrd].class];
                let tcw = TileWireCoord {
                    cell: cid,
                    wire: w.slot,
                };
                if let Some(ins) = tcls.pips_bwd.get(&tcw) {
                    for &tcwi in ins {
                        let wire_in_raw = self.tile_wire(tcrd, tcwi.tw);
                        if let Some(wire_in) = self.resolve_wire(wire_in_raw) {
                            res.push(TilePip {
                                wire_out: wire,
                                wire_in,
                                wire_out_raw: w,
                                wire_in_raw,
                                tile: tcrd,
                                tile_wire_out: tcw,
                                tile_wire_in: tcwi.tw,
                                inv: tcwi.inv,
                            });
                        }
                    }
                }
            }
        }
        res
    }

    pub fn wire_pips_fwd(&self, wire: WireCoord) -> Vec<TilePip> {
        let wires = self.wire_tree(wire);
        let mut res = vec![];
        for w in wires {
            for &(tcrd, cid) in &self[w.cell].tile_index {
                let tcls = &self.db_index[self[tcrd].class];
                let tcw = TileWireCoord {
                    cell: cid,
                    wire: w.slot,
                };
                if let Some(outs) = tcls.pips_fwd.get(&tcw) {
                    for &tcwo in outs {
                        let wire_out_raw = self.tile_wire(tcrd, tcwo.tw);
                        if let Some(wire_out) = self.resolve_wire(wire_out_raw) {
                            res.push(TilePip {
                                wire_out,
                                wire_in: wire,
                                wire_out_raw,
                                wire_in_raw: w,
                                tile: tcrd,
                                tile_wire_out: tcwo.tw,
                                tile_wire_in: tcw,
                                inv: tcwo.inv,
                            });
                        }
                    }
                }
            }
        }
        res
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub tiles: EntityPartVec<TileSlotId, Tile>,
    pub conns: EntityPartVec<ConnectorSlotId, Connector>,
    pub tile_index: Vec<(TileCoord, CellSlotId)>,
    pub region_root: EntityPartVec<RegionSlotId, CellCoord>,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub class: TileClassId,
    pub cells: EntityPartVec<CellSlotId, CellCoord>,
}

#[derive(Clone, Debug)]
pub struct Connector {
    pub class: ConnectorClassId,
    pub target: Option<CellCoord>,
}
