use prjcombine_entity::{EntityId, EntityIdArith, EntityRange, EntityVec};
use prjcombine_interconnect::db::{ConnectorClassId, IntDb};
use prjcombine_interconnect::dir::{DirH, DirV};
use prjcombine_interconnect::grid::builder::GridBuilder;
use prjcombine_interconnect::grid::{BelCoord, CellCoord, ColId, DieId, DieIdExt, RowId, RowIdExt};
use std::collections::{BTreeSet, HashMap};

use crate::chip::{
    AiArray, AiKind, BramKind, Chip, CleKind, ColumnKind, DisabledPart, FabricKind, GtRowKind,
    HardRowKind, HbmRow, HsrNKind, HsrSKind, Interposer, InterposerKind, Me2HNoc, Me2Row, MeRow,
    MeVNocConn, NicBlock, NicRow, PsKind, RegId, X5ioClkBli, X5ioDdrmcKind, X5ioDdrmcTestClk,
    X5ioHNoc, X5ioPllBli, X5ioRow, X5ioScanBli, XpioClkBli, XpioRow, XpioTestBli,
};
use crate::defs::{bslots, ccls, rslots, tcls};
use crate::expanded::{ExpandedDevice, SllConn};
use crate::noc::{BelCoordExt, NocBuilder, NocPortCoord};

struct DieInfo {
    col_cfrm: ColId,
    cols_vnoc: Vec<ColId>,
}

struct Expander<'a> {
    chips: EntityVec<DieId, &'a Chip>,
    interposer: &'a Interposer,
    disabled: BTreeSet<DisabledPart>,
    egrid: GridBuilder<'a>,
    die: EntityVec<DieId, DieInfo>,
    noc: NocBuilder<'a>,
}

