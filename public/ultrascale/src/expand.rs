#![allow(clippy::type_complexity)]

use bimap::BiHashMap;
use prjcombine_entity::{EntityId, EntityPartVec, EntityVec};
use prjcombine_interconnect::coord::{
    CellCoord, ColId, DieId, DieIdExt, RowId, RowIdExt, TileIobId,
};
use prjcombine_interconnect::db::IntDb;
use prjcombine_interconnect::dir::DirH;
use prjcombine_interconnect::grid::builder::GridBuilder;
use std::collections::BTreeSet;

use crate::chip::{
    Chip, ChipKind, CleMKind, ColumnKind, ConfigKind, DisabledPart, DspKind, HardKind, HardRowKind,
    Interposer, IoRowKind, RegId,
};
use crate::defs::rslots;
use crate::expanded::{ExpandedDevice, HdioCoord, HpioCoord, IoCoord, Xp5ioCoord};

use crate::bond::SharedCfgPad;
use crate::defs::{ccls, tcls};

struct DieExpander<'a, 'b, 'c> {
    chip: &'b Chip,
    chips: &'a EntityVec<DieId, &'b Chip>,
    disabled: &'a BTreeSet<DisabledPart>,
    egrid: &'a mut GridBuilder<'b>,
    die: DieId,
    io: &'c mut Vec<IoCoord>,
    gt: &'c mut Vec<CellCoord>,
}

