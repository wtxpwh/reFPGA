use bincode::{Decode, Encode};
use prjcombine_entity::{
    EntityBundleItemIndex, EntityIdArith, EntityRange, entity_id_u8, entity_id_u16, id::EntityDelta,
};
use prjcombine_types::enumerable::EnumMap;

use crate::{
    db::{BelKind, BelPadId, BelSlotId, ConnectorSlotId, IntDb, TileSlotId, WireSlotId},
    dir::{Dir, DirH, DirV},
};

entity_id_u8!(pub DieId arith, DieTag tag, "D");
entity_id_u16!(pub ColId arith, ColTag tag, "X");
entity_id_u16!(pub RowId arith, RowTag tag, "Y");

pub trait ColIdExt {
    fn side_of(self, anchor: ColId) -> DirH;
    fn step(self, dir: DirH, num: impl EntityDelta) -> ColId;
    fn col_on_side(self, side: DirH) -> ColId;
    fn snap_to_side_of(self, anchor: ColId) -> ColId;
}

impl ColIdExt for ColId {
    fn side_of(self, anchor: ColId) -> DirH {
        if self < anchor { DirH::W } else { DirH::E }
    }

    fn step(self, dir: DirH, num: impl EntityDelta) -> ColId {
        match dir {
            DirH::W => self - num,
            DirH::E => self + num,
        }
    }

    fn col_on_side(self, side: DirH) -> ColId {
        match side {
            DirH::W => self - 1,
            DirH::E => self,
        }
    }

    fn snap_to_side_of(self, anchor: ColId) -> ColId {
        anchor.col_on_side(self.side_of(anchor))
    }
}

pub trait ColRangeExt: Copy {
    fn with_col(self, side: DirH, col: ColId) -> Self;

    fn split_at(self, col: ColId) -> EnumMap<DirH, Self> {
        EnumMap::from_fn(|side| self.with_col(!side, col))
    }
}

impl ColRangeExt for EntityRange<ColId> {
    fn with_col(self, side: DirH, col: ColId) -> Self {
        match side {
            DirH::W => col.range_until(self.end()),
            DirH::E => self.start().range_until(col),
        }
    }
}

pub trait RowIdExt {
    fn side_of(self, anchor: RowId) -> DirV;
    fn step(self, dir: DirV, num: impl EntityDelta) -> RowId;
    fn row_on_side(self, side: DirV) -> RowId;
    fn snap_to_side_of(self, anchor: RowId) -> RowId;
}

impl RowIdExt for RowId {
    fn side_of(self, anchor: RowId) -> DirV {
        if self < anchor { DirV::S } else { DirV::N }
    }

    fn step(self, dir: DirV, num: impl EntityDelta) -> RowId {
        match dir {
            DirV::S => self - num,
            DirV::N => self + num,
        }
    }

    fn row_on_side(self, side: DirV) -> RowId {
        match side {
            DirV::S => self - 1,
            DirV::N => self,
        }
    }

    fn snap_to_side_of(self, anchor: RowId) -> RowId {
        anchor.row_on_side(self.side_of(anchor))
    }
}

pub trait RowRangeExt: Copy {
    fn with_row(self, side: DirV, row: RowId) -> Self;

    fn split_at(self, row: RowId) -> EnumMap<DirV, Self> {
        EnumMap::from_fn(|side| self.with_row(!side, row))
    }
}

impl RowRangeExt for EntityRange<RowId> {
    fn with_row(self, side: DirV, row: RowId) -> Self {
        match side {
            DirV::S => row.range_until(self.end()),
            DirV::N => self.start().range_until(row),
        }
    }
}

pub trait DieIdExt {
    fn cell(self, col: ColId, row: RowId) -> CellCoord;
    fn rect(self, cols: EntityRange<ColId>, rows: EntityRange<RowId>) -> Rect;
}

impl DieIdExt for DieId {
    fn cell(self, col: ColId, row: RowId) -> CellCoord {
        CellCoord {
            die: self,
            col,
            row,
        }
    }

    fn rect(self, cols: EntityRange<ColId>, rows: EntityRange<RowId>) -> Rect {
        Rect {
            die: self,
            cols,
            rows,
        }
    }
}

entity_id_u8!(pub TileIobId, IobTag tag, "IOB");

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Encode, Decode)]
pub enum EdgeIoCoord {
    W(RowId, TileIobId),
    E(RowId, TileIobId),
    S(ColId, TileIobId),
    N(ColId, TileIobId),
}

impl EdgeIoCoord {
    pub fn with_iob(self, iob: TileIobId) -> Self {
        match self {
            EdgeIoCoord::W(row, _) => EdgeIoCoord::W(row, iob),
            EdgeIoCoord::E(row, _) => EdgeIoCoord::E(row, iob),
            EdgeIoCoord::S(col, _) => EdgeIoCoord::S(col, iob),
            EdgeIoCoord::N(col, _) => EdgeIoCoord::N(col, iob),
        }
    }

