use prjcombine_entity::{EntityId, EntityVec};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, ColId, DieId, DieIdExt, RowId},
    dir::DirV,
    grid::ExpandedGrid,
};
use std::collections::{BTreeSet, HashMap};

use crate::{
    chip::{Chip, DisabledPart, HsrNKind, HsrSKind, Interposer, RegId, X5io, X5ioDdrmc},
    defs::cslots,
    noc::NocPortCoord,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SllConn {
    pub target: Option<BelCoord>,
    pub cursed: bool,
}

impl From<BelCoord> for SllConn {
    fn from(value: BelCoord) -> Self {
        SllConn {
            target: Some(value),
            cursed: false,
        }
    }
}

#[derive(Debug)]
pub struct ExpandedDevice<'a> {
    pub chips: EntityVec<DieId, &'a Chip>,
    pub egrid: ExpandedGrid<'a>,
    pub interposer: &'a Interposer,
    pub disabled: BTreeSet<DisabledPart>,
    pub col_cfrm: EntityVec<DieId, ColId>,
    pub cols_vnoc: EntityVec<DieId, Vec<ColId>>,
    pub sll: HashMap<BelCoord, SllConn>,
    pub noc: HashMap<NocPortCoord, NocPortCoord>,
}

impl<'a> std::ops::Deref for ExpandedDevice<'a> {
    type Target = ExpandedGrid<'a>;

    fn deref(&self) -> &Self::Target {
        &self.egrid
    }
}

impl ExpandedDevice<'_> {
    pub fn row_to_reg(&self, row: RowId) -> RegId {
        RegId::from_idx(row.to_idx() / Chip::ROWS_PER_REG)
    }

    pub fn row_reg_bot(&self, reg: RegId) -> RowId {
        RowId::from_idx(reg.to_idx() * Chip::ROWS_PER_REG)
    }

    pub fn row_reg_hclk(&self, die: DieId, reg: RegId) -> RowId {
        let reg = if self.is_reg_n(die, reg) {
            reg
        } else {
            reg + 1
        };
        self.row_reg_bot(reg)
    }

    pub fn is_reg_n(&self, die: DieId, reg: RegId) -> bool {
        reg.to_idx() == self.chips[die].regs - 1 || reg.to_idx() % 2 == 1
    }

    pub fn bel_gt_neighbour(&self, bcrd: BelCoord, dir: DirV) -> Option<BelCoord> {
        let reg = self.row_to_reg(bcrd.row);
        let chip = self.chips[bcrd.die];
        let edge = chip.col_side(bcrd.col);
        match dir {
            DirV::S => {
                if reg == chip.regs().first().unwrap() && chip.hsr_s == HsrSKind::Ssit {
                    let odie = bcrd.die - 1;
                    let ochip = self.chips[odie];
                    let bcrd = odie
                        .cell(
                            ochip.col_edge(edge),
                            ochip.row_reg_bot(ochip.regs().last().unwrap()),
                        )
                        .bel(bcrd.slot);
                    if self.has_bel(bcrd) { Some(bcrd) } else { None }
                } else {
                    self.bel_delta(bcrd.cell, 0, -(Chip::ROWS_PER_REG as i32), bcrd.slot)
                }
            }
            DirV::N => {
                if reg == chip.regs().last().unwrap()
                    && chip.hsr_n == HsrNKind::Ssit
                    && bcrd.die != self.chips.last_id().unwrap()
                {
                    let odie = bcrd.die + 1;
                    let ochip = self.chips[odie];
                    let bcrd = odie
                        .cell(
                            ochip.col_edge(edge),
                            ochip.row_reg_bot(ochip.regs().first().unwrap()),
                        )
                        .bel(bcrd.slot);
                    if self.has_bel(bcrd) { Some(bcrd) } else { None }
                } else {
                    self.bel_delta(bcrd.cell, 0, Chip::ROWS_PER_REG as i32, bcrd.slot)
                }
            }
        }
    }

    pub fn x5io(&self, cell: CellCoord) -> Option<X5io> {
        self.chips[cell.die].x5io(cell)
    }

    pub fn x5io_ddrmc(&self, cell: CellCoord) -> Option<X5ioDdrmc> {
        self.chips[cell.die].x5io_ddrmc(cell)
    }

    pub fn x5io_to_w(&self, cell: CellCoord) -> Option<CellCoord> {
        self.chips[cell.die].x5io_to_w(cell)
    }

    pub fn x5io_to_e(&self, cell: CellCoord) -> Option<CellCoord> {
        self.chips[cell.die].x5io_to_e(cell)
    }

    pub fn x5io_pair(&self, cell: CellCoord) -> Option<CellCoord> {
        self[cell]
            .conns
            .get(cslots::X5IO_PAIR)
            .and_then(|c| c.target)
    }

    pub fn x5io_to_ddrmc(&self, cell: CellCoord) -> Option<BelCoord> {
        self.chips[cell.die].x5io_to_ddrmc(cell)
    }

    pub fn x5pll_casc_source(&self, bcrd: BelCoord) -> Option<BelCoord> {
        let side = self.x5io(bcrd.cell).unwrap().side;
        if let Some(ddrmc) = self.x5io_to_ddrmc(bcrd.cell) {
            if self.x5io_ddrmc(ddrmc.cell).unwrap().kind.is_2hb() {
                Some(bcrd)
            } else if Some(ddrmc.cell) == self.x5io_to_w(bcrd.cell) {
                let bcrd_src = self.bel_delta(bcrd.cell, -8, 0, bcrd.slot)?;
                if self.x5io(bcrd_src.cell).unwrap().side == side {
                    Some(bcrd_src)
                } else {
                    None
                }
            } else if Some(ddrmc.cell) == self.x5io_to_e(bcrd.cell) {
                let bcrd_src = self.bel_delta(bcrd.cell, 8, 0, bcrd.slot)?;
                if self.x5io(bcrd_src.cell).unwrap().side == side {
                    Some(bcrd_src)
                } else {
                    None
                }
            } else {
                Some(bcrd)
            }
        } else {
            if self.x5io_to_ddrmc(self.x5io_pair(bcrd.cell)?).is_some() {
                Some(bcrd)
            } else {
                None
            }
        }
    }
}