impl Expander<'_> {
    fn in_int_hole(&self, cell: CellCoord) -> bool {
        cell.row.to_idx() < self.chips[cell.die].ps.height()
            && cell.col < self.die[cell.die].col_cfrm
    }

    fn in_bli(&self, cell: CellCoord) -> bool {
        let chip = self.chips[cell.die];
        if cell.row < chip.row_s() + 4 {
            chip.columns[cell.col].has_bli_s
        } else if cell.row >= chip.row_n() - 3 {
            chip.columns[cell.col].has_bli_n
        } else {
            false
        }
    }

    fn column_pl(
        &self,
        die: DieId,
        col: ColId,
    ) -> impl DoubleEndedIterator<Item = CellCoord> + ExactSizeIterator + 'static {
        self.chips[die].rows_pl().map(move |row| die.cell(col, row))
    }

    pub fn die_cells_pl(&self, die: DieId) -> impl DoubleEndedIterator<Item = CellCoord> + 'static {
        let rows = self.chips[die].rows_pl();
        self.egrid
            .cols(die)
            .into_iter()
            .flat_map(move |col| rows.map(move |row| die.cell(col, row)))
    }

    fn fill_die(&mut self) {
        for (_, &chip) in &self.chips {
            self.egrid
                .add_die(chip.columns.len(), chip.rows_ai().end().to_idx());
            self.die.push(DieInfo {
                col_cfrm: chip
                    .columns
                    .iter()
                    .find(|(_, cd)| cd.kind == ColumnKind::Cfrm)
                    .unwrap()
                    .0,
                cols_vnoc: chip
                    .columns
                    .iter()
                    .filter_map(|(col, cd)| {
                        if matches!(
                            cd.kind,
                            ColumnKind::VNoc | ColumnKind::VNoc2 | ColumnKind::VNoc4
                        ) {
                            Some(col)
                        } else {
                            None
                        }
                    })
                    .collect(),
            });
        }
    }

    fn fill_int(&mut self) {
        for (die, chip) in &self.chips {
            for col in self.egrid.cols(die) {
                if self.disabled.contains(&DisabledPart::Column(die, col)) {
                    continue;
                }
                for cell in self.column_pl(die, col) {
                    let reg = chip.row_to_reg(cell.row);
                    if self.disabled.contains(&DisabledPart::Region(die, reg)) {
                        continue;
                    }
                    if self.in_int_hole(cell) {
                        continue;
                    }
                    if chip.col_side(col) == DirH::W {
                        self.egrid.add_tile_e(cell, tcls::INT, 2);
                        if cell.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG)
                            && chip.is_reg_n(reg)
                        {
                            self.egrid.add_tile_e(cell, tcls::RCLK, 2);
                        }
                    }
                }
            }

            for cell in self.die_cells_pl(die) {
                if self.in_int_hole(cell) {
                    continue;
                }
                let cell_e = cell.delta(1, 0);
                if cell.col == chip.col_e() || self.in_int_hole(cell_e) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_E);
                } else {
                    self.egrid
                        .fill_conn_pair(cell, cell_e, ccls::PASS_E, ccls::PASS_W);
                }
                if cell.col == chip.col_w() || self.in_int_hole(cell.delta(-1, 0)) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_W);
                }
            }

            for cell in self.die_cells_pl(die) {
                if chip.col_side(cell.col) != DirH::W {
                    continue;
                }
                if self.in_int_hole(cell) {
                    continue;
                }
                let cell_e = cell.delta(2, 0);
                if cell.col == chip.col_e() - 1 || self.in_int_hole(cell_e) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_LE);
                } else {
                    self.egrid
                        .fill_conn_pair(cell, cell_e, ccls::PASS_LE, ccls::PASS_LW);
                }
                if cell.col == chip.col_w() || self.in_int_hole(cell.delta(-2, 0)) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_LW);
                }
            }

            for cell in self.die_cells_pl(die) {
                let cell_n = cell.delta(0, 1);
                if cell.row == chip.row_n() || self.in_int_hole(cell_n) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_N);
                } else {
                    self.egrid
                        .fill_conn_pair(cell, cell_n, ccls::PASS_N, ccls::PASS_S);
                }
                if cell.row == chip.row_s() || self.in_int_hole(cell.delta(0, -1)) {
                    self.egrid.fill_conn_term(cell, ccls::TERM_S);
                }
            }
        }
    }

    fn fill_cle_bc(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if matches!(cd.kind, ColumnKind::Cle(_)) && chip.col_side(col) == DirH::E {
                    for cell in self.column_pl(die, col) {
                        if self.in_int_hole(cell) {
                            continue;
                        }
                        let tcid = match cd.kind {
                            ColumnKind::Cle(CleKind::Plain) => tcls::CLE_BC,
                            ColumnKind::Cle(CleKind::Sll) => {
                                if self.in_bli(cell) {
                                    tcls::CLE_BC
                                } else {
                                    tcls::CLE_BC_SLL
                                }
                            }
                            ColumnKind::Cle(CleKind::Sll2) => {
                                if self.in_bli(cell) {
                                    tcls::CLE_BC
                                } else {
                                    tcls::CLE_BC_SLL2
                                }
                            }
                            _ => unreachable!(),
                        };
                        self.egrid[cell.delta(1, 0)].region_root[rslots::CLE] = cell;
                        self.egrid.add_tile_e(cell, tcid, 2);
                        if self.in_bli(cell) {
                            self.egrid.fill_conn_term(cell, ccls::CLE_BLI_E);
                            self.egrid.fill_conn_term(cell.delta(1, 0), ccls::CLE_BLI_W);
                        } else {
                            self.egrid.fill_conn_term(cell, ccls::CLE_E);
                            self.egrid.fill_conn_term(cell.delta(1, 0), ccls::CLE_W);
                        }
                        let reg = chip.row_to_reg(cell.row);
                        if cell.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG) {
                            let cell = cell.delta(1, 0);
                            if chip.is_reg_half(reg) || chip.is_reg_n(reg) {
                                self.egrid.add_tile_single(cell, tcls::RCLK_CLE);
                            }
                        }
                    }
                }
            }
        }
    }

    fn fill_intf(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if cd.has_bli_s {
                    let cell = die.cell(col, chip.row_s());
                    if matches!(cd.kind, ColumnKind::Cle(_)) {
                        self.egrid.add_tile_n(cell, tcls::BLI_CLE_S, 4);
                        self.egrid.add_tile_single(cell, tcls::INTF_BLI_CLE_D_S);
                        for dy in 1..=3 {
                            self.egrid
                                .add_tile_single(cell.delta(0, dy), tcls::INTF_BLI_CLE_ABC_S);
                        }
                    } else {
                        self.egrid.add_tile_n(cell, tcls::BLI_S, 4);
                    }
                    if chip.hsr_s.has_bli_mux_noc() {
                        self.egrid.add_tile_single(cell, tcls::BLI_MUX_NOC);
                    } else {
                        self.egrid.add_tile_single(cell, tcls::BLI_MUX);
                    }
                }

                if cd.has_bli_n {
                    let cell = die.cell(col, chip.row_n());
                    if matches!(cd.kind, ColumnKind::Cle(_)) {
                        self.egrid.add_tile_sn(cell, tcls::BLI_CLE_N, 3, 4);
                        self.egrid.add_tile_single(cell, tcls::INTF_BLI_CLE_D_N);
                        for dy in -3..=-1 {
                            self.egrid
                                .add_tile_single(cell.delta(0, dy), tcls::INTF_BLI_CLE_ABC_N);
                        }
                    } else {
                        self.egrid.add_tile_sn(cell, tcls::BLI_N, 3, 4);
                    }
                    self.egrid.add_tile_single(cell, tcls::BLI_MUX);
                }

                for cell in self.column_pl(die, col) {
                    if self.in_int_hole(cell) {
                        continue;
                    }

                    let side = chip.col_side(col);
                    if !matches!(cd.kind, ColumnKind::Cle(_) | ColumnKind::None) {
                        let tcid = match (cd.kind, side) {
                            (ColumnKind::Gt, DirH::W) => tcls::INTF_W_TERM_GT,
                            (
                                ColumnKind::Term
                                | ColumnKind::TermSsit
                                | ColumnKind::Gt
                                | ColumnKind::HNicX,
                                DirH::E,
                            ) => tcls::INTF_E_TERM_GT,
                            (ColumnKind::Cfrm, DirH::W) => {
                                if cell.row.to_idx() < chip.ps.height() {
                                    tcls::INTF_W_TERM_PSS
                                } else {
                                    tcls::INTF_W_PSS
                                }
                            }
                            (ColumnKind::Hard, DirH::E) => {
                                let ch = chip.get_col_hard(col).unwrap();
                                match ch.regs[chip.row_to_reg(cell.row)] {
                                    HardRowKind::Hdio => tcls::INTF_E_HDIO,
                                    _ => tcls::INTF_E_HB,
                                }
                            }
                            (ColumnKind::ContHard, DirH::W) => {
                                let ch = chip.get_col_hard(col - 1).unwrap();
                                match ch.regs[chip.row_to_reg(cell.row)] {
                                    HardRowKind::Hdio => tcls::INTF_W_HDIO,
                                    _ => tcls::INTF_W_HB,
                                }
                            }
                            (_, DirH::W) => tcls::INTF_W,
                            (_, DirH::E) => tcls::INTF_E,
                        };
                        self.egrid.add_tile_single(cell, tcid);
                    }
                    let reg = chip.row_to_reg(cell.row);
                    if cell.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG)
                        && chip.is_reg_n(reg)
                        && cd.kind == ColumnKind::TermSsit
                    {
                        self.egrid.add_tile_single(cell, tcls::RCLK_SPLITTER_SSIT);
                    }

                    if cell.row.to_idx().is_multiple_of(Chip::ROWS_PER_REG)
                        && chip.is_reg_n(reg)
                        && !matches!(
                            cd.kind,
                            ColumnKind::Cle(_) | ColumnKind::TermSsit | ColumnKind::None
                        )
                    {
                        self.egrid.add_tile_single(cell, tcls::RCLK_INTF);
                        if matches!(
                            cd.kind,
                            ColumnKind::ContDsp | ColumnKind::Bram(_) | ColumnKind::Uram
                        ) {
                            self.egrid.add_tile_single(cell, tcls::RCLK_DFX);
                        }
                        if let ColumnKind::Bram(kind) = cd.kind {
                            if kind == BramKind::SplitterPd {
                                self.egrid.add_tile_e(
                                    cell,
                                    if chip.fabric == FabricKind::Ulvt {
                                        tcls::RCLK_SPLITTER_PD_VR
                                    } else {
                                        tcls::RCLK_SPLITTER_PD
                                    },
                                    2,
                                );
                            } else if kind == BramKind::Splitter
                                || (kind == BramKind::OptSplitter
                                    && cell.row.to_idx() >= chip.ps.height())
                            {
                                self.egrid.add_tile_e(
                                    cell,
                                    if chip.fabric == FabricKind::Ulvt {
                                        tcls::RCLK_SPLITTER_VR
                                    } else {
                                        tcls::RCLK_SPLITTER
                                    },
                                    2,
                                );
                            }
                        }
                        if cd.kind == ColumnKind::Hard {
                            let hc = chip.get_col_hard(col).unwrap();
                            if hc.regs[reg] == HardRowKind::Hdio {
                                self.egrid.add_tile_single(
                                    cell,
                                    if chip.fabric == FabricKind::Ulvt {
                                        tcls::RCLK_HDIO_VR
                                    } else {
                                        tcls::RCLK_HDIO
                                    },
                                );
                            } else if !reg.to_idx().is_multiple_of(2)
                                && hc.regs[reg - 1] == HardRowKind::Hdio
                            {
                                self.egrid.add_tile_single(
                                    cell,
                                    if chip.fabric == FabricKind::Ulvt {
                                        tcls::RCLK_HB_HDIO_VR
                                    } else {
                                        tcls::RCLK_HB_HDIO
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn fill_cle(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if !matches!(cd.kind, ColumnKind::Cle(_)) {
                    continue;
                }
                for cell in self.column_pl(die, col) {
                    if self.in_int_hole(cell) || self.in_bli(cell) {
                        continue;
                    }

                    self.egrid.add_tile_single(
                        cell,
                        if chip.fabric == FabricKind::Ulvt {
                            tcls::CLE_VR
                        } else {
                            tcls::CLE
                        },
                    );
                }
            }
        }
    }

    fn fill_dsp(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if cd.kind != ColumnKind::Dsp {
                    continue;
                }
                for cell in self.column_pl(die, col) {
                    if self.in_int_hole(cell) || self.in_bli(cell) {
                        continue;
                    }
                    if !cell.row.to_idx().is_multiple_of(2) {
                        continue;
                    }
                    self.egrid.add_tile(
                        cell,
                        tcls::DSP,
                        &[
                            cell.delta(0, 0),
                            cell.delta(0, 1),
                            cell.delta(1, 0),
                            cell.delta(1, 1),
                        ],
                    );
                }
            }
        }
    }

    fn fill_bram(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if !matches!(cd.kind, ColumnKind::Bram(_)) {
                    continue;
                }
                for cell in self.column_pl(die, col) {
                    if self.in_int_hole(cell) || self.in_bli(cell) {
                        continue;
                    }
                    if !cell.row.to_idx().is_multiple_of(4) {
                        continue;
                    }
                    self.egrid.add_tile_n(cell, tcls::BRAM, 4);
                }
            }
        }
    }

    fn fill_uram(&mut self) {
        for (die, chip) in &self.chips {
            for (col, &cd) in &chip.columns {
                if cd.kind != ColumnKind::Uram {
                    continue;
                }
                for cell in self.column_pl(die, col) {
                    if self.in_int_hole(cell) || self.in_bli(cell) {
                        continue;
                    }
                    if !cell.row.to_idx().is_multiple_of(4) {
                        continue;
                    }
                    self.egrid.add_tile_n(cell, tcls::URAM, 4);
                }
            }
        }
    }

    fn fill_hard(&mut self) {
        for (die, chip) in &self.chips {
            for hc in &chip.cols_hard {
                for reg in chip.regs() {
                    if self
                        .disabled
                        .contains(&DisabledPart::HardIp(die, hc.col, reg))
                    {
                        continue;
                    }
                    if self.disabled.contains(&DisabledPart::Region(die, reg)) {
                        continue;
                    }
                    let kind = hc.regs[reg];
                    let tcid = match kind {
                        HardRowKind::None => continue,
                        HardRowKind::Hdio => tcls::HDIO,
                        HardRowKind::CpmExt => {
                            // handled with PS
                            continue;
                        }
                        HardRowKind::Pcie4 => tcls::PCIE4,
                        HardRowKind::Pcie5 => tcls::PCIE5,
                        HardRowKind::Mrmac => tcls::MRMAC,
                        HardRowKind::Dcmac => tcls::DCMAC,
                        HardRowKind::Ilkn => tcls::ILKN,
                        HardRowKind::Hsc => tcls::HSC,
                        HardRowKind::Sdfec => tcls::SDFEC,
                        HardRowKind::DfeCfc => tcls::DFE_CFC,
                    };
                    let cell = die.cell(hc.col, chip.row_reg_bot(reg));
                    let mut tcells = vec![];
                    let height = if kind.is_double() {
                        Chip::ROWS_PER_REG * 2
                    } else {
                        Chip::ROWS_PER_REG
                    };
                    tcells.extend(cell.cells_n(height));
                    tcells.extend(cell.delta(1, 0).cells_n(height));
                    self.egrid.add_tile(cell, tcid, &tcells);
                }
            }
        }
    }

    fn fill_vnoc(&mut self) {
        for (die, chip) in &self.chips {
            for (col, cd) in &chip.columns {
                if !matches!(
                    cd.kind,
                    ColumnKind::VNoc | ColumnKind::VNoc2 | ColumnKind::VNoc4
                ) {
                    continue;
                }
                if self.disabled.contains(&DisabledPart::Column(die, col)) {
                    continue;
                }
                let mut prev: Option<CellCoord> = None;
                for reg in chip.regs() {
                    if self.disabled.contains(&DisabledPart::Region(die, reg)) {
                        prev = None;
                        continue;
                    }
                    let cell = die.cell(col, chip.row_reg_bot(reg));
                    let mut tcells = vec![];
                    tcells.extend(cell.cells_n(Chip::ROWS_PER_REG));
                    tcells.extend(cell.delta(1, 0).cells_n(Chip::ROWS_PER_REG));
                    let tcid = match cd.kind {
                        ColumnKind::VNoc => tcls::VNOC,
                        ColumnKind::VNoc2 => tcls::VNOC2,
                        ColumnKind::VNoc4 => tcls::VNOC4,
                        _ => unreachable!(),
                    };
                    self.egrid.add_tile(cell, tcid, &tcells);

                    let bel_nmu = cell.bel(bslots::VNOC_NMU512);
                    let bel_nsu = cell.bel(bslots::VNOC_NSU512);
                    if cd.kind == ColumnKind::VNoc4 {
                        let bel_nps_nmu = cell.bel(bslots::VNOC_NPS6X_NMU);
                        let bel_nps_nsu = cell.bel(bslots::VNOC_NPS6X_NSU);
                        self.noc
                            .connect(bel_nmu.noc_port(0), bel_nps_nmu.noc_port(0));
                        self.noc
                            .connect(bel_nsu.noc_port(0), bel_nps_nsu.noc_port(3));
                        self.noc
                            .connect(bel_nps_nmu.noc_port(3), bel_nps_nsu.noc_port(0));
                        if let Some(prev) = prev {
                            let bel_prev_nps_nmu = prev.bel(bslots::VNOC_NPS6X_NMU);
                            let bel_prev_nps_nsu = prev.bel(bslots::VNOC_NPS6X_NSU);
                            self.noc
                                .connect(bel_nps_nmu.noc_port(4), bel_prev_nps_nmu.noc_port(2));
                            self.noc
                                .connect(bel_nps_nmu.noc_port(5), bel_prev_nps_nmu.noc_port(1));
                            self.noc
                                .connect(bel_nps_nsu.noc_port(4), bel_prev_nps_nsu.noc_port(2));
                            self.noc
                                .connect(bel_nps_nsu.noc_port(5), bel_prev_nps_nsu.noc_port(1));
                        }
                    } else {
                        let bel_nps_nmu = cell.bel(bslots::VNOC_NPS_NMU);
                        let bel_nps_nsu = cell.bel(bslots::VNOC_NPS_NSU);
                        self.noc
                            .connect(bel_nmu.noc_port(0), bel_nps_nmu.noc_port(3));
                        self.noc
                            .connect(bel_nsu.noc_port(0), bel_nps_nsu.noc_port(3));
                        self.noc
                            .connect(bel_nps_nmu.noc_port(1), bel_nps_nsu.noc_port(1));
                        if let Some(prev) = prev {
                            let bel_prev_nps_nmu = prev.bel(bslots::VNOC_NPS_NMU);
                            let bel_prev_nps_nsu = prev.bel(bslots::VNOC_NPS_NSU);
                            self.noc
                                .connect(bel_nps_nmu.noc_port(2), bel_prev_nps_nmu.noc_port(0));
                            self.noc
                                .connect(bel_nps_nsu.noc_port(0), bel_prev_nps_nsu.noc_port(2));
                        }
                    }
                    prev = Some(cell);

                    if chip.is_reg_n(reg) {
                        self.egrid.add_tile(cell.delta(1, 0), tcls::MISR, &tcells);
                        let tcid = if col == self.die[die].cols_vnoc[0]
                            && reg.to_idx() / 2 < chip.ps.bufg_ps_height()
                        {
                            tcls::RCLK_VNOC_PS
                        } else {
                            tcls::RCLK_VNOC
                        };
                        self.egrid.add_tile(cell, tcid, &tcells);
                        self.egrid.fill_conn_term(cell, ccls::CLK_DRV);
                        self.egrid.fill_conn_term(cell.delta(1, 0), ccls::CLK_DRV);
                        if matches!(chip.hsr_s, HsrSKind::Ssit) {
                            self.egrid.add_tile_e(
                                cell,
                                [
                                    tcls::CLK_V_BUF_VNOC_SSIT_MID_S,
                                    tcls::CLK_V_BUF_VNOC_SSIT_MID_N,
                                    tcls::CLK_V_BUF_VNOC_SSIT_N,
                                ][reg.to_idx() / 2],
                                2,
                            );
                        } else if matches!(chip.hsr_n, HsrNKind::Ssit) {
                            self.egrid.add_tile_e(
                                cell,
                                [
                                    tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S,
                                    tcls::CLK_V_BUF_VNOC_SSIT_MID_S,
                                    tcls::CLK_V_BUF_VNOC_SSIT_MID_N,
                                    tcls::CLK_V_BUF_VNOC_SSIT_N,
                                ][reg.to_idx() / 2],
                                2,
                            );
                        } else if matches!(chip.hsr_n, HsrNKind::Ssit2)
                            && reg.to_idx() == chip.regs - 1
                        {
                            self.egrid
                                .add_tile_e(cell, tcls::CLK_V_BUF_VNOC_SSIT2_EDGE, 2);
                            if self.chips[die].hsr_n == HsrNKind::Ssit2 && (die.to_idx() & 1) != 0 {
                                let odie = DieId::from_idx(die.to_idx() ^ 1);
                                self.egrid[cell].region_root[rslots::GCLK_PD_V] =
                                    odie.cell(cell.col, cell.row);
                                self.egrid[cell.delta(1, 0)].region_root[rslots::GCLK_PD_V] =
                                    odie.cell(cell.col, cell.row).delta(1, 0);
                            }
                        } else {
                            self.egrid.add_tile_e(cell, tcls::CLK_V_BUF_VNOC, 2);
                        }
                    } else {
                        self.egrid.add_tile(cell, tcls::SYSMON_SAT_VNOC, &tcells);
                    }
                }
            }
        }
    }

    fn fill_gt(&mut self) {
        for (die, chip) in &self.chips {
            for (col, cd) in &chip.columns {
                if !matches!(chip.columns[col].kind, ColumnKind::Gt | ColumnKind::HNicX) {
                    continue;
                }

                let mut is_no_loc = true;
                if col == chip.col_w() {
                    is_no_loc = false;
                } else {
                    for col in self.die[die].cols_vnoc.last().unwrap().range(col) {
                        if chip.columns[col].kind == ColumnKind::Bram(BramKind::SplitterPd) {
                            is_no_loc = false;
                        }
                    }
                }

                let mut was_vcu = false;
                let side = chip.col_side(col);
                let regs_gt = match side {
                    DirH::W => &chip.regs_gt_w,
                    DirH::E => &chip.regs_gt_e,
                };
                let mut prev_noc: Option<(BelCoord, BelCoord)> = None;
                for (reg, &kind) in regs_gt {
                    let cell = die.cell(col, chip.row_reg_bot(reg));
                    if self.in_int_hole(cell) {
                        continue;
                    }

                    let is_vdu = matches!(kind, GtRowKind::Vdu | GtRowKind::Vcu2 | GtRowKind::Isp2)
                        || was_vcu;

                    self.egrid
                        .add_tile_n(cell, tcls::SYSMON_SAT_GT, Chip::ROWS_PER_REG);
                    self.egrid
                        .add_tile_n(cell, tcls::DPLL_GT, Chip::ROWS_PER_REG);
                    self.egrid.fill_conn_term(
                        cell,
                        if is_vdu {
                            ccls::DPLL_GT_VDU
                        } else {
                            ccls::DPLL_GT
                        },
                    );
                    if chip.is_reg_n(reg) {
                        self.egrid
                            .add_tile_n(cell, tcls::RCLK_GT, Chip::ROWS_PER_REG);
                        if !is_no_loc {
                            self.egrid.fill_conn_term(cell, ccls::CLK_DRV);
                        }
                        if matches!(chip.hsr_s, HsrSKind::Ssit) && side == DirH::E {
                            self.egrid.add_tile_single(
                                cell,
                                [
                                    tcls::CLK_V_BUF_GT_SSIT_MID_S,
                                    tcls::CLK_V_BUF_GT_SSIT_MID_N,
                                    tcls::CLK_V_BUF_GT_SSIT_N,
                                ][reg.to_idx() / 2],
                            );
                        } else if matches!(chip.hsr_n, HsrNKind::Ssit) && side == DirH::E {
                            self.egrid.add_tile_single(
                                cell,
                                [
                                    tcls::CLK_V_BUF_GT_SSIT_HNOC_S,
                                    tcls::CLK_V_BUF_GT_SSIT_MID_S,
                                    tcls::CLK_V_BUF_GT_SSIT_MID_N,
                                    tcls::CLK_V_BUF_GT_SSIT_N,
                                ][reg.to_idx() / 2],
                            );
                        } else if matches!(chip.hsr_n, HsrNKind::Ssit2)
                            && reg.to_idx() == chip.regs - 1
                        {
                            self.egrid
                                .add_tile_single(cell, tcls::CLK_V_BUF_GT_SSIT2_EDGE);
                            if self.chips[die].hsr_n == HsrNKind::Ssit2 && (die.to_idx() & 1) != 0 {
                                let odie = DieId::from_idx(die.to_idx() ^ 1);
                                self.egrid[cell].region_root[rslots::GCLK_PD_V] =
                                    odie.cell(cell.col, cell.row);
                            }
                        } else {
                            self.egrid.add_tile_single(cell, tcls::CLK_V_BUF_GT);
                        }
                    }
                    match kind {
                        GtRowKind::None => (),
                        GtRowKind::Gty => {
                            self.egrid.add_tile_n(cell, tcls::GTY, Chip::ROWS_PER_REG);
                            if side == DirH::W {
                                self.egrid.add_tile_n(cell, tcls::XPIPE, Chip::ROWS_PER_REG);
                            }
                        }
                        GtRowKind::Gtyp => {
                            let tcid = if reg.to_idx() != 0 && regs_gt[reg - 1] == GtRowKind::Xram {
                                tcls::GTYP_XRAM
                            } else {
                                tcls::GTYP
                            };
                            self.egrid.add_tile_n(cell, tcid, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::Gtm => {
                            let tcid = if cd.kind == ColumnKind::HNicX {
                                tcls::GTM_HNIC
                            } else {
                                tcls::GTM
                            };
                            self.egrid.add_tile_n(cell, tcid, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::Gtm2 => {
                            let tcid = if chip.is_reg_n(reg) {
                                tcls::GTM2_N
                            } else {
                                tcls::GTM2_S
                            };
                            self.egrid.add_tile_n(cell, tcid, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::RfAdc => {
                            self.egrid
                                .add_tile_n(cell, tcls::RFADC, Chip::ROWS_PER_REG * 2);
                        }
                        GtRowKind::RfDac => {
                            self.egrid
                                .add_tile_n(cell, tcls::RFDAC, Chip::ROWS_PER_REG * 2);
                        }
                        GtRowKind::Xram => {
                            self.egrid.add_tile_n(cell, tcls::XRAM, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::Vdu => {
                            self.egrid.add_tile_n(cell, tcls::VDU, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::BfrB => {
                            self.egrid.add_tile_n(cell, tcls::BFR_B, Chip::ROWS_PER_REG);
                        }
                        GtRowKind::Isp2 => {
                            self.egrid.add_tile_n(cell, tcls::ISP2, Chip::ROWS_PER_REG);
                            let bel_nps_0 = cell.bel(bslots::VCU2_NPS6X[0]);
                            let bel_nps_1 = cell.bel(bslots::VCU2_NPS6X[1]);
                            let bel_nmu_0 = cell.bel(bslots::VCU2_NMU[0]);
                            let bel_nmu_1 = cell.bel(bslots::VCU2_NMU[1]);
                            let bel_nsu = cell.bel(bslots::VCU2_NSU);
                            self.noc
                                .connect(bel_nmu_0.noc_port(0), bel_nps_0.noc_port(0));
                            self.noc
                                .connect(bel_nmu_1.noc_port(0), bel_nps_1.noc_port(0));
                            self.noc.connect(bel_nsu.noc_port(0), bel_nps_0.noc_port(3));
                            if let Some((prev_nps_0, prev_nps_1)) = prev_noc {
                                self.noc
                                    .connect(bel_nps_0.noc_port(1), prev_nps_0.noc_port(5));
                                self.noc
                                    .connect(bel_nps_0.noc_port(2), prev_nps_0.noc_port(4));
                                self.noc
                                    .connect(bel_nps_1.noc_port(1), prev_nps_1.noc_port(5));
                                self.noc
                                    .connect(bel_nps_1.noc_port(2), prev_nps_1.noc_port(4));
                            }
                            prev_noc = Some((bel_nps_0, bel_nps_1));
                        }
                        GtRowKind::Vcu2 => {
                            self.egrid
                                .add_tile_n(cell, tcls::VCU2, Chip::ROWS_PER_REG * 2);
                            let bel_nps_0 = cell.bel(bslots::VCU2_NPS6X[0]);
                            let bel_nps_1 = cell.bel(bslots::VCU2_NPS6X[1]);
                            let bel_nps_2 = cell.bel(bslots::VCU2_NPS6X[2]);
                            let bel_nps_3 = cell.bel(bslots::VCU2_NPS6X[3]);
                            let bel_nmu_0 = cell.bel(bslots::VCU2_NMU[0]);
                            let bel_nmu_1 = cell.bel(bslots::VCU2_NMU[1]);
                            let bel_nmu_2 = cell.bel(bslots::VCU2_NMU[2]);
                            let bel_nmu_3 = cell.bel(bslots::VCU2_NMU[3]);
                            let bel_nsu = cell.bel(bslots::VCU2_NSU);
                            self.noc
                                .connect(bel_nmu_0.noc_port(0), bel_nps_0.noc_port(0));
                            self.noc
                                .connect(bel_nmu_1.noc_port(0), bel_nps_1.noc_port(0));
                            self.noc
                                .connect(bel_nmu_2.noc_port(0), bel_nps_2.noc_port(0));
                            self.noc
                                .connect(bel_nmu_3.noc_port(0), bel_nps_3.noc_port(0));
                            self.noc.connect(bel_nsu.noc_port(0), bel_nps_1.noc_port(3));
                            self.noc
                                .connect(bel_nps_2.noc_port(1), bel_nps_0.noc_port(5));
                            self.noc
                                .connect(bel_nps_2.noc_port(2), bel_nps_0.noc_port(4));
                            self.noc
                                .connect(bel_nps_3.noc_port(1), bel_nps_1.noc_port(5));
                            self.noc
                                .connect(bel_nps_3.noc_port(2), bel_nps_1.noc_port(4));
                            self.noc
                                .connect(bel_nps_2.noc_port(3), bel_nps_3.noc_port(3));
                            if let Some((prev_nps_0, prev_nps_1)) = prev_noc {
                                self.noc
                                    .connect(bel_nps_0.noc_port(1), prev_nps_0.noc_port(5));
                                self.noc
                                    .connect(bel_nps_0.noc_port(2), prev_nps_0.noc_port(4));
                                self.noc
                                    .connect(bel_nps_1.noc_port(1), prev_nps_1.noc_port(5));
                                self.noc
                                    .connect(bel_nps_1.noc_port(2), prev_nps_1.noc_port(4));
                            }
                            prev_noc = Some((bel_nps_2, bel_nps_3));
                        }
                    }
                    was_vcu = kind == GtRowKind::Vcu2;
                }
                if cd.kind == ColumnKind::HNicX {
                    let cell = die.cell(col, chip.row_reg_bot(RegId::from_idx(2)));
                    self.egrid
                        .add_tile_n(cell, tcls::HNICX, Chip::ROWS_PER_REG * 7);
                }
            }
        }
    }

    fn fill_ps9(&mut self, die: DieId) {
        let chip = self.chips[die];
        let cell = die.cell(self.die[die].col_cfrm, chip.row_s());
        let cell_hsr = die.cell(chip.col_w(), chip.row_hsr(DirV::S));
        let mut tcells = cell.cells_n(Chip::ROWS_PER_REG * 2);
        tcells.extend([
            cell_hsr,
            cell_hsr.with_col(ColId::from_idx(match self.chips[die].hsr_s {
                HsrSKind::Ssit => 1,
                HsrSKind::Xpio(_) => 18,
                HsrSKind::X5io(_) => 16,
            })),
        ]);
        self.egrid.add_tile(cell, tcls::PS9, &tcells);
    }

    fn fill_ps(&mut self) {
        for die in self.chips.ids() {
            let chip = self.chips[die];
            let cell = die.cell(self.die[die].col_cfrm, chip.row_s());
            let cell_hsr = die.cell(chip.col_w(), chip.row_hsr(DirV::S));
            match chip.ps {
                PsKind::Ps9 => {
                    self.fill_ps9(die);
                }
                PsKind::Ps9Xram => {
                    self.fill_ps9(die);
                    if self.chips[die].regs == 2 {
                        self.egrid.add_tile(
                            cell.delta(0, Chip::ROWS_PER_REG as i32 * 2 - 1),
                            tcls::XRAM_NC,
                            &[],
                        );
                    }
                }
                PsKind::Ps9Cpm4 => {
                    self.fill_ps9(die);
                    let cpm_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 2);
                    let mut cells = cpm_cell.cells_n(Chip::ROWS_PER_REG);
                    let cell_ext = die.cell(
                        self.chips[die].cols_hard[0].col,
                        cpm_cell.row + Chip::ROWS_PER_REG,
                    );
                    cells.extend(cell_ext.cells_n(Chip::ROWS_PER_REG));
                    cells.extend(cell_ext.delta(1, 0).cells_n(Chip::ROWS_PER_REG));
                    self.egrid.add_tile(cpm_cell, tcls::CPM4, &cells);
                    let chip = self.chips[die];
                    for reg in chip.regs() {
                        let cell = die.cell(chip.col_w(), chip.row_reg_bot(reg));
                        self.egrid[cell].region_root[rslots::CPM] = cpm_cell;
                    }
                }
                PsKind::Ps9Cpm5 => {
                    self.fill_ps9(die);
                    let cpm_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 2);
                    self.egrid
                        .add_tile_n(cpm_cell, tcls::CPM5, Chip::ROWS_PER_REG * 4);
                    for i in 0..4 {
                        let gt_cell = cpm_cell.delta(0, Chip::ROWS_PER_REG as i32 * i);
                        self.egrid.add_tile(gt_cell, tcls::GTYP_PS, &[]);
                    }
                }
                PsKind::Ps9Cpm6 => {
                    self.fill_ps9(die);
                    let cpm_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 2);
                    self.egrid
                        .add_tile_n(cpm_cell, tcls::CPM6, Chip::ROWS_PER_REG * 4);
                    for i in 0..4 {
                        let gt_cell = cpm_cell.delta(0, Chip::ROWS_PER_REG as i32 * i);
                        self.egrid.add_tile(gt_cell, tcls::GTM2_PS, &[]);
                        if matches!(i, 1 | 3) {
                            self.egrid
                                .add_tile_n(gt_cell, tcls::RCLK_GT_CPM6, Chip::ROWS_PER_REG);
                        }
                    }
                }
                PsKind::PsXCpm5N => {
                    let mut tcells = cell.cells_n(Chip::ROWS_PER_REG * 5);
                    tcells.push(cell_hsr);
                    self.egrid.add_tile(cell, tcls::PSXL, &tcells);
                    let cpm_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 5);
                    self.egrid
                        .add_tile_n(cpm_cell, tcls::CPM5N, Chip::ROWS_PER_REG * 4);
                    for i in 0..4 {
                        let gt_cell = cpm_cell.delta(0, Chip::ROWS_PER_REG as i32 * i);
                        self.egrid.add_tile(gt_cell, tcls::GTYP_PS, &[]);
                    }
                }
                PsKind::PsXcMmi => {
                    let mut tcells = cell.cells_n(Chip::ROWS_PER_REG * 4);
                    tcells.push(cell_hsr);
                    self.egrid.add_tile(cell, tcls::PSXC, &tcells);
                    if self.chips[die].regs == 4 {
                        let mmi_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 4 - 1);
                        self.egrid.add_tile(mmi_cell, tcls::MMI_NC, &[]);
                        self.egrid.add_tile(mmi_cell, tcls::GTYP_PS, &[]);
                    } else {
                        let mmi_cell = cell.delta(0, Chip::ROWS_PER_REG as i32 * 4);
                        self.egrid.add_tile_n(mmi_cell, tcls::MMI, 96);
                        self.egrid.add_tile(
                            mmi_cell.delta(0, Chip::ROWS_PER_REG as i32),
                            tcls::GTYP_PS,
                            &[],
                        );
                    }
                }
                PsKind::PsXt => {
                    let mut tcells = cell.cells_n(Chip::ROWS_PER_REG * 4);
                    tcells.push(cell_hsr);
                    self.egrid.add_tile(cell, tcls::PSXT, &tcells);
                    self.egrid.add_tile(
                        cell.delta(0, Chip::ROWS_PER_REG as i32 * 3),
                        tcls::GTYP_PS,
                        &[],
                    );
                }
            }
        }
    }

    fn fill_hsr_xpio(&mut self, die: DieId, xpio: &XpioRow, side: DirV) {
        let chip = &self.chips[die];
        let row = chip.row_hsr(side);
        let row_bli = chip.row_edge(side);
        if side == DirV::S {
            let n0 = die.cell(ColId::from_idx(0), row);
            let n1 = die.cell(ColId::from_idx(9), row);
            let n2 = die.cell(ColId::from_idx(18), row);

            for (pc, pi, si, sp) in [
                (n0, 0, 2, 3), // PCIE 0
                (n0, 1, 3, 1), // PCIE 1
                (n0, 2, 3, 0), // PMC
                (n0, 3, 2, 1), // RPU
                (n1, 0, 9, 1), // NCI 0
                (n1, 1, 8, 3), // NCI 1
                (n1, 2, 6, 3), // CCI 0
                (n1, 3, 7, 3), // CCI 1
                (n1, 4, 4, 1), // CCI 2
                (n1, 5, 5, 1), // CCI 3
            ] {
                self.noc.connect(
                    pc.bel(bslots::HNOC_PS_NMU[pi]).noc_port(0),
                    [n0, n1, n2][si / 4]
                        .bel(bslots::HNOC_NPS[si % 4])
                        .noc_port(sp),
                );
            }
            for (pc, pi, si, sp) in [
                (n0, 0, 0, 1),  // PCIE
                (n0, 1, 0, 0),  // PMC
                (n1, 0, 10, 1), // CCI 0
                (n1, 1, 11, 3), // CCI 1
                (n1, 2, 8, 1),  // NCI 0
                (n1, 3, 9, 3),  // NCI 1
            ] {
                self.noc.connect(
                    pc.bel(bslots::HNOC_PS_NSU[pi]).noc_port(0),
                    [n0, n1, n2][si / 4]
                        .bel(bslots::HNOC_NPS[si % 4])
                        .noc_port(sp),
                );
            }
            self.noc.connect(
                n0.bel(bslots::HNOC_NPS[0]).noc_port(3),
                n0.bel(bslots::HNOC_NPS[1]).noc_port(1),
            );
            self.noc.connect(
                n0.bel(bslots::HNOC_NPS[1]).noc_port(0),
                n0.bel(bslots::HNOC_NPS[2]).noc_port(0),
            );
            self.noc.connect(
                n0.bel(bslots::HNOC_NPS[1]).noc_port(3),
                n0.bel(bslots::HNOC_NPS[3]).noc_port(3),
            );
        } else {
            for &col in &self.die[die].cols_vnoc {
                let cell = die.cell(col, row_bli);
                let cell_vnoc = die.cell(col, chip.row_reg_bot(chip.regs().last().unwrap()));
                self.egrid.add_tile(cell, tcls::HNOC_NCRB, &[]);
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NCRB[0]).noc_port(0),
                    cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                );
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NCRB[0]).noc_port(1),
                    cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                );
                self.noc.connect_uni(
                    cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                    cell.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                );
                self.noc.connect_uni(
                    cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                    cell.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                );
            }
        }
        let mut prev: Option<CellCoord> = None;
        let mut clk_conn_distr = HashMap::new();
        let mut clk_conn_route = HashMap::new();
        for (reg, rd) in &xpio.regions {
            let col = ColId::from_idx(reg.to_idx() * 9);
            let cell = die.cell(col, row);
            let (tcid, col_bli_clk) = match rd.clk_bli {
                XpioClkBli::Minimal(c) => (tcls::CLK_XPIO_MIN, c),
                XpioClkBli::Full(c) => (tcls::CLK_XPIO_FULL, c),
            };
            let bli_clk = die.cell(col_bli_clk, row_bli);
            let bli_mmcm = die.cell(rd.mmcm_bli, row_bli);
            let bli_dpll = die.cell(rd.dpll_bli, row_bli);
            self.egrid.add_tile(
                cell,
                tcid,
                &[
                    cell,
                    bli_clk,
                    bli_clk.delta(1, 0),
                    bli_mmcm,
                    bli_mmcm.delta(1, 0),
                ],
            );
            self.egrid.add_tile(
                cell,
                tcls::MMCM_XPIO,
                &[cell, bli_mmcm, bli_mmcm.delta(1, 0)],
            );
            self.egrid.add_tile(
                cell,
                tcls::DPLL_XPIO,
                &[cell, bli_dpll, bli_dpll.delta(1, 0)],
            );
            let bli_xpll = die.cell(rd.xpll_bli, row_bli);
            let mut tcells = cell.cells_e(9);
            tcells.extend([bli_xpll, bli_xpll.delta(1, 0)]);
            self.egrid.add_tile(cell, tcls::XPLL, &tcells);
            for i in 0..4 {
                self.egrid[tcells[i]].region_root[rslots::XPIO_LEAF] = tcells[3];
            }
            for i in 4..9 {
                self.egrid[tcells[i]].region_root[rslots::XPIO_LEAF] = tcells[4];
            }

            for (i, col_bli) in rd.xpio_bli.into_iter().enumerate() {
                let cell_io = cell.delta(i as i32, 0);
                if let Some(col_bli) = col_bli {
                    let bli = die.cell(col_bli, row_bli);
                    self.egrid
                        .add_tile(cell_io, tcls::XPIO, &[cell_io, bli, bli.delta(1, 0)]);
                } else {
                    self.egrid.add_tile_single(cell_io, tcls::XPIO_NC);
                }
            }

            let (tcid, dci_bli) = match rd.dci_bli {
                XpioTestBli::PrimaryA(c) => (tcls::XPIO_DCI_A, c),
                XpioTestBli::PrimaryB(c) => (tcls::XPIO_DCI_B, c),
                XpioTestBli::PrimaryC(c) => (tcls::XPIO_DCI_C, c),
                XpioTestBli::Secondary(c) => (tcls::XPIO_DCI_S, c),
            };
            let dci_bli = die.cell(dci_bli, row_bli);
            self.egrid
                .add_tile(cell, tcid, &[dci_bli, dci_bli.delta(1, 0)]);

            let (tcid, scan_bli) = match rd.scan_bli {
                XpioTestBli::PrimaryA(c) => (tcls::XPIO_SCAN_A, c),
                XpioTestBli::PrimaryB(c) => (tcls::XPIO_SCAN_B, c),
                XpioTestBli::PrimaryC(c) => (tcls::XPIO_SCAN_C, c),
                XpioTestBli::Secondary(c) => (tcls::XPIO_SCAN_S, c),
            };
            let scan_bli = die.cell(scan_bli, row_bli);
            self.egrid
                .add_tile(cell, tcid, &[scan_bli, scan_bli.delta(1, 0)]);

            if let Some(cols_bli) = rd.ddrmc_bli {
                let bli = cols_bli.map(|c| die.cell(c, row_bli));
                self.egrid.add_tile(
                    cell,
                    tcls::DDRMC,
                    &[cell, bli[0], bli[0].delta(1, 0), bli[1], bli[1].delta(1, 0)],
                );

                if side == DirV::N || reg.to_idx() >= 3 {
                    let ncell = cell.delta(9, 0);
                    self.egrid.add_tile_single(
                        ncell,
                        match chip.fabric {
                            FabricKind::Orig => tcls::HSR_CLK_TEST_XPIO_REV1,
                            _ => tcls::HSR_CLK_TEST_XPIO_REV2,
                        },
                    );
                    self.egrid[ncell].region_root[rslots::PS] =
                        die.cell(chip.col_w(), chip.row_hsr(DirV::S));
                }
            }
            if let Some(col_bli) = rd.sysmon_bli {
                let bli = die.cell(col_bli, row_bli);
                self.egrid
                    .add_tile(cell, tcls::SYSMON_SAT_HSR_XPIO, &[bli, bli.delta(1, 0)]);
            }

            if reg.to_idx() == 0 && side == DirV::S {
                self.egrid.add_tile(cell, tcls::HNOC_XPIO_PS9_A, &[]);
            } else if reg.to_idx() == 1 && side == DirV::S {
                self.egrid.add_tile(cell, tcls::HNOC_XPIO_PS9_B, &[]);
                for i in 0..4 {
                    self.noc.connect(
                        cell.bel(bslots::DDRMC).noc_port(i),
                        cell.bel(bslots::HNOC_NPS[[0, 1, 3, 2][i]])
                            .noc_port([3, 3, 1, 1][i]),
                    );
                }
            } else if reg.to_idx() == 2 && side == DirV::S {
                self.egrid.add_tile(cell, tcls::HNOC_XPIO_PS9_C, &[]);
            } else {
                self.egrid.add_tile(cell, tcls::HNOC_XPIO, &[]);
                for i in 0..2 {
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[2 * i]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS_MID[i]).noc_port(1),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS_MID[i]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[2 * i + 1]).noc_port(1),
                    );
                }
                self.noc.connect(
                    cell.bel(bslots::HNOC_NPS_MID[0]).noc_port(2),
                    cell.bel(bslots::HNOC_NPS_MID[1]).noc_port(0),
                );
                if rd.ddrmc_bli.is_some() {
                    for i in 0..4 {
                        self.noc.connect(
                            cell.bel(bslots::DDRMC).noc_port(i),
                            cell.bel(bslots::HNOC_NPS[[0, 1, 3, 2][i]])
                                .noc_port([1, 3, 3, 1][i]),
                        );
                    }
                }
                for (i, vnoc) in rd.vnoc.into_iter().enumerate() {
                    let port = [
                        cell.bel(bslots::HNOC_NPS_MID[0]).noc_port(0),
                        cell.bel(bslots::HNOC_NPS_MID[1]).noc_port(2),
                    ][i];
                    if let Some(vnoc) = vnoc {
                        let cell_vnoc = die.cell(vnoc.0, row_bli);
                        match side {
                            DirV::S => {
                                let port_vnoc = [
                                    cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(2),
                                    cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(0),
                                ][vnoc.1];
                                self.noc.connect(port, port_vnoc);
                            }
                            DirV::N => {
                                self.noc.connect_uni(
                                    port,
                                    cell_vnoc.bel(bslots::HNOC_NCRB[0]).noc_port(vnoc.1),
                                );
                                self.noc.connect_uni(
                                    cell_vnoc.bel(bslots::HNOC_NCRB[1]).noc_port(vnoc.1),
                                    port,
                                );
                            }
                        }
                    }
                }
            }
            if let Some(prev) = prev {
                for i in 0..4 {
                    self.noc.connect(
                        prev.bel(bslots::HNOC_NPS[i]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[i]).noc_port(0),
                    );
                }
            }

            self.egrid[cell].region_root[rslots::HSM] =
                die.cell(chip.col_w(), chip.row_hsr(DirV::S));

            if let Some(ocol) = rd.clk_distr_col {
                match side {
                    DirV::S => {
                        let ocell = die.cell(ocol, chip.row_s());
                        clk_conn_distr.insert(ocell, cell);
                    }
                    DirV::N => {
                        let ocell =
                            die.cell(ocol, chip.row_reg_bot(RegId::from_idx(chip.regs - 1)));
                        self.egrid
                            .fill_conn_pair(ocell, cell, ccls::CLK_N_HSR, ccls::CLK_S_HSR);
                    }
                }
            }
            if let Some(ocol) = rd.clk_route_col
                && side == DirV::S
            {
                let ocell = die.cell(ocol, chip.row_s());
                clk_conn_route.insert(ocell, cell);
            }
            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::HSR_E, ccls::HSR_W);
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::XPIO_E, ccls::XPIO_W);
            }
            prev = Some(cell);
        }
        for (cell_fabric, cell_distr) in clk_conn_distr {
            let cell_route = clk_conn_route[&cell_fabric];
            let tcid = if chip.columns[cell_fabric.col].kind.is_vnoc() {
                tcls::CLK_S_BUF_VNOC
            } else {
                tcls::CLK_S_BUF_GT
            };
            self.egrid
                .add_tile(cell_fabric, tcid, &[cell_fabric, cell_route, cell_distr]);
        }
        if chip.columns[chip.col_e()].kind == ColumnKind::TermSsit {
            let col = ColId::from_idx(xpio.regions.len() * 9);
            let cell = die.cell(col, row);
            self.egrid.add_tile(cell, tcls::HNOC_SSIT2_H, &[]);
            let bel = cell.bel(bslots::HNOC_NIDB_H);
            self.noc.connect(
                prev.unwrap().bel(bslots::HNOC_NPS[0]).noc_port(2),
                bel.noc_port(0),
            );
            if die.to_idx() >= 2 {
                let odie = DieId::from_idx(die.to_idx() ^ 3);
                let ocell = odie.cell(col, row);
                let obel = ocell.bel(bslots::HNOC_NIDB_H);
                self.noc.connect(bel.noc_port(1), obel.noc_port(1));
                self.egrid[cell].region_root[rslots::HSR_SSIT2_H] = ocell;
            }

            self.egrid.add_tile_single(cell, tcls::CLK_HSR_SSIT2_H);

            self.egrid
                .fill_conn_pair(prev.unwrap(), cell, ccls::HSR_E, ccls::HSR_W);
        }
    }

    fn fill_hsr_x5io(&mut self, die: DieId, x5io: &X5ioRow) {
        let chip = &self.chips[die];
        let row = chip.row_hsr(DirV::S);
        let row_bli = chip.row_edge(DirV::S);
        let mut prev_clk: Option<CellCoord> = None;
        let mut prev_io: Option<(CellCoord, ConnectorClassId)> = None;
        let mut was_hnoc_ddrmc = false;
        let mut clk_conn_distr = HashMap::new();
        let mut clk_conn_route = HashMap::new();
        let mut prev_hnoc: [Option<NocPortCoord>; 4] = [None; 4];
        let mut cell_clk = die.cell(
            ColId::from_idx(
                x5io.regions
                    .iter()
                    .find(|(_, region)| region.clk.is_some())
                    .unwrap()
                    .0
                    .to_idx()
                    * 4,
            ),
            row,
        );
        for (reg, rd) in &x5io.regions {
            let col = ColId::from_idx(reg.to_idx() * 4);
            let cell = die.cell(col, row);
            if let Some(clk) = rd.clk {
                cell_clk = cell;
                let (tcid, col_bli_clk) = match clk.clk_bli {
                    X5ioClkBli::MinimalA(c) => (tcls::CLK_X5IO_MIN_A, c),
                    X5ioClkBli::MinimalB(c) => (tcls::CLK_X5IO_MIN_B, c),
                    X5ioClkBli::Full(c) => (tcls::CLK_X5IO_FULL, c),
                };
                let bli_clk = die.cell(col_bli_clk, row_bli);
                let bli_mmcm = die.cell(clk.mmcm_bli, row_bli);
                self.egrid.add_tile(
                    cell,
                    tcid,
                    &[
                        cell,
                        bli_clk,
                        bli_clk.delta(1, 0),
                        bli_mmcm,
                        bli_mmcm.delta(1, 0),
                    ],
                );
                self.egrid.add_tile(
                    cell,
                    tcls::MMCM_X5IO,
                    &[cell, bli_mmcm, bli_mmcm.delta(1, 0)],
                );
                if let Some(bli_dpll) = clk.dpll_bli {
                    let bli_dpll = die.cell(bli_dpll, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::DPLL_X5IO,
                        &[cell, bli_dpll, bli_dpll.delta(1, 0)],
                    );
                } else {
                    self.egrid.add_tile_single(cell, tcls::DPLL_X5IO_NC);
                }

                self.egrid[cell].region_root[rslots::HSM] =
                    die.cell(chip.col_w(), chip.row_hsr(DirV::S));

                if let Some(ocol) = clk.clk_distr_col {
                    let ocell = die.cell(ocol, chip.row_s());
                    clk_conn_distr.insert(ocell, cell);
                }
                if let Some(ocol) = clk.clk_route_col {
                    let ocell = die.cell(ocol, chip.row_s());
                    clk_conn_route.insert(ocell, cell);
                }

                if reg != x5io.regions.last_id().unwrap() && x5io.regions[reg + 1].io.is_some() {
                    self.egrid.add_tile_e(cell, tcls::X5PLL_GASKET, 8);
                } else {
                    self.egrid.add_tile_e(cell, tcls::X5PLL_GASKET_HALF, 4);
                }

                if let Some(prev_clk) = prev_clk {
                    self.egrid
                        .fill_conn_pair(prev_clk, cell, ccls::HSR_E, ccls::HSR_W);
                }
                prev_clk = Some(cell);
            }

            if let Some(io) = rd.io {
                if let Some(bank_bli) = io.bank_bli {
                    let bli = die.cell(bank_bli, row_bli);
                    self.egrid
                        .add_tile(cell, tcls::X5IO_BANK, &[bli, bli.delta(1, 0)]);
                } else {
                    self.egrid.add_tile(cell, tcls::X5IO_BANK_NC, &[]);
                }
                let cell_pair = match io.side {
                    DirH::W => cell,
                    DirH::E => cell.delta(-4, 0),
                };
                let (col_bli, tcid) = match io.scan_bli {
                    X5ioScanBli::A(col) => (col, tcls::X5IO_SCAN_A),
                    X5ioScanBli::B(col) => (col, tcls::X5IO_SCAN_B),
                    X5ioScanBli::C(col) => (col, tcls::X5IO_SCAN_C),
                };
                let bli = die.cell(col_bli, row_bli);
                self.egrid.add_tile(cell, tcid, &[bli, bli.delta(1, 0)]);
                let mut cells_pll = cell.cells_e(4);
                cells_pll.push(cell_clk);
                cells_pll.push(cell_pair);
                let (col_bli, tcid) = match io.x5pll_bli {
                    X5ioPllBli::None => (None, tcls::X5PLL_NC),
                    X5ioPllBli::PrimaryA(col) => (Some(col), tcls::X5PLL_A),
                    X5ioPllBli::PrimaryB(col) => (Some(col), tcls::X5PLL_B),
                    X5ioPllBli::Secondary(col) => (Some(col), tcls::X5PLL_S),
                };
                if let Some(col_bli) = col_bli {
                    let bli = die.cell(col_bli, row_bli);
                    cells_pll.extend([bli, bli.delta(1, 0)]);
                }
                self.egrid.add_tile(cell, tcid, &cells_pll);

                for (j, col_bli) in io.octad_bli.into_iter().enumerate() {
                    let cell_oct = cells_pll[j];
                    if let Some(col_bli) = col_bli {
                        let bli = die.cell(col_bli, row_bli);
                        self.egrid.add_tile(
                            cell_oct,
                            tcls::X5IO,
                            &[cell_oct, bli, bli.delta(1, 0)],
                        );
                    } else {
                        self.egrid.add_tile_single(cell_oct, tcls::X5IO_NC);
                    }
                }

                if let Some(ddrmc) = io.ddrmc {
                    let tcid = match ddrmc.kind {
                        X5ioDdrmcKind::Ddrmc5 => tcls::DDRMC5,
                        X5ioDdrmcKind::Ddrmc5C => tcls::DDRMC5C,
                        X5ioDdrmcKind::Ddrmc5E => tcls::DDRMC5E,
                        X5ioDdrmcKind::Ddrmc5EAlt => tcls::DDRMC5E_ALT,
                        X5ioDdrmcKind::Ddrmc5E2 => tcls::DDRMC5E2,
                        X5ioDdrmcKind::Ddrmc5E2Cut => tcls::DDRMC5E2_CUT,
                        X5ioDdrmcKind::Ddrmc5X => tcls::DDRMC5X,
                    };
                    let bli = die.cell(ddrmc.bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcid,
                        &[cell.step(!io.side, 4), cell, bli, bli.delta(1, 0)],
                    );
                    let (tcid, treg) = match ddrmc.test_clk {
                        X5ioDdrmcTestClk::A(treg) => (tcls::HSR_CLK_TEST_X5IO_A, treg),
                        X5ioDdrmcTestClk::B(treg) => (tcls::HSR_CLK_TEST_X5IO_B, treg),
                    };
                    let cell_test = cell.with_col(ColId::from_idx(treg.to_idx() * 4));
                    self.egrid.add_tile(cell, tcid, &[cell_test, cell]);

                    if ddrmc.kind.is_2hb() {
                        let cell_io0 = cell.step(!io.side, 4);
                        self.egrid.fill_conn_pair(
                            cell,
                            cell,
                            ccls::X5IO_HS_DESKEW_CLKIN,
                            ccls::X5IO_HS_DESKEW_CLKIN_REV,
                        );
                        self.egrid.fill_conn_pair(
                            cell_io0,
                            cell_io0,
                            ccls::X5IO_HS_DESKEW_CLKIN,
                            ccls::X5IO_HS_DESKEW_CLKIN_REV,
                        );
                    } else {
                        let cell_io2 = cell.step(io.side, 4);
                        self.egrid.fill_conn_pair(
                            cell_io2,
                            cell,
                            ccls::X5IO_HS_DESKEW_CLKFB,
                            ccls::X5IO_HS_DESKEW_CLKFB_REV,
                        );
                        self.egrid.fill_conn_pair(
                            cell_io2,
                            cell_io2,
                            ccls::X5IO_HS_DESKEW_CLKIN,
                            ccls::X5IO_HS_DESKEW_CLKIN_REV,
                        );
                        if io.side == DirH::W {
                            let cell_io3 = cell.step(io.side, 8);
                            self.egrid.fill_conn_pair(
                                cell_io3,
                                cell_io3,
                                ccls::X5IO_HS_DESKEW_CLKIN,
                                ccls::X5IO_HS_DESKEW_CLKIN_REV,
                            );
                        }
                    }
                }

                if io.side == DirH::W {
                    let is_paired = if let Some(nreg) = x5io.regions.get(reg + 1)
                        && let Some(nio) = nreg.io
                        && nio.side == DirH::E
                    {
                        self.egrid.fill_conn_pair(
                            cell,
                            cell.delta(4, 0),
                            ccls::X5IO_PAIR,
                            ccls::X5IO_PAIR,
                        );
                        true
                    } else {
                        false
                    };
                    if let Some((prev, ccid)) = prev_io {
                        self.egrid.fill_conn_pair(prev, cell, ccls::X5IO_E, ccid);
                    }
                    prev_io = Some((
                        cell,
                        if is_paired {
                            ccls::X5IO_W
                        } else {
                            ccls::X5IO_W_SINGLE
                        },
                    ));
                }
            }

            match rd.hnoc {
                X5ioHNoc::None => (),
                X5ioHNoc::Ps9A => {
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_PS9_A, &[]);
                    let nps: [_; 3] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let nsu: [_; 2] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NSU[i]).noc_port(0));

                    self.noc.connect(nps[0].noc_port(0), nsu[0]);
                    self.noc.connect(nps[0].noc_port(3), nmu[3]);
                    self.noc.connect(nps[1].noc_port(0), nsu[1]);
                    self.noc.connect(nps[1].noc_port(1), nmu[2]);
                    self.noc.connect(nps[2].noc_port(0), nmu[0]);
                    self.noc.connect(nps[2].noc_port(1), nmu[1]);

                    prev_hnoc = [
                        nps[0].noc_port(2),
                        nps[1].noc_port(2),
                        nps[2].noc_port(3),
                        nps[2].noc_port(2),
                    ]
                    .map(Option::Some);
                }
                X5ioHNoc::Ps9B => {
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_PS9_B, &[]);
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 2] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let xbr = cell.bel(bslots::HNOC_XBR4X2);
                    let ddrmc = cell.bel(bslots::DDRMC5);
                    let prev_nps = prev_hnoc.map(Option::unwrap);
                    for i in 0..4 {
                        self.noc.connect(nps[i].noc_port(0), prev_nps[i]);
                    }

                    self.noc.connect(nps[0].noc_port(3), nmu[0]);
                    self.noc.connect(nps[3].noc_port(1), nmu[1]);
                    self.noc.connect(nps[2].noc_port(3), xbr.noc_port(0));
                    self.noc.connect(nps[3].noc_port(3), xbr.noc_port(1));
                    self.noc.connect(nps[1].noc_port(1), xbr.noc_port(2));
                    self.noc.connect(nps[0].noc_port(1), xbr.noc_port(3));

                    self.noc.connect(ddrmc.noc_port(0), xbr.noc_port(4));
                    self.noc.connect(ddrmc.noc_port(1), xbr.noc_port(5));

                    prev_hnoc = [
                        nps[0].noc_port(2),
                        nps[1].noc_port(2),
                        nps[2].noc_port(2),
                        nps[3].noc_port(2),
                    ]
                    .map(Option::Some);
                }
                X5ioHNoc::Ps9C { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_X5IO_PS9_C,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    let nps: [_; 8] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let nsu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NSU[i]).noc_port(0));
                    let xbr = cell.bel(bslots::HNOC_XBR4X2);
                    let ddrmc = cell.bel(bslots::DDRMC5);
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    for i in 0..4 {
                        self.noc.connect(nps[i].noc_port(0), prev_nps[i]);
                    }

                    self.noc.connect(nps[1].noc_port(3), nmu[0]);
                    self.noc.connect(nps[2].noc_port(1), nmu[1]);
                    self.noc.connect(nps[3].noc_port(1), nmu[2]);
                    self.noc.connect(nps[0].noc_port(3), nmu[3]);
                    self.noc.connect(nps[2].noc_port(3), xbr.noc_port(0));
                    self.noc.connect(nps[3].noc_port(3), xbr.noc_port(1));
                    self.noc.connect(nps[0].noc_port(1), xbr.noc_port(2));
                    self.noc.connect(nps[1].noc_port(1), xbr.noc_port(3));

                    self.noc.connect(ddrmc.noc_port(0), xbr.noc_port(4));
                    self.noc.connect(ddrmc.noc_port(1), xbr.noc_port(5));

                    for i in 0..4 {
                        self.noc.connect(nps[i + 4].noc_port(0), nps[i].noc_port(2));
                    }

                    self.noc.connect(nps[4].noc_port(3), nsu[0]);
                    self.noc.connect(nps[6].noc_port(1), nsu[1]);
                    self.noc.connect(nps[5].noc_port(3), nsu[2]);
                    self.noc.connect(nps[7].noc_port(1), nsu[3]);
                    self.noc.connect(nps[4].noc_port(1), nps[6].noc_port(3));
                    self.noc.connect(nps[5].noc_port(1), nps[7].noc_port(3));

                    prev_hnoc = [
                        nps[4].noc_port(2),
                        nps[5].noc_port(2),
                        nps[6].noc_port(2),
                        nps[7].noc_port(2),
                    ]
                    .map(Option::Some);
                }
                X5ioHNoc::PsXA => {
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_PSX_A, &[]);
                    let nps = cell.bel(bslots::HNOC_NPS[0]);
                    let nmu: [_; 3] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let nsu: [_; 2] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NSU[i]).noc_port(0));

                    self.noc.connect(nps.noc_port(0), nsu[1]);
                    self.noc.connect(nps.noc_port(1), nmu[1]);
                    self.noc.connect(nps.noc_port(3), nmu[0]);

                    prev_hnoc = [Some(nps.noc_port(2)), Some(nsu[0]), Some(nmu[2]), None];
                }
                X5ioHNoc::PsXB { flip } => {
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_PSX_B, &[]);
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let xbr = cell.bel(bslots::HNOC_XBR4X2);
                    let ddrmc = cell.bel(bslots::DDRMC5);

                    let (port_prev, port_next) = if flip { (2, 0) } else { (0, 2) };
                    for i in 0..4 {
                        if let Some(prev) = prev_hnoc[i] {
                            self.noc.connect(prev, nps[i].noc_port(port_prev));
                        }
                    }

                    self.noc.connect(nps[2].noc_port(1), nmu[0]);
                    self.noc.connect(nps[1].noc_port(3), nmu[1]);
                    self.noc.connect(nps[0].noc_port(3), nmu[2]);
                    self.noc.connect(nps[3].noc_port(1), nmu[3]);
                    self.noc.connect(nps[2].noc_port(3), xbr.noc_port(0));
                    self.noc.connect(nps[3].noc_port(3), xbr.noc_port(1));
                    self.noc.connect(nps[1].noc_port(1), xbr.noc_port(2));
                    self.noc.connect(nps[0].noc_port(1), xbr.noc_port(3));

                    self.noc.connect(ddrmc.noc_port(0), xbr.noc_port(4));
                    self.noc.connect(ddrmc.noc_port(1), xbr.noc_port(5));

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(port_next)));
                }
                X5ioHNoc::PsXC { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    let tcid = match chip.ps {
                        PsKind::PsXCpm5N => tcls::HNOC_X5IO_PSXL_C,
                        PsKind::PsXcMmi => tcls::HNOC_X5IO_PSXC_C,
                        _ => unreachable!(),
                    };
                    self.egrid
                        .add_tile(cell, tcid, &[scan_bli, scan_bli.delta(1, 0)]);
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let nsu: [_; 4] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NSU[i]).noc_port(0));
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    for i in 0..4 {
                        self.noc.connect(nps[i].noc_port(0), prev_nps[i]);
                    }

                    self.noc.connect(nps[0].noc_port(3), nsu[0]);
                    self.noc.connect(nps[1].noc_port(3), nsu[1]);
                    self.noc.connect(nps[2].noc_port(1), nsu[2]);
                    self.noc.connect(nps[3].noc_port(1), nsu[3]);

                    if tcid == tcls::HNOC_X5IO_PSXL_C {
                        self.noc.connect(nps[0].noc_port(1), nmu[0]);
                        self.noc.connect(nps[1].noc_port(1), nmu[1]);
                        self.noc.connect(nps[2].noc_port(3), nmu[2]);
                        self.noc.connect(nps[3].noc_port(3), nmu[3]);
                    } else {
                        self.noc.connect(nps[0].noc_port(1), nmu[0]);
                    }

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(2)));
                }
                X5ioHNoc::PsXTA => {
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_PSXT_A, &[]);
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 7] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NMU[i]).noc_port(0));
                    let nsu = cell.bel(bslots::HNOC_PS_NSU[0]).noc_port(0);

                    self.noc.connect(nps[0].noc_port(0), nmu[4]);
                    self.noc.connect(nps[1].noc_port(0), nsu);
                    self.noc.connect(nps[2].noc_port(0), nmu[6]);
                    self.noc.connect(nps[3].noc_port(0), nmu[5]);

                    self.noc.connect(nps[0].noc_port(3), nmu[2]);
                    self.noc.connect(nps[1].noc_port(3), nmu[1]);
                    self.noc.connect(nps[2].noc_port(1), nmu[0]);
                    self.noc.connect(nps[3].noc_port(1), nmu[3]);

                    self.noc.connect(nps[0].noc_port(1), nps[2].noc_port(3));
                    self.noc.connect(nps[1].noc_port(1), nps[3].noc_port(3));

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(2)));
                }
                X5ioHNoc::PsXTB { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_X5IO_PSXT_B,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu = cell.bel(bslots::HNOC_PS_NMU[0]).noc_port(0);
                    let nsu: [_; 2] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_PS_NSU[i]).noc_port(0));
                    let xbr = cell.bel(bslots::HNOC_XBR4X2);
                    let ddrmc = cell.bel(bslots::DDRMC5);
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    for i in 0..4 {
                        self.noc.connect(prev_nps[i], nps[i].noc_port(0));
                    }

                    self.noc.connect(nps[0].noc_port(3), nsu[0]);
                    self.noc.connect(nps[1].noc_port(3), nsu[1]);
                    self.noc.connect(nps[3].noc_port(1), nmu);

                    self.noc.connect(nps[2].noc_port(3), xbr.noc_port(0));
                    self.noc.connect(nps[3].noc_port(3), xbr.noc_port(1));
                    self.noc.connect(nps[1].noc_port(1), xbr.noc_port(2));
                    self.noc.connect(nps[0].noc_port(1), xbr.noc_port(3));

                    self.noc.connect(ddrmc.noc_port(0), xbr.noc_port(4));
                    self.noc.connect(ddrmc.noc_port(1), xbr.noc_port(5));

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(2)));
                }
                X5ioHNoc::PsXTC { scan_bli, vnoc } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_X5IO_PSXT_C,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    for i in 0..4 {
                        self.noc.connect(prev_nps[i], nps[i].noc_port(0));
                    }

                    let cell_vnoc = die.cell(vnoc, row_bli);
                    let vnoc_nps_nmu = cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(2);
                    let vnoc_nps_nsu = cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(0);
                    self.noc.connect(nps[2].noc_port(1), vnoc_nps_nsu);
                    self.noc.connect(nps[3].noc_port(1), vnoc_nps_nmu);

                    self.noc.connect(nps[0].noc_port(3), nps[1].noc_port(3));
                    self.noc.connect(nps[0].noc_port(1), nps[2].noc_port(3));
                    self.noc.connect(nps[1].noc_port(1), nps[3].noc_port(3));

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(2)));
                }
                X5ioHNoc::Cross {
                    alt,
                    flip,
                    scan_bli,
                    vnoc,
                } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    let tcid = if alt {
                        tcls::HNOC_X5IO_CROSS_ALT
                    } else {
                        tcls::HNOC_X5IO_CROSS
                    };
                    self.egrid
                        .add_tile(cell, tcid, &[scan_bli, scan_bli.delta(1, 0)]);

                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    let (port_prev, port_next) = if flip { (2, 0) } else { (0, 2) };
                    for i in 0..4 {
                        self.noc.connect(nps[i].noc_port(port_prev), prev_nps[i]);
                    }

                    self.noc.connect(nps[0].noc_port(1), nps[2].noc_port(3));
                    self.noc.connect(nps[1].noc_port(1), nps[3].noc_port(3));

                    if let Some(col_vnoc) = vnoc {
                        let cell_vnoc = die.cell(col_vnoc, row_bli);
                        if chip.columns[col_vnoc].kind == ColumnKind::VNoc4 {
                            let vnoc_nps_nmu = cell_vnoc.bel(bslots::VNOC_NPS6X_NMU);
                            let vnoc_nps_nsu = cell_vnoc.bel(bslots::VNOC_NPS6X_NSU);
                            if flip {
                                self.noc
                                    .connect(nps[0].noc_port(3), vnoc_nps_nsu.noc_port(4));
                                self.noc
                                    .connect(nps[1].noc_port(3), vnoc_nps_nmu.noc_port(5));
                                self.noc
                                    .connect(nps[2].noc_port(1), vnoc_nps_nsu.noc_port(5));
                                self.noc
                                    .connect(nps[3].noc_port(1), vnoc_nps_nmu.noc_port(4));
                            } else {
                                self.noc
                                    .connect(nps[0].noc_port(3), vnoc_nps_nmu.noc_port(4));
                                self.noc
                                    .connect(nps[1].noc_port(3), vnoc_nps_nsu.noc_port(5));
                                self.noc
                                    .connect(nps[2].noc_port(1), vnoc_nps_nmu.noc_port(5));
                                self.noc
                                    .connect(nps[3].noc_port(1), vnoc_nps_nsu.noc_port(4));
                            }
                        } else {
                            let vnoc_nps_nmu = cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(2);
                            let vnoc_nps_nsu = cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(0);
                            if flip {
                                self.noc.connect(nps[2].noc_port(1), vnoc_nps_nsu);
                                self.noc.connect(nps[3].noc_port(1), vnoc_nps_nmu);
                            } else {
                                self.noc.connect(nps[2].noc_port(1), vnoc_nps_nmu);
                                self.noc.connect(nps[3].noc_port(1), vnoc_nps_nsu);
                            }

                            self.noc.connect(nps[0].noc_port(3), nps[1].noc_port(3));
                        }
                    } else {
                        self.noc.connect(nps[0].noc_port(3), nps[1].noc_port(3));
                    }

                    prev_hnoc = nps.map(|nps| Some(nps.noc_port(port_next)));
                    was_hnoc_ddrmc = false;
                }
                X5ioHNoc::Ddrmc {
                    flip,
                    swap,
                    scan_bli,
                    nmu_bli,
                    vnoc,
                } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    let mut tcells = vec![];
                    for bli in nmu_bli {
                        if let Some(bli) = bli {
                            let bli = bli.map(|col| die.cell(col, row_bli));
                            tcells.extend([
                                bli[0],
                                bli[0].delta(1, 0),
                                bli[1],
                                bli[1].delta(1, 0),
                                bli[2],
                                bli[2].delta(1, 0),
                            ]);
                        } else {
                            // dummy cells for BLI stubbing
                            tcells.extend([cell; 6]);
                        }
                    }
                    tcells.extend([scan_bli, scan_bli.delta(1, 0)]);
                    self.egrid.add_tile(cell, tcls::HNOC_X5IO_DDRMC, &tcells);

                    let nps: [_; 4] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let nmu: [_; 2] =
                        core::array::from_fn(|i| cell.bel(bslots::HNOC_NMU512[i]).noc_port(0));
                    let xbr = cell.bel(bslots::HNOC_XBR4X2);
                    let ddrmc = cell.bel(bslots::DDRMC5);
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    let (port_prev, port_next) = if flip { (2, 0) } else { (0, 2) };
                    for i in 0..4 {
                        self.noc.connect(nps[i].noc_port(port_prev), prev_nps[i]);
                    }

                    for (vnoc, port) in vnoc
                        .into_iter()
                        .zip([nps[1].noc_port(3), nps[3].noc_port(1)])
                    {
                        if let Some(vnoc) = vnoc {
                            let cell_vnoc = die.cell(vnoc.0, row_bli);
                            self.noc.connect(
                                port,
                                [
                                    cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(2),
                                    cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(0),
                                ][vnoc.1],
                            );
                        }
                    }

                    self.noc.connect(nps[2].noc_port(3), xbr.noc_port(0));
                    self.noc.connect(nps[3].noc_port(3), xbr.noc_port(1));
                    self.noc.connect(nps[1].noc_port(1), xbr.noc_port(2));
                    self.noc.connect(nps[0].noc_port(1), xbr.noc_port(3));

                    self.noc.connect(nps[0].noc_port(3), nmu[0]);
                    self.noc.connect(nps[2].noc_port(1), nmu[1]);

                    self.noc.connect(ddrmc.noc_port(0), xbr.noc_port(4));
                    self.noc.connect(ddrmc.noc_port(1), xbr.noc_port(5));

                    if swap {
                        prev_hnoc = [1, 0, 3, 2].map(|i| Some(nps[i].noc_port(port_next)));
                    } else {
                        prev_hnoc = nps.map(|nps| Some(nps.noc_port(port_next)));
                    }
                    was_hnoc_ddrmc = true;
                }
                X5ioHNoc::Cap { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_X5IO_CAP,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );

                    let nps: [_; 2] = core::array::from_fn(|i| cell.bel(bslots::HNOC_NPS[i]));
                    let prev_nps = prev_hnoc.map(Option::unwrap);

                    self.noc.connect(nps[0].noc_port(0), prev_nps[0]);
                    self.noc.connect(nps[0].noc_port(1), prev_nps[1]);
                    self.noc.connect(nps[1].noc_port(0), prev_nps[2]);
                    self.noc.connect(nps[1].noc_port(1), prev_nps[3]);

                    prev_hnoc = [
                        Some(nps[0].noc_port(2)),
                        Some(nps[0].noc_port(3)),
                        Some(nps[1].noc_port(2)),
                        Some(nps[1].noc_port(3)),
                    ];
                    was_hnoc_ddrmc = false;
                }
            }

            if let Some(col_bli) = rd.sysmon_bli {
                let bli = die.cell(col_bli, row_bli);
                self.egrid
                    .add_tile(cell, tcls::SYSMON_SAT_HSR_X5IO, &[bli, bli.delta(1, 0)]);
            }
        }

        if chip.columns[chip.col_e()].kind == ColumnKind::Gt
            && chip.regs_gt_e.first().unwrap().has_noc()
        {
            let cell = die.cell(chip.col_e(), chip.row_s());
            let nps: [_; 2] = core::array::from_fn(|i| cell.bel(bslots::VCU2_NPS6X[i]));
            let prev_nps = prev_hnoc.map(Option::unwrap);

            self.noc.connect(prev_nps[0], nps[1].noc_port(2));
            self.noc.connect(prev_nps[1], nps[0].noc_port(2));
            self.noc.connect(prev_nps[2], nps[1].noc_port(1));
            self.noc.connect(prev_nps[3], nps[0].noc_port(1));
        } else if was_hnoc_ddrmc {
            let prev_nps = prev_hnoc.map(Option::unwrap);
            self.noc.connect(prev_nps[0], prev_nps[2]);
            self.noc.connect(prev_nps[1], prev_nps[3]);
        }

        for (cell_fabric, cell_distr) in clk_conn_distr {
            let cell_route = clk_conn_route[&cell_fabric];
            let tcid = if chip.columns[cell_fabric.col].kind.is_vnoc() {
                tcls::CLK_S_BUF_VNOC
            } else {
                tcls::CLK_S_BUF_GT
            };
            self.egrid
                .add_tile(cell_fabric, tcid, &[cell_fabric, cell_route, cell_distr]);
        }
    }

    fn fill_ai(&mut self, die: DieId, ai: &AiArray, regs: usize) {
        let chip = &self.chips[die];
        let row_bli = chip.row_edge(DirV::N);
        let row_hsr = chip.row_hsr(DirV::N);
        let row_intf = chip.rows_ai().first().unwrap();

        for (col, cd) in &ai.columns {
            let cell = die.cell(col, row_intf);
            let is_pll = col == ai.columns.first_id().unwrap();
            if is_pll {
                let bli = die.cell(ai.pll_bli, row_bli);
                self.egrid.add_tile(
                    cell,
                    match ai.kind {
                        AiKind::Ai => tcls::AIE_PLL,
                        AiKind::AiMl => tcls::AIE_ML_PLL,
                        AiKind::AiMl2 => tcls::AIE2PS_PLL,
                    },
                    &[
                        die.cell(col + if ai.kind == AiKind::AiMl2 { 2 } else { 1 }, row_hsr),
                        bli,
                        bli.delta(1, 0),
                    ],
                );
            }
            if ai.kind == AiKind::AiMl2 {
                let mut tcells = vec![cell; 4];
                for (i, col_bli) in cd.axis_bli.into_iter().enumerate() {
                    if let Some(col_bli) = col_bli {
                        let bli = die.cell(col_bli, row_bli);
                        tcells[2 * i] = bli;
                        tcells[2 * i + 1] = bli.delta(1, 0);
                    }
                }
                self.egrid.add_tile(cell, tcls::AIE2PS_SHIM_NOC_PL, &tcells);
                if let Some(cols_bli) = cd.test_bli {
                    let bli = cols_bli.map(|col| die.cell(col, row_bli));
                    self.egrid.add_tile(
                        cell,
                        tcls::AIE2PS_INTF_TEST,
                        &[bli[0], bli[0].delta(1, 0), bli[1], bli[1].delta(1, 0)],
                    );
                } else {
                    self.egrid.add_tile(cell, tcls::AIE2PS_INTF_TEST_NC, &[]);
                }
            } else if is_pll {
                self.egrid.add_tile(
                    cell,
                    match ai.kind {
                        AiKind::Ai => tcls::AIE_SHIM,
                        AiKind::AiMl => tcls::AIE_ML_SHIM,
                        _ => unreachable!(),
                    },
                    &[],
                );
            } else {
                let (intf_pl, intf_noc, test, test_nc) = match ai.kind {
                    AiKind::Ai => (
                        tcls::AIE_SHIM_PL,
                        tcls::AIE_SHIM_NOC_PL,
                        tcls::AIE_INTF_TEST,
                        tcls::AIE_INTF_TEST_NC,
                    ),
                    AiKind::AiMl => (
                        tcls::AIE_ML_SHIM_PL,
                        tcls::AIE_ML_SHIM_NOC_PL,
                        tcls::AIE_ML_INTF_TEST,
                        tcls::AIE_ML_INTF_TEST_NC,
                    ),
                    AiKind::AiMl2 => unreachable!(),
                };
                let tcid = if col.to_idx() < regs * 4 + 1 {
                    if let Some(cols_bli) = cd.test_bli {
                        let bli = cols_bli.map(|col| die.cell(col, row_bli));
                        self.egrid.add_tile(
                            cell,
                            test,
                            &[bli[0], bli[0].delta(1, 0), bli[1], bli[1].delta(1, 0)],
                        );
                    }
                    [intf_pl, intf_noc, intf_noc, intf_pl][(col.to_idx() - 1) % 4]
                } else {
                    self.egrid.add_tile(cell, test_nc, &[]);
                    intf_pl
                };
                // dummy sinks
                let mut tcells = vec![cell; 4];
                for (i, col_bli) in cd.axis_bli.into_iter().enumerate() {
                    if let Some(col_bli) = col_bli {
                        let bli = die.cell(col_bli, row_bli);
                        tcells[2 * i] = bli;
                        tcells[2 * i + 1] = bli.delta(1, 0);
                    }
                }
                if tcid == intf_noc {
                    if let Some(reg) = cd.axi_clk_test {
                        tcells.push(die.cell(ColId::from_idx(reg.to_idx() * 4 + 1), row_hsr));
                    } else {
                        // dummy sink
                        tcells.push(cell);
                    }
                }
                self.egrid.add_tile(cell, tcid, &tcells);
            }
        }
        for dy in 0..ai.rows_mem {
            let row = row_intf + 1 + dy;
            for col in ai.columns.ids() {
                let cell = die.cell(col, row);
                let tcid = match ai.kind {
                    AiKind::Ai => unreachable!(),
                    AiKind::AiMl => tcls::AIE_ML_MEM,
                    AiKind::AiMl2 => tcls::AIE2PS_MEM,
                };
                self.egrid.add_tile(cell, tcid, &[]);
            }
        }
        for dy in 0..ai.rows {
            let row = row_intf + 1 + ai.rows_mem + dy;
            for col in ai.columns.ids() {
                let cell = die.cell(col, row);
                let tcid = match ai.kind {
                    AiKind::Ai => tcls::AIE,
                    AiKind::AiMl => tcls::AIE_ML,
                    AiKind::AiMl2 => tcls::AIE2PS,
                };
                self.egrid.add_tile(cell, tcid, &[]);
            }
        }
    }

    fn fill_hsr_me(&mut self, die: DieId, me: &MeRow) {
        let chip = &self.chips[die];
        let row = chip.row_hsr(DirV::N);
        let row_bli = chip.row_edge(DirV::N);
        let mut prev: Option<CellCoord> = None;
        let mut was_ncrb = false;
        for (reg, rd) in &me.regions {
            let col = ColId::from_idx(reg.to_idx() * 4 + 1);
            let cell = die.cell(col, row);

            self.egrid.add_tile_single(cell, tcls::CLK_ME);

            let tcid = if matches!(chip.hsr_s, HsrSKind::Ssit) {
                tcls::HSR_CLK_TEST_ME_SSIT
            } else {
                match chip.fabric {
                    FabricKind::Orig => tcls::HSR_CLK_TEST_ME_REV1,
                    FabricKind::Rev2 | FabricKind::Ssit | FabricKind::Ssit2 => {
                        tcls::HSR_CLK_TEST_ME_REV2
                    }
                    FabricKind::Ulvt => tcls::HSR_CLK_TEST_ME_ULVT,
                }
            };
            self.egrid.add_tile_single(cell, tcid);
            self.egrid[cell].region_root[rslots::PS] =
                die.cell(chip.col_w(), chip.row_hsr(DirV::S));

            if rd.connect_hsm {
                self.egrid[cell].region_root[rslots::HSM] =
                    die.cell(chip.col_w(), chip.row_hsr(DirV::S));
            }
            if rd.pll_test {
                let cell_pll = die.cell(ColId::from_idx(1), row);
                self.egrid
                    .add_tile(cell, tcls::HSR_CLK_AIE_PLL, &[cell, cell_pll]);
            }

            if let Some(cols_bli) = rd.sysmon_bli {
                let bli0 = die.cell(cols_bli[0], row_bli);
                let bli1 = die.cell(cols_bli[1], row_bli);
                self.egrid.add_tile(
                    cell,
                    tcls::SYSMON_SAT_HSR_ME,
                    &[bli0, bli0.delta(1, 0), bli1, bli1.delta(1, 0)],
                );
            } else {
                self.egrid.add_tile(cell, tcls::SYSMON_SAT_HSR_NC, &[]);
            }

            let mut is_ncrb = false;
            match rd.vnoc {
                MeVNocConn::None => {
                    self.egrid.add_tile(cell, tcls::HNOC_ME, &[]);
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(1),
                        cell.bel(bslots::HNOC_ME_NMU[0]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(3),
                        cell.bel(bslots::HNOC_ME_NMU[1]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(1),
                        cell.bel(bslots::HNOC_ME_NSU[0]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                        cell.bel(bslots::HNOC_ME_NSU[1]).noc_port(0),
                    );
                }
                MeVNocConn::Port1(conns) => {
                    let cell_vnoc =
                        die.cell(conns[0].0, chip.row_reg_bot(chip.regs().last().unwrap()));
                    self.egrid.add_tile(cell, tcls::HNOC_ME_VNOC, &[]);
                    for (i, conn) in conns.into_iter().enumerate() {
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(1),
                            cell_vnoc
                                .bel([bslots::VNOC_NPS_NMU, bslots::VNOC_NPS_NSU][conn.1])
                                .noc_port([0, 2][conn.1]),
                        );
                    }
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                    );
                    is_ncrb = cell_vnoc.col != *self.die[die].cols_vnoc.last().unwrap();
                }
                MeVNocConn::Port3(conns) => {
                    let cell_vnoc =
                        die.cell(conns[0].0, chip.row_reg_bot(chip.regs().last().unwrap()));
                    self.egrid.add_tile(cell, tcls::HNOC_ME_VNOC, &[]);
                    for (i, conn) in conns.into_iter().enumerate() {
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(3),
                            cell_vnoc
                                .bel([bslots::VNOC_NPS_NMU, bslots::VNOC_NPS_NSU][conn.1])
                                .noc_port([0, 2][conn.1]),
                        );
                    }
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(1),
                    );
                    is_ncrb = cell_vnoc.col != *self.die[die].cols_vnoc.last().unwrap();
                }
            }
            if let Some(prev) = prev {
                if was_ncrb {
                    self.noc.connect_uni(
                        prev.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(0),
                    );
                    self.noc.connect_uni(
                        prev.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(0),
                    );
                    self.noc.connect_uni(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(0),
                        prev.bel(bslots::HNOC_NCRB[0]).noc_port(0),
                    );
                    self.noc.connect_uni(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(0),
                        prev.bel(bslots::HNOC_NCRB[0]).noc_port(1),
                    );
                } else {
                    self.noc.connect(
                        prev.bel(bslots::HNOC_NPS[0]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(0),
                    );
                    self.noc.connect(
                        prev.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(0),
                    );
                }
            } else if me.ai.is_some() {
                self.noc.connect(
                    cell.bel(bslots::HNOC_NPS[0]).noc_port(0),
                    cell.bel(bslots::HNOC_NPS[1]).noc_port(0),
                );
            }
            if is_ncrb {
                self.egrid.add_tile(cell, tcls::HNOC_NCRB, &[]);
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NCRB[0]).noc_port(0),
                    cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                );
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NCRB[0]).noc_port(1),
                    cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                );
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                    cell.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                );
                self.noc.connect_uni(
                    cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                    cell.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                );
            }
            if reg == me.regions.last_id().unwrap()
                && me.ai.is_some()
                && rd.vnoc == MeVNocConn::None
            {
                self.noc.connect(
                    cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                    cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                );
            }
            was_ncrb = is_ncrb;

            if let Some(ocol) = rd.clk_col {
                let ocell = die.cell(ocol, chip.row_reg_bot(RegId::from_idx(chip.regs - 1)));
                self.egrid
                    .fill_conn_pair(ocell, cell, ccls::CLK_N_HSR, ccls::CLK_S_HSR);
            }

            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::HSR_E, ccls::HSR_W);
            }
            prev = Some(cell);
        }
        if let Some(ref ai) = me.ai {
            self.fill_ai(die, ai, me.regions.len());
        }
    }

    fn fill_hsr_me2(&mut self, die: DieId, me: &Me2Row) {
        let chip = &self.chips[die];
        let row = chip.row_hsr(DirV::N);
        let row_bli = chip.row_edge(DirV::N);

        let cell = die.cell(ColId::from_idx(0), row);
        let bli = die.cell(me.scan_bli_w, row_bli);
        self.egrid
            .add_tile(cell, tcls::HNOC_ME2_CAP_W, &[bli, bli.delta(1, 0)]);
        let mut prev_hnoc = [0, 1].map(|i| cell.bel(bslots::HNOC_ME_NMU[i]).noc_port(0));

        let mut prev: Option<CellCoord> = None;
        for (reg, rd) in &me.regions {
            let col = ColId::from_idx(reg.to_idx() * 4 + 2);
            let cell = die.cell(col, row);

            self.egrid.add_tile_single(cell, tcls::CLK_ME);
            if reg.to_idx() == 0 {
                let cell_pll = die.cell(ColId::from_idx(2), row);
                self.egrid
                    .add_tile(cell, tcls::HSR_CLK_AIE_PLL, &[cell, cell_pll]);
            }

            let tcid = if matches!(chip.hsr_s, HsrSKind::Ssit) {
                tcls::HSR_CLK_TEST_ME_SSIT
            } else {
                match chip.fabric {
                    FabricKind::Orig => tcls::HSR_CLK_TEST_ME_REV1,
                    FabricKind::Rev2 | FabricKind::Ssit | FabricKind::Ssit2 => {
                        tcls::HSR_CLK_TEST_ME_REV2
                    }
                    FabricKind::Ulvt => tcls::HSR_CLK_TEST_ME_ULVT,
                }
            };
            self.egrid.add_tile_single(cell, tcid);
            self.egrid[cell].region_root[rslots::PS] =
                die.cell(chip.col_w(), chip.row_hsr(DirV::S));

            if rd.connect_hsm {
                self.egrid[cell].region_root[rslots::HSM] =
                    die.cell(chip.col_w(), chip.row_hsr(DirV::S));
            }

            let bli0 = die.cell(rd.sysmon_bli[0], row_bli);
            let bli1 = die.cell(rd.sysmon_bli[1], row_bli);
            self.egrid.add_tile(
                cell,
                tcls::SYSMON_SAT_HSR_ME,
                &[bli0, bli0.delta(1, 0), bli1, bli1.delta(1, 0)],
            );

            if let Some(ocol) = rd.clk_col {
                let ocell = die.cell(ocol, chip.row_reg_bot(RegId::from_idx(chip.regs - 1)));
                self.egrid
                    .fill_conn_pair(ocell, cell, ccls::CLK_N_HSR, ccls::CLK_S_HSR);
            }

            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::HSR_E, ccls::HSR_W);
            }
            prev = Some(cell);

            let nps = [0, 1, 2].map(|i| cell.bel(bslots::HNOC_NPS6X[i]));
            let nmu = [0, 1, 2, 3].map(|i| cell.bel(bslots::HNOC_ME_NMU[i]).noc_port(0));
            let nsu = cell.bel(bslots::HNOC_ME_NSU[0]).noc_port(0);
            if matches!(rd.hnoc, Me2HNoc::VNocNcrb(_)) {
                let ncrb = [0, 1].map(|i| cell.bel(bslots::HNOC_ME2_NCRB[i]));
                self.noc.connect_uni(ncrb[0].noc_port(0), prev_hnoc[0]);
                self.noc.connect_uni(ncrb[0].noc_port(1), prev_hnoc[1]);
                self.noc.connect_uni(prev_hnoc[0], ncrb[1].noc_port(0));
                self.noc.connect_uni(prev_hnoc[1], ncrb[1].noc_port(1));
                self.noc
                    .connect_uni(ncrb[1].noc_port(0), nps[0].noc_port(0));
                self.noc
                    .connect_uni(ncrb[1].noc_port(1), nps[1].noc_port(0));
                self.noc
                    .connect_uni(nps[0].noc_port(0), ncrb[0].noc_port(0));
                self.noc
                    .connect_uni(nps[1].noc_port(0), ncrb[0].noc_port(1));
            } else {
                self.noc.connect(prev_hnoc[0], nps[0].noc_port(0));
                self.noc.connect(prev_hnoc[1], nps[1].noc_port(0));
            }
            self.noc.connect(nps[0].noc_port(1), nsu);
            self.noc.connect(nps[0].noc_port(5), nmu[0]);
            self.noc.connect(nps[1].noc_port(5), nmu[1]);
            self.noc.connect(nps[0].noc_port(4), nmu[2]);
            self.noc.connect(nps[1].noc_port(4), nmu[3]);
            prev_hnoc = [nps[0].noc_port(3), nps[1].noc_port(3)];
            let bli = die.cell(rd.scan_bli, row_bli);
            let tcells = [bli, bli.delta(1, 0)];
            match rd.hnoc {
                Me2HNoc::Plain => {
                    self.egrid.add_tile(cell, tcls::HNOC_ME2, &tcells);
                }
                Me2HNoc::Cross => {
                    self.noc.connect(nps[0].noc_port(2), nps[1].noc_port(2));
                    self.egrid.add_tile(cell, tcls::HNOC_ME2, &tcells);
                }
                Me2HNoc::VNoc(col) => {
                    let cell_vnoc = die.cell(col, chip.row_reg_bot(chip.regs().last().unwrap()));
                    self.noc.connect(
                        nps[0].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                    );
                    self.noc.connect(
                        nps[1].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                    );
                    self.egrid.add_tile(cell, tcls::HNOC_ME2, &tcells);
                }
                Me2HNoc::VNocNcrb(col) => {
                    let cell_vnoc = die.cell(col, chip.row_reg_bot(chip.regs().last().unwrap()));
                    self.noc.connect(
                        nps[0].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                    );
                    self.noc.connect(
                        nps[1].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                    );
                    self.egrid.add_tile(cell, tcls::HNOC_ME2_NCRB, &tcells);
                }
                Me2HNoc::VNoc4(col) => {
                    let cell_vnoc = die.cell(col, chip.row_reg_bot(chip.regs().last().unwrap()));
                    self.noc.connect(
                        nps[0].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS6X_NMU).noc_port(1),
                    );
                    self.noc.connect(
                        nps[1].noc_port(2),
                        cell_vnoc.bel(bslots::VNOC_NPS6X_NMU).noc_port(2),
                    );
                    self.noc.connect(
                        nps[2].noc_port(0),
                        cell_vnoc.bel(bslots::VNOC_NPS6X_NSU).noc_port(1),
                    );
                    self.noc.connect(
                        nps[2].noc_port(3),
                        cell_vnoc.bel(bslots::VNOC_NPS6X_NSU).noc_port(2),
                    );
                    self.noc.connect(nps[0].noc_port(3), nps[2].noc_port(1));
                    self.noc.connect(nps[1].noc_port(3), nps[2].noc_port(2));
                    prev_hnoc = [nps[2].noc_port(5), nps[2].noc_port(4)];
                    self.egrid.add_tile(cell, tcls::HNOC_ME2_VNOC4, &tcells);
                }
            }
        }

        let col = ColId::from_idx(me.regions.len() * 4 + 2);
        let cell = die.cell(col, row);
        let bli = die.cell(me.scan_bli_e, row_bli);
        self.egrid
            .add_tile(cell, tcls::HNOC_ME2_CAP_E, &[bli, bli.delta(1, 0)]);
        let nmu = [0, 1, 2].map(|i| cell.bel(bslots::HNOC_ME_NMU[i]).noc_port(0));
        let nps = cell.bel(bslots::HNOC_NPS6X[0]);
        let nsu = cell.bel(bslots::HNOC_ME_NSU[0]).noc_port(0);
        self.noc.connect(nps.noc_port(0), nmu[2]);
        self.noc.connect(nps.noc_port(1), prev_hnoc[0]);
        self.noc.connect(nps.noc_port(2), prev_hnoc[1]);
        self.noc.connect(nps.noc_port(3), nmu[0]);
        self.noc.connect(nps.noc_port(4), nmu[1]);
        self.noc.connect(nps.noc_port(5), nsu);

        self.fill_ai(die, &me.ai, me.regions.len());
    }

    fn fill_hsr_hbm(&mut self, die: DieId, hbm: &HbmRow) {
        let chip = &self.chips[die];
        let row = chip.row_hsr(DirV::N);
        let row_bli = chip.row_edge(DirV::N);
        for &col in &self.die[die].cols_vnoc {
            let cell = die.cell(col, row_bli);
            let cell_vnoc = die.cell(col, chip.row_reg_bot(chip.regs().last().unwrap()));
            self.egrid.add_tile(cell, tcls::HNOC_NCRB, &[]);
            self.noc.connect_uni(
                cell.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
            );
            self.noc.connect_uni(
                cell.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
            );
            self.noc.connect_uni(
                cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                cell.bel(bslots::HNOC_NCRB[0]).noc_port(0),
            );
            self.noc.connect_uni(
                cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                cell.bel(bslots::HNOC_NCRB[0]).noc_port(1),
            );
        }
        let mut prev: Option<CellCoord> = None;
        for (i, (tcid_conn, c0, c1)) in [
            (tcls::HSR_CLK_HBM_EVEN, 0, 2),
            (tcls::HSR_CLK_HBM_MS_EVEN, 4, 4),
            (tcls::HSR_CLK_HBM_ODD, 6, 8),
            (tcls::HSR_CLK_HBM_ODD, 10, 12),
            (tcls::HSR_CLK_HBM_ODD, 14, 16),
            (tcls::HSR_CLK_HBM_MS_ODD, 20, 18),
            (tcls::HSR_CLK_HBM_EVEN, 20, 22),
            (tcls::HSR_CLK_HBM_EVEN, 24, 26),
            (tcls::HSR_CLK_HBM_EVEN, 28, 30),
        ]
        .into_iter()
        .enumerate()
        {
            let col = ColId::from_idx(i * 4);
            let cell = die.cell(col, row);

            self.egrid.add_tile_single(cell, tcls::CLK_HBM);
            self.egrid.add_tile_single(cell, tcls::HSR_CLK_TEST_ME_SSIT);
            self.egrid.add_tile(
                cell,
                tcid_conn,
                &[
                    cell,
                    cell.with_col(ColId::from_idx(c0)),
                    cell.with_col(ColId::from_idx(c1)),
                ],
            );

            self.egrid[cell].region_root[rslots::PS] =
                die.cell(chip.col_w(), chip.row_hsr(DirV::S));
            self.egrid[cell].region_root[rslots::HSM] =
                die.cell(chip.col_w(), chip.row_hsr(DirV::S));

            if !i.is_multiple_of(2) {
                let ocell = die.cell(
                    self.die[die].cols_vnoc[i / 2],
                    chip.row_reg_bot(RegId::from_idx(chip.regs - 1)),
                );
                self.egrid
                    .fill_conn_pair(ocell, cell, ccls::CLK_N_HSR, ccls::CLK_S_HSR);
            }

            if let Some(prev) = prev {
                self.egrid
                    .fill_conn_pair(prev, cell, ccls::HSR_E, ccls::HSR_W);
            }
            prev = Some(cell);
        }

        for (col, &bli) in &hbm.intf_bli {
            let bli = bli.map(|col_bli| die.cell(col_bli, row_bli));
            self.egrid.add_tile(
                die.cell(col, row),
                tcls::HBM_INTF,
                &[
                    bli[0],
                    bli[0].delta(1, 0),
                    bli[1],
                    bli[1].delta(1, 0),
                    bli[2],
                    bli[2].delta(1, 0),
                ],
            );
        }

        for i in 0..16 {
            let col = ColId::from_idx(i * 2);
            let cell = die.cell(col, row);
            self.egrid.add_tile_single(cell, tcls::HBM_CHNL);
        }

        let mut prev: Option<[NocPortCoord; 4]> = None;
        for i in 0..4 {
            for (dx, tcid) in [
                (0, tcls::HNOC_HBM),
                (2, tcls::HNOC_HBM_VNOC),
                (4, tcls::HNOC_HBM),
            ] {
                let col = ColId::from_idx(i * 8 + dx);
                let cell = die.cell(col, row);
                self.egrid.add_tile(cell, tcid, &[]);

                if tcid == tcls::HNOC_HBM {
                    for j in 0..4 {
                        let nps = cell.bel(bslots::HNOC_NPS6[j]);
                        if let Some(prev) = prev {
                            self.noc.connect(prev[j], nps.noc_port(0));
                        }
                        let cell_nmu = cell.delta(j as i32, 0);
                        self.noc.connect(
                            nps.noc_port(1),
                            cell_nmu.bel(bslots::NOC_NMU_HBM2E[0]).noc_port(0),
                        );
                        self.noc.connect(
                            nps.noc_port(2),
                            cell_nmu.bel(bslots::NOC_NMU_HBM2E[1]).noc_port(0),
                        );
                        self.noc
                            .connect(nps.noc_port(5), cell.bel(bslots::HNOC_NPS4[0]).noc_port(j));
                        self.noc
                            .connect(nps.noc_port(4), cell.bel(bslots::HNOC_NPS4[1]).noc_port(j));
                    }
                    for j in 0..2 {
                        let mc = cell.delta(j as i32 * 2, 0).bel(bslots::HBM_CHNL);
                        for k in 0..4 {
                            let nps = cell.bel(bslots::HNOC_NPS4[k % 2]);
                            self.noc
                                .connect(mc.noc_port(k), nps.noc_port(4 + j * 2 + k / 2));
                        }
                    }
                    prev = Some(core::array::from_fn(|j| {
                        cell.bel(bslots::HNOC_NPS6[j]).noc_port(3)
                    }));
                } else {
                    if let Some(prev) = prev {
                        self.noc
                            .connect(prev[0], cell.bel(bslots::HNOC_NPS[0]).noc_port(0));
                        self.noc
                            .connect(prev[1], cell.bel(bslots::HNOC_NPS[1]).noc_port(1));
                        self.noc
                            .connect(prev[2], cell.bel(bslots::HNOC_NPS[3]).noc_port(1));
                        self.noc
                            .connect(prev[3], cell.bel(bslots::HNOC_NPS[2]).noc_port(0));
                    }

                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(3),
                    );

                    let cell_ncrb = die.cell(self.die[die].cols_vnoc[i], row_bli);
                    self.noc.connect_uni(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                        cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                    );
                    self.noc.connect_uni(
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(3),
                        cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                    );
                    self.noc.connect_uni(
                        cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(0),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                    );
                    self.noc.connect_uni(
                        cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(3),
                    );

                    prev = Some([
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(1),
                    ]);
                }
            }
        }

        for (i, col_bli) in hbm.ms_bli.into_iter().enumerate() {
            let col = ColId::from_idx(i * 16);
            let cell = die.cell(col, row);
            let bli = die.cell(col_bli, row_bli);
            self.egrid.add_tile(
                cell,
                tcls::HBM_MS,
                &[cell.delta(4, 0), cell.delta(8, 0), bli, bli.delta(1, 0)],
            );
        }
    }

    fn fill_hsr_nic(&mut self, die: DieId, nic: &NicRow) {
        let chip = self.chips[die];
        let row = chip.row_hsr(DirV::N);
        let row_bli = chip.row_edge(DirV::N);
        let mut vnocs = self.die[die].cols_vnoc.iter().copied();
        let mut prev: Option<[NocPortCoord; 4]> = None;
        for (col, &cd) in &nic.blocks {
            let cell = die.cell(col, row);
            match cd {
                NicBlock::Cpm5NCdx => {
                    self.egrid.add_tile(cell, tcls::HNOC_NIC_CPM_CDX, &[]);
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NIC_AXI_NMU).noc_port(0),
                        cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(4),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NIC_AXI_NSU).noc_port(0),
                        cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(5),
                    );
                    for i in 0..4 {
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(0),
                            cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(i),
                        );
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(1),
                            cell.bel(bslots::HNOC_NIC_AXIS_NSU[i]).noc_port(0),
                        );
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(3),
                            cell.bel(bslots::HNOC_NIC_AXIS_NMU[i]).noc_port(0),
                        );
                    }
                    prev = Some([
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(2),
                    ]);
                }
                NicBlock::Cpm5NDpu { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_NIC_CPM_DPU,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    for i in 0..4 {
                        self.noc
                            .connect(cell.bel(bslots::HNOC_NPS[i]).noc_port(0), prev.unwrap()[i]);
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(1),
                            cell.bel(bslots::HNOC_NIC_AXIS_NSU[i]).noc_port(0),
                        );
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(3),
                            cell.bel(bslots::HNOC_NIC_AXIS_NMU[i]).noc_port(0),
                        );
                    }
                    prev = Some([
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(2),
                    ]);
                }
                NicBlock::Bli {
                    nsu_bli,
                    nmu_bli,
                    scan_bli,
                } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    let nmu_bli = nmu_bli.map(|col| die.cell(col, row_bli));
                    let nsu_bli = nsu_bli.map(|col| die.cell(col, row_bli));
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_NIC_BLI,
                        &[
                            nmu_bli[0],
                            nmu_bli[0].delta(1, 0),
                            nmu_bli[1],
                            nmu_bli[1].delta(1, 0),
                            nmu_bli[2],
                            nmu_bli[2].delta(1, 0),
                            nsu_bli[0],
                            nsu_bli[0].delta(1, 0),
                            nsu_bli[1],
                            nsu_bli[1].delta(1, 0),
                            nsu_bli[2],
                            nsu_bli[2].delta(1, 0),
                            nsu_bli[3],
                            nsu_bli[3].delta(1, 0),
                            scan_bli,
                            scan_bli.delta(1, 0),
                        ],
                    );
                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[0]).noc_port(0), prev.unwrap()[0]);
                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[1]).noc_port(0), prev.unwrap()[1]);
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPP_RPTR[0]).noc_port(0),
                        prev.unwrap()[2],
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPP_RPTR[1]).noc_port(0),
                        prev.unwrap()[3],
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NMU512[0]).noc_port(0),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(1),
                        cell.bel(bslots::HNOC_NSU512).noc_port(0),
                    );
                    prev = Some([
                        cell.bel(bslots::HNOC_NPP_RPTR[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NPP_RPTR[1]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                    ]);
                }
                NicBlock::VNoc {
                    scan_bli,
                    ncrb_scan_bli,
                    swap,
                } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    let ncrb_scan_bli = die.cell(ncrb_scan_bli, row_bli);
                    let col_vnoc = vnocs.next().unwrap();
                    let cell_vnoc =
                        die.cell(col_vnoc, chip.row_reg_bot(chip.regs().last().unwrap()));
                    let cell_ncrb = die.cell(col_vnoc, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_NIC_VNOC,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    self.egrid.add_tile(
                        cell_ncrb,
                        tcls::HNOC2_NCRB,
                        &[ncrb_scan_bli, ncrb_scan_bli.delta(1, 0)],
                    );

                    self.noc.connect_uni(
                        cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(0),
                        cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                    );
                    self.noc.connect_uni(
                        cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(1),
                        cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                    );
                    self.noc.connect_uni(
                        cell_vnoc.bel(bslots::VNOC_NPS_NMU).noc_port(0),
                        cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(0),
                    );
                    self.noc.connect_uni(
                        cell_vnoc.bel(bslots::VNOC_NPS_NSU).noc_port(2),
                        cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(1),
                    );

                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[0]).noc_port(0), prev.unwrap()[0]);
                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[1]).noc_port(0), prev.unwrap()[1]);
                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[2]).noc_port(0), prev.unwrap()[2]);
                    self.noc
                        .connect(cell.bel(bslots::HNOC_NPS[3]).noc_port(0), prev.unwrap()[3]);
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(1),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(1),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(1),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(3),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(1),
                    );

                    let (p0, p1) = if swap {
                        (
                            cell.bel(bslots::HNOC_NPS[3]).noc_port(3),
                            cell.bel(bslots::HNOC_NPS[2]).noc_port(3),
                        )
                    } else {
                        (
                            cell.bel(bslots::HNOC_NPS[2]).noc_port(3),
                            cell.bel(bslots::HNOC_NPS[3]).noc_port(3),
                        )
                    };
                    self.noc
                        .connect_uni(cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(0), p0);
                    self.noc
                        .connect_uni(cell_ncrb.bel(bslots::HNOC_NCRB[0]).noc_port(1), p1);
                    self.noc
                        .connect_uni(p0, cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(0));
                    self.noc
                        .connect_uni(p1, cell_ncrb.bel(bslots::HNOC_NCRB[1]).noc_port(1));

                    prev = Some([
                        cell.bel(bslots::HNOC_NPS[0]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[1]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[2]).noc_port(2),
                        cell.bel(bslots::HNOC_NPS[3]).noc_port(2),
                    ]);
                }
                NicBlock::HNicX { scan_bli } => {
                    let scan_bli = die.cell(scan_bli, row_bli);
                    self.egrid.add_tile(
                        cell,
                        tcls::HNOC_NIC_HNICX,
                        &[scan_bli, scan_bli.delta(1, 0)],
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NIC_AXI_NMU).noc_port(0),
                        cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(4),
                    );
                    self.noc.connect(
                        cell.bel(bslots::HNOC_NIC_AXI_NSU).noc_port(0),
                        cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(5),
                    );
                    for i in 0..4 {
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(0),
                            cell.bel(bslots::HNOC_NIC_XBR2X4).noc_port(i),
                        );
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(1),
                            cell.bel(bslots::HNOC_NIC_AXIS_NSU[i]).noc_port(0),
                        );
                        self.noc
                            .connect(cell.bel(bslots::HNOC_NPS[i]).noc_port(2), prev.unwrap()[i]);
                        self.noc.connect(
                            cell.bel(bslots::HNOC_NPS[i]).noc_port(3),
                            cell.bel(bslots::HNOC_NIC_AXIS_NMU[i]).noc_port(0),
                        );
                    }
                }
            }
        }
    }

    fn fill_hnoc_ssit_s(&mut self, die: DieId) {
        let pdie = die - 1;
        let chip = &self.chips[die];
        let pchip = &self.chips[pdie];
        let row = chip.row_s();
        let row_hsr = chip.row_hsr(DirV::S);
        let prow = pchip.row_reg_bot(RegId::from_idx(pchip.regs - 1));
        for (&col, &pcol) in self.die[die]
            .cols_vnoc
            .iter()
            .zip(self.die[pdie].cols_vnoc.iter())
        {
            let cell = die.cell(col, row);
            self.egrid.add_tile_e(cell, tcls::CLK_V_BUF_VNOC_SSIT_S, 2);
            let pcell = pdie.cell(pcol, prow);
            self.egrid
                .fill_conn_pair(pcell, cell, ccls::CLK_N_SSIT, ccls::CLK_S);
            self.egrid[cell].region_root[rslots::GCLK_PD_V] = pcell;
            self.egrid[cell.delta(1, 0)].region_root[rslots::GCLK_PD_V] = pcell.delta(1, 0);
            self.egrid.fill_conn_pair(
                pcell.delta(1, 0),
                cell.delta(1, 0),
                ccls::CLK_N_SSIT,
                ccls::CLK_S,
            );
        }
        let cell = die.cell(chip.col_e(), row);
        let pcell = pdie.cell(pchip.col_e(), prow);
        self.egrid.add_tile_single(cell, tcls::CLK_V_BUF_GT_SSIT_S);
        // what. why. what the fuck.
        if matches!(chip.hsr_n, HsrNKind::Hbm(_)) {
            self.egrid
                .fill_conn_pair(pcell, cell, ccls::CLK_N_SSIT_HBM, ccls::CLK_S);
        } else {
            self.egrid
                .fill_conn_pair(pcell, cell, ccls::CLK_N_SSIT, ccls::CLK_S);
        }
        self.egrid[cell].region_root[rslots::GCLK_PD_V] = pcell;

        let cell = die.cell(chip.col_w(), row_hsr);
        self.egrid.add_tile(cell, tcls::HNOC_SSIT_PS9_A, &[]);
        self.egrid
            .add_tile(cell.delta(1, 0), tcls::HNOC_SSIT_PS9_BD, &[]);
        self.egrid
            .add_tile(cell.delta(2, 0), tcls::HNOC_SSIT_PS9_C, &[]);
        self.egrid
            .add_tile(cell.delta(3, 0), tcls::HNOC_SSIT_PS9_BD, &[]);
        let bels_nps: [_; 8] =
            core::array::from_fn(|i| cell.delta(i as i32 / 2, 0).bel(bslots::HNOC_NPS[i % 2]));
        for i in 0..6 {
            self.noc
                .connect(bels_nps[i].noc_port(2), bels_nps[i + 2].noc_port(0));
        }
        for (pc, pi, si, sp) in [
            (0, 0, 1, 1), // PCIE 0
            (0, 1, 0, 3), // PCIE 1
            (0, 2, 0, 0), // PMC
            (0, 3, 0, 1), // RPU
            (2, 0, 7, 1), // NCI 0
            (2, 1, 6, 1), // NCI 1
            (2, 2, 5, 1), // CCI 0
            (2, 3, 4, 1), // CCI 1
            (2, 4, 3, 3), // CCI 2
            (2, 5, 2, 3), // CCI 3
        ] {
            self.noc.connect(
                cell.delta(pc, 0).bel(bslots::HNOC_PS_NMU[pi]).noc_port(0),
                bels_nps[si].noc_port(sp),
            );
        }
        for (pc, pi, si, sp) in [
            (0, 0, 1, 3), // PCIE
            (0, 1, 1, 0), // PMC
            (2, 0, 5, 3), // CCI 0
            (2, 1, 4, 3), // CCI 1
            (2, 2, 7, 3), // NCI 0
            (2, 3, 6, 3), // NCI 1
        ] {
            self.noc.connect(
                cell.delta(pc, 0).bel(bslots::HNOC_PS_NSU[pi]).noc_port(0),
                bels_nps[si].noc_port(sp),
            );
        }

        let mut port_prev0_o = bels_nps[6].noc_port(2);
        let mut port_prev0_i = bels_nps[6].noc_port(2);
        let mut port_prev1_o = bels_nps[7].noc_port(2);
        let mut port_prev1_i = bels_nps[7].noc_port(2);
        let orow = self.chips[pdie].row_n();
        for (&col, &ocol) in self.die[die]
            .cols_vnoc
            .iter()
            .zip(self.die[pdie].cols_vnoc.iter())
        {
            let cell = die.cell(col, row);
            self.egrid.add_tile(cell, tcls::HNOC_SSIT, &[]);
            self.egrid.add_tile(cell, tcls::HNOC_NCRB, &[]);
            let bel_ncrb0 = cell.bel(bslots::HNOC_NCRB[0]);
            let bel_ncrb1 = cell.bel(bslots::HNOC_NCRB[1]);
            let bel_nidb0 = cell.bel(bslots::HNOC_NIDB_V[0]);
            let bel_nidb1 = cell.bel(bslots::HNOC_NIDB_V[1]);
            let bel_nps0 = cell.bel(bslots::HNOC_NPS[0]);
            let bel_nps1 = cell.bel(bslots::HNOC_NPS[1]);
            let bel_rptr0 = cell.bel(bslots::HNOC_NPP_RPTR[0]);
            let bel_rptr1 = cell.bel(bslots::HNOC_NPP_RPTR[1]);
            let bel_rptr2 = cell.bel(bslots::HNOC_NPP_RPTR[2]);
            let bel_rptr3 = cell.bel(bslots::HNOC_NPP_RPTR[3]);
            let bel_vnoc_nps_nmu = cell.bel(bslots::VNOC_NPS_NMU);
            let bel_vnoc_nps_nsu = cell.bel(bslots::VNOC_NPS_NSU);

            self.noc.connect_uni(port_prev0_i, bel_rptr0.noc_port(0));
            self.noc.connect_uni(port_prev1_i, bel_rptr1.noc_port(0));
            self.noc.connect_uni(bel_rptr0.noc_port(0), port_prev0_o);
            self.noc.connect_uni(bel_rptr1.noc_port(0), port_prev1_o);

            self.noc
                .connect(bel_rptr0.noc_port(1), bel_nps0.noc_port(0));
            self.noc
                .connect(bel_rptr1.noc_port(1), bel_nps1.noc_port(0));
            self.noc
                .connect(bel_nps0.noc_port(1), bel_vnoc_nps_nsu.noc_port(0));
            self.noc
                .connect(bel_nps1.noc_port(3), bel_vnoc_nps_nmu.noc_port(2));
            self.noc
                .connect(bel_nps0.noc_port(3), bel_nidb1.noc_port(0));
            self.noc
                .connect(bel_nps1.noc_port(1), bel_nidb0.noc_port(0));
            self.noc
                .connect(bel_nps0.noc_port(2), bel_rptr2.noc_port(0));
            self.noc
                .connect(bel_nps1.noc_port(2), bel_rptr3.noc_port(0));
            let ocell = pdie.cell(ocol, orow);
            let obel_nidb0 = ocell.bel(bslots::HNOC_NIDB_V[0]);
            let obel_nidb1 = ocell.bel(bslots::HNOC_NIDB_V[1]);
            self.noc
                .connect(bel_nidb0.noc_port(1), obel_nidb0.noc_port(1));
            self.noc
                .connect(bel_nidb1.noc_port(1), obel_nidb1.noc_port(1));

            self.noc
                .connect_uni(bel_ncrb0.noc_port(0), bel_rptr2.noc_port(1));
            self.noc
                .connect_uni(bel_ncrb0.noc_port(1), bel_rptr3.noc_port(1));
            self.noc
                .connect_uni(bel_rptr2.noc_port(1), bel_ncrb1.noc_port(0));
            self.noc
                .connect_uni(bel_rptr3.noc_port(1), bel_ncrb1.noc_port(1));

            port_prev0_i = bel_ncrb1.noc_port(0);
            port_prev1_i = bel_ncrb1.noc_port(1);
            port_prev0_o = bel_ncrb0.noc_port(0);
            port_prev1_o = bel_ncrb0.noc_port(1);
        }
    }

    fn fill_hnoc_ssit_n(&mut self, die: DieId) {
        let chip = self.chips[die];
        let row = chip.row_n();
        let row_vnoc = self.chips[die].row_reg_bot(chip.regs().last().unwrap());
        let mut prev: Option<(BelCoord, BelCoord)> = None;
        let tcid = if chip.hsr_n == HsrNKind::Ssit2 {
            tcls::HNOC_SSIT2_V
        } else {
            tcls::HNOC_SSIT
        };
        for &col in &self.die[die].cols_vnoc {
            let cell = die.cell(col, row);
            let cell_vnoc = die.cell(col, row_vnoc);
            self.egrid.add_tile(cell, tcid, &[]);
            self.egrid.add_tile(cell, tcls::HNOC_NCRB, &[]);
            let bel_ncrb0 = cell.bel(bslots::HNOC_NCRB[0]);
            let bel_ncrb1 = cell.bel(bslots::HNOC_NCRB[1]);
            let bel_nidb0 = cell.bel(bslots::HNOC_NIDB_V[0]);
            let bel_nidb1 = cell.bel(bslots::HNOC_NIDB_V[1]);
            let bel_nps0 = cell.bel(bslots::HNOC_NPS[0]);
            let bel_nps1 = cell.bel(bslots::HNOC_NPS[1]);
            let bel_rptr0 = cell.bel(bslots::HNOC_NPP_RPTR[0]);
            let bel_rptr1 = cell.bel(bslots::HNOC_NPP_RPTR[1]);
            let bel_rptr2 = cell.bel(bslots::HNOC_NPP_RPTR[2]);
            let bel_rptr3 = cell.bel(bslots::HNOC_NPP_RPTR[3]);
            let bel_vnoc_nps_nmu = cell_vnoc.bel(bslots::VNOC_NPS_NMU);
            let bel_vnoc_nps_nsu = cell_vnoc.bel(bslots::VNOC_NPS_NSU);
            if let Some((prev_rptr2, prev_rptr3)) = prev {
                self.noc
                    .connect_uni(bel_ncrb1.noc_port(0), prev_rptr2.noc_port(1));
                self.noc
                    .connect_uni(bel_ncrb1.noc_port(1), prev_rptr3.noc_port(1));
                self.noc
                    .connect_uni(prev_rptr2.noc_port(1), bel_ncrb0.noc_port(0));
                self.noc
                    .connect_uni(prev_rptr3.noc_port(1), bel_ncrb0.noc_port(1));
            }
            self.noc
                .connect_uni(bel_ncrb0.noc_port(0), bel_rptr0.noc_port(0));
            self.noc
                .connect_uni(bel_ncrb0.noc_port(1), bel_rptr1.noc_port(0));
            self.noc
                .connect_uni(bel_rptr0.noc_port(0), bel_ncrb1.noc_port(0));
            self.noc
                .connect_uni(bel_rptr1.noc_port(0), bel_ncrb1.noc_port(1));
            self.noc
                .connect(bel_rptr0.noc_port(1), bel_nps0.noc_port(0));
            self.noc
                .connect(bel_rptr1.noc_port(1), bel_nps1.noc_port(0));
            self.noc
                .connect(bel_nps0.noc_port(1), bel_vnoc_nps_nsu.noc_port(2));
            self.noc
                .connect(bel_nps1.noc_port(3), bel_vnoc_nps_nmu.noc_port(0));
            self.noc
                .connect(bel_nps0.noc_port(3), bel_nidb1.noc_port(0));
            self.noc
                .connect(bel_nps1.noc_port(1), bel_nidb0.noc_port(0));
            self.noc
                .connect(bel_nps0.noc_port(2), bel_rptr2.noc_port(0));
            self.noc
                .connect(bel_nps1.noc_port(2), bel_rptr3.noc_port(0));
            prev = Some((bel_rptr2, bel_rptr3));
            if chip.hsr_n == HsrNKind::Ssit2 && (die.to_idx() & 1) != 0 {
                let odie = DieId::from_idx(die.to_idx() ^ 1);
                let ocell = odie.cell(col, row);
                let obel_nidb0 = ocell.bel(bslots::HNOC_NIDB_V[0]);
                let obel_nidb1 = ocell.bel(bslots::HNOC_NIDB_V[1]);
                self.noc
                    .connect(bel_nidb0.noc_port(1), obel_nidb0.noc_port(1));
                self.noc
                    .connect(bel_nidb1.noc_port(1), obel_nidb1.noc_port(1));
            }
        }
    }

    fn fill_hsr(&mut self) {
        for die in self.chips.ids() {
            let chip = self.chips[die];
            match chip.hsr_s {
                HsrSKind::Xpio(ref xpio) => {
                    self.fill_hsr_xpio(die, xpio, DirV::S);
                }
                HsrSKind::X5io(ref x5io) => {
                    self.fill_hsr_x5io(die, x5io);
                }
                HsrSKind::Ssit => {
                    self.fill_hnoc_ssit_s(die);
                }
            }
            match chip.hsr_n {
                HsrNKind::Xpio(ref xpio) => {
                    self.fill_hsr_xpio(die, xpio, DirV::N);
                }
                HsrNKind::Ssit | HsrNKind::Ssit2 => {
                    self.fill_hnoc_ssit_n(die);
                }
                HsrNKind::Me(ref me) => {
                    self.fill_hsr_me(die, me);
                }
                HsrNKind::Me2(ref me2) => {
                    self.fill_hsr_me2(die, me2);
                }
                HsrNKind::Hbm(ref hbm) => {
                    self.fill_hsr_hbm(die, hbm);
                }
                HsrNKind::Nic(ref nic) => {
                    self.fill_hsr_nic(die, nic);
                }
                HsrNKind::None => (),
            }
        }
    }

    fn fill_clkroot(&mut self) {
        for (die, chip) in &self.chips {
            for cell in self.die_cells_pl(die) {
                let reg = chip.row_to_reg(cell.row);
                let crow = cell.row.snap_to_side_of(chip.row_reg_hclk(reg));
                self.egrid[cell].region_root[rslots::LEAF] = cell.with_row(crow);
            }
            let mut prev_row = Some(chip.row_s());
            for reg in chip.regs() {
                if !chip.is_reg_n(reg) {
                    continue;
                }
                let row = chip.row_reg_hclk(reg);
                let mut root_pd = die.cell(chip.col_w(), row);
                let mut root_route = root_pd;
                let mut root_distr = root_pd;
                let mut root_distr_loc = root_pd;
                for cell in self.egrid.row(die, row) {
                    self.egrid[cell].region_root[rslots::GCLK_PD_H] = root_pd;
                    self.egrid[cell].region_root[rslots::HDISTR_LOC] = root_distr_loc;
                    self.egrid[cell].region_root[rslots::HROUTE] = root_route;
                    self.egrid[cell].region_root[rslots::HDISTR] = root_distr;
                    match chip.columns[cell.col].kind {
                        ColumnKind::Bram(BramKind::SplitterPd) => {
                            root_route = cell.delta(1, 0);
                            root_distr = cell.delta(1, 0);
                            root_distr_loc = cell.delta(1, 0);
                        }
                        ColumnKind::Bram(BramKind::Splitter) => {
                            root_route = cell.delta(1, 0);
                            if chip.fabric != FabricKind::Ulvt {
                                root_distr = cell.delta(1, 0);
                            }
                        }
                        ColumnKind::Bram(BramKind::OptSplitter) => {
                            if cell.row.to_idx() >= chip.ps.height() {
                                root_route = cell.delta(1, 0);
                                if chip.fabric != FabricKind::Ulvt {
                                    root_distr = cell.delta(1, 0);
                                }
                            }
                        }
                        ColumnKind::VNoc | ColumnKind::VNoc2 | ColumnKind::VNoc4 => {
                            root_route = cell.delta(1, 0);
                            root_distr = cell.delta(1, 0);
                            root_distr_loc = cell.delta(1, 0);
                            root_pd = cell.delta(1, 0);
                            if cell.col == *self.die[die].cols_vnoc.last().unwrap()
                                && self.interposer.kind == InterposerKind::MirrorSquare
                                && die.to_idx() >= 2
                            {
                                root_pd.die = DieId::from_idx(die.to_idx() ^ 3);
                            }
                        }
                        _ => (),
                    }
                }
                if !chip.is_reg_half(reg) {
                    for cell in self.egrid.row(die, row) {
                        self.egrid.fill_conn_pair(
                            cell.delta(0, -(Chip::ROWS_PER_REG as i32)),
                            cell,
                            ccls::CLK_HALF_N,
                            ccls::CLK_HALF_S,
                        );
                    }
                }
                if let Some(prev_row) = prev_row {
                    for cell in self.egrid.row(die, row) {
                        if matches!(
                            chip.columns[cell.col].kind,
                            ColumnKind::Gt | ColumnKind::HNicX
                        ) {
                            let clk_n = if self.chips.len() != 1 {
                                ccls::CLK_N_GT_LEV3
                            } else {
                                ccls::CLK_N_GT
                            };
                            self.egrid.fill_conn_pair(
                                cell.with_row(prev_row),
                                cell,
                                clk_n,
                                ccls::CLK_S,
                            );
                        } else if chip.columns[cell.col].kind.is_vnoc() {
                            let clk_n = if self.chips.len() != 1 {
                                ccls::CLK_N_VNOC_LEV3
                            } else {
                                ccls::CLK_N_VNOC
                            };
                            self.egrid.fill_conn_pair(
                                cell.with_row(prev_row),
                                cell,
                                clk_n,
                                ccls::CLK_S,
                            );
                            self.egrid.fill_conn_pair(
                                cell.delta(1, 0).with_row(prev_row),
                                cell.delta(1, 0),
                                clk_n,
                                ccls::CLK_S,
                            );
                        }
                    }
                }
                prev_row = Some(row);
            }
        }
    }
}