impl DieExpander<'_, '_, '_> {
    fn in_int_hole(&self, cell: CellCoord) -> bool {
        self.chip.in_int_hole(cell.col, cell.row)
    }

    fn in_site_hole(&self, cell: CellCoord) -> bool {
        self.chip.in_site_hole(cell.col, cell.row)
    }

    fn fill_int(&mut self) {
        for cell in self.egrid.die_cells(self.die) {
            if self.disabled.contains(&DisabledPart::Region(
                self.die,
                self.chip.row_to_reg(cell.row),
            )) {
                continue;
            }
            if self.in_int_hole(cell) {
                continue;
            }
            if self.chip.col_side(cell.col) == DirH::W {
                self.egrid.add_tile_e(cell, tcls::INT, 2);
                if cell.row.to_idx() % 60 == 30 {
                    self.egrid.add_tile(
                        cell,
                        tcls::RCLK_INT,
                        &[
                            cell.delta(0, 0),
                            cell.delta(1, 0),
                            cell.delta(0, -1),
                            cell.delta(1, -1),
                        ],
                    );
                }
            }
            match self.chip.columns[cell.col].kind {
                ColumnKind::CleL(_) | ColumnKind::CleM(_) => (),
                ColumnKind::Bram(_)
                | ColumnKind::Dsp(_)
                | ColumnKind::Uram
                | ColumnKind::ContUram => {
                    self.egrid.add_tile_single(cell, tcls::INTF);
                }
                ColumnKind::Gt(idx) | ColumnKind::Io(idx) => {
                    let iocol = &self.chip.cols_io[idx];
                    let rk = iocol.regs[self.chip.row_to_reg(cell.row)];
                    match (self.chip.kind, rk) {
                        (_, IoRowKind::None) => (),

                        (
                            ChipKind::UltrascalePlus,
                            IoRowKind::Hpio
                            | IoRowKind::Hrio
                            | IoRowKind::HdioL
                            | IoRowKind::HdioLMrc
                            | IoRowKind::Xp5io,
                        ) => {
                            self.egrid.add_tile_single(cell, tcls::INTF_IO);
                        }
                        _ => {
                            self.egrid.add_tile_single(cell, tcls::INTF_DELAY);
                        }
                    }
                }
                ColumnKind::Hard(_, _)
                | ColumnKind::DfeC
                | ColumnKind::DfeDF
                | ColumnKind::DfeE
                | ColumnKind::HdioS
                | ColumnKind::ContHard
                | ColumnKind::Sdfec
                | ColumnKind::DfeB => {
                    self.egrid.add_tile_single(cell, tcls::INTF_DELAY);
                }
            }
        }
    }

    fn fill_ps(&mut self) {
        if let Some(ps) = self.chip.ps {
            let cell = self.die.cell(ps.col, RowId::from_idx(0));
            let height = ps.height();
            let width = ps.col.to_idx();
            if height != self.chip.regs * 60 {
                let row_t = RowId::from_idx(height);
                for dx in 0..width {
                    let col = ColId::from_idx(dx);
                    self.egrid.fill_conn_term(
                        self.die.cell(col, row_t),
                        [ccls::TERM_S0, ccls::TERM_S1][col.to_idx() % 2],
                    );
                }
            }
            for dy in 0..height {
                self.egrid.fill_conn_term(cell.delta(0, dy), ccls::TERM_W);
                self.egrid.fill_conn_term(cell.delta(0, dy), ccls::TERM_LW);
                self.egrid.add_tile_single(cell.delta(0, dy), tcls::INTF_IO);
                if dy % 60 == 30 {
                    self.egrid.add_tile_single(cell.delta(0, dy), tcls::RCLK_PS);
                }
            }
            let dy = if ps.has_vcu { 60 } else { 0 };
            self.egrid.add_tile_n(cell.delta(0, dy), tcls::PS, 180);
            if ps.has_vcu {
                self.egrid.add_tile_n(cell, tcls::VCU, 60);
            }
        }
    }

    fn fill_term(&mut self) {
        for cell in self
            .egrid
            .row(self.die, self.egrid.rows(self.die).first().unwrap())
        {
            if !self.in_int_hole(cell)
                && !self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                ))
            {
                self.egrid
                    .fill_conn_term(cell, [ccls::TERM_S0, ccls::TERM_S1][cell.col.to_idx() % 2]);
            }
        }
        for cell in self
            .egrid
            .row(self.die, self.egrid.rows(self.die).last().unwrap())
        {
            if !self.in_int_hole(cell)
                && !self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                ))
            {
                self.egrid
                    .fill_conn_term(cell, [ccls::TERM_N0, ccls::TERM_N1][cell.col.to_idx() % 2]);
            }
        }
        for cell in self
            .egrid
            .column(self.die, self.egrid.cols(self.die).first().unwrap())
        {
            if !self.in_int_hole(cell) {
                self.egrid.fill_conn_term(cell, ccls::TERM_W);
                self.egrid.fill_conn_term(cell, ccls::TERM_LW);
            }
        }
        for cell in self
            .egrid
            .column(self.die, self.egrid.cols(self.die).last().unwrap())
        {
            if !self.in_int_hole(cell) {
                self.egrid.fill_conn_term(cell, ccls::TERM_E);
                self.egrid.fill_conn_term(cell.delta(-1, 0), ccls::TERM_LE);
            }
        }
    }

    fn fill_main_passes(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if col == self.chip.columns.first_id().unwrap() {
                continue;
            }
            let is_io_mid = matches!(cd.kind, ColumnKind::Io(_))
                && col != self.chip.columns.last_id().unwrap()
                && self.chip.kind == ChipKind::UltrascalePlus;
            for cell in self.egrid.column(self.die, col) {
                if self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                )) {
                    continue;
                }
                if self.in_int_hole(cell) || self.in_int_hole(cell.delta(-1, 0)) {
                    continue;
                }
                if is_io_mid {
                    self.egrid
                        .fill_conn_pair(cell.delta(-1, 0), cell, ccls::IO_E, ccls::IO_W);
                    self.egrid
                        .fill_conn_pair(cell.delta(-2, 0), cell, ccls::IO_LE, ccls::IO_LW);
                } else {
                    self.egrid
                        .fill_conn_pair(cell.delta(-1, 0), cell, ccls::PASS_E, ccls::PASS_W);
                    if self.chip.col_side(col) == DirH::W {
                        self.egrid.fill_conn_pair(
                            cell.delta(-2, 0),
                            cell,
                            ccls::PASS_LE,
                            ccls::PASS_LW,
                        );
                    }
                }
            }
        }
        for row in self.egrid.rows(self.die) {
            if row == self.chip.rows().last().unwrap() {
                continue;
            }
            if self
                .disabled
                .contains(&DisabledPart::Region(self.die, self.chip.row_to_reg(row)))
            {
                continue;
            }
            if self.disabled.contains(&DisabledPart::Region(
                self.die,
                self.chip.row_to_reg(row + 1),
            )) {
                continue;
            }
            for cell in self.egrid.row(self.die, row) {
                if self.in_int_hole(cell) || self.in_int_hole(cell.delta(0, 1)) {
                    continue;
                }
                self.egrid
                    .fill_conn_pair(cell, cell.delta(0, 1), ccls::PASS_N, ccls::PASS_S);
            }
        }
    }

    fn fill_clb(&mut self) {
        for (col, &cd) in &self.chip.columns {
            let Some(tcid) = (match cd.kind {
                ColumnKind::CleL(_) => Some(tcls::CLEL),
                ColumnKind::CleM(_) => Some(tcls::CLEM),
                _ => None,
            }) else {
                continue;
            };
            for cell in self.egrid.column(self.die, col) {
                if self.in_site_hole(cell) {
                    continue;
                }
                if self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                )) {
                    continue;
                }
                if cd.kind == ColumnKind::CleM(CleMKind::Laguna)
                    && self.chip.is_laguna_row(cell.row)
                {
                    self.egrid.add_tile_single(cell, tcls::LAGUNA);
                } else {
                    self.egrid.add_tile_single(cell, tcid);
                }
                if cell.row == self.chip.row_rclk(cell.row) {
                    if matches!(cd.kind, ColumnKind::CleM(CleMKind::ClkBuf)) {
                        self.egrid
                            .add_tile_single(cell, tcls::RCLK_HROUTE_SPLITTER_CLE);
                        self.egrid.fill_conn_pair(
                            cell,
                            cell.delta(-1, 0),
                            ccls::GCLK_H,
                            ccls::GCLK_H_REV,
                        );
                    } else if cd.kind == ColumnKind::CleM(CleMKind::Laguna)
                        && self.chip.is_laguna_row(cell.row)
                    {
                        if self.chip.kind == ChipKind::Ultrascale {
                            continue;
                        }
                        self.egrid.add_tile_single(cell, tcls::RCLK_V_SINGLE_LAG);
                    } else if self.chip.col_side(col) == DirH::W
                        || self.chip.kind != ChipKind::UltrascalePlus
                    {
                        self.egrid.add_tile_single(cell, tcls::RCLK_V_SINGLE_CLE);
                    }
                }
            }
        }
    }

    fn fill_bram(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if !matches!(cd.kind, ColumnKind::Bram(_)) {
                continue;
            }
            for cell in self.egrid.column(self.die, col) {
                if !cell.row.to_idx().is_multiple_of(5) {
                    continue;
                }
                if self.in_site_hole(cell) {
                    continue;
                }
                if self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                )) {
                    continue;
                }
                self.egrid.add_tile_n(cell, tcls::BRAM, 5);
                if cell.row == self.chip.row_rclk(cell.row) {
                    self.egrid.add_tile_single(cell, tcls::HARD_SYNC);
                    if self.chip.kind == ChipKind::Ultrascale {
                        self.egrid.add_tile_single(cell, tcls::RCLK_V_DOUBLE_BRAM);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::RCLK_V_QUAD_BRAM);
                    }
                }
            }
        }
    }

    fn fill_dsp(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if !matches!(cd.kind, ColumnKind::Dsp(_)) {
                continue;
            }
            for cell in self.egrid.column(self.die, col) {
                if !cell.row.to_idx().is_multiple_of(5) {
                    continue;
                }
                if self.in_int_hole(cell) {
                    continue;
                }
                if self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                )) {
                    continue;
                }
                if self.chip.has_hbm && cell.row.to_idx() < 15 {
                    if cell.row.to_idx() != 0 {
                        continue;
                    }
                    if cell.col < self.chip.cols_io[1].col
                        && self.disabled.contains(&DisabledPart::HbmLeft)
                    {
                        continue;
                    }
                    self.egrid.add_tile_n(cell, tcls::HBM_BLI, 15);
                } else {
                    self.egrid.add_tile_n(cell, tcls::DSP, 5);
                }
                if cell.row == self.chip.row_rclk(cell.row) {
                    if matches!(cd.kind, ColumnKind::Dsp(DspKind::ClkBuf)) {
                        self.egrid.add_tile_single(cell, tcls::RCLK_SPLITTER);
                        let ccls = if matches!(self.chip.columns[col + 1].kind, ColumnKind::Io(_)) {
                            ccls::GCLK_H_IO
                        } else {
                            ccls::GCLK_H
                        };
                        self.egrid
                            .fill_conn_pair(cell, cell.delta(1, 0), ccls, ccls::GCLK_H_REV);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::RCLK_V_DOUBLE_DSP);
                    }
                }
            }
        }
    }

    fn fill_uram(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if cd.kind != ColumnKind::Uram {
                continue;
            }
            for cell in self.egrid.column(self.die, col) {
                if !cell.row.to_idx().is_multiple_of(15) {
                    continue;
                }
                if self.in_int_hole(cell) {
                    continue;
                }
                if self.disabled.contains(&DisabledPart::Region(
                    self.die,
                    self.chip.row_to_reg(cell.row),
                )) {
                    continue;
                }
                let mut tcells = vec![];
                tcells.extend(cell.cells_n_const::<15>());
                tcells.extend(cell.delta(1, 0).cells_n_const::<15>());
                self.egrid.add_tile(cell, tcls::URAM, &tcells);
                if cell.row == self.chip.row_rclk(cell.row) {
                    self.egrid
                        .add_tile_single(cell.delta(1, 0), tcls::RCLK_V_QUAD_URAM);
                }
            }
        }
    }

    fn fill_hard_single(&mut self, cell: CellCoord, kind: HardRowKind) {
        if cell.col != self.chip.columns.last_id().unwrap() {
            self.egrid.fill_conn_pair(
                cell.delta(0, 30),
                cell.delta(1, 30),
                ccls::GCLK_H,
                ccls::GCLK_H_REV,
            );
        }

        let reg = self.chip.row_to_reg(cell.row);
        if self.disabled.contains(&DisabledPart::Region(self.die, reg)) {
            return;
        }

        let tcid = match kind {
            HardRowKind::None => return,
            HardRowKind::Hdio | HardRowKind::HdioAms => {
                self.egrid
                    .add_tile_sn(cell.delta(0, 30), tcls::HDIO, 30, 60);
                for idx in 0..24 {
                    self.io.push(IoCoord::Hdio(HdioCoord {
                        cell: cell.delta(0, 30),
                        iob: TileIobId::from_idx(idx),
                    }));
                }
                self.egrid
                    .add_tile_sn(cell.delta(0, 30), tcls::RCLK_HDIO, 30, 60);
                return;
            }
            HardRowKind::HdioS => {
                let mut tcells = vec![];
                tcells.extend(cell.cells_n_const::<60>());
                tcells.extend(cell.delta(1, 0).cells_n_const::<60>());
                self.egrid
                    .add_tile_sn(cell.delta(0, 30), tcls::RCLK_HDIOS, 30, 60);
                self.egrid.fill_conn_pair(
                    cell.delta(0, 30),
                    cell.delta(1, 30),
                    ccls::GCLK_H,
                    ccls::GCLK_H_REV,
                );
                self.egrid.add_tile(cell.delta(0, 30), tcls::HDIOS, &tcells);
                for i in 0..42 {
                    self.io.push(IoCoord::HdioLc(HdioCoord {
                        cell: cell.delta(0, 30),
                        iob: TileIobId::from_idx(i),
                    }));
                }
                return;
            }
            HardRowKind::HdioL => {
                for (i, tcid) in [tcls::HDIOL_S, tcls::HDIOL_N].into_iter().enumerate() {
                    let cell = cell.delta(0, i * 30);
                    self.egrid.add_tile_n(cell, tcid, 30);
                    for j in 0..42 {
                        self.io.push(IoCoord::HdioLc(HdioCoord {
                            cell,
                            iob: TileIobId::from_idx(j),
                        }));
                    }
                }
                self.egrid
                    .add_tile_sn(cell.delta(0, 30), tcls::RCLK_HDIOL, 30, 60);
                return;
            }
            HardRowKind::Cfg => {
                let tcid = match self.chip.config_kind {
                    ConfigKind::Config => tcls::CFG,
                    ConfigKind::Csec => tcls::CFG_CSEC,
                    ConfigKind::CsecV2 => tcls::CFG_CSEC_V2,
                };
                self.egrid.add_tile_n(cell, tcid, 60);
                self.egrid
                    .add_tile_single(cell.delta(0, 30), tcls::RCLK_HROUTE_SPLITTER_HARD);
                return;
            }
            HardRowKind::Ams => {
                self.egrid.add_tile_n(cell, tcls::CFGIO, 30);
                self.egrid
                    .add_tile_single(cell.delta(0, 30), tcls::RCLK_HROUTE_SPLITTER_HARD);
                self.egrid.add_tile_n(cell.delta(0, 30), tcls::AMS, 30);
                return;
            }
            HardRowKind::Pcie3 => tcls::PCIE3,
            HardRowKind::Pcie4 => tcls::PCIE4,
            HardRowKind::Pcie4C => tcls::PCIE4C,
            HardRowKind::Pcie4CE => tcls::PCIE4CE,
            HardRowKind::Cmac => tcls::CMAC,
            HardRowKind::Ilkn => tcls::ILKN,
            HardRowKind::DfeA => tcls::DFE_A,
            HardRowKind::DfeG => tcls::DFE_G,
        };
        self.egrid
            .add_tile_single(cell.delta(0, 30), tcls::RCLK_HROUTE_SPLITTER_HARD);
        let mut tcells = vec![];
        tcells.extend(cell.cells_n_const::<60>());
        tcells.extend(cell.delta(1, 0).cells_n_const::<60>());
        self.egrid.add_tile(cell, tcid, &tcells);
    }

    fn fill_hard(&mut self, has_pcie_cfg: &mut bool) {
        for hc in &self.chip.cols_hard {
            let is_cfg = hc.regs.values().any(|&x| x == HardRowKind::Cfg);
            for reg in self.chip.regs() {
                let kind = hc.regs[reg];
                if kind == HardRowKind::Cfg
                    && reg.to_idx() != 0
                    && matches!(
                        hc.regs[reg - 1],
                        HardRowKind::Pcie3 | HardRowKind::Pcie4 | HardRowKind::Pcie4C
                    )
                {
                    *has_pcie_cfg = true;
                }
                self.fill_hard_single(self.die.cell(hc.col, self.chip.row_reg_bot(reg)), kind);
            }
            if is_cfg && self.chip.has_hbm {
                self.egrid.add_tile(
                    self.die.cell(hc.col, RowId::from_idx(0)),
                    tcls::HBM_ABUS_SWITCH,
                    &[],
                );
            }
        }
    }

    fn fill_io(&mut self) {
        for ioc in &self.chip.cols_io {
            for reg in self.chip.regs() {
                if self.disabled.contains(&DisabledPart::Region(self.die, reg)) {
                    continue;
                }
                let kind = ioc.regs[reg];
                let cell = self.die.cell(ioc.col, self.chip.row_reg_rclk(reg));
                if ioc.col != self.chip.columns.first_id().unwrap()
                    && ioc.col != self.chip.columns.last_id().unwrap()
                {
                    assert_eq!(self.chip.col_side(ioc.col), DirH::W);
                    if self.chip.columns[ioc.col - 1].kind != ColumnKind::Dsp(DspKind::ClkBuf) {
                        self.egrid.fill_conn_pair(
                            cell,
                            cell.delta(-1, 0),
                            ccls::GCLK_H,
                            ccls::GCLK_H_REV,
                        );
                    }
                }
                match kind {
                    IoRowKind::None => (),
                    IoRowKind::Hpio | IoRowKind::Hrio => {
                        for idx in 0..52 {
                            self.io.push(IoCoord::Hpio(HpioCoord {
                                cell,
                                iob: TileIobId::from_idx(idx),
                            }));
                        }
                        let is_hbm = self.chip.has_hbm && reg.to_idx() == 0;
                        let tcid = if is_hbm { tcls::CMT_HBM } else { tcls::CMT };
                        self.egrid.add_tile_sn(cell, tcid, 30, 60);

                        for i in 0..4 {
                            self.egrid
                                .add_tile_n(cell.delta(0, -30 + i * 15), tcls::XIPHY, 15);
                        }
                        if self.chip.kind == ChipKind::UltrascalePlus {
                            self.egrid
                                .add_tile(cell, tcls::RCLK_XIPHY, &[cell, cell.delta(0, -1)]);
                        }

                        if kind == IoRowKind::Hpio {
                            for i in 0..2 {
                                self.egrid
                                    .add_tile_n(cell.delta(0, -30 + i * 30), tcls::HPIO, 30);
                            }
                            self.egrid.add_tile_sn(cell, tcls::RCLK_HPIO, 30, 60);
                        } else {
                            for i in 0..2 {
                                self.egrid
                                    .add_tile_n(cell.delta(0, -30 + i * 30), tcls::HRIO, 30);
                            }
                            self.egrid.add_tile_sn(cell, tcls::RCLK_HRIO, 30, 60);
                        }
                    }
                    IoRowKind::HdioL | IoRowKind::HdioLMrc => {
                        for (i, tcid) in [tcls::HDIOL_S, tcls::HDIOL_N].into_iter().enumerate() {
                            let cell = cell.delta(0, -30 + i as i32 * 30);
                            self.egrid.add_tile_n(cell, tcid, 30);
                            for idx in 0..42 {
                                self.io.push(IoCoord::HdioLc(HdioCoord {
                                    cell,
                                    iob: TileIobId::from_idx(idx),
                                }));
                            }
                        }
                        if kind == IoRowKind::HdioLMrc {
                            self.egrid.fill_conn_term(cell, ccls::RCLK_HDIO_CMT);
                        }
                        self.egrid.add_tile_sn(cell, tcls::CMT, 30, 60);
                        self.egrid.add_tile_sn(
                            cell,
                            match self.chip.col_side(ioc.col) {
                                DirH::W => tcls::RCLK_HDIOL_CMT_W,
                                DirH::E => tcls::RCLK_HDIOL_CMT_E,
                            },
                            30,
                            60,
                        );
                    }
                    IoRowKind::Xp5io => {
                        self.egrid.add_tile_sn(cell, tcls::CMTXP, 30, 60);
                        self.egrid.add_tile_sn(cell, tcls::XP5IO, 30, 60);
                        for idx in 0..66 {
                            self.io.push(IoCoord::Xp5io(Xp5ioCoord {
                                cell,
                                iob: TileIobId::from_idx(idx),
                            }));
                        }
                    }
                    _ => {
                        let tcid = match kind {
                            IoRowKind::Gth => tcls::GTH,
                            IoRowKind::Gty => tcls::GTY,
                            IoRowKind::Gtf => tcls::GTF,
                            IoRowKind::Gtm => tcls::GTM,
                            IoRowKind::HsAdc => tcls::HSADC,
                            IoRowKind::HsDac => tcls::HSDAC,
                            IoRowKind::RfAdc => tcls::RFADC,
                            IoRowKind::RfDac => tcls::RFDAC,
                            _ => unreachable!(),
                        };
                        self.egrid.add_tile_sn(cell, tcid, 30, 60);
                        self.gt.push(cell);
                    }
                }
            }
        }
    }

    fn fill_fe(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if cd.kind != ColumnKind::Sdfec {
                continue;
            }
            for reg in self.chip.regs() {
                if self.disabled.contains(&DisabledPart::Region(self.die, reg)) {
                    continue;
                }
                let cell = self.die.cell(col, self.chip.row_reg_bot(reg));
                self.egrid.add_tile_n(cell, tcls::FE, 60);
            }
        }
    }

    fn fill_dfe(&mut self) {
        for (col, &cd) in &self.chip.columns {
            let (tcid, bi) = match cd.kind {
                ColumnKind::DfeB => (tcls::DFE_B, false),
                ColumnKind::DfeC => (tcls::DFE_C, true),
                ColumnKind::DfeDF => (tcls::DFE_D, true),
                ColumnKind::DfeE => (tcls::DFE_E, true),
                _ => continue,
            };
            for reg in self.chip.regs() {
                let cell = self.die.cell(col, self.chip.row_reg_bot(reg));
                let tcid = if tcid == tcls::DFE_D && reg.to_idx() == 2 {
                    tcls::DFE_F
                } else {
                    tcid
                };
                if matches!(cd.kind, ColumnKind::DfeB | ColumnKind::DfeE) {
                    self.egrid
                        .add_tile_single(cell.delta(0, 30), tcls::RCLK_HROUTE_SPLITTER_HARD);
                    self.egrid.fill_conn_pair(
                        cell.delta(0, 30),
                        cell.delta(1, 30),
                        ccls::GCLK_H,
                        ccls::GCLK_H_REV,
                    );
                }
                if bi {
                    let mut tcells = vec![];
                    tcells.extend(cell.cells_n_const::<60>());
                    tcells.extend(cell.delta(1, 0).cells_n_const::<60>());
                    self.egrid.add_tile(cell, tcid, &tcells);
                } else {
                    self.egrid.add_tile_n(cell, tcid, 60);
                }
            }
        }
    }

    fn fill_hdios(&mut self) {
        for (col, &cd) in &self.chip.columns {
            if cd.kind != ColumnKind::HdioS {
                continue;
            }
            for reg in self.chip.regs() {
                let row = self.chip.row_reg_rclk(reg);
                let cell = self.die.cell(col, row);
                let mut tcells = vec![];
                tcells.extend(cell.delta(0, -30).cells_n_const::<60>());
                tcells.extend(cell.delta(1, -30).cells_n_const::<60>());
                self.egrid.add_tile_sn(cell, tcls::RCLK_HDIOS, 30, 60);
                self.egrid
                    .fill_conn_pair(cell, cell.delta(1, 0), ccls::GCLK_H, ccls::GCLK_H_REV);
                self.egrid.add_tile(cell, tcls::HDIOS, &tcells);
                for i in 0..42 {
                    self.io.push(IoCoord::HdioLc(HdioCoord {
                        cell,
                        iob: TileIobId::from_idx(i),
                    }));
                }
            }
        }
    }

    fn fill_rclk(&mut self) {
        for (col, cd) in &self.chip.columns {
            let Some((start, num)) = cd.clk else { continue };
            let ccls = match num {
                1 => [
                    ccls::RCLK1_0,
                    ccls::RCLK1_1,
                    ccls::RCLK1_2,
                    ccls::RCLK1_3,
                    ccls::RCLK1_4,
                    ccls::RCLK1_5,
                    ccls::RCLK1_6,
                    ccls::RCLK1_7,
                    ccls::RCLK1_8,
                    ccls::RCLK1_9,
                    ccls::RCLK1_10,
                    ccls::RCLK1_11,
                    ccls::RCLK1_12,
                    ccls::RCLK1_13,
                    ccls::RCLK1_14,
                    ccls::RCLK1_15,
                    ccls::RCLK1_16,
                    ccls::RCLK1_17,
                    ccls::RCLK1_18,
                    ccls::RCLK1_19,
                    ccls::RCLK1_20,
                    ccls::RCLK1_21,
                    ccls::RCLK1_22,
                    ccls::RCLK1_23,
                ][start],
                2 => [
                    ccls::RCLK2_0,
                    ccls::RCLK2_1,
                    ccls::RCLK2_2,
                    ccls::RCLK2_3,
                    ccls::RCLK2_4,
                    ccls::RCLK2_5,
                    ccls::RCLK2_6,
                    ccls::RCLK2_7,
                    ccls::RCLK2_8,
                    ccls::RCLK2_9,
                    ccls::RCLK2_10,
                    ccls::RCLK2_11,
                    ccls::RCLK2_12,
                    ccls::RCLK2_13,
                    ccls::RCLK2_14,
                    ccls::RCLK2_15,
                    ccls::RCLK2_16,
                    ccls::RCLK2_17,
                    ccls::RCLK2_18,
                    ccls::RCLK2_19,
                    ccls::RCLK2_20,
                    ccls::RCLK2_21,
                    ccls::RCLK2_22,
                ][start],
                3 => [
                    ccls::RCLK3_0,
                    ccls::RCLK3_1,
                    ccls::RCLK3_2,
                    ccls::RCLK3_3,
                    ccls::RCLK3_4,
                    ccls::RCLK3_5,
                    ccls::RCLK3_6,
                    ccls::RCLK3_7,
                    ccls::RCLK3_8,
                    ccls::RCLK3_9,
                    ccls::RCLK3_10,
                    ccls::RCLK3_11,
                    ccls::RCLK3_12,
                    ccls::RCLK3_13,
                    ccls::RCLK3_14,
                    ccls::RCLK3_15,
                    ccls::RCLK3_16,
                    ccls::RCLK3_17,
                    ccls::RCLK3_18,
                    ccls::RCLK3_19,
                    ccls::RCLK3_20,
                    ccls::RCLK3_21,
                ][start],
                4 => [
                    ccls::RCLK4_0,
                    ccls::RCLK4_1,
                    ccls::RCLK4_2,
                    ccls::RCLK4_3,
                    ccls::RCLK4_4,
                    ccls::RCLK4_5,
                    ccls::RCLK4_6,
                    ccls::RCLK4_7,
                    ccls::RCLK4_8,
                    ccls::RCLK4_9,
                    ccls::RCLK4_10,
                    ccls::RCLK4_11,
                    ccls::RCLK4_12,
                    ccls::RCLK4_13,
                    ccls::RCLK4_14,
                    ccls::RCLK4_15,
                    ccls::RCLK4_16,
                    ccls::RCLK4_17,
                    ccls::RCLK4_18,
                    ccls::RCLK4_19,
                    ccls::RCLK4_20,
                ][start],
                _ => unreachable!(),
            };
            for cell in self.egrid.column(self.die, col) {
                if cell.row == self.chip.row_rclk(cell.row) {
                    self.egrid.fill_conn_term(cell, ccls);
                }
            }
        }
        for (col, cd) in &self.chip.columns {
            let mut prev = None;
            if self.die.to_idx() != 0 {
                let prev_die: DieId = self.die - 1;
                let prev_chip = self.chips[prev_die];
                prev = Some(prev_die.cell(
                    col,
                    prev_chip.row_reg_rclk(RegId::from_idx(prev_chip.regs - 1)),
                ));
            }
            for cell in self.egrid.column(self.die, col) {
                if cell.row == self.chip.row_rclk(cell.row) {
                    if let Some(prev) = prev {
                        if prev.die == cell.die {
                            self.egrid
                                .fill_conn_pair(prev, cell, ccls::GCLK_N, ccls::GCLK_S);
                        } else if let Some((_, num)) = cd.clk {
                            let ccls_s = match num {
                                1 => ccls::GCLK_S_SLR1,
                                2 => ccls::GCLK_S_SLR2,
                                3 => ccls::GCLK_S_SLR3,
                                4 => ccls::GCLK_S_SLR4,
                                _ => unreachable!(),
                            };
                            self.egrid.fill_conn_pair(prev, cell, ccls::GCLK_N, ccls_s);
                        }
                    }
                    prev = Some(cell);
                }
            }
        }
    }

    fn fill_clkroot(&mut self) {
        let mut hdistr_col = EntityVec::new();
        let mut hroute_col = EntityVec::new();
        let mut cur_hd = self.chip.columns.first_id().unwrap();
        let mut cur_hr = self.chip.columns.first_id().unwrap();
        for (col, cd) in &self.chip.columns {
            match cd.kind {
                ColumnKind::CleM(CleMKind::ClkBuf) => {
                    cur_hr = col;
                }
                ColumnKind::Io(_) if col != self.chip.columns.last_id().unwrap() => {
                    cur_hr = col;
                    cur_hd = col;
                }
                _ => (),
            }
            hdistr_col.push(cur_hd);
            hroute_col.push(cur_hr);
            match cd.kind {
                ColumnKind::Dsp(DspKind::ClkBuf) => {
                    cur_hd = col + 1;
                    cur_hr = col + 1;
                }
                ColumnKind::Hard(_, _)
                | ColumnKind::DfeE
                | ColumnKind::DfeB
                | ColumnKind::HdioS => {
                    cur_hr = col + 1;
                }
                _ => (),
            }
        }
        for cell in self.egrid.die_cells(self.die) {
            let cell_leaf = cell.with_row(cell.row.snap_to_side_of(self.chip.row_rclk(cell.row)));
            self.egrid[cell].region_root[rslots::LEAF] = cell_leaf;
            self.egrid[cell].region_root[rslots::HDISTR] = cell.with_col(hdistr_col[cell.col]);
            self.egrid[cell].region_root[rslots::HROUTE] = cell.with_col(hroute_col[cell.col]);
        }
    }
}

