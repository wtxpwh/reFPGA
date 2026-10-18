use prjcombine_ecp::{
    chip::{ChipKind, IoGroupKind, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{DirH, DirHV, DirV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    fn process_serdes_scm(&mut self) {
        for hv in [DirHV::NW, DirHV::NE] {
            let bcrd = self.chip.bel_pll(hv, 0).bel(bslots::SERDES_CORNER);

            let mut cells_serdes = vec![];
            match hv.h {
                DirH::W => {
                    for (col, cd) in &self.chip.columns {
                        if cd.io_n == IoGroupKind::Serdes && col < self.chip.col_clk {
                            cells_serdes.push(bcrd.with_cr(col + 6, self.chip.row_n() - 11));
                        }
                    }
                }
                DirH::E => {
                    for (col, cd) in self.chip.columns.iter().rev() {
                        if cd.io_n == IoGroupKind::Serdes && col >= self.chip.col_clk {
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
                let bank = self.chip.columns[bcrd.col].bank_n.unwrap();
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
                    .find(|&col| self.chip.columns[col].bank_n == Some(bank + 2))
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
                    h: if bcrd.col < self.chip.col_clk {
                        DirH::W
                    } else {
                        DirH::E
                    },
                    v: if bcrd.row < self.chip.row_clk {
                        DirV::S
                    } else {
                        DirV::N
                    },
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
            let bank = self.chip.columns[bcrd.cell.col].bank_s.unwrap();
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
            let bank = self.chip.columns[bcrd.cell.col].bank_s.unwrap();
            let dual = match bank {
                50 => 0,
                51 => 1,
                _ => unreachable!(),
            };
            let cell_prev = if dual != 0 {
                let mut cell_prev = cell.delta(-1, 0);
                while self.chip.columns[cell_prev.col].io_s != IoGroupKind::Serdes {
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
                let bcrd = tcrd.bel(bslots::MIPI);
                let bank = self.chip.columns[bcrd.cell.col].bank_n.unwrap();
                let idx = match bank {
                    60 => 0,
                    61 => 1,
                    _ => unreachable!(),
                };

                self.hybrid_out_rc_io(
                    bcrd,
                    "JCLKHSBYTE_MIPIDPHY",
                    wires::OUT_MIPI_CLKHSBYTE.cell(0),
                );

                self.extract_switchbox(bcrd.bel(bslots::BEL_INT))
                    .output_bind_rc_io("CLKREF", wires::IMUX_MIPI_CLKREF.cell(0))
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

    pub fn process_serdes(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => self.process_serdes_scm(),
            ChipKind::Ecp2M => self.process_serdes_ecp2(),
            ChipKind::Ecp3(_) => self.process_serdes_ecp3(),
            ChipKind::Ecp4 => self.process_serdes_ecp4(),
            ChipKind::Ecp5 => self.process_serdes_ecp5(),
            ChipKind::Crosslink => self.process_mipi_crosslink(),
            _ => (),
        }
    }
}