    pub fn iob(self) -> TileIobId {
        match self {
            EdgeIoCoord::W(_, iob) => iob,
            EdgeIoCoord::E(_, iob) => iob,
            EdgeIoCoord::S(_, iob) => iob,
            EdgeIoCoord::N(_, iob) => iob,
        }
    }

    pub fn edge(&self) -> Dir {
        match self {
            EdgeIoCoord::W(..) => Dir::W,
            EdgeIoCoord::E(..) => Dir::E,
            EdgeIoCoord::S(..) => Dir::S,
            EdgeIoCoord::N(..) => Dir::N,
        }
    }
}

impl std::fmt::Display for EdgeIoCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeIoCoord::W(row, iob) => write!(f, "IOB_W{row:#}_{iob:#}"),
            EdgeIoCoord::E(row, iob) => write!(f, "IOB_E{row:#}_{iob:#}"),
            EdgeIoCoord::S(col, iob) => write!(f, "IOB_S{col:#}_{iob:#}"),
            EdgeIoCoord::N(col, iob) => write!(f, "IOB_N{col:#}_{iob:#}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CellCoord {
    pub die: DieId,
    pub col: ColId,
    pub row: RowId,
}

impl CellCoord {
    pub fn with_cr(self, col: ColId, row: RowId) -> CellCoord {
        CellCoord { col, row, ..self }
    }

    pub fn with_col(self, col: ColId) -> CellCoord {
        CellCoord { col, ..self }
    }

    pub fn with_row(self, row: RowId) -> CellCoord {
        CellCoord { row, ..self }
    }

    pub fn tile(self, slot: TileSlotId) -> TileCoord {
        TileCoord { cell: self, slot }
    }

    pub fn wire(self, slot: WireSlotId) -> WireCoord {
        WireCoord { cell: self, slot }
    }

    pub fn bel(self, slot: BelSlotId) -> BelCoord {
        BelCoord { cell: self, slot }
    }

    pub fn connector(self, slot: ConnectorSlotId) -> ConnectorCoord {
        ConnectorCoord { cell: self, slot }
    }

    pub fn delta(self, dx: impl EntityDelta, dy: impl EntityDelta) -> CellCoord {
        self.with_cr(self.col + dx, self.row + dy)
    }

    pub fn step(self, dir: impl Into<Dir>, num: impl EntityDelta) -> CellCoord {
        match dir.into() {
            Dir::H(h) => self.with_col(self.col.step(h, num)),
            Dir::V(v) => self.with_row(self.row.step(v, num)),
        }
    }

    pub fn cells_e_const<const N: usize>(self) -> [CellCoord; N] {
        core::array::from_fn(|i| self.delta(i, 0))
    }

    pub fn cells_n_const<const N: usize>(self) -> [CellCoord; N] {
        core::array::from_fn(|i| self.delta(0, i))
    }

    pub fn cells_e(self, num: usize) -> Vec<CellCoord> {
        (0..num).map(|i| self.delta(i, 0)).collect()
    }

    pub fn cells_n(self, num: usize) -> Vec<CellCoord> {
        (0..num).map(|i| self.delta(0, i)).collect()
    }

    pub fn rect_single(self) -> Rect {
        self.rect(1, 1)
    }

    pub fn rect_dir_inclusive(self, dir: impl Into<Dir>, num: usize) -> Rect {
        self.rect_single().enlarge_to(dir, num)
    }

    pub fn rect(self, width: usize, height: usize) -> Rect {
        Rect {
            die: self.die,
            cols: self.col.range_up(width),
            rows: self.row.range_up(height),
        }
    }
}

impl std::fmt::Display for CellCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{die}{col}{row}",
            die = self.die,
            col = self.col,
            row = self.row
        )
    }
}

impl std::fmt::Debug for CellCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{die}{col}{row}",
            die = self.die,
            col = self.col,
            row = self.row
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct TileCoord {
    pub cell: CellCoord,
    pub slot: TileSlotId,
}

impl TileCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        format!(
            "{cell}.{slot}",
            cell = self.cell,
            slot = db.tile_slots[self.slot]
        )
    }
}

impl std::ops::Deref for TileCoord {
    type Target = CellCoord;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl std::ops::DerefMut for TileCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ConnectorCoord {
    pub cell: CellCoord,
    pub slot: ConnectorSlotId,
}

impl ConnectorCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        format!(
            "{cell}.{slot}",
            cell = self.cell,
            slot = db.conn_slots[self.slot].name
        )
    }
}

impl std::ops::Deref for ConnectorCoord {
    type Target = CellCoord;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl std::ops::DerefMut for ConnectorCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct WireCoord {
    pub cell: CellCoord,
    pub slot: WireSlotId,
}

impl WireCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        format!(
            "{cell}.{slot}",
            cell = self.cell,
            slot = db.wires[self.slot].name
        )
    }

    pub fn pos(self) -> PolWireCoord {
        PolWireCoord {
            wire: self,
            inv: false,
        }
    }
}