fn fill_cfg_io_hpio(chip: &Chip, cell: CellCoord) -> BiHashMap<SharedCfgPad, IoCoord> {
    let mut res = BiHashMap::new();
    for idx in 0..52 {
        if let Some(cfg) = if !chip.is_alt_cfg {
            match idx {
                0 => Some(SharedCfgPad::Rs(0)),
                1 => Some(SharedCfgPad::Rs(1)),
                2 => Some(SharedCfgPad::FoeB),
                3 => Some(SharedCfgPad::FweB),
                4 => Some(SharedCfgPad::Addr(26)),
                5 => Some(SharedCfgPad::Addr(27)),
                6 => Some(SharedCfgPad::Addr(24)),
                7 => Some(SharedCfgPad::Addr(25)),
                8 => Some(SharedCfgPad::Addr(22)),
                9 => Some(SharedCfgPad::Addr(23)),
                10 => Some(SharedCfgPad::Addr(20)),
                11 => Some(SharedCfgPad::Addr(21)),
                12 => Some(SharedCfgPad::Addr(28)),
                13 => Some(SharedCfgPad::Addr(18)),
                14 => Some(SharedCfgPad::Addr(19)),
                15 => Some(SharedCfgPad::Addr(16)),
                16 => Some(SharedCfgPad::Addr(17)),
                17 => Some(SharedCfgPad::Data(30)),
                18 => Some(SharedCfgPad::Data(31)),
                19 => Some(SharedCfgPad::Data(28)),
                20 => Some(SharedCfgPad::Data(29)),
                21 => Some(SharedCfgPad::Data(26)),
                22 => Some(SharedCfgPad::Data(27)),
                23 => Some(SharedCfgPad::Data(24)),
                24 => Some(SharedCfgPad::Data(25)),
                25 => Some(if chip.kind == ChipKind::Ultrascale {
                    SharedCfgPad::PerstN1
                } else {
                    SharedCfgPad::SmbAlert
                }),
                26 => Some(SharedCfgPad::Data(22)),
                27 => Some(SharedCfgPad::Data(23)),
                28 => Some(SharedCfgPad::Data(20)),
                29 => Some(SharedCfgPad::Data(21)),
                30 => Some(SharedCfgPad::Data(18)),
                31 => Some(SharedCfgPad::Data(19)),
                32 => Some(SharedCfgPad::Data(16)),
                33 => Some(SharedCfgPad::Data(17)),
                34 => Some(SharedCfgPad::Data(14)),
                35 => Some(SharedCfgPad::Data(15)),
                36 => Some(SharedCfgPad::Data(12)),
                37 => Some(SharedCfgPad::Data(13)),
                38 => Some(SharedCfgPad::CsiB),
                39 => Some(SharedCfgPad::Data(10)),
                40 => Some(SharedCfgPad::Data(11)),
                41 => Some(SharedCfgPad::Data(8)),
                42 => Some(SharedCfgPad::Data(9)),
                43 => Some(SharedCfgPad::Data(6)),
                44 => Some(SharedCfgPad::Data(7)),
                45 => Some(SharedCfgPad::Data(4)),
                46 => Some(SharedCfgPad::Data(5)),
                47 => Some(SharedCfgPad::I2cScl),
                48 => Some(SharedCfgPad::I2cSda),
                49 => Some(SharedCfgPad::EmCclk),
                50 => Some(SharedCfgPad::Dout),
                51 => Some(SharedCfgPad::PerstN0),
                _ => None,
            }
        } else {
            match idx {
                0 => Some(SharedCfgPad::Rs(1)),
                1 => Some(SharedCfgPad::FweB),
                2 => Some(SharedCfgPad::Rs(0)),
                3 => Some(SharedCfgPad::FoeB),
                4 => Some(SharedCfgPad::Addr(28)),
                5 => Some(SharedCfgPad::Addr(26)),
                6 => Some(SharedCfgPad::SmbAlert),
                7 => Some(SharedCfgPad::Addr(27)),
                8 => Some(SharedCfgPad::Addr(24)),
                9 => Some(SharedCfgPad::Addr(22)),
                10 => Some(SharedCfgPad::Addr(25)),
                11 => Some(SharedCfgPad::Addr(23)),
                12 => Some(SharedCfgPad::Addr(20)),
                13 => Some(SharedCfgPad::Addr(18)),
                14 => Some(SharedCfgPad::Addr(16)),
                15 => Some(SharedCfgPad::Addr(19)),
                16 => Some(SharedCfgPad::Addr(17)),
                17 => Some(SharedCfgPad::Data(30)),
                18 => Some(SharedCfgPad::Data(28)),
                19 => Some(SharedCfgPad::Data(31)),
                20 => Some(SharedCfgPad::Data(29)),
                21 => Some(SharedCfgPad::Data(26)),
                22 => Some(SharedCfgPad::Data(24)),
                23 => Some(SharedCfgPad::Data(27)),
                24 => Some(SharedCfgPad::Data(25)),
                25 => Some(SharedCfgPad::Addr(21)),
                26 => Some(SharedCfgPad::CsiB),
                27 => Some(SharedCfgPad::Data(22)),
                28 => Some(SharedCfgPad::EmCclk),
                29 => Some(SharedCfgPad::Data(23)),
                30 => Some(SharedCfgPad::Data(20)),
                31 => Some(SharedCfgPad::Data(18)),
                32 => Some(SharedCfgPad::Data(21)),
                33 => Some(SharedCfgPad::Data(19)),
                34 => Some(SharedCfgPad::Data(16)),
                35 => Some(SharedCfgPad::Data(14)),
                36 => Some(SharedCfgPad::Data(17)),
                37 => Some(SharedCfgPad::Data(15)),
                38 => Some(SharedCfgPad::Data(12)),
                39 => Some(SharedCfgPad::Data(10)),
                40 => Some(SharedCfgPad::Data(8)),
                41 => Some(SharedCfgPad::Data(11)),
                42 => Some(SharedCfgPad::Data(9)),
                43 => Some(SharedCfgPad::Data(6)),
                44 => Some(SharedCfgPad::Data(4)),
                45 => Some(SharedCfgPad::Data(7)),
                46 => Some(SharedCfgPad::Data(5)),
                47 => Some(SharedCfgPad::I2cScl),
                48 => Some(SharedCfgPad::Dout),
                49 => Some(SharedCfgPad::I2cSda),
                50 => Some(SharedCfgPad::PerstN0),
                51 => Some(SharedCfgPad::Data(13)),
                _ => None,
            }
        } {
            res.insert(
                cfg,
                IoCoord::Hpio(HpioCoord {
                    cell,
                    iob: TileIobId::from_idx(idx),
                }),
            );
        }
    }
    res
}