pub fn expand_grid<'a>(
    chips: &EntityVec<DieId, &'a Chip>,
    interposer: &'a Interposer,
    disabled: &BTreeSet<DisabledPart>,
    db: &'a IntDb,
) -> ExpandedDevice<'a> {
    let mut expander = Expander {
        chips: chips.clone(),
        interposer,
        disabled: disabled.clone(),
        egrid: GridBuilder::new(db),
        die: EntityVec::new(),
        noc: NocBuilder::new(db),
    };
    expander.fill_die();
    expander.fill_int();
    expander.fill_cle_bc();
    expander.fill_intf();
    expander.fill_cle();
    expander.fill_dsp();
    expander.fill_bram();
    expander.fill_uram();
    expander.fill_hard();
    expander.fill_vnoc();
    expander.fill_gt();
    expander.fill_ps();
    expander.fill_hsr();
    expander.fill_clkroot();
    let egrid = expander.egrid.finish();
    let noc = expander.noc.finish();

    let col_cfrm = expander.die.map_values(|die| die.col_cfrm);
    let cols_vnoc = expander.die.map_values(|die| die.cols_vnoc.clone());

    let mut sll = HashMap::new();
    match interposer.kind {
        InterposerKind::Single => (),
        InterposerKind::Column => {
            fill_sll_column(interposer, chips, &mut sll);
        }
        InterposerKind::MirrorSquare => {
            fill_sll_mirror_square(interposer, chips, &mut sll);
        }
    }

    ExpandedDevice {
        chips: expander.chips,
        interposer,
        egrid,
        disabled: expander.disabled,
        col_cfrm,
        cols_vnoc,
        sll,
        noc,
    }
}

