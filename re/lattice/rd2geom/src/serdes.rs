use prjcombine_ecp::{
    chip::{ChipKind, IoGroupKind, NexusKind, SpecialLocKey},
    defs::{self, bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{DirH, DirHV, DirV},
    grid::{ColIdExt, RowIdExt},
};

use crate::ChipContext;

const NEXUS_PCS_FROM_PCIE_CH: &[(&str, &str, usize)] = &[
    ("PIPE_BLOCK_ALIGN_CONTROL_LL", "PIPE_BLOCK_ALIGN_CONTROL", 1),
    ("PIPE_TX_ELEC_IDLE_LL", "PIPE_TX_ELEC_IDLE", 1),
    ("PIPE_TX_COMPLIANCE_LL", "PIPE_TX_COMPLIANCE", 1),
    ("PIPE_TX_DATAK_LL", "PIPE_TX_DATAK", 4),
    ("PIPE_TX_DATA_LL", "PIPE_TX_DATA", 32),
    ("PIPE_TX_SYNC_HEADER_LL", "PIPE_TX_SYNC_HEADER", 2),
    ("PIPE_TX_START_BLOCK_LL", "PIPE_TX_START_BLOCK", 1),
    ("PIPE_TX_DATA_VALID_LL", "PIPE_TX_DATA_VALID", 1),
    ("PIPE_REMOTE_EQ_RX_PRESET_LL", "PIPE_REMOTE_EQ_RX_PRESET", 4),
    (
        "PIPE_REMOTE_EQ_RX_DEEMPH_LL",
        "PIPE_REMOTE_EQ_RX_DEEMPH",
        18,
    ),
    ("PIPE_REMOTE_LF_LL", "PIPE_REMOTE_LF", 6),
    ("PIPE_REMOTE_FS_LL", "PIPE_REMOTE_FS", 6),
    (
        "PIPE_LOCAL_GET_PRESET_INDEX_LL",
        "PIPE_LOCAL_GET_PRESET_INDEX",
        4,
    ),
    (
        "PIPE_LOCAL_GET_PRESET_COEF_LL",
        "PIPE_LOCAL_GET_PRESET_COEF",
        1,
    ),
    ("PIPE_TX_DEEMPH_WINDOW_LL", "PIPE_TX_DEEMPH_WINDOW", 1),
    ("PIPE_TX_DEEMPH_LL", "PIPE_TX_DEEMPH", 18),
    ("PIPE_TX_MARGIN_LL", "PIPE_TX_MARGIN", 3),
    ("PIPE_TX_SWING_LL", "PIPE_TX_SWING", 1),
    ("PIPE_RX_EI_DISABLE_LL", "PIPE_RX_EI_DISABLE", 1),
    ("PIPE_TX_CM_DISABLE_LL", "PIPE_TX_CM_DISABLE", 1),
    ("PIPE_TX_CLKREQ_N_LL", "PIPE_TX_CLKREQ_N", 1),
    ("PIPE_RX_POLARITY_LL", "PIPE_RX_POLARITY", 1),
    ("PIPE_RX_PRESET_HINT_LL", "PIPE_RX_PRESET_HINT", 3),
    (
        "PIPE_RX_PRESET_HINT_ENABLE_LL",
        "PIPE_RX_PRESET_HINT_ENABLE",
        1,
    ),
    ("PIPE_INVALID_REQUEST_LL", "PIPE_INVALID_REQUEST", 1),
    ("PIPE_RX_EQ_EVAL_LL", "PIPE_RX_EQ_EVAL", 1),
    (
        "PIPE_TX_DETECT_RX_LOOPBACK_LL",
        "PIPE_TX_DETECT_RX_LOOPBACK",
        1,
    ),
    ("PIPE_RATE_LL", "PIPE_RATE", 2),
    ("PIPE_POWER_DOWN_LL", "PIPE_POWER_DOWN", 2),
    ("LINK0_CSR_PIPE_RESET", "LINK0_CSR_PIPE_RESET", 1),
];

const NEXUS_PCS_TO_PCIE_CH: &[(&str, &str, usize)] = &[
    ("PIPE_WIDTH_2G5_LL", "PIPE_WIDTH_2G5", 2),
    ("PIPE_WIDTH_5G0_LL", "PIPE_WIDTH_5G0", 2),
    ("PIPE_WIDTH_8G0_LL", "PIPE_WIDTH_8G0", 2),
    (
        "PIPE_RX_EQ_EVAL_FEEDBACK_FOM_LL",
        "PIPE_RX_EQ_EVAL_FEEDBACK_FROM",
        8,
    ),
    (
        "PIPE_RX_EQ_EVAL_FEEDBACK_DIR_LL",
        "PIPE_RX_EQ_EVAL_FEEDBACK_DIR",
        6,
    ),
    ("PIPE_PHY_STATUS_LL", "PIPE_PHY_STATUS", 1),
    ("PIPE_RX_ELEC_IDLE_LL", "PIPE_RX_ELEC_IDLE", 1),
    ("PIPE_RX_CLKREQ_N_LL", "PIPE_RX_CLKREQ_N", 1),
    ("PIPE_LOCAL_FS_LL", "PIPE_LOCAL_FS", 6),
    ("PIPE_LOCAL_LF_LL", "PIPE_LOCAL_IF", 6),
    (
        "PIPE_LOCAL_GET_TX_COEF_VALID_LL",
        "PIPE_LOCAL_GET_TX_COEF_VALID",
        1,
    ),
    (
        "PIPE_LOCAL_GET_TX_PRESET_COEF_LL",
        "PIPE_LOCAL_GET_TX_PRESET_COEF",
        18,
    ),
    ("PIPE_TX_DATA_ENABLE_LL", "PIPE_TX_DATA_EN", 1),
    ("PIPE_RX_DATAEN_LL", "PIPE_RX_DATA_EN", 1),
    ("PIPE_RX_DATA_VALID_LL", "PIPE_RX_DATA_VALID", 1),
    ("PIPE_RX_START_BLOCK_LL", "PIPE_RX_START_BLOCK", 1),
    ("PIPE_RX_SYNC_HEADER_LL", "PIPE_RX_SYNC_HEADER", 2),
    ("PIPE_RX_DATA_LL", "PIPE_RX_DATA", 32),
    ("PIPE_RX_DATAK_LL", "PIPE_RX_DATAK", 4),
    ("PIPE_RX_VALID_LL", "PIPE_RX_VALID", 1),
    ("PIPE_RX_STATUS_LL", "PIPE_RX_STATUS", 3),
];

const NEXUS_PCS_FROM_PCIE: &[(&str, &str)] = &[
    ("PCIELL_MODESEL2", "BIFUR_SEL2"),
    ("PCIELL_MODESEL1", "BIFUR_SEL1"),
    ("PCIELL_MODESEL0", "BIFUR_SEL0"),
    ("LINK1_CSR_PIPE_RESET", "LINK1_CSR_PIPE_RESET"),
];

const NEXUS_PCS_TO_PCIE: &[(&str, &str)] = &[
    ("CH3_PCLKAUX_SEL_LL", "LINK1_SEL_PCLK1_AUX0"),
    ("CH0_PCLKAUX_SEL_LL", "LINK0_SEL_PCLK1_AUX0"),
    ("PIPE_PCLKOUT_LL3", "LINK1_CK250I"),
    ("PIPE_PCLKOUT_LL0", "LINK0_CK250I"),
    ("CH3_CLKREQOE", "PIPE_TX_CLKACK_N_X1"),
    ("CH0_CLKREQOE", "PIPE_TX_CLKACK_N_X40"),
    ("CH1_CLKREQOE", "PIPE_TX_CLKACK_N_X41"),
    ("CH2_CLKREQOE", "PIPE_TX_CLKACK_N_X42"),
    ("CH3_CLKREQOE", "PIPE_TX_CLKACK_N_X43"),
];

impl ChipContext<'_> {
    fn process_serdes_scm(&mut self) {
        for hv in [DirHV::NW, DirHV::NE] {
            let bcrd = self.chip.bel_pll(hv, 0).bel(bslots::SERDES_CORNER);

            let mut cells_serdes = vec![];
            match hv.h {
                DirH::W => {
                    for (col, cd) in &self.chip.columns {
                        if cd.io[DirV::N] == IoGroupKind::Serdes && col < self.chip.col_clk {
                            cells_serdes.push(bcrd.with_cr(col + 6, self.chip.row_n() - 11));
                        }
                    }
                }
                DirH::E => {
                    for (col, cd) in self.chip.columns.iter().rev() {
                        if cd.io[DirV::N] == IoGroupKind::Serdes && col >= self.chip.col_clk {
                            cells_serdes.push(bcrd.with_cr(col, self.chip.row_n() - 11));
                        }
                    }
                }
            };

            {
                let mut bel = self.extract_bel(bcrd);
                bel.name_null();

                for i in 0..13 {
                    let wires = Vec::from_iter(
                        cells_serdes
                            .iter()
                            .copied()
                            .map(|cell| bel.ctx.rc_wire(cell, &format!("JCIN_{i}_PCS"))),
                    );
                    bel.input_sc_xlat(bcls::SERDES_CORNER::CIN[i], &wires);
                }
                {
                    let wires = Vec::from_iter(
                        cells_serdes
                            .iter()
                            .copied()
                            .map(|cell| bel.ctx.rc_wire(cell, "JTESTCLK_MACO_PCS")),
                    );
                    bel.input_sc_xlat(bcls::SERDES_CORNER::TESTCLK_MACO, &wires);
                }
                for i in 0..22 {
                    let wire = bel.ctx.rc_wire(cells_serdes[0], &format!("JCOUT_{i}_PCS"));
                    bel.output_sc_xlat(bcls::SERDES_CORNER::COUT[i], wire);
                }
                {
                    let wire = bel.ctx.rc_wire(cells_serdes[0], "JTCK_FMAC_PCS");
                    bel.output_sc_xlat(bcls::SERDES_CORNER::TCK_FMAC_PCS, wire);
                }
            }

            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            for (i, cell_serdes) in cells_serdes.into_iter().enumerate() {
                for j in 0..4 {
                    let wname_pcs = ex.ctx.rc_wire(cell_serdes, &format!("JBS4PAD_{j}_PCS"));
                    let wname = ex.ctx.pips_fwd[&wname_pcs]
                        .iter()
                        .copied()
                        .find(|&w| !ex.ctx.int_wires.contains_key(&w))
                        .unwrap();
                    ex.ctx.claim_node(wname);
                    let src = wires::OUT_SERDES_BS4PAD[j].cell(3 + i);
                    let dst = ex.ctx.xlat_sc_wire(wname, ex.bcrd);
                    ex.bind_wire(wname, src).permabuf(dst, src);
                }
            }
        }
        for tcid in [tcls::SERDES_W, tcls::SERDES_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::SERDES);
                let bank = self.chip.columns[bcrd.col].bank[DirV::N];
                let (edge, quad) = match bank {
                    9 => (DirH::W, 0),
                    11 => (DirH::W, 1),
                    13 => (DirH::W, 2),
                    15 => (DirH::W, 3),
                    10 => (DirH::E, 0),
                    12 => (DirH::E, 1),
                    14 => (DirH::E, 2),
                    16 => (DirH::E, 3),
                    _ => unreachable!(),
                };
                let name_pcs = match edge {
                    DirH::W => format!("PCS36{quad}00"),
                    DirH::E => format!("PCS3E{quad}00"),
                };
                let abcd = ['A', 'B', 'C', 'D'][quad];
                let lr = match edge {
                    DirH::W => 'L',
                    DirH::E => 'R',
                };
                let cell = match edge {
                    DirH::W => bcrd.delta(6, 1),
                    DirH::E => bcrd.delta(0, 1),
                };
                let cell_next = self
                    .chip
                    .columns
                    .ids()
                    .into_iter()
                    .find(|&col| self.chip.columns[col].bank.get(DirV::N) == Some(&(bank + 2)))
                    .map(|col| match edge {
                        DirH::W => bcrd.with_col(col).delta(6, 1),
                        DirH::E => bcrd.with_col(col).delta(0, 1),
                    });
                let cell_apio = match edge {
                    DirH::W => [cell.delta(0, 11), cell.delta(-1, 11)],
                    DirH::E => [cell.delta(0, 11), cell.delta(1, 11)],
                };
                let (_, c_apio1) = self.rc(cell_apio[1]);

                self.extract_bel_cell(bcrd, cell)
                    .names([
                        name_pcs,
                        format!("{abcd}_REFCLKP_{lr}"),
                        format!("{abcd}_REFCLKN_{lr}"),
                        format!("{abcd}_HDINP0_{lr}"),
                        format!("{abcd}_HDINN0_{lr}"),
                        format!("{abcd}_HDINP1_{lr}"),
                        format!("{abcd}_HDINN1_{lr}"),
                        format!("{abcd}_HDINP2_{lr}"),
                        format!("{abcd}_HDINN2_{lr}"),
                        format!("{abcd}_HDINP3_{lr}"),
                        format!("{abcd}_HDINN3_{lr}"),
                        format!("{abcd}_HDOUTP0_{lr}"),
                        format!("{abcd}_HDOUTN0_{lr}"),
                        format!("{abcd}_HDOUTP1_{lr}"),
                        format!("{abcd}_HDOUTN1_{lr}"),
                        format!("{abcd}_HDOUTP2_{lr}"),
                        format!("{abcd}_HDOUTN2_{lr}"),
                        format!("{abcd}_HDOUTP3_{lr}"),
                        format!("{abcd}_HDOUTN3_{lr}"),
                        format!("PT{c_apio1}I"),
                        format!("PT{c_apio1}J"),
                    ])
                    .suffix("PCS")
                    .output_bind_rc(
                        bcls::SERDES::FF_RXCLK_P1,
                        "JFF_RXCLK_P1_PCS",
                        wires::OUT_SERDES_FF_RXCLK_P1.cell(0),
                    )
                    .output_bind_rc(
                        bcls::SERDES::FF_RXCLK_P2,
                        "JFF_RXCLK_P2_PCS",
                        wires::OUT_SERDES_FF_RXCLK_P2.cell(0),
                    )
                    .output_bind_rc(
                        bcls::SERDES::FF_SYSCLK_P1,
                        "JFF_SYSCLK_P1_PCS",
                        wires::OUT_SERDES_FF_SYSCLK_P1.cell(0),
                    );

                for (cell_pio, abcd, pin) in [
                    (cell_apio[0], 'A', "HDOUTP3"),
                    (cell_apio[0], 'B', "HDOUTN3"),
                    (cell_apio[0], 'C', "HDINP3"),
                    (cell_apio[0], 'D', "HDINN3"),
                    (cell_apio[0], 'E', "HDOUTP2"),
                    (cell_apio[0], 'F', "HDOUTN2"),
                    (cell_apio[0], 'G', "HDINP2"),
                    (cell_apio[0], 'H', "HDINN2"),
                    (cell_apio[0], 'I', "REFCLKP"),
                    (cell_apio[0], 'J', "REFCLKN"),
                    (cell_apio[1], 'A', "HDOUTP1"),
                    (cell_apio[1], 'B', "HDOUTN1"),
                    (cell_apio[1], 'C', "HDINP1"),
                    (cell_apio[1], 'D', "HDINN1"),
                    (cell_apio[1], 'E', "HDOUTP0"),
                    (cell_apio[1], 'F', "HDOUTN0"),
                    (cell_apio[1], 'G', "HDINP0"),
                    (cell_apio[1], 'H', "HDINN0"),
                    (cell_apio[1], 'I', "RXREFCLKP"),
                    (cell_apio[1], 'J', "RXREFCLKN"),
                ] {
                    for pin_pio in ["INPUT", "OUTPUT", "CLOCK"] {
                        let wire_apio =
                            self.rc_io_wire(cell_pio, &format!("J{pin_pio}{abcd}_APIO"));
                        self.add_bel_wire(bcrd, format!("{pin}_APIO_{pin_pio}"), wire_apio);
                        if pin_pio == "INPUT"
                            && (pin.starts_with("HDIN") || pin.starts_with("REFCLK"))
                        {
                            let wire = self.rc_wire(cell, &format!("J{pin}_PCS"));
                            self.add_bel_wire(bcrd, pin, wire);
                            self.claim_pip(wire, wire_apio);
                        }
                        if pin_pio == "OUTPUT" && pin.starts_with("HDOUT") {
                            let wire = self.rc_wire(cell, &format!("J{pin}_PCS"));
                            self.add_bel_wire(bcrd, pin, wire);
                            self.claim_pip(wire_apio, wire);
                        }
                    }
                }

                {
                    let wire = self.rc_wire(cell, "JCS_QIF_PCS");
                    self.add_bel_wire(bcrd, "CS_QIF", wire);
                    let cell_sysbus = self.chip.special_loc[&SpecialLocKey::Config].delta(2, 0);
                    let wire_sysbus = self.rc_wire(cell_sysbus, &format!("J{lr}PCSQ{quad}_SYSBUS"));
                    self.claim_pip(wire, wire_sysbus);
                }

                for i in 0..22 {
                    let cout = self.rc_wire(cell, &format!("JCOUT_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("COUT_{i}"), cout);
                    let bist_coutp = self.rc_wire(cell, &format!("JBIST_COUTP_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("BIST_COUTP_{i}"), bist_coutp);
                    let scan_coutp = self.rc_wire(cell, &format!("JSCAN_COUTP_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("SCAN_COUTP_{i}"), scan_coutp);
                    self.claim_pip(cout, scan_coutp);
                    self.claim_pip(cout, bist_coutp);
                    if let Some(cell_next) = cell_next {
                        let cout_next = self.rc_wire(cell_next, &format!("JCOUT_{i}_PCS"));
                        self.claim_pip(scan_coutp, cout_next);
                        self.claim_pip(bist_coutp, cout_next);
                    }
                }
                for i in 0..22 {
                    let cout = self.rc_wire(cell, &format!("JSERDES_COUT_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("SERDES_COUT_{i}"), cout);
                    let coutp = self.rc_wire(cell, &format!("JSERDES_COUTP_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("SERDES_COUTP_{i}"), coutp);
                    self.claim_pip(cout, coutp);
                    if let Some(cell_next) = cell_next {
                        let cout_next = self.rc_wire(cell_next, &format!("JSERDES_COUT_{i}_PCS"));
                        self.claim_pip(coutp, cout_next);
                    }
                }
                {
                    let testclk = self.rc_wire(cell, "JTESTCLK_PCS");
                    self.add_bel_wire(bcrd, "TESTCLK", testclk);
                    let testclk_maco = self.rc_wire(cell, "JTESTCLK_MACO_PCS");
                    self.add_bel_wire(bcrd, "TESTCLK_MACO", testclk_maco);
                    self.claim_pip(testclk, testclk_maco);
                    if let Some(cell_next) = cell_next {
                        let testclk_maco_next = self.rc_wire(cell_next, "JTESTCLK_MACO_PCS");
                        self.claim_pip(testclk_maco_next, testclk);
                    }
                }
                {
                    let tck_fmac = self.rc_wire(cell, "JTCK_FMAC_PCS");
                    self.add_bel_wire(bcrd, "TCK_FMAC", tck_fmac);
                    let tck_fmacp = self.rc_wire(cell, "JTCK_FMACP_PCS");
                    self.add_bel_wire(bcrd, "TCK_FMACP", tck_fmacp);
                    self.claim_pip(tck_fmac, tck_fmacp);
                    if let Some(cell_next) = cell_next {
                        let tck_fmac_next = self.rc_wire(cell_next, "JTCK_FMAC_PCS");
                        self.claim_pip(tck_fmacp, tck_fmac_next);
                    }
                }
                for i in 0..13 {
                    let cin = self.rc_wire(cell, &format!("JCIN_{i}_PCS"));
                    self.add_bel_wire(bcrd, format!("CIN_{i}"), cin);
                }
                for pin in ["RXREFCLKP", "RXREFCLKN"] {
                    let wire = self.rc_wire(cell, &format!("J{pin}_PCS"));
                    self.add_bel_wire(bcrd, pin, wire);
                }
            }
        }
    }

    fn process_serdes_ecp2(&mut self) {
        for tcid in [tcls::SERDES_S, tcls::SERDES_N] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::SERDES);
                let hv = DirHV {
                    h: bcrd.col.side_of(self.chip.col_clk),
                    v: bcrd.row.side_of(self.chip.row_clk),
                };
                let cell = bcrd.cell.delta(
                    match hv.h {
                        DirH::W => 12,
                        DirH::E => 13,
                    },
                    match hv.v {
                        DirV::S => -1,
                        DirV::N => 1,
                    },
                );
                let cell_apio = bcrd.cell.delta(
                    match hv.h {
                        DirH::W => 12,
                        DirH::E => 13,
                    },
                    match hv.v {
                        DirV::S => -7,
                        DirV::N => 7,
                    },
                );
                let corner = match hv {
                    DirHV::SW => "LL",
                    DirHV::SE => "LR",
                    DirHV::NW => "UL",
                    DirHV::NE => "UR",
                };
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.names([
                    format!("{corner}PCS"),
                    format!("{corner}C_SQ_REFCLKP"),
                    format!("{corner}C_SQ_REFCLKN"),
                    format!("{corner}C_SQ_HDINP0"),
                    format!("{corner}C_SQ_HDINN0"),
                    format!("{corner}C_SQ_HDINP1"),
                    format!("{corner}C_SQ_HDINN1"),
                    format!("{corner}C_SQ_HDINP2"),
                    format!("{corner}C_SQ_HDINN2"),
                    format!("{corner}C_SQ_HDINP3"),
                    format!("{corner}C_SQ_HDINN3"),
                    format!("{corner}C_SQ_HDOUTP0"),
                    format!("{corner}C_SQ_HDOUTN0"),
                    format!("{corner}C_SQ_HDOUTP1"),
                    format!("{corner}C_SQ_HDOUTN1"),
                    format!("{corner}C_SQ_HDOUTP2"),
                    format!("{corner}C_SQ_HDOUTN2"),
                    format!("{corner}C_SQ_HDOUTP3"),
                    format!("{corner}C_SQ_HDOUTN3"),
                ])
                .suffix("PCS");
                for (dx, name) in [
                    (-11, "IN3"),
                    (-10, "OUT3"),
                    (-6, "IN2"),
                    (-5, "OUT2"),
                    (0, "REFCLK"),
                    (5, "IN1"),
                    (6, "OUT1"),
                    (10, "IN0"),
                    (11, "OUT0"),
                ] {
                    for pin in [
                        "JINPUTA_APIO",
                        "JOUTPUTA_APIO",
                        "JCLOCKA_APIO",
                        "JINPUTB_APIO",
                        "JOUTPUTB_APIO",
                        "JCLOCKB_APIO",
                    ] {
                        let wire = bel.ctx.rc_io_wire(cell_apio.delta(dx, 0), pin);
                        bel.add_wire(format!("{name}_{pin}"), wire);
                    }
                }
                for (dx, wp, wn) in [
                    (-11, "HDINP3", "HDINN3"),
                    (-6, "HDINP2", "HDINN2"),
                    (0, "REFCLKP", "REFCLKN"),
                    (5, "HDINP1", "HDINN1"),
                    (10, "HDINP0", "HDINN0"),
                ] {
                    let wire_apio = bel.ctx.rc_io_wire(cell_apio.delta(dx, 0), "JINPUTA_APIO");
                    let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wp}_PCS"));
                    bel.ctx.claim_pip(wire_pcs, wire_apio);
                    bel.add_wire(wp, wire_pcs);
                    let wire_apio = bel.ctx.rc_io_wire(cell_apio.delta(dx, 0), "JINPUTB_APIO");
                    let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wn}_PCS"));
                    bel.ctx.claim_pip(wire_pcs, wire_apio);
                    bel.add_wire(wn, wire_pcs);
                }
                for (dx, wp, wn) in [
                    (-10, "HDOUTP3", "HDOUTN3"),
                    (-5, "HDOUTP2", "HDOUTN2"),
                    (6, "HDOUTP1", "HDOUTN1"),
                    (11, "HDOUTP0", "HDOUTN0"),
                ] {
                    let wire_apio = bel.ctx.rc_io_wire(cell_apio.delta(dx, 0), "JOUTPUTA_APIO");
                    let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wp}_PCS"));
                    bel.ctx.claim_pip(wire_apio, wire_pcs);
                    bel.add_wire(wp, wire_pcs);
                    let wire_apio = bel.ctx.rc_io_wire(cell_apio.delta(dx, 0), "JOUTPUTB_APIO");
                    let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wn}_PCS"));
                    bel.ctx.claim_pip(wire_apio, wire_pcs);
                    bel.add_wire(wn, wire_pcs);
                }
            }
        }
    }

    fn process_serdes_ecp3(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::SERDES] {
            let bcrd = tcrd.bel(bslots::SERDES);
            let bank = self.chip.columns[bcrd.cell.col].bank[DirV::S];
            let name = match bank {
                50 => "PCSA",
                51 => "PCSB",
                52 => "PCSC",
                53 => "PCSD",
                _ => unreachable!(),
            };
            let cell = bcrd.cell.delta(0, -1);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.names([
                name.to_string(),
                format!("{name}_REFCLKP"),
                format!("{name}_REFCLKN"),
                format!("{name}_HDINP0"),
                format!("{name}_HDINN0"),
                format!("{name}_HDINP1"),
                format!("{name}_HDINN1"),
                format!("{name}_HDINP2"),
                format!("{name}_HDINN2"),
                format!("{name}_HDINP3"),
                format!("{name}_HDINN3"),
                format!("{name}_HDOUTP0"),
                format!("{name}_HDOUTN0"),
                format!("{name}_HDOUTP1"),
                format!("{name}_HDOUTN1"),
                format!("{name}_HDOUTP2"),
                format!("{name}_HDOUTN2"),
                format!("{name}_HDOUTP3"),
                format!("{name}_HDOUTN3"),
            ])
            .suffix("PCS");
            for (dx, name) in [
                (0, "IN3"),
                (1, "OUT3"),
                (2, "OUT2"),
                (3, "IN2"),
                (4, "REFCLK"),
                (5, "IN1"),
                (6, "OUT1"),
                (7, "OUT0"),
                (8, "IN0"),
            ] {
                for pin in [
                    "JINPUTA_APIO",
                    "JOUTPUTA_APIO",
                    "JCLOCKA_APIO",
                    "JINPUTB_APIO",
                    "JOUTPUTB_APIO",
                    "JCLOCKB_APIO",
                ] {
                    let wire = bel.ctx.rc_io_wire(cell.delta(dx, -8), pin);
                    bel.add_wire(format!("{name}_{pin}"), wire);
                }
            }
            for (dx, wp, wn) in [
                (0, "HDINP3", "HDINN3"),
                (3, "HDINN2", "HDINP2"),
                (4, "REFCLKP", "REFCLKN"),
                (5, "HDINP1", "HDINN1"),
                (8, "HDINN0", "HDINP0"),
            ] {
                let wire_apio = bel.ctx.rc_io_wire(cell.delta(dx, -8), "JINPUTA_APIO");
                let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wp}_PCS"));
                bel.ctx.claim_pip(wire_pcs, wire_apio);
                bel.add_wire(wp, wire_pcs);
                let wire_apio = bel.ctx.rc_io_wire(cell.delta(dx, -8), "JINPUTB_APIO");
                let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wn}_PCS"));
                bel.ctx.claim_pip(wire_pcs, wire_apio);
                bel.add_wire(wn, wire_pcs);
            }
            for (dx, wp, wn) in [
                (1, "HDOUTP3", "HDOUTN3"),
                (2, "HDOUTN2", "HDOUTP2"),
                (6, "HDOUTP1", "HDOUTN1"),
                (7, "HDOUTN0", "HDOUTP0"),
            ] {
                let wire_apio = bel.ctx.rc_io_wire(cell.delta(dx, -8), "JOUTPUTA_APIO");
                let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wp}_PCS"));
                bel.ctx.claim_pip(wire_apio, wire_pcs);
                bel.add_wire(wp, wire_pcs);
                let wire_apio = bel.ctx.rc_io_wire(cell.delta(dx, -8), "JOUTPUTB_APIO");
                let wire_pcs = bel.ctx.rc_wire(cell, &format!("J{wn}_PCS"));
                bel.ctx.claim_pip(wire_apio, wire_pcs);
                bel.add_wire(wn, wire_pcs);
            }
            let wire = bel.ctx.rc_wire(cell, "JREFCLK_TO_NQ_PCS");
            bel.add_wire("REFCLK_TO_NQ_PCS", wire);
            let wire = bel.ctx.rc_wire(cell, "JREFCLK_FROM_NQ_PCS");
            bel.add_wire("REFCLK_FROM_NQ_PCS", wire);
            if let Some(cell_prev) = bel.ctx.edev.cell_delta(bcrd.cell, -36, 0)
                && bel.ctx.edev.has_bel(cell_prev.bel(bslots::SERDES))
            {
                let wire_prev = bel.ctx.rc_wire(cell.delta(-36, 0), "JREFCLK_TO_NQ_PCS");
                bel.ctx.claim_pip(wire, wire_prev);
            }
        }
    }

    fn process_serdes_ecp4(&mut self) {
        let (cell, num_quads) =
            if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::SerdesSingle) {
                (cell, 1)
            } else if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::SerdesDouble) {
                (cell, 2)
            } else if let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::SerdesTriple) {
                (cell, 3)
            } else {
                unreachable!()
            };
        let bcrd = cell.bel(bslots::SERDES);

        let mut names = vec![
            "ASB".to_string(),
            "REFCLKP_CORNER".to_string(),
            "REFCLKN_CORNER".to_string(),
        ];
        for quad in 0..num_quads {
            names.extend([format!("REFCLKP_Q{quad}"), format!("REFCLKN_Q{quad}")]);
            for ch in 0..4 {
                names.extend([
                    format!("HDRXP_Q{quad}CH{ch}"),
                    format!("HDRXN_Q{quad}CH{ch}"),
                    format!("HDTXP_Q{quad}CH{ch}"),
                    format!("HDTXN_Q{quad}CH{ch}"),
                ]);
            }
        }
        let cell = cell.with_col(self.chip.col_w());
        self.extract_bel_cell(bcrd, cell).names(names).suffix("ASB");

        for quad in 0..num_quads {
            let cell_apio = cell.delta(1 + quad, 0);
            if quad == 0 {
                for pn in ['P', 'N'] {
                    let wire_apio = self.rc_wire(cell_apio, &format!("JINPUT_CORNREF{pn}_APIO"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_INPUT_CORNREF{pn}_APIO"), wire_apio);
                    let wire = self.rc_io_sn_wire(cell, &format!("JCORNER_REFCLK{pn}_ASB"));
                    self.add_bel_wire(bcrd, format!("CORNER_REFCLK{pn}"), wire);
                    self.claim_pip(wire, wire_apio);
                    for pin in ["OUTPUT", "CLK"] {
                        let wire = self.rc_wire(cell_apio, &format!("{pin}_CORNREF{pn}_APIO"));
                        self.add_bel_wire(bcrd, format!("Q{quad}_{pin}_CORNREF{pn}_APIO"), wire);
                    }
                }
            }
            for pn in ['P', 'N'] {
                let wire_apio = self.rc_wire(cell_apio, &format!("JINPUT_REF{pn}_APIO"));
                self.add_bel_wire(bcrd, format!("Q{quad}_INPUT_REF{pn}_APIO"), wire_apio);
                let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}_REFCLK{pn}_ASB"));
                self.add_bel_wire(bcrd, format!("Q{quad}_REFCLK{pn}"), wire);
                self.claim_pip(wire, wire_apio);
                for pin in ["OUTPUT", "CLK"] {
                    let wire = self.rc_wire(cell_apio, &format!("{pin}_REF{pn}_APIO"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_{pin}_REF{pn}_APIO"), wire);
                }
            }
            for ch in 0..4 {
                for pn in ['P', 'N'] {
                    let wire_apio = self.rc_wire(cell_apio, &format!("JOUTPUT_O{pn}{ch}_APIO"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_OUTPUT_O{pn}{ch}_APIO"), wire_apio);
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_HDOUT{pn}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_HDOUT{pn}"), wire);
                    self.claim_pip(wire_apio, wire);
                    for pin in ["INPUT", "CLK"] {
                        let wire = self.rc_wire(cell_apio, &format!("{pin}_O{pn}{ch}_APIO"));
                        self.add_bel_wire(bcrd, format!("Q{quad}_{pin}_O{pn}{ch}_APIO"), wire);
                    }
                }
                for pn in ['P', 'N'] {
                    let wire_apio = self.rc_wire(cell_apio, &format!("JINPUT_I{pn}{ch}_APIO"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_INPUT_I{pn}{ch}_APIO"), wire_apio);
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_HDIN{pn}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_HDIN{pn}"), wire);
                    self.claim_pip(wire, wire_apio);
                    for pin in ["OUTPUT", "CLK"] {
                        let wire = self.rc_wire(cell_apio, &format!("{pin}_I{pn}{ch}_APIO"));
                        self.add_bel_wire(bcrd, format!("Q{quad}_{pin}_I{pn}{ch}_APIO"), wire);
                    }
                }
            }
        }

        for quad in num_quads..4 {
            for i in 0..6 {
                for j in 0..8 {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}_FCDFECOEFF{i}_{j}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_FCDFECOEFF{i}_{j}"), wire);
                }
            }
            for pin in [
                "FCDFESIGN1",
                "FCDFESIGN2",
                "FCDFESIGN3",
                "FCDFESIGN4",
                "FCDFESIGN5",
                "FCMPWRUP",
                "FCMRST",
                "FCSCANMODE",
                "FIRXTESTCLK",
                "FISYNCCLK",
                "FITMRCLK",
                "FITXTESTCLK",
                "FOREFCLK2FPGA",
                "HSPLLLOL",
                "HSPLLPWRUP",
                "HSPLLREFCLKI",
                "HSPLLRST",
                "LSPLLLOL",
                "LSPLLPWRUP",
                "LSPLLREFCLKI",
                "LSPLLRST",
            ] {
                let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}_{pin}_ASB"));
                self.add_bel_wire(bcrd, format!("Q{quad}_{pin}"), wire);
            }
            for (pin, width) in [
                ("FDDFECHSEL", 2),
                ("FDDFEDATA", 10),
                ("FDDFEERR", 10),
                ("FIGRPFBRRCLK", 2),
                ("FIGRPFBTWCLK", 2),
            ] {
                for j in 0..width {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}_{pin}{j}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}_{pin}{j}"), wire);
                }
            }
            for ch in 0..4 {
                for pin in [
                    "FOPCLKA",
                    "FOPCLKB",
                    "FCALIGNEN",
                    "FCCDRFORCEDLOCK",
                    "FCDFERDEN",
                    "FCDFEUPD",
                    "FCLDRTXEN",
                    "FCLSMEN",
                    "FCPCIEDETEN",
                    "FCPCSRXRST",
                    "FCPCSTXRST",
                    "FCPIPEPHYRESETN",
                    "FCPLLLOL",
                    "FCRATE0",
                    "FCRATE1",
                    "FCRATE2",
                    "FCRRST",
                    "FCRXPOLARITY",
                    "FCRXPWRUP",
                    "FCTMRSTART",
                    "FCTMRSTOP",
                    "FCTRST",
                    "FCTXMARGIN0",
                    "FCTXMARGIN1",
                    "FCTXMARGIN2",
                    "FCTXPWRUP",
                    "FCWORDALGNEN",
                    "FDLDRRX",
                    "FDLDRTX",
                    "FIRCLK",
                    "FIREFRXCLK",
                    "FITCLK",
                    "FITMRSTARTCLK",
                    "FITMRSTOPCLK",
                    "FSCCOVERRUN",
                    "FSCCUNDERRUN",
                    "FSDFEVLD",
                    "FSLSM",
                    "FSPCIECON",
                    "FSPCIEDONE",
                    "FSRCDONE",
                    "FSRLOL",
                    "FSRLOS",
                    "FSSKPADDED",
                    "FSSKPDELETED",
                ] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_{pin}"), wire);
                }
                for i in 0..48 {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_FDRX{i}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_FDRX{i}"), wire);
                }
                for i in 0..50 {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_FDTX{i}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_FDTX{i}"), wire);
                }
            }
            for i in 0..2 {
                for pin in ["FCDERST", "FSDE", "FSDM"] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}D{i}_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}D{i}_{pin}"), wire);
                }
            }
            for ch in 0..4 {
                for pin in [
                    "CIRXFULL",
                    "CIRXIGNOREPKT",
                    "CITXDATAAVAIL",
                    "CITXEMPTY",
                    "CITXEOF",
                    "CITXFIFOCTRL",
                    "CITXFORCEERR",
                    "CITXLASTBYTEVLD",
                    "CITXPAUSREQ",
                    "CORXEOF",
                    "CORXERROR",
                    "CORXFIFOFULLERROR",
                    "CORXLASTBYTEVLD",
                    "CORXSTATEN",
                    "CORXWRITE",
                    "COTXDISCFRM",
                    "COTXDONE",
                    "COTXREAD",
                    "COTXSTATEN",
                    "GIIPGSHRINK",
                    "GINONPADRXDV",
                    "GISYNCCOL",
                    "GISYNCCRS",
                    "GISYNCNIBDRIB",
                    "GISYNCRXDV",
                    "GISYNCRXER",
                    "GODISCARDFCS",
                    "GOTXMACERR",
                    "GOTXMACWR",
                    "KIRSTN",
                    "KIRXMACCLK",
                    "KIRXMACCLKENEXT",
                    "KIRXTXFECLK",
                    "KITXGMIILPBK",
                    "KITXMACCLK",
                    "KITXMACCLKENEXT",
                    "KOGBITEN",
                    "KORXMACCLKEN",
                ] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}EA{ch}_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}EA{ch}_{pin}"), wire);
                }
                for (pin, width) in [
                    ("CITXDATA", 16),
                    ("CITXPAUSTIM", 16),
                    ("CORXDATA", 16),
                    ("CORXSTATVEC", 8),
                    ("COTXSTATVEC", 8),
                    ("GISYNCRXD", 8),
                    ("GOTXMACDATA", 8),
                ] {
                    for j in 0..width {
                        let wire =
                            self.rc_io_sn_wire(cell, &format!("JQ{quad}EA{ch}_{pin}{j}_ASB"));
                        self.add_bel_wire(bcrd, format!("Q{quad}EA{ch}_{pin}{j}"), wire);
                    }
                }
            }
            if quad == 1 {
                for pin in [
                    "BUFFDEQACKADVPTRN",
                    "ENABLETXFLOWCONTROLN",
                    "GEAR",
                    "LNKCLK",
                    "LNKCLKDIV2",
                    "LNKCLKDIV2RSTN",
                    "LNKCLKRSTN",
                    "LNKDIV2RSTN",
                    "LNKMCERXACKN",
                    "LNKMCERXREQN",
                    "LNKMCETXACKN",
                    "LNKMCETXREQN",
                    "LNKRSTN",
                    "LNKTOUTPUTPORTENABLE",
                    "MASTERENABLE",
                    "MGTCLK",
                    "MGTCLKRSTN",
                    "MGTRDN",
                    "MGTRSTN",
                    "OLLMMGTINTN",
                    "OLLMMGTRDYN",
                    "OUTPUTUNRECOVERREVENTACKN",
                    "OUTPUTUNRECOVERREVENTREQN",
                    "PHYCLK",
                    "PHYEMEVENTACKN",
                    "PHYEMEVENTREQN",
                    "PHYMSTB",
                    "PHYRINITN",
                    "PHYRSTN",
                    "PHYTINITN",
                    "PHYUSTB",
                    "PORTDISABLE",
                    "RECOVERRSTN",
                    "RIOCLK",
                    "RIORSTN",
                    "RLNKDSTDSCN",
                    "RLNKDSTRDYN",
                    "RLNKEOFN",
                    "RLNKSOFN",
                    "RLNKSRCRDYN",
                    "RXINITN",
                    "RXPFORCERETRYN",
                    "SOFTRSTN",
                    "SYSRSTIN",
                    "SYSRSTON",
                    "TIMBISTMODE",
                    "TIMBISTRE",
                    "TIMBISTWE",
                    "TISCANCLK",
                    "TISCANENA",
                    "TISCANMODE",
                    "TISCANRSTN",
                    "TLNKDSTRDYN",
                    "TLNKEOFN",
                    "TLNKREM0",
                    "TLNKREM1",
                    "TLNKREM2",
                    "TLNKSOFN",
                    "TLNKSRCDSCN",
                    "TLNKSRCRDYN",
                    "TPORTENABLEN",
                    "TXINITN",
                ] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}S_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}S_{pin}"), wire);
                }
                for (pin, width) in [
                    ("BAUDSUPPORT", 5),
                    ("DECRBUFCNTVECTOR", 10),
                    ("FLOWFIFOACKON", 4),
                    ("LOOPBACK", 3),
                    ("MGTA", 22),
                    ("MGTDI", 32),
                    ("MGTWRN", 4),
                    ("OLLMEFPTR", 16),
                    ("OLLMMGTDI", 32),
                    ("PORTNERRORDETECT", 32),
                    ("RCVCLKSEL", 4),
                    ("RESPTIMEOUT", 24),
                    ("RLNKBEATS", 8),
                    ("RLNKD", 64),
                    ("RLNKREM", 3),
                    ("RXSYNCCTRL", 2),
                    ("STATUS", 22),
                    ("TIMBISTBANKSEL", 4),
                    ("TIMBISTRA", 8),
                    ("TIMBISTSDI", 36),
                    ("TIMBISTWA", 8),
                    ("TISCANI", 30),
                    ("TISCANO", 30),
                    ("TLNKD", 64),
                    ("TOMBISTDO", 36),
                    ("TPORTCHARISK", 8),
                    ("TPORTTDI", 64),
                    ("TXENQUEUEFLOWN", 4),
                    ("TXFLOWCTRLSTATE", 5),
                    ("TXLANESILENCECH", 4),
                    ("TXSYNCCTRL", 2),
                    ("WM0", 4),
                    ("WM1", 4),
                    ("WM2", 4),
                ] {
                    for j in 0..width {
                        let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}S_{pin}{j}_ASB"));
                        self.add_bel_wire(bcrd, format!("Q{quad}S_{pin}{j}"), wire);
                    }
                }
            }
            if quad >= 2 {
                for ch in 0..4 {
                    for pin in [
                        "CIRXFULL",
                        "CIRXIGNOREPKT",
                        "CITXDATAAVAIL",
                        "CITXEMPTY",
                        "CITXEOF",
                        "CITXFIFOCTRL",
                        "CITXFORCEERR",
                        "CITXPAUSEREQ",
                        "GIIPGSHRINK",
                        "GINONPADRXDV",
                        "GISYNCCOL",
                        "GISYNCCRS",
                        "GISYNCNIBDRIB",
                        "GISYNCRXDV",
                        "GISYNCRXER",
                        "KIRSTN",
                        "KIRXMACCLK",
                        "KIRXMACCLKENEXT",
                        "KIRXTXFECLK",
                        "KITXGMIILPBK",
                        "KITXMACCLK",
                        "KITXMACCLKENEXT",
                    ] {
                        let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}EB{ch}_{pin}_ASB"));
                        self.add_bel_wire(bcrd, format!("Q{quad}EB{ch}_{pin}"), wire);
                    }
                    for (pin, width) in [("CITXDATA", 8), ("CITXPAUSTIM", 16), ("GISYNCRXD", 8)] {
                        for j in 0..width {
                            let wire =
                                self.rc_io_sn_wire(cell, &format!("JQ{quad}EB{ch}_{pin}{j}_ASB"));
                            self.add_bel_wire(bcrd, format!("Q{quad}EB{ch}_{pin}{j}"), wire);
                        }
                    }
                }
                for pin in [
                    "IIGNOREPKT",
                    "IRESETN",
                    "IRXMACCLK",
                    "ITSMSGAVAIL",
                    "ITXDATAAVAIL",
                    "ITXEMPTY",
                    "ITXEOF",
                    "ITXFORCEERR",
                    "ITXMACCLK",
                    "ITXPAUSREQ",
                    "TIBISTREN",
                    "TIBISTTESTMODE",
                    "TIBISTWEN",
                    "TISCANEN",
                    "TISCANMODE",
                    "TISCANRSTN",
                ] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}X_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}X_{pin}"), wire);
                }
                for (pin, width) in [
                    ("CSO", 141),
                    ("GSO", 44),
                    ("ITXBYTEN", 3),
                    ("ITXDATA", 64),
                    ("ITXMSG", 64),
                    ("ITXPAUSTIM", 16),
                    ("KSO", 20),
                    ("TIBISTBANKSEL", 4),
                    ("TIBISTRA", 8),
                    ("TIBISTTDI", 36),
                    ("TIBISTWA", 8),
                    ("TISCANIN", 30),
                    ("TOSCANOUT", 29),
                ] {
                    for j in 0..width {
                        let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}X_{pin}{j}_ASB"));
                        self.add_bel_wire(bcrd, format!("Q{quad}X_{pin}{j}"), wire);
                    }
                }
            }

            for pn in ['P', 'N'] {
                let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}_REFCLK{pn}_ASB"));
                self.add_bel_wire(bcrd, format!("Q{quad}_REFCLK{pn}"), wire);
            }
            for ch in 0..4 {
                for pn in ['P', 'N'] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_HDOUT{pn}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_HDOUT{pn}"), wire);
                }
                for pn in ['P', 'N'] {
                    let wire = self.rc_io_sn_wire(cell, &format!("JQ{quad}CH{ch}_HDIN{pn}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_HDIN{pn}"), wire);
                }
            }
        }
        for quad in 0..4 {
            for ch in 0..4 {
                for pin in ["TXBITCLK", "FORECCLK"] {
                    let wire = self.rc_io_sn_wire(cell, &format!("Q{quad}CH{ch}_{pin}_ASB"));
                    self.add_bel_wire(bcrd, format!("Q{quad}CH{ch}_{pin}"), wire);
                }
            }
            for pin in ["REFCLKO", "HSPLLCLKO", "LSPLLCLKO", "FCDFESIGN0"] {
                let wire = self.rc_io_sn_wire(cell, &format!("Q{quad}_{pin}_ASB"));
                self.add_bel_wire(bcrd, format!("Q{quad}_{pin}"), wire);
            }
        }
        for pin in [
            "Q0P_SCANO25",
            "Q0P_SCANO26",
            "Q2X_TOSCANOUT29",
            "Q3X_TOSCANOUT29",
            "SCIINT",
            "SCIRTYO",
            "SCIACKO",
        ] {
            let wire = self.rc_io_sn_wire(cell, &format!("J{pin}_ASB"));
            self.add_bel_wire(bcrd, pin, wire);
        }
        for pin in ["CORNER_REFCLKO", "CORNER_LSPLLCLKO"] {
            let wire = self.rc_io_sn_wire(cell, &format!("{pin}_ASB"));
            self.add_bel_wire(bcrd, pin, wire);
        }
        for i in 0..8 {
            let wire = self.rc_io_sn_wire(cell, &format!("JSCIDATO{i}_ASB"));
            self.add_bel_wire(bcrd, format!("SCIDATO{i}"), wire);
        }

        let cell_efb = self.chip.special_loc[&SpecialLocKey::Config];
        for (pin, pin_efb, range) in [
            ("SCISTBI", "SCISTBO", 0..64),
            ("SCIDATI", "SCIDATO", 0..8),
            ("SCIADRI", "SCIADRO", 0..12),
        ] {
            for i in range {
                let wire = self.rc_io_sn_wire(cell, &format!("J{pin}{i}_ASB"));
                self.add_bel_wire(bcrd, format!("{pin}{i}"), wire);
                let wire_efb = self.rc_io_wire(cell_efb, &format!("J{pin_efb}{i}_EFB"));
                self.claim_pip(wire, wire_efb);
            }
        }
        for (pin, pin_efb) in [
            ("SCIWEI", "SCIWEO"),
            ("SCICYCI", "SCICYCO"),
            ("SCICLKI", "SCICLKO"),
            ("SCISLEEP", "SCISLEEP"),
            ("SCIINITEN0", "SCIINITEN0"),
            ("SCIINITEN1", "SCIINITEN1"),
            ("SCIRSTN", "SCIRSTN"),
            ("SCIRSTI", "SCIRSTO"),
        ] {
            let wire = self.rc_io_sn_wire(cell, &format!("J{pin}_ASB"));
            self.add_bel_wire(bcrd, pin, wire);
            let wire_efb = self.rc_io_wire(cell_efb, &format!("J{pin_efb}_EFB"));
            self.claim_pip(wire, wire_efb);
        }
    }

    fn process_serdes_ecp5(&mut self) {
        if self.skip_serdes {
            return;
        }
        for &tcrd in &self.edev.tile_index[tcls::SERDES] {
            let bcrd = tcrd.bel(bslots::SERDES);
            let cell = bcrd.cell;
            let bank = self.chip.columns[bcrd.cell.col].bank[DirV::S];
            let dual = match bank {
                50 => 0,
                51 => 1,
                _ => unreachable!(),
            };
            let cell_prev = if dual != 0 {
                let mut cell_prev = cell.delta(-1, 0);
                while self.chip.columns[cell_prev.col].io[DirV::S] != IoGroupKind::Serdes {
                    cell_prev.col -= 1;
                }
                Some(cell_prev)
            } else {
                None
            };

            let mut bel = self.extract_bel(bcrd);
            bel.names([
                format!("DCU{dual}"),
                format!("EXTREF{dual}"),
                format!("REFCLKP_D{dual}"),
                format!("REFCLKN_D{dual}"),
                format!("HDRXP0_D{dual}CH0"),
                format!("HDRXN0_D{dual}CH0"),
                format!("HDRXP0_D{dual}CH1"),
                format!("HDRXN0_D{dual}CH1"),
                format!("HDTXP0_D{dual}CH0"),
                format!("HDTXN0_D{dual}CH0"),
                format!("HDTXP0_D{dual}CH1"),
                format!("HDTXN0_D{dual}CH1"),
            ])
            .suffix("DCU");

            for (name, pin) in [
                ("IP0", "CH0_HDINP"),
                ("IN0", "CH0_HDINN"),
                ("IP1", "CH1_HDINP"),
                ("IN1", "CH1_HDINN"),
            ] {
                bel.add_wire_rc_io(format!("INPUT_{name}_APIO"), format!("JINPUT_{name}_APIO"))
                    .add_wire_rc_io(pin, format!("J{pin}_DCU"))
                    .claim_pip(pin, format!("INPUT_{name}_APIO"));
                for apin in ["CLK", "OUTPUT"] {
                    bel.add_wire_rc_io(
                        format!("{apin}_{name}_APIO"),
                        format!("{apin}_{name}_APIO"),
                    );
                }
            }
            for (name, pin) in [
                ("OP0", "CH0_HDOUTP"),
                ("ON0", "CH0_HDOUTN"),
                ("OP1", "CH1_HDOUTP"),
                ("ON1", "CH1_HDOUTN"),
            ] {
                bel.add_wire_rc_io(
                    format!("OUTPUT_{name}_APIO"),
                    format!("JOUTPUT_{name}_APIO"),
                )
                .add_wire_rc_io(pin, format!("J{pin}_DCU"))
                .claim_pip(format!("OUTPUT_{name}_APIO"), pin);
                for apin in ["CLK", "INPUT"] {
                    bel.add_wire_rc_io(
                        format!("{apin}_{name}_APIO"),
                        format!("{apin}_{name}_APIO"),
                    );
                }
            }

            for (name, pin) in [("REFP", "REFCLKP"), ("REFN", "REFCLKN")] {
                bel.add_wire_rc_io(format!("INPUT_{name}_APIO"), format!("INPUT_{name}_APIO"))
                    .add_wire_rc_io(pin, format!("{pin}_EXTREF"))
                    .claim_pip(pin, format!("INPUT_{name}_APIO"));
                for apin in ["CLK", "OUTPUT"] {
                    bel.add_wire_rc_io(
                        format!("{apin}_{name}_APIO"),
                        format!("{apin}_{name}_APIO"),
                    );
                }
            }

            bel.output_xlat_rc_io(bcls::SERDES::REFCLKO, "JREFCLKO_EXTREF")
                .add_wire_rc_io("REFCLKO_OUT", "EXTREFCLK")
                .claim_pip("REFCLKO_OUT", "REFCLKO")
                .add_wire_rc_io("KEEPWIRE", "KEEPWIRE");

            if let Some(cell_prev) = cell_prev {
                let wire_prev = bel.ctx.rc_io_wire(cell_prev, "JTXREFCLK");
                bel.add_wire_rc_io("REFCLK_PREV", "JREFCLKFROMND")
                    .claim_pip_from("REFCLK_PREV", wire_prev);
            }

            for ch in 0..2 {
                bel.add_wire_rc_io(format!("CH{ch}_RX_REFCLK"), format!("CH{ch}_RX_REFCLK_DCU"))
                    .add_wire_rc_io(format!("CH{ch}_RX_REFCLK_IN"), format!("CH{ch}_RX_REFCLK"))
                    .claim_pip(format!("CH{ch}_RX_REFCLK"), format!("CH{ch}_RX_REFCLK_IN"))
                    .input_xlat_rename_rc_io(
                        [bcls::SERDES::CH0_RX_REFCLK, bcls::SERDES::CH1_RX_REFCLK][ch],
                        format!("CH{ch}_RX_REFCLK_INT"),
                        format!("JCH{ch}RXREFCLKCIB"),
                    )
                    .claim_pip(
                        format!("CH{ch}_RX_REFCLK_IN"),
                        format!("CH{ch}_RX_REFCLK_INT"),
                    )
                    .add_wire_rc_io(format!("CH{ch}_RX_REFCLK_MUX"), format!("RXREFCLK{ch}"))
                    .claim_pip(
                        format!("CH{ch}_RX_REFCLK_IN"),
                        format!("CH{ch}_RX_REFCLK_MUX"),
                    )
                    .claim_pip(format!("CH{ch}_RX_REFCLK_MUX"), "REFCLKO_OUT")
                    .claim_pip(format!("CH{ch}_RX_REFCLK_MUX"), "KEEPWIRE");
                if cell_prev.is_some() {
                    bel.claim_pip(format!("CH{ch}_RX_REFCLK_MUX"), "REFCLK_PREV");
                }
            }

            {
                bel.add_wire_rc_io("D_REFCLKI", "D_REFCLKI_DCU")
                    .add_wire_rc_io("D_REFCLKI_IN", "D_REFCLKI")
                    .claim_pip("D_REFCLKI", "D_REFCLKI_IN")
                    .input_xlat_rename_rc_io(
                        bcls::SERDES::D_REFCLKI,
                        "D_REFCLKI_INT",
                        "JTXREFCLKCIB",
                    )
                    .claim_pip("D_REFCLKI_IN", "D_REFCLKI_INT")
                    .add_wire_rc_io("D_REFCLKI_MUX", "JTXREFCLK")
                    .claim_pip("D_REFCLKI_IN", "D_REFCLKI_MUX")
                    .claim_pip("D_REFCLKI_MUX", "REFCLKO_OUT")
                    .claim_pip("D_REFCLKI_MUX", "KEEPWIRE");
                if cell_prev.is_some() {
                    bel.claim_pip("D_REFCLKI_MUX", "REFCLK_PREV");
                }
            }

            for (pin_to, pin_from) in [
                ("D_SYNC_ND", "D_SYNC_PULSE2ND"),
                ("D_TXPLL_LOL_FROM_ND", "D_TXPLL_LOL_TO_ND"),
                ("D_TXBIT_CLKP_FROM_ND", "D_TXBIT_CLKP_TO_ND"),
                ("D_TXBIT_CLKN_FROM_ND", "D_TXBIT_CLKN_TO_ND"),
            ] {
                bel.add_wire_rc_io(pin_from, format!("J{pin_from}_DCU"))
                    .add_wire_rc_io(pin_to, format!("J{pin_to}_DCU"));
                if let Some(cell_prev) = cell_prev {
                    let wire_prev = bel.ctx.rc_io_wire(cell_prev, &format!("J{pin_from}_DCU"));
                    bel.claim_pip_from(pin_to, wire_prev);
                }
            }
        }
    }

    fn process_mipi_crosslink(&mut self) {
        for tcid in [tcls::MIPI_W, tcls::MIPI_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::MIPI[0]);
                let bank = self.chip.columns[bcrd.cell.col].bank[DirV::N];
                let idx = match bank {
                    60 => 0,
                    61 => 1,
                    _ => unreachable!(),
                };

                self.hybrid_out_rc_io(
                    bcrd,
                    "JCLKHSBYTE_MIPIDPHY",
                    wires::OUT_MIPI_CLKHSBYTE[0].cell(0),
                );

                self.extract_switchbox(bcrd.bel(bslots::BEL_INT))
                    .output_bind_rc_io("CLKREF", wires::IMUX_MIPI_CLKREF[0].cell(0))
                    .input_xlat_rc_io("JREFCLK1")
                    .input_xlat_rc_io("JREFCLK2")
                    .input_xlat_rc_io("JREFCLK3");

                let mut bel = self.extract_bel(bcrd);
                bel.names([
                    format!("MIPIDPHY{idx}"),
                    format!("CLKTEST_MIPIDPHY{idx}"),
                    format!("DPHY{idx}_CKP"),
                    format!("DPHY{idx}_CKN"),
                    format!("DPHY{idx}_DP0"),
                    format!("DPHY{idx}_DN0"),
                    format!("DPHY{idx}_DP1"),
                    format!("DPHY{idx}_DN1"),
                    format!("DPHY{idx}_DP2"),
                    format!("DPHY{idx}_DN2"),
                    format!("DPHY{idx}_DP3"),
                    format!("DPHY{idx}_DN3"),
                ])
                .suffix("MIPIDPHY")
                .input_xlat_rc_io(bcls::MIPI::CLKREF, "CLKREF_MIPIDPHY")
                .output_bind_rc_io(
                    bcls::MIPI::HSBYTECLKD,
                    "JHSBYTECLKD_MIPIDPHY",
                    wires::OUT_MIPI_HSBYTECLKD.cell(0),
                )
                .output_bind_rc_io(
                    bcls::MIPI::HSBYTECLKS,
                    "JHSBYTECLKS_MIPIDPHY",
                    wires::OUT_MIPI_HSBYTECLKS.cell(0),
                );

                for pin in [
                    "DP0", "DN0", "DP1", "DN1", "DP2", "DN2", "DP3", "DN3", "CKP", "CKN",
                ] {
                    bel.add_wire_rc_io(format!("PAD{pin}"), format!("PAD{pin}_ABPIO"))
                        .add_wire_rc_io(pin, format!("{pin}_MIPIDPHY"))
                        .claim_pip(format!("PAD{pin}"), pin);
                }

                for i in 0..2 {
                    bel.input_xlat_rc_io(
                        bcls::MIPI::CLKTEST_TESTIN[i],
                        format!("JTESTIN{i}_CLKTEST"),
                    );
                }
                for i in 2..6 {
                    bel.add_wire_rc_io(format!("CLKTEST_TESTIN{i}"), format!("TESTIN{i}_CLKTEST"));
                }
            }
        }
    }

    pub fn process_pcie_x1_nexus(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PCIE_JE5D30] {
            let (_r, c) = self.rc(tcrd.cell);
            let mut bel = self.extract_bel(tcrd.bel(bslots::PCIE_X1));
            bel.names([
                format!("TPCIE_CORE{c}"),
                "SD0_TXDP".to_string(),
                "SD0_TXDN".to_string(),
                "SD0_RXDP".to_string(),
                "SD0_RXDN".to_string(),
                "SD0_REFCLKP".to_string(),
                "SD0_REFCLKN".to_string(),
                "SD0_REXT".to_string(),
                "SD0_REFRET".to_string(),
            ])
            .suffix("PCIE_CORE_PCIE0")
            .output_bind_rc_io(
                bcls::PCIE_X1::ECKINDO,
                "JECKINDO_PCIE_CORE_PCIE0",
                wires::OUT_PCIE_LINKCLK.cell(0),
            )
            .output_bind_rc_io(
                bcls::PCIE_X1::CKUSRO,
                "JCKUSRO_PCIE_CORE_PCIE0",
                wires::OUT_PCIE_USRCLK[0].cell(0),
            )
            .output_bind_rc_io(
                bcls::PCIE_X1::ETXCKDO,
                "JETXCKDO_PCIE_CORE_PCIE0",
                wires::OUT_PCS_CLKTX[0].cell(0),
            )
            .output_bind_rc_io(
                bcls::PCIE_X1::ERXCKDO,
                "JERXCKDO_PCIE_CORE_PCIE0",
                wires::OUT_PCS_CLKRX[0].cell(0),
            );
            for (pin, pad) in [
                ("S0RXP", "SD0_RXP"),
                ("S0RXN", "SD0_RXN"),
                ("S0TXP", "SD0_TXP"),
                ("S0TXN", "SD0_TXN"),
                ("S0REXT", "SD0_REXT"),
                ("S0REFRET", "SD0_REFRET"),
                ("S0REFCKP", "SD0_REFCLKP"),
                ("S0REFCKN", "SD0_REFCLKN"),
            ] {
                let pad_apio = &format!("{pad}_APIO");
                bel.add_wire_rc_io(pin, format!("J{pin}_PCIE_CORE_PCIE0"))
                    .add_wire_rc_io(pad_apio, format!("JPAD_APIO_{pad}"))
                    .claim_pip(pin, pad_apio)
                    .claim_pip(pad_apio, pin);
            }
        }
    }

    pub fn process_pcs_nexus(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PCS_REFCLK] {
            let (r, c) = self.rc(tcrd.cell);
            let mut bel = self.extract_bel(tcrd.bel(bslots::PCS_REFCLK));
            bel.names([
                format!("DIFFCLKIO_CORE_R{r}C{c}"),
                "SD_EXT0_REFCLKP".to_string(),
                "SD_EXT0_REFCLKN".to_string(),
                "SD_EXT1_REFCLKP".to_string(),
                "SD_EXT1_REFCLKN".to_string(),
            ]);
            for i in 0..2 {
                bel.add_wire_rc_io(
                    format!("CLKOUT{i}"),
                    format!("JCLKOUT{i}_DIFFCLKIO_CORE_DIFFCLKIO0"),
                );
                for pn in ["P", "N"] {
                    bel.add_wire_rc_io(
                        format!("REFCLK{i}{pn}"),
                        format!("JREFCLK{i}{pn}_DIFFCLKIO_CORE_DIFFCLKIO0"),
                    )
                    .add_wire_rc_io(
                        format!("REFCLK{i}{pn}_APIO"),
                        format!("JPAD_APIO_REFCLK{i}{pn}"),
                    )
                    .claim_pip(format!("REFCLK{i}{pn}"), format!("REFCLK{i}{pn}_APIO"))
                    .claim_pip(format!("REFCLK{i}{pn}_APIO"), format!("REFCLK{i}{pn}"));
                }
            }
        }
        for &tcrd in &self.edev.tile_index[tcls::PCS_REFMUX_JD5D] {
            let cell_refclk = self.chip.special_loc[&SpecialLocKey::PcsRefclk];
            let mut bel = self.extract_bel(tcrd.bel(bslots::PCS_REFMUX));
            bel.name_rc("PCSREFMUX8_CORE").suffix("PCSREFMUX8_CORE");
            for i in 0..2 {
                let wire_diffclk = bel.ctx.rc_io_wire(
                    cell_refclk,
                    &format!("JCLKOUT{i}_DIFFCLKIO_CORE_DIFFCLKIO0"),
                );
                bel.add_wire_rc_io(
                    format!("Q{i}CLKOUT"),
                    format!("JQ{i}CLKOUT_PCSREFMUX8_CORE"),
                )
                .add_wire_rc_io(
                    format!("DIFFIOCLK{i}"),
                    format!("JDIFFIOCLK{i}_PCSREFMUX8_CORE"),
                )
                .claim_pip_from(format!("DIFFIOCLK{i}"), wire_diffclk);
            }
        }
        for i in 0..2 {
            let Some(&cell) = self.chip.special_loc.get(&SpecialLocKey::Pcs(i)) else {
                continue;
            };
            let cell_refmux = self.chip.special_loc[&SpecialLocKey::PcsRefmux];
            let cell_pcs0 = self.chip.special_loc[&SpecialLocKey::Pcs(0)];
            let cell_pcs_prev = if i == 0 {
                None
            } else {
                Some(self.chip.special_loc[&SpecialLocKey::Pcs(0)])
            };
            let (r, c) = self.rc(cell);
            let mut names = vec![format!("PCS_CORE_R{r}C{c}")];
            for j in 0..4 {
                let ii = i * 4 + j;
                for pad in ["TXDP", "TXDN", "RXDP", "RXDN", "REXT", "REFRET"] {
                    names.push(format!("SD{ii}_{pad}"));
                }
            }
            for pad in ["REFCLKP", "REFCLKN"] {
                names.push(format!("SDQ{i}_{pad}"));
            }
            let mut bel = self.extract_bel(cell.bel(bslots::PCS));
            bel.names(names).suffix("PCS_CORE_PCS0");
            for j in 0..4 {
                bel.output_bind_rc_io(
                    [
                        bcls::PCS::CH0_PIPE_PCS_TXCLKOUT,
                        bcls::PCS::CH1_PIPE_PCS_TXCLKOUT,
                        bcls::PCS::CH2_PIPE_PCS_TXCLKOUT,
                        bcls::PCS::CH3_PIPE_PCS_TXCLKOUT,
                    ][j],
                    format!("JCH{j}_PIPE_PCS_TXCLKOUT_PCS_CORE_PCS0"),
                    wires::OUT_PCS_CLKTX[j].cell(0),
                )
                .output_bind_rc_io(
                    [
                        bcls::PCS::CH0_RXOUTCLK,
                        bcls::PCS::CH1_RXOUTCLK,
                        bcls::PCS::CH2_RXOUTCLK,
                        bcls::PCS::CH3_RXOUTCLK,
                    ][j],
                    format!("JCH{j}_RXOUTCLK_PCS_CORE_PCS0"),
                    wires::OUT_PCS_CLKRX[j].cell(0),
                )
                .add_wire_rc_io(
                    format!("CH{j}_PIPE_L1PMSSEN_LL"),
                    format!("JCH{j}_PIPE_L1PMSSEN_LL_PCS_CORE_PCS0"),
                )
                .add_wire_rc_io(
                    format!("PIPE_PCLKOUT_LL{j}"),
                    format!("JPIPE_PCLKOUT_LL{j}_PCS_CORE_PCS0"),
                );
                for pad in ["RXP", "RXN", "TXP", "TXN", "REXT", "REFRET"] {
                    let pin_pcs = &format!("CH{j}_{pad}");
                    let pin_apio = &format!("CH{j}_{pad}_APIO");
                    bel.add_wire_rc_io(pin_pcs, format!("JCH{j}_SD{pad}_PCS_CORE_PCS0"))
                        .add_wire_rc_io(pin_apio, format!("JPAD_APIO_SD{j}_{pad}"))
                        .claim_pip(pin_pcs, pin_apio)
                        .claim_pip(pin_apio, pin_pcs);
                }
                for &(pin, _, num) in NEXUS_PCS_FROM_PCIE_CH {
                    if num == 1 {
                        bel.add_wire_rc_io(
                            format!("CH{j}_{pin}"),
                            format!("JCH{j}_{pin}_PCS_CORE_PCS0"),
                        );
                    } else {
                        for k in 0..num {
                            bel.add_wire_rc_io(
                                format!("CH{j}_{pin}{k}"),
                                format!("JCH{j}_{pin}{k}_PCS_CORE_PCS0"),
                            );
                        }
                    }
                }
                for &(pin, _, num) in NEXUS_PCS_TO_PCIE_CH {
                    if num == 1 {
                        bel.add_wire_rc_io(
                            format!("CH{j}_{pin}"),
                            format!("JCH{j}_{pin}_PCS_CORE_PCS0"),
                        );
                    } else {
                        for k in 0..num {
                            bel.add_wire_rc_io(
                                format!("CH{j}_{pin}{k}"),
                                format!("JCH{j}_{pin}{k}_PCS_CORE_PCS0"),
                            );
                        }
                    }
                }
            }
            for pad in ["REFCLKP", "REFCLKN"] {
                let pad_apio = &format!("{pad}_APIO");
                bel.add_wire_rc_io(pad, format!("JSDQ_{pad}_PCS_CORE_PCS0"))
                    .add_wire_rc_io(pad_apio, format!("JPAD_APIO_SDQ_{pad}"))
                    .claim_pip(pad, pad_apio)
                    .claim_pip(pad_apio, pad);
            }
            for &(pin, _) in NEXUS_PCS_FROM_PCIE {
                bel.add_wire_rc_io(pin, format!("J{pin}_PCS_CORE_PCS0"));
            }
            let wire_refmux = bel
                .ctx
                .rc_io_wire(cell_refmux, &format!("JQ{i}CLKOUT_PCSREFMUX8_CORE"));
            let wire_align_rx = bel
                .ctx
                .rc_io_wire(cell_pcs0, "JRX_ALIGN_CLKOUT_PCS_CORE_PCS0");
            let wire_align_tx = bel
                .ctx
                .rc_io_wire(cell_pcs0, "JTX_ALIGN_CLKOUT_PCS_CORE_PCS0");
            bel.add_wire_rc_io("ALTCLKIN", "JALTCLKIN_PCS_CORE_PCS0")
                .claim_pip_from("ALTCLKIN", wire_refmux)
                .add_wire_rc_io("RX_ALIGN_CLKIN", "JRX_ALIGN_CLKIN_PCS_CORE_PCS0")
                .add_wire_rc_io("RX_ALIGN_CLKOUT", "JRX_ALIGN_CLKOUT_PCS_CORE_PCS0")
                .add_wire_rc_io("TX_ALIGN_CLKIN", "JTX_ALIGN_CLKIN_PCS_CORE_PCS0")
                .add_wire_rc_io("TX_ALIGN_CLKOUT", "JTX_ALIGN_CLKOUT_PCS_CORE_PCS0")
                .claim_pip_from("RX_ALIGN_CLKIN", wire_align_rx)
                .claim_pip_from("TX_ALIGN_CLKIN", wire_align_tx);
            for j in 0..8 {
                bel.add_wire_rc_io(
                    format!("TX_LALIGN_IN_DOWN{j}"),
                    format!("JTX_LALIGN_IN_DOWN{j}_PCS_CORE_PCS0"),
                )
                .add_wire_rc_io(
                    format!("TX_LALIGN_IN_UP{j}"),
                    format!("JTX_LALIGN_IN_UP{j}_PCS_CORE_PCS0"),
                )
                .add_wire_rc_io(
                    format!("TX_LALIGN_OUT_DOWN{j}"),
                    format!("JTX_LALIGN_OUT_DOWN{j}_PCS_CORE_PCS0"),
                )
                .add_wire_rc_io(
                    format!("TX_LALIGN_OUT_UP{j}"),
                    format!("JTX_LALIGN_OUT_UP{j}_PCS_CORE_PCS0"),
                );
                if let Some(cell_prev) = cell_pcs_prev {
                    let prev_out = bel
                        .ctx
                        .rc_io_wire(cell_prev, &format!("JTX_LALIGN_OUT_DOWN{j}_PCS_CORE_PCS0"));
                    let prev_in = bel
                        .ctx
                        .rc_io_wire(cell_prev, &format!("JTX_LALIGN_IN_DOWN{j}_PCS_CORE_PCS0"));
                    bel.claim_pip_from(format!("TX_LALIGN_IN_UP{j}"), prev_out)
                        .claim_pip_to(prev_in, format!("TX_LALIGN_OUT_UP{j}"));
                }
            }
        }
    }

    pub fn process_pcie_ll_nexus(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PCIE_JD5D] {
            let cell_pcs = self.chip.special_loc[&SpecialLocKey::Pcs(0)];
            let mut bel = self.extract_bel_cell(tcrd.bel(bslots::PCIE_LL), tcrd.delta(-9, 0));
            bel.name_rc("PCIELL_CORE")
                .suffix("PCIELL_CORE")
                .output_bind_rc_io(
                    bcls::PCIE_LL::LINK0_CKUSRO,
                    "JLINK0_CKUSRO_PCIELL_CORE",
                    wires::OUT_PCIE_USRCLK[0].cell(0),
                )
                .output_bind_rc_io(
                    bcls::PCIE_LL::LINK1_CKUSRO,
                    "JLINK1_CKUSRO_PCIELL_CORE",
                    wires::OUT_PCIE_USRCLK[1].cell(0),
                );
            for i in 0..4 {
                for &(pin_pcs, pin, num) in NEXUS_PCS_FROM_PCIE_CH {
                    if num == 1 {
                        let wire_pcs = bel
                            .ctx
                            .rc_io_wire(cell_pcs, &format!("JCH{i}_{pin_pcs}_PCS_CORE_PCS0"));
                        let pname = &format!("{pin}{i}");
                        bel.add_wire_rc_io(pname, format!("J{pname}_PCIELL_CORE"))
                            .claim_pip_to(wire_pcs, pname);
                    } else {
                        for j in 0..num {
                            let wire_pcs = bel.ctx.rc_io_wire(
                                cell_pcs,
                                &format!("JCH{i}_{pin_pcs}{j}_PCS_CORE_PCS0"),
                            );
                            let ii = i * num + j;
                            let pname = &format!("{pin}{ii}");
                            bel.add_wire_rc_io(pname, format!("J{pname}_PCIELL_CORE"))
                                .claim_pip_to(wire_pcs, pname);
                        }
                    }
                }
                for &(pin_pcs, pin, num) in NEXUS_PCS_TO_PCIE_CH {
                    if num == 1 {
                        let wire_pcs = bel
                            .ctx
                            .rc_io_wire(cell_pcs, &format!("JCH{i}_{pin_pcs}_PCS_CORE_PCS0"));
                        let pname = &format!("{pin}{i}");
                        bel.add_wire_rc_io(pname, format!("J{pname}_PCIELL_CORE"))
                            .claim_pip_from(pname, wire_pcs);
                    } else {
                        for j in 0..num {
                            let wire_pcs = bel.ctx.rc_io_wire(
                                cell_pcs,
                                &format!("JCH{i}_{pin_pcs}{j}_PCS_CORE_PCS0"),
                            );
                            let ii = i * num + j;
                            let pname = &format!("{pin}{ii}");
                            bel.add_wire_rc_io(pname, format!("J{pname}_PCIELL_CORE"))
                                .claim_pip_from(pname, wire_pcs);
                        }
                    }
                }
            }
            for &(pin_pcs, pin) in NEXUS_PCS_FROM_PCIE {
                let wire_pcs = bel
                    .ctx
                    .rc_io_wire(cell_pcs, &format!("J{pin_pcs}_PCS_CORE_PCS0"));
                bel.add_wire_rc_io(pin, format!("J{pin}_PCIELL_CORE"))
                    .claim_pip_to(wire_pcs, pin);
            }
            for &(pin_pcs, pin) in NEXUS_PCS_TO_PCIE {
                let wire_pcs = bel
                    .ctx
                    .rc_io_wire(cell_pcs, &format!("J{pin_pcs}_PCS_CORE_PCS0"));
                bel.add_wire_rc_io(pin, format!("J{pin}_PCIELL_CORE"))
                    .claim_pip_from(pin, wire_pcs);
            }
        }
    }

    pub fn process_mipi_nexus(&mut self) {
        let Some(&cell_tile) = self.chip.special_loc.get(&SpecialLocKey::Mipi) else {
            return;
        };
        if !self.naming.strings.contains("JLMMIREADY_DPHY_CORE_DPHY0") {
            self.skip_mipi = true;
            return;
        }
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!();
        };
        for i in 0..2 {
            let dx = match kind {
                NexusKind::SmallMipi => [0, 26][i],
                NexusKind::SmallMipiPcie => [0, 24][i],
                _ => unreachable!(),
            };
            let (cslot_io_s, cslot_io_n) = match kind {
                NexusKind::SmallMipi => (
                    defs::nexus::tcls::MIPI_JE5D15::IO_S,
                    defs::nexus::tcls::MIPI_JE5D15::IO_N,
                ),
                NexusKind::SmallMipiPcie => (
                    defs::nexus::tcls::MIPI_JE5D30::IO_S,
                    defs::nexus::tcls::MIPI_JE5D30::IO_N,
                ),
                _ => unreachable!(),
            };
            let cell = cell_tile.delta(dx, 0);
            {
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::BEL_INT));
                ex.cell(cell)
                    .output_bind_rc_io(
                        "JCLKOUT_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0",
                        wires::IMUX_MIPI_CLKREF[i].cell(0),
                    )
                    .input_xlat_rc_io("JCLKIN0_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0")
                    .input_io_rc_io(
                        "JCLKIN1_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0",
                        cslot_io_s[0].wire(wires::OUT_IO_INCK[0]),
                    )
                    .input_io_rc_io(
                        "JCLKIN2_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0",
                        cslot_io_s[1].wire(wires::OUT_IO_INCK[0]),
                    )
                    .input_io_rc_io(
                        "JCLKIN3_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0",
                        cslot_io_s[2].wire(wires::OUT_IO_INCK[0]),
                    )
                    .input_io_rc_io(
                        "JCLKIN4_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0",
                        cslot_io_n[i].wire(wires::OUT_IO_INCK[0]),
                    );
            }
            let bcrd = cell_tile.bel(bslots::MIPI[i]);
            self.hybrid_out_rc_io_cell(
                bcrd,
                cell,
                "JUCRXWCHS_DPHY_CORE_DPHY0",
                wires::OUT_MIPI_CLKHSBYTE[i].cell(0),
            );
            let (_r, c) = self.rc(cell);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.names([
                format!("DPHY{i}"),
                format!("TDPHY_REFCLOCK_MUX_CORE{c}"),
                format!("DPHY{i}_CKP"),
                format!("DPHY{i}_CKN"),
                format!("DPHY{i}_DP0"),
                format!("DPHY{i}_DN0"),
                format!("DPHY{i}_DP1"),
                format!("DPHY{i}_DN1"),
                format!("DPHY{i}_DP2"),
                format!("DPHY{i}_DN2"),
                format!("DPHY{i}_DP3"),
                format!("DPHY{i}_DN3"),
            ])
            .suffix("DPHY_CORE_DPHY0")
            .output_bind_rc_io(
                bcls::MIPI::UTWDCKHS,
                "JUTWDCKHS_DPHY_CORE_DPHY0",
                wires::OUT_MIPI_UTWDCKHS[i].cell(0),
            )
            .output_bind_rc_io(
                bcls::MIPI::URWDCKHS,
                "JURWDCKHS_DPHY_CORE_DPHY0",
                wires::OUT_MIPI_URWDCKHS[i].cell(0),
            );
            for j in 0..3 {
                bel.input_xlat_rc_io(
                    bcls::MIPI::REFCLOCK_MUX_TESTINP[j],
                    format!("JTESTINP{j}_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0"),
                );
            }
            for pad in [
                "DP0", "DP1", "DP2", "DP3", "DN0", "DN1", "DN2", "DN3", "CKP", "CKN",
            ] {
                let pad_apio = &format!("{pad}_APIO");
                bel.add_wire_rc_io(pad, format!("J{pad}_DPHY_CORE_DPHY0"))
                    .add_wire_rc_io(pad_apio, format!("JPAD_APIO_{pad}"))
                    .claim_pip(pad, pad_apio)
                    .claim_pip(pad_apio, pad);
            }
        }
    }

    pub fn process_serdes(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => self.process_serdes_scm(),
            ChipKind::Ecp2M => self.process_serdes_ecp2(),
            ChipKind::Ecp3(_) => self.process_serdes_ecp3(),
            ChipKind::Ecp4 => self.process_serdes_ecp4(),
            ChipKind::Ecp5 => self.process_serdes_ecp5(),
            ChipKind::Crosslink => self.process_mipi_crosslink(),
            ChipKind::Nexus(_) => {
                self.process_pcie_x1_nexus();
                self.process_pcs_nexus();
                self.process_pcie_ll_nexus();
                self.process_mipi_nexus();
            }
            _ => (),
        }
    }
}