fn fill_cfg_io_hdiol(cell: CellCoord) -> BiHashMap<SharedCfgPad, IoCoord> {
    let mut res = BiHashMap::new();
    for idx in 0..84 {
        if let Some(cfg) = match idx {
            14 => Some(SharedCfgPad::Data(31)),
            15 => Some(SharedCfgPad::Data(30)),
            16 => Some(SharedCfgPad::Data(28)),
            17 => Some(SharedCfgPad::Data(26)),
            18 => Some(SharedCfgPad::Data(24)),
            19 => Some(SharedCfgPad::Data(22)),
            21 => Some(SharedCfgPad::Data(20)),
            22 => Some(SharedCfgPad::Data(18)),
            23 => Some(SharedCfgPad::Data(16)),
            24 => Some(SharedCfgPad::Data(14)),
            30 => Some(SharedCfgPad::Data(29)),
            31 => Some(SharedCfgPad::Data(27)),
            32 => Some(SharedCfgPad::Data(25)),
            33 => Some(SharedCfgPad::Data(23)),
            35 => Some(SharedCfgPad::Data(21)),
            36 => Some(SharedCfgPad::Data(19)),
            37 => Some(SharedCfgPad::Data(17)),
            38 => Some(SharedCfgPad::Data(15)),
            39 => Some(SharedCfgPad::Data(13)),
            40 => Some(SharedCfgPad::Data(12)),
            43 => Some(SharedCfgPad::EmCclk),
            57 => Some(SharedCfgPad::Data(11)),
            58 => Some(SharedCfgPad::Data(10)),
            59 => Some(SharedCfgPad::Data(8)),
            60 => Some(SharedCfgPad::Data(7)),
            61 => Some(SharedCfgPad::Data(5)),
            62 => Some(SharedCfgPad::Busy),
            64 => Some(SharedCfgPad::Fcs1B),
            65 => Some(SharedCfgPad::CsiB),
            66 => Some(SharedCfgPad::I2cSda),
            67 => Some(SharedCfgPad::I2cScl),
            68 => Some(SharedCfgPad::PerstN0),
            69 => Some(SharedCfgPad::SmbAlert),
            73 => Some(SharedCfgPad::Data(9)),
            74 => Some(SharedCfgPad::OspiDs),
            75 => Some(SharedCfgPad::Data(6)),
            76 => Some(SharedCfgPad::Data(4)),
            80 => Some(SharedCfgPad::OspiRstB),
            81 => Some(SharedCfgPad::OspiEccFail),
            _ => None,
        } {
            res.insert(
                cfg,
                if idx < 42 {
                    IoCoord::HdioLc(HdioCoord {
                        cell,
                        iob: TileIobId::from_idx(idx),
                    })
                } else {
                    IoCoord::HdioLc(HdioCoord {
                        cell: cell.delta(0, 30),
                        iob: TileIobId::from_idx(idx - 42),
                    })
                },
            );
        }
    }
    res
}