fn fill_sll_column(
    interposer: &Interposer,
    chips: &EntityVec<DieId, &Chip>,
    sll: &mut HashMap<BelCoord, SllConn>,
) {
    let mut curse_queue = vec![];
    for (die, cols) in &interposer.sll_columns {
        let chip = chips[die];
        let all_rows = chip.rows_pl();
        let has_link_bot = die != chips.first_id().unwrap();
        let has_link_top = die != chips.last_id().unwrap();
        for (cidx, &col) in cols.iter().enumerate() {
            let start = if chip.columns[col].has_bli_s {
                assert!(!has_link_bot);
                4
            } else {
                0
            };
            let end = if chip.columns[col].has_bli_n {
                assert!(!has_link_top);
                all_rows.len() - 4
            } else {
                all_rows.len()
            };
            let rows: EntityRange<RowId> = EntityRange::new(start, end);
            for row in rows {
                let cell = die.cell(col, row);
                if has_link_bot && row.to_idx() < start + 75 {
                    let odie = die - 1;
                    let orow = RowId::from_idx(chips[odie].rows_pl().len() - 75 + row.to_idx());
                    let ocol = interposer.sll_columns[odie][cidx];
                    let ocell = odie.cell(ocol, orow);
                    for (bump, obump) in [(0, 0), (1, 1), (3, 2), (4, 4), (5, 5)] {
                        sll.insert(
                            cell.bel(bslots::LAGUNA[bump]),
                            ocell.bel(bslots::LAGUNA[obump]).into(),
                        );
                    }
                    sll.insert(
                        cell.bel(bslots::LAGUNA[2]),
                        cell.delta(0, 75).bel(bslots::LAGUNA[3]).into(),
                    );
                } else if has_link_top && row.to_idx() >= end - 75 {
                    let odie = die + 1;
                    let orow = RowId::from_idx(row.to_idx() - (end - 75));
                    let ocol = interposer.sll_columns[odie][cidx];
                    let ocell = odie.cell(ocol, orow);
                    for (bump, obump) in [(0, 0), (1, 1), (2, 3), (4, 4), (5, 5)] {
                        sll.insert(
                            cell.bel(bslots::LAGUNA[bump]),
                            ocell.bel(bslots::LAGUNA[obump]).into(),
                        );
                    }
                    sll.insert(
                        cell.bel(bslots::LAGUNA[3]),
                        cell.delta(0, -75).bel(bslots::LAGUNA[2]).into(),
                    );
                } else {
                    if row.to_idx() < start + 75 {
                        let triad = (row.to_idx() - start) / 3;
                        let sub = (row.to_idx() - start) % 3;
                        let orow = RowId::from_idx(start + (24 - triad) * 3 + sub);
                        if orow != row {
                            sll.insert(
                                cell.bel(bslots::LAGUNA[3]),
                                cell.with_row(orow).bel(bslots::LAGUNA[3]).into(),
                            );
                        }
                    } else {
                        sll.insert(
                            cell.bel(bslots::LAGUNA[3]),
                            cell.delta(0, -75).bel(bslots::LAGUNA[2]).into(),
                        );
                    }
                    if row.to_idx() >= end - 75 {
                        let triad = (row.to_idx() - (end - 75)) / 3;
                        let sub = (row.to_idx() - (end - 75)) % 3;
                        let orow = RowId::from_idx(end - 75 + (24 - triad) * 3 + sub);
                        if orow != row {
                            sll.insert(
                                cell.bel(bslots::LAGUNA[2]),
                                cell.with_row(orow).bel(bslots::LAGUNA[2]).into(),
                            );
                        }
                    } else {
                        sll.insert(
                            cell.bel(bslots::LAGUNA[2]),
                            cell.delta(0, 75).bel(bslots::LAGUNA[3]).into(),
                        );
                    }
                    if cidx < 10 {
                        for bump in [1, 5] {
                            let ocol = cols[9 - cidx];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[bump]).into(),
                            );
                        }
                    } else {
                        for (bump, obump) in [(1, 0), (5, 4)] {
                            let ocol = cols[cidx - 10];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[obump]).into(),
                            );
                        }
                    }
                    if cidx >= cols.len() - 10 {
                        for bump in [0, 4] {
                            let ocol = cols[cols.len() - 10 + (9 - (cidx - (cols.len() - 10)))];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[bump]).into(),
                            );
                        }
                    } else {
                        for (bump, obump) in [(0, 1), (4, 5)] {
                            let ocol = cols[cidx + 10];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[obump]).into(),
                            );
                        }
                    }
                }
            }
            curse_queue.push(die.cell(col, RowId::from_idx(start)));
            if has_link_top {
                curse_queue.push(die.cell(col, RowId::from_idx(end - 75)));
            }
            if has_link_bot {
                curse_queue.push(die.cell(col, RowId::from_idx(start + 75)));
            }
        }
    }
    for cell in curse_queue {
        for bslot in bslots::LAGUNA {
            let conn = sll.entry(cell.bel(bslot)).or_default();
            conn.cursed = true;
            if let Some(target) = conn.target {
                sll.get_mut(&target).unwrap().cursed = true;
            }
        }
    }
}