impl std::ops::Deref for WireCoord {
    type Target = CellCoord;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl std::ops::DerefMut for WireCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct PolWireCoord {
    pub wire: WireCoord,
    pub inv: bool,
}

impl std::ops::Deref for PolWireCoord {
    type Target = WireCoord;

    fn deref(&self) -> &Self::Target {
        &self.wire
    }
}

impl std::ops::DerefMut for PolWireCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wire
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct BelCoord {
    pub cell: CellCoord,
    pub slot: BelSlotId,
}

impl BelCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        format!(
            "{cell}.{slot}",
            cell = self.cell,
            slot = db.bel_slots[self.slot].name
        )
    }

    pub fn pad(self, pad: BelPadId) -> BelPadCoord {
        BelPadCoord { bel: self, pad }
    }
}

impl std::ops::Deref for BelCoord {
    type Target = CellCoord;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl std::ops::DerefMut for BelCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct BelPadCoord {
    pub bel: BelCoord,
    pub pad: BelPadId,
}

impl BelPadCoord {
    pub fn to_string(&self, db: &IntDb) -> String {
        let BelKind::Class(bcid) = db.bel_slots[self.slot].kind else {
            unreachable!()
        };
        let (name, idx) = db.bel_classes[bcid].pads.key(self.pad);
        match idx {
            EntityBundleItemIndex::Single => format!("{bel}.{name}", bel = self.bel.to_string(db)),

            EntityBundleItemIndex::Array { index, .. } => {
                format!("{bel}.{name}[{index}]", bel = self.bel.to_string(db))
            }
        }
    }
}

impl std::ops::Deref for BelPadCoord {
    type Target = BelCoord;

    fn deref(&self) -> &Self::Target {
        &self.bel
    }
}

impl std::ops::DerefMut for BelPadCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bel
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect {
    pub die: DieId,
    pub cols: EntityRange<ColId>,
    pub rows: EntityRange<RowId>,
}

impl Rect {
    pub fn contains(self, cell: CellCoord) -> bool {
        cell.die == self.die && self.cols.contains(cell.col) && self.rows.contains(cell.row)
    }

    pub fn enlarge_by(self, dir: impl Into<Dir>, num: usize) -> Rect {
        match dir.into() {
            Dir::W => self.with_col(DirH::W, self.cols.start() - num),
            Dir::E => self.with_col(DirH::E, self.cols.end() + num),
            Dir::S => self.with_row(DirV::S, self.rows.start() - num),
            Dir::N => self.with_row(DirV::N, self.rows.end() + num),
        }
    }

    pub fn enlarge_to(self, dir: impl Into<Dir>, num: usize) -> Rect {
        match dir.into() {
            Dir::W => self.with_col(DirH::W, self.cols.end() - num),
            Dir::E => self.with_col(DirH::E, self.cols.start() + num),
            Dir::S => self.with_row(DirV::S, self.rows.end() - num),
            Dir::N => self.with_row(DirV::N, self.rows.start() + num),
        }
    }

    pub fn with_cols(self, cols: EntityRange<ColId>) -> Rect {
        Rect { cols, ..self }
    }

    pub fn with_rows(self, rows: EntityRange<RowId>) -> Rect {
        Rect { rows, ..self }
    }

    pub fn with_col(self, side: DirH, col: ColId) -> Rect {
        self.with_cols(self.cols.with_col(side, col))
    }

    pub fn with_row(self, side: DirV, row: RowId) -> Rect {
        self.with_rows(self.rows.with_row(side, row))
    }
}

impl IntoIterator for Rect {
    type Item = CellCoord;
    type IntoIter = RectIter;

    fn into_iter(self) -> Self::IntoIter {
        RectIter {
            die: self.die,
            cols: self.cols,
            col_next: self.cols.start(),
            col_back: self.cols.start(),
            row_next: self.rows.start(),
            row_back: self.rows.end(),
        }
    }
}

pub struct RectIter {
    die: DieId,
    cols: EntityRange<ColId>,
    col_next: ColId,
    row_next: RowId,
    col_back: ColId,
    row_back: RowId,
}

impl Iterator for RectIter {
    type Item = CellCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col_next == self.col_back && self.row_next == self.row_back {
            None
        } else {
            let result = self.die.cell(self.col_next, self.row_next);
            self.col_next += 1;
            if self.col_next == self.cols.end() {
                self.col_next = self.cols.start();
                self.row_next += 1;
            }
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let result = (self.row_back - self.row_next) as usize * self.cols.len()
            + (self.col_back - self.col_next) as usize;
        (result, Some(result))
    }
}

impl DoubleEndedIterator for RectIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.col_next == self.col_back && self.row_next == self.row_back {
            None
        } else {
            if self.col_back == self.cols.start() {
                self.col_back = self.cols.end();
                self.row_back -= 1;
            }
            self.col_back -= 1;
            Some(self.die.cell(self.col_back, self.row_back))
        }
    }
}

impl ExactSizeIterator for RectIter {}