fn fill_cfg_io_hdios(cell: CellCoord) -> BiHashMap<SharedCfgPad, IoCoord> {
    let mut res = BiHashMap::new();
    for idx in 0..42 {
        if let Some(cfg) = match idx {
            0 => Some(SharedCfgPad::Data(31)),
            1 => Some(SharedCfgPad::Data(30)),
            2 => Some(SharedCfgPad::Data(28)),
            3 => Some(SharedCfgPad::Data(26)),
            4 => Some(SharedCfgPad::Data(24)),
            5 => Some(SharedCfgPad::Data(22)),
            6 => Some(SharedCfgPad::Data(20)),
            7 => Some(SharedCfgPad::Data(18)),
            8 => Some(SharedCfgPad::Data(16)),
            9 => Some(SharedCfgPad::Data(14)),
            12 => Some(SharedCfgPad::Data(29)),
            13 => Some(SharedCfgPad::Data(27)),
            14 => Some(SharedCfgPad::Data(25)),
            15 => Some(SharedCfgPad::Data(23)),
            16 => Some(SharedCfgPad::Data(21)),
            17 => Some(SharedCfgPad::Data(19)),
            18 => Some(SharedCfgPad::Data(17)),
            19 => Some(SharedCfgPad::Data(15)),
            20 => Some(SharedCfgPad::Data(13)),
            21 => Some(SharedCfgPad::Data(12)),
            22 => Some(SharedCfgPad::EmCclk),
            24 => Some(SharedCfgPad::Data(8)),
            25 => Some(SharedCfgPad::Data(7)),
            26 => Some(SharedCfgPad::Data(5)),
            27 => Some(SharedCfgPad::Busy),
            28 => Some(SharedCfgPad::Fcs1B),
            29 => Some(SharedCfgPad::CsiB),
            30 => Some(SharedCfgPad::I2cSda),
            31 => Some(SharedCfgPad::I2cScl),
            32 => Some(SharedCfgPad::PerstN0),
            33 => Some(SharedCfgPad::SmbAlert),
            34 => Some(SharedCfgPad::Data(11)),
            35 => Some(SharedCfgPad::Data(10)),
            36 => Some(SharedCfgPad::Data(9)),
            37 => Some(SharedCfgPad::OspiDs),
            38 => Some(SharedCfgPad::Data(6)),
            39 => Some(SharedCfgPad::Data(4)),
            40 => Some(SharedCfgPad::OspiRstB),
            41 => Some(SharedCfgPad::OspiEccFail),
            _ => None,
        } {
            res.insert(
                cfg,
                IoCoord::HdioLc(HdioCoord {
                    cell,
                    iob: TileIobId::from_idx(idx),
                }),
            );
        }
    }
    res
}