fn fill_sll_mirror_square(
    interposer: &Interposer,
    chips: &EntityVec<DieId, &Chip>,
    sll: &mut HashMap<BelCoord, SllConn>,
) {
    let mut curse_queue = vec![];
    for (die, cols) in &interposer.sll_columns {
        let chip = chips[die];
        let all_rows = chip.rows_pl();
        let col_cfrm = chip
            .columns
            .iter()
            .find(|(_, cd)| cd.kind == ColumnKind::Cfrm)
            .unwrap()
            .0;
        let ps_height = chip.ps.height();
        let cidx_ps = cols.binary_search(&col_cfrm).unwrap_err();
        for (cidx, &col) in cols.iter().enumerate() {
            let start = if col < col_cfrm {
                ps_height
            } else if chip.columns[col].has_bli_s {
                4
            } else {
                0
            };
            let end = all_rows.len();
            let rows: EntityRange<RowId> = EntityRange::new(start, end);
            for row in rows {
                let cell = die.cell(col, row);
                let cidx_l = if row.to_idx() < ps_height { cidx_ps } else { 0 };
                if row == RowId::from_idx(end - 63) {
                    // do nothing
                } else if cidx < cols.len() - 10 {
                    if row < RowId::from_idx(end - 63) {
                        if row.to_idx() < start + 75 {
                            let triad = (row.to_idx() - start) / 3;
                            let sub = (row.to_idx() - start) % 3;
                            let orow = RowId::from_idx(start + (24 - triad) * 3 + sub);
                            if orow != row {
                                sll.insert(
                                    cell.bel(bslots::LAGUNA[3]),
                                    cell.with_row(orow).bel(bslots::LAGUNA[3]).into(),
                                );
                            }
                        } else {
                            sll.insert(
                                cell.bel(bslots::LAGUNA[3]),
                                cell.delta(0, -75).bel(bslots::LAGUNA[2]).into(),
                            );
                        }
                        if row.to_idx() >= end - 63 - 75 {
                            // nothing
                        } else {
                            sll.insert(
                                cell.bel(bslots::LAGUNA[2]),
                                cell.delta(0, 75).bel(bslots::LAGUNA[3]).into(),
                            );
                        }
                        if cidx < cidx_l + 10 {
                            for bump in [1, 5] {
                                let ocol = cols[cidx_l + 9 - (cidx - cidx_l)];
                                sll.insert(
                                    cell.bel(bslots::LAGUNA[bump]),
                                    cell.with_col(ocol).bel(bslots::LAGUNA[bump]).into(),
                                );
                            }
                        } else {
                            for (bump, obump) in [(1, 0), (5, 4)] {
                                let ocol = cols[cidx - 10];
                                sll.insert(
                                    cell.bel(bslots::LAGUNA[bump]),
                                    cell.with_col(ocol).bel(bslots::LAGUNA[obump]).into(),
                                );
                            }
                        }
                        for (bump, obump) in [(0, 1), (4, 5)] {
                            let ocol = cols[cidx + 10];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[obump]).into(),
                            );
                        }
                    } else {
                        for bump in 0..6 {
                            let odie = DieId::from_idx(die.to_idx() ^ 1);
                            let orow = RowId::from_idx(end - 62 + (end - 1 - row.to_idx()));
                            let ocell = odie.cell(col, orow);
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                ocell.bel(bslots::LAGUNA[bump]).into(),
                            );
                        }
                    }
                } else {
                    if !(46..50).contains(&(row.to_idx() % (Chip::ROWS_PER_REG * 2))) {
                        for bump in [0, 2, 4] {
                            let odie = DieId::from_idx(die.to_idx() ^ 3);
                            let ocol = cols[cols.len() - 10 + (cols.len() - 1 - cidx)];
                            let ocell = odie.cell(ocol, row);
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                ocell.bel(bslots::LAGUNA[bump]).into(),
                            );
                        }
                    }
                    if row.to_idx() % (Chip::ROWS_PER_REG * 2) == 50 {
                        for bump in [0, 2, 4] {
                            curse_queue.push(die.cell(col, row).bel(bslots::LAGUNA[bump]));
                        }
                    }
                    if row < RowId::from_idx(end - 63) {
                        for (bump, obump) in [(1, 0), (5, 4)] {
                            let ocol = cols[cidx - 10];
                            sll.insert(
                                cell.bel(bslots::LAGUNA[bump]),
                                cell.with_col(ocol).bel(bslots::LAGUNA[obump]).into(),
                            );
                        }
                        if row.to_idx() % (Chip::ROWS_PER_REG * 2) == 50 {
                            for bump in [1, 5] {
                                curse_queue.push(die.cell(col, row).bel(bslots::LAGUNA[bump]));
                            }
                        }
                    }
                }
            }
            for bump in 0..6 {
                if cidx < cols.len() - 10 || bump != 3 {
                    curse_queue.push(
                        die.cell(col, RowId::from_idx(start))
                            .bel(bslots::LAGUNA[bump]),
                    );
                }
                if cidx < cols.len() - 10 || matches!(bump, 0 | 2 | 4) {
                    curse_queue.push(
                        die.cell(col, RowId::from_idx(end - 63))
                            .bel(bslots::LAGUNA[bump]),
                    );
                }
            }
            if cidx < 10 {
                let row = RowId::from_idx(ps_height);
                for bump in [1, 5] {
                    curse_queue.push(die.cell(col, row).bel(bslots::LAGUNA[bump]));
                }
            }
        }
    }
    for bel in curse_queue {
        let conn = sll.entry(bel).or_default();
        conn.cursed = true;
        if let Some(target) = conn.target {
            sll.get_mut(&target).unwrap().cursed = true;
        }
    }
}