pub fn expand_grid<'a>(
    chips: &EntityVec<DieId, &'a Chip>,
    interposer: &'a Interposer,
    disabled: &BTreeSet<DisabledPart>,
    db: &'a IntDb,
) -> ExpandedDevice<'a> {
    let mut egrid = GridBuilder::new(db);
    let pchip = chips[interposer.primary];
    let mut has_pcie_cfg = false;
    let mut io = vec![];
    let mut gt = vec![];
    for (_, chip) in chips {
        let die = egrid.add_die(chip.columns.len(), chip.regs * 60);

        let mut expander = DieExpander {
            chip,
            chips,
            disabled,
            egrid: &mut egrid,
            die,
            io: &mut io,
            gt: &mut gt,
        };
        expander.fill_int();
        expander.fill_ps();
        expander.fill_term();
        expander.fill_main_passes();
        expander.fill_clb();
        expander.fill_bram();
        expander.fill_dsp();
        expander.fill_uram();
        expander.fill_fe();
        expander.fill_dfe();
        expander.fill_hdios();
        expander.fill_hard(&mut has_pcie_cfg);
        expander.fill_io();
        expander.fill_rclk();
        expander.fill_clkroot();
    }

    let is_cut = disabled
        .iter()
        .any(|x| matches!(x, DisabledPart::Region(..)));
    let is_cut_d = disabled.contains(&DisabledPart::Region(
        DieId::from_idx(0),
        RegId::from_idx(0),
    ));

    let egrid = egrid.finish();

    let mut col_cfg_io = None;
    let reg_cfg = pchip.reg_cfg();
    for (col, &cd) in &pchip.columns {
        if let ColumnKind::Io(idx) = cd.kind
            && (col_cfg_io.is_none() || pchip.col_side(col) == DirH::W)
            && (!pchip.config_kind.is_csec()
                || matches!(
                    pchip.cols_io[idx].regs[reg_cfg],
                    IoRowKind::HdioL | IoRowKind::HdioLMrc
                ))
        {
            col_cfg_io = Some(col);
        }
        if cd.kind == ColumnKind::HdioS {
            col_cfg_io = Some(col);
        }
        if let ColumnKind::Hard(HardKind::Term, idx) = cd.kind {
            let mut has_hdiolc = false;
            for chip in chips.values() {
                if chip.cols_hard[idx]
                    .regs
                    .values()
                    .any(|&kind| kind == HardRowKind::HdioL)
                {
                    has_hdiolc = true;
                }
            }
            if has_hdiolc {
                col_cfg_io = Some(col);
            }
        }
    }
    let col_cfg_io = col_cfg_io.unwrap();

    let mut ioxlut = EntityPartVec::new();
    let mut bankxlut = EntityPartVec::new();
    let mut iox = 0;
    for (col, &cd) in &pchip.columns {
        match cd.kind {
            ColumnKind::Io(_) => {
                ioxlut.insert(col, iox);
                iox += 1;
            }
            ColumnKind::HdioS => {
                ioxlut.insert(col, iox);
                iox += 1;
            }
            ColumnKind::Hard(_, idx) => {
                let regs = &pchip.cols_hard[idx].regs;
                if regs.values().any(|x| {
                    matches!(
                        x,
                        HardRowKind::Hdio
                            | HardRowKind::HdioAms
                            | HardRowKind::HdioL
                            | HardRowKind::HdioS
                    )
                }) {
                    ioxlut.insert(col, iox);
                    iox += 1;
                }
            }
            _ => (),
        }
    }
    let iox_cfg = ioxlut[col_cfg_io];
    for (col, &iox) in &ioxlut {
        let mut bank = (40 + iox * 20 - iox_cfg * 20) as u32;
        if col.to_idx() == 0
            && iox != iox_cfg
            && pchip.kind == ChipKind::UltrascalePlus
            && pchip.cols_hard.len() == 1
            && !pchip.config_kind.is_csec()
        {
            bank -= 20;
        }
        bankxlut.insert(col, bank);
    }

    let mut bank = 0;
    let mut bankylut = EntityVec::new();
    let mut cfg_bank = None;
    for (die, &chip) in chips {
        let mut ylut = EntityPartVec::new();
        for reg in chip.regs() {
            let mut has_io = false;
            let mut has_hdiolc = false;
            for hcol in &chip.cols_hard {
                match hcol.regs[reg] {
                    HardRowKind::Cfg => {
                        if die == interposer.primary {
                            cfg_bank = Some(bank);
                        }
                    }
                    HardRowKind::Hdio | HardRowKind::HdioAms => {
                        has_io = true;
                    }
                    HardRowKind::HdioL => {
                        has_hdiolc = true;
                    }
                    _ => (),
                }
            }
            for iocol in &chip.cols_io {
                match iocol.regs[reg] {
                    IoRowKind::Hpio | IoRowKind::Hrio => {
                        has_io = true;
                    }
                    IoRowKind::HdioL | IoRowKind::HdioLMrc => {
                        has_hdiolc = true;
                    }
                    _ => (),
                }
            }
            if has_hdiolc {
                ylut.insert(reg, bank);
                bank += 2;
            } else if has_io {
                ylut.insert(reg, bank);
                bank += 1;
            }
        }
        bankylut.push(ylut);
    }
    let cfg_bank = cfg_bank.unwrap();
    for ylut in bankylut.values_mut() {
        for bank in ylut.values_mut() {
            *bank += 25;
            *bank -= cfg_bank;
        }
    }

    let mut cfg_io = EntityVec::new();
    for (die, &chip) in chips {
        let mut die_cfg_io = BiHashMap::new();
        if let Some(iocol) = chip.cols_io.iter().find(|iocol| iocol.col == col_cfg_io) {
            if matches!(
                iocol.regs[chip.reg_cfg()],
                IoRowKind::Hpio | IoRowKind::Hrio
            ) {
                let cell = die.cell(iocol.col, chip.row_reg_rclk(chip.reg_cfg()));
                die_cfg_io = fill_cfg_io_hpio(chip, cell);
            } else if matches!(
                iocol.regs[chip.reg_cfg()],
                IoRowKind::HdioL | IoRowKind::HdioLMrc
            ) {
                let cell = die.cell(iocol.col, chip.row_reg_bot(chip.reg_cfg()));
                die_cfg_io = fill_cfg_io_hdiol(cell);
            }
        } else if let Some(hcol) = chip.cols_hard.iter().find(|hcol| hcol.col == col_cfg_io) {
            let cell = die.cell(hcol.col, chip.row_reg_bot(chip.reg_cfg()));
            die_cfg_io = fill_cfg_io_hdiol(cell);
        } else {
            let cell = die.cell(col_cfg_io, chip.row_reg_rclk(chip.reg_cfg()));
            die_cfg_io = fill_cfg_io_hdios(cell);
        }
        cfg_io.push(die_cfg_io);
    }

    ExpandedDevice {
        kind: chips[interposer.primary].kind,
        chips: chips.clone(),
        interposer,
        egrid,
        disabled: disabled.clone(),
        has_pcie_cfg,
        is_cut,
        is_cut_d,
        io,
        cfg_io,
        gt,
        col_cfg_io,
        bankxlut,
        bankylut,
    }
}
