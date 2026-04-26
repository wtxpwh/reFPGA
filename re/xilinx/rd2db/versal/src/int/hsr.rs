use prjcombine_interconnect::db::{BelSlotId, TileClassId, WireSlotIdExt};
use prjcombine_re_xilinx_rawdump::Coord;
use prjcombine_versal::{
    chip::FabricKind,
    defs::{bslots, tcls, wires},
};

use crate::{grid::get_fabric, int::IntMaker};

impl<'rd> IntMaker<'rd> {
    fn fill_tiles_hnoc_ncrb(&mut self) {
        for (tkn, naming, bn) in [
            ("NOC_TNOC_SSIT_NCRB_CORE", "HNOC_NCRB_SSIT", "NOC_NCRB_SSIT"),
            (
                "NOC_TNOC_SSIT_NCRB_CORE_MY",
                "HNOC_NCRB_SSIT",
                "NOC_NCRB_SSIT",
            ),
            ("NOC_TNOC_NCRB_CORE_MX", "HNOC_NCRB", "NOC_NCRB"),
            ("NOC_HBM_NCRB_CORE_MX", "HNOC_NCRB", "NOC_NCRB"),
            ("NOC_XPIO_NCRB_CORE_MX", "HNOC_NCRB", "NOC_NCRB"),
            ("NOC2_NCRB_TILE", "HNOC_NCRB", "NOC_NCRB"),
        ] {
            let tcid = tcls::HNOC_NCRB;
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                for i in 0..2 {
                    bels.push(self.bel_name_only(xy, bslots::HNOC_NCRB[i], bn, 0, i));
                }
                self.builder.xtile_id(tcid, naming, xy).bels(bels).extract();
            }
        }
    }

    fn fill_tiles_hnoc_ssit(&mut self) {
        if let Some(&xy) = self
            .rd
            .tiles_by_kind_name("NOC_TNOC_SSIT_S2_CORE")
            .iter()
            .next()
        {
            let &pnoc_xy = self
                .rd
                .tiles_by_kind_name("NOC_PNOC_SSIT_CORE")
                .iter()
                .next()
                .unwrap();

            for (tcid, naming, nmu, nsu) in [
                (tcls::HNOC_SSIT_PS9_A, "HNOC_SSIT_PS9_A", 0..4, 0..2),
                (tcls::HNOC_SSIT_PS9_BD, "HNOC_SSIT_PS9_BD", 0..0, 0..0),
                (tcls::HNOC_SSIT_PS9_C, "HNOC_SSIT_PS9_C", 4..10, 2..6),
            ] {
                let mut bels = vec![];
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS7575", 0, i)
                            .pin_name_only("OUT_0", 1)
                            .pin_name_only("OUT_1", 1)
                            .pin_name_only("OUT_2", 1)
                            .pin_name_only("OUT_3", 1)
                            .pin_name_only("IN_0", 1)
                            .pin_name_only("IN_1", 1)
                            .pin_name_only("IN_2", 1)
                            .pin_name_only("IN_3", 1),
                    );
                }

                if tcid == tcls::HNOC_SSIT_PS9_A {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_NPI_NIR, "NPI_NIR", 0, 0)
                            .raw_tile(1),
                    );
                }
                for (i, sy) in nmu.enumerate() {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_PS_NMU[i], "NOC_NMU128", 0, sy)
                            .raw_tile(1),
                    );
                }
                for (i, sy) in nsu.enumerate() {
                    bels.push(
                        self.bel_name_only(pnoc_xy, bslots::HNOC_PS_NSU[i], "NOC_NSU128", 0, sy)
                            .raw_tile(1),
                    );
                }

                self.builder
                    .xtile_id(tcid, naming, xy)
                    .raw_tile(pnoc_xy)
                    .bels(bels)
                    .extract();
            }
        }
        for (tcid, naming, tkn) in [
            (tcls::HNOC_SSIT, "HNOC_SSIT", "NOC_TNOC_BRIDGE_BOT_CORE"),
            (tcls::HNOC_SSIT, "HNOC_SSIT", "NOC_TNOC_BRIDGE_TOP_CORE"),
            (tcls::HNOC_SSIT2_V, "HNOC_SSIT2_V", "NOC2_TNOC_NIDB_CORE"),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS7575", 0, i)
                            .pin_name_only("OUT_0", 1)
                            .pin_name_only("OUT_1", 1)
                            .pin_name_only("OUT_2", 1)
                            .pin_name_only("OUT_3", 1)
                            .pin_name_only("IN_0", 1)
                            .pin_name_only("IN_1", 1)
                            .pin_name_only("IN_2", 1)
                            .pin_name_only("IN_3", 1),
                    );
                }
                for i in 0..2 {
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::HNOC_NIDB_V[i],
                                if tcid == tcls::HNOC_SSIT2_V {
                                    "NOC2_NIDBV"
                                } else {
                                    "NOC_NIDB"
                                },
                                0,
                                i,
                            )
                            .pin_name_only("OUT_0", 1)
                            .pin_name_only("IN_0", 1)
                            .pin_name_only("RX", 1)
                            .pin_name_only("TX", 1),
                    );
                }
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HNOC_NPP_RPTR[i], "NOC_NPP_RPTR", 0, i)
                            .pin_name_only("OUT_LEFT", 1)
                            .pin_name_only("IN_LEFT", 1)
                            .pin_name_only("OUT_RIGHT", 1)
                            .pin_name_only("IN_RIGHT", 1),
                    );
                }
                self.builder.xtile_id(tcid, naming, xy).bels(bels).extract();
            }
        }
    }

    fn fill_tiles_hnoc_nidb_h(&mut self) {
        let tkn = "NOC2_HNOC_NIDB_CORE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let bel = self
                .builder
                .bel_xy(bslots::HNOC_NIDB_H, "NOC2_NIDBH", 0, 0)
                .pin_name_only("OUT_0", 1)
                .pin_name_only("IN_0", 1)
                .pin_name_only("RX", 1)
                .pin_name_only("TX", 1);
            self.builder
                .xtile_id(tcls::HNOC_SSIT2_H, "HNOC_SSIT2_H", xy)
                .bel(bel)
                .extract();
        }
    }

    fn fill_tiles_hsr_sysmon_sat(&mut self) {
        for tkn in [
            "HSM_AMS_SAT_LC",
            "HSM_AMS_SAT_LC_MX",
            "HSM_AMS_SAT_ME",
            "HSM_AMS_SAT_ME_MX",
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                let Some(slot) = self.rd.slot_xy("SYSMON_SAT", 0, 0) else {
                    continue;
                };
                if let Some((bli0_xy, wn0)) =
                    self.xlat_bli_full(xy, "AMS_SATELLITE_ATOM_0_CSSDTRIGN")
                {
                    let bli1_xy = self
                        .xlat_bli(xy, "AMS_SATELLITE_ATOM_0_TESTADCOUT_8_")
                        .unwrap();
                    let bli2_xy = self
                        .xlat_bli(xy, "AMS_SATELLITE_ATOM_0_SCANCLKNEXT")
                        .unwrap();
                    let bli3_xy = self.xlat_bli(xy, "AMS_SATELLITE_ATOM_0_FABRICCLK").unwrap();
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::SYSMON_SAT_HSR, "SYSMON_SAT", 0, 0)
                        .pin_name_only("DATAREADY", 1);
                    for i in 0..16 {
                        bel = bel
                            .pin_name_only(&format!("PVALUE_{i}_"), 1)
                            .pin_name_only(&format!("VVALUE_{i}_"), 1)
                            .pin_name_only(&format!("TVALUE_{i}_"), 1);
                    }
                    for i in 0..8 {
                        bel = bel.pin_name_only(&format!("ALARM_{i}_"), 1);
                    }
                    let (tcid, naming, bli_xy, bli_naming) = match wn0 {
                        "BLI_GRP6_D_BLI_IMUX_BYP10" => (
                            tcls::SYSMON_SAT_HSR_XPIO,
                            "SYSMON_SAT_HSR_XPIO",
                            vec![None, Some(bli0_xy)],
                            "BLI_MUX",
                        ),
                        "BLI_GRP0_A_BLI_IMUX0" | "BLI_GRP0_B_BLI_IMUX0" => {
                            // dealt with as part of HBM_MS
                            continue;
                        }
                        "BLI_GRP7_D_BLI_IMUX4" => (
                            tcls::SYSMON_SAT_HSR_ME,
                            "SYSMON_SAT_HSR_ME",
                            vec![Some(bli1_xy), Some(bli3_xy), None, Some(bli0_xy)],
                            "BLI_MUX",
                        ),
                        "BLI_GRP7_D_BLI_IMUX_BYP5" => (
                            tcls::SYSMON_SAT_HSR_X5IO,
                            "SYSMON_SAT_HSR_XPIO",
                            vec![Some(bli0_xy), Some(bli2_xy)],
                            "BLI_MUX_NOC",
                        ),
                        _ => panic!("ummm {wn0}"),
                    };
                    let bli = self.builder.ndb.get_tile_class_naming(bli_naming);
                    let mut xn = self.builder.xtile_id(tcid, naming, xy);
                    for (i, bli_xy) in bli_xy.into_iter().enumerate() {
                        if let Some(bli_xy) = bli_xy {
                            xn = xn.ref_single(bli_xy, i, bli);
                        }
                    }
                    xn.bel(bel).extract();
                } else {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::SYSMON_SAT_HSR, "SYSMON_SAT", 0, 0);
                    let tksite = tk.sites.get(&slot).unwrap().1;
                    for pin in tksite.pins.keys() {
                        bel = bel.pin_name_only(pin, 1);
                    }

                    self.builder
                        .xtile_id(tcls::SYSMON_SAT_HSR_NC, "SYSMON_SAT_HSR_ME", xy)
                        .bel(bel)
                        .extract();
                }
            }
        }
    }

    fn fill_tiles_hnoc_me(&mut self) {
        let tiles = self.rd.tiles_by_kind_name("NOC_TNOC_S2_CORE_MX");
        let tiles_nxu = Vec::from_iter(
            self.rd
                .tiles_by_kind_name("NOC_TNOC_ME_CORE_MX")
                .iter()
                .chain(self.rd.tiles_by_kind_name("NOC_TNOC_ME_CORE_R180"))
                .copied(),
        );
        if !tiles.is_empty() {
            let tcid = tcls::HNOC_ME_VNOC;
            let naming = "HNOC_ME_VNOC";
            let mut bels = vec![];
            for i in 0..2 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS5555", 0, i)
                        .pin_name_only("OUT_0", 1)
                        .pin_name_only("OUT_1", 1)
                        .pin_name_only("OUT_2", 1)
                        .pin_name_only("OUT_3", 1)
                        .pin_name_only("IN_0", 1)
                        .pin_name_only("IN_1", 1)
                        .pin_name_only("IN_2", 1)
                        .pin_name_only("IN_3", 1),
                );
            }
            self.builder
                .xtile_id(tcid, naming, tiles[0])
                .bels(bels)
                .extract();

            let tcid = tcls::HNOC_ME;
            let naming = "HNOC_ME";
            let mut bels = vec![];
            for i in 0..2 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_NPS[i], "NOC_NPS5555", 0, i)
                        .pin_name_only("OUT_0", 1)
                        .pin_name_only("OUT_1", 1)
                        .pin_name_only("OUT_2", 1)
                        .pin_name_only("OUT_3", 1)
                        .pin_name_only("IN_0", 1)
                        .pin_name_only("IN_1", 1)
                        .pin_name_only("IN_2", 1)
                        .pin_name_only("IN_3", 1),
                );
            }
            for i in 0..2 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_ME_NMU[i], "NOC_NMU128", 0, 0)
                        .raw_tile(1 + i)
                        .pin_name_only("FROM_NOC", 1)
                        .pin_name_only("TO_NOC", 1)
                        .pin_name_only("AXI_IN", 1),
                );
            }
            for i in 0..2 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::HNOC_ME_NSU[i], "NOC_NSU128", 0, 0)
                        .raw_tile(1 + i)
                        .pin_name_only("FROM_NOC", 1)
                        .pin_name_only("TO_NOC", 1)
                        .pin_name_only("AXI_OUT", 1),
                );
            }
            self.builder
                .xtile_id(tcid, naming, tiles[0])
                .raw_tile(tiles_nxu[0])
                .raw_tile(tiles_nxu[1])
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_hnoc_me2(&mut self) {
        let nps = self.rd.tiles_by_kind_name("NOC2_NPS6X_TOP_MX");
        let nmu = self.rd.tiles_by_kind_name("NOC2_NMU128_TOP_MX");
        let nsu = self.rd.tiles_by_kind_name("NOC2_NSU128_TOP_MX");
        let ncrb = self.rd.tiles_by_kind_name("NOC2_NCRB_TILE_MX");

        for &scan in self.rd.tiles_by_kind_name("NOC2_SCAN_TOP_MX") {
            if self
                .xlat_bli(scan, "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_FROM_PL_4_")
                .is_none()
            {
                self.fill_tile_noc2(
                    tcls::HNOC_ME2_CAP_W,
                    "HNOC_ME2_CAP_W",
                    &[
                        (nmu[0], bslots::HNOC_ME_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_ME_NMU[1], "NOC2_NMU128", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((4, 4))),
                    ],
                    &[
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                        None,
                    ],
                );
            } else if self
                .xlat_bli(scan, "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_FROM_PL_7_")
                .is_none()
            {
                self.fill_tile_noc2(
                    tcls::HNOC_ME2_CAP_E,
                    "HNOC_ME2_CAP_E",
                    &[
                        (nmu[0], bslots::HNOC_ME_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_ME_NMU[1], "NOC2_NMU128", None),
                        (nmu[2], bslots::HNOC_ME_NMU[2], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_ME_NSU[0], "NOC2_NSU128", None),
                        (nps[0], bslots::HNOC_NPS6X[0], "NOC2_NPS6X", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((7, 7))),
                    ],
                    &[
                        None,
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                    ],
                );
            } else if self
                .xlat_bli(scan, "NOC2_SCAN_ATOM_0_NOC2_SCAN_CHNL_FROM_PL_9_")
                .is_none()
            {
                self.fill_tile_noc2(
                    tcls::HNOC_ME2,
                    "HNOC_ME2",
                    &[
                        (nmu[0], bslots::HNOC_ME_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_ME_NMU[1], "NOC2_NMU128", None),
                        (nmu[2], bslots::HNOC_ME_NMU[2], "NOC2_NMU128", None),
                        (nmu[3], bslots::HNOC_ME_NMU[3], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_ME_NSU[0], "NOC2_NSU128", None),
                        (nps[0], bslots::HNOC_NPS6X[0], "NOC2_NPS6X", None),
                        (nps[1], bslots::HNOC_NPS6X[1], "NOC2_NPS6X", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((9, 9))),
                    ],
                    &[
                        None,
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                    ],
                );
            } else if ncrb.is_empty() {
                self.fill_tile_noc2(
                    tcls::HNOC_ME2_VNOC4,
                    "HNOC_ME2_VNOC4",
                    &[
                        (nmu[0], bslots::HNOC_ME_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_ME_NMU[1], "NOC2_NMU128", None),
                        (nmu[2], bslots::HNOC_ME_NMU[2], "NOC2_NMU128", None),
                        (nmu[3], bslots::HNOC_ME_NMU[3], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_ME_NSU[0], "NOC2_NSU128", None),
                        (nps[0], bslots::HNOC_NPS6X[0], "NOC2_NPS6X", None),
                        (nps[1], bslots::HNOC_NPS6X[1], "NOC2_NPS6X", None),
                        (nps[2], bslots::HNOC_NPS6X[2], "NOC2_NPS6X", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((10, 10))),
                    ],
                    &[
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N")),
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                    ],
                );
            } else {
                self.fill_tile_noc2(
                    tcls::HNOC_ME2_NCRB,
                    "HNOC_ME2_NCRB",
                    &[
                        (nmu[0], bslots::HNOC_ME_NMU[0], "NOC2_NMU128", None),
                        (nmu[1], bslots::HNOC_ME_NMU[1], "NOC2_NMU128", None),
                        (nmu[2], bslots::HNOC_ME_NMU[2], "NOC2_NMU128", None),
                        (nmu[3], bslots::HNOC_ME_NMU[3], "NOC2_NMU128", None),
                        (nsu[0], bslots::HNOC_ME_NSU[0], "NOC2_NSU128", None),
                        (nps[0], bslots::HNOC_NPS6X[0], "NOC2_NPS6X", None),
                        (nps[1], bslots::HNOC_NPS6X[1], "NOC2_NPS6X", None),
                        (scan, bslots::HNOC_SCAN[0], "NOC2_SCAN", Some((10, 10))),
                        (ncrb[0], bslots::HNOC_ME2_NCRB[0], "NOC_NCRB", None),
                    ],
                    &[
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_EN_N")),
                        Some((scan, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")),
                    ],
                );
            }
        }
    }

    fn fill_tiles_hnoc_hbm(&mut self) {
        if let Some(&xy_nps6) = self
            .rd
            .tiles_by_kind_name("NOC_HBM_S4_NPS6_CORE")
            .iter()
            .next()
            && let Some(&xy_nps8) = self
                .rd
                .tiles_by_kind_name("NOC_HBM_NPS8_CORE")
                .iter()
                .next()
        {
            let mut bels = vec![];
            for i in 0..4 {
                bels.push(self.bel_name_only(xy_nps6, bslots::HNOC_NPS6[i], "NOC_NPS6", 0, i));
            }
            for i in 0..2 {
                bels.push(
                    self.bel_name_only(xy_nps8, bslots::HNOC_NPS4[i], "NOC_NPS4", 0, i)
                        .raw_tile(1),
                );
            }
            self.builder
                .xtile_id(tcls::HNOC_HBM, "HNOC_HBM", xy_nps6)
                .raw_tile(xy_nps8)
                .bels(bels)
                .extract();
        }
        if let Some(&xy) = self.rd.tiles_by_kind_name("NOC_HBM_S4_CORE").iter().next() {
            let mut bels = vec![];
            for i in 0..4 {
                bels.push(self.bel_name_only(xy, bslots::HNOC_NPS[i], "NOC_NPS5555", 0, i));
            }
            self.builder
                .xtile_id(tcls::HNOC_HBM_VNOC, "HNOC_HBM_VNOC", xy)
                .bels(bels)
                .extract();
        }
    }

    fn fill_tiles_hsr_clk_ssit2_h(&mut self) {
        let tkn = "CLKE2_REBUF_HSR_EDGE_CORE";
        for i in 0..24 {
            self.builder
                .extra_name_tile(tkn, format!("CLK_HROUTE_L{i}"), wires::HSR_HROUTE_W[i]);
            self.builder
                .extra_name_tile(tkn, format!("CLK_HDISTR_L{i}"), wires::HSR_HDISTR_W[i]);
            self.builder.extra_name_tile(
                tkn,
                format!("CLK_HROUTE_SLR{i}"),
                wires::HSR_HROUTE_SLR[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("CLK_HDISTR_SLR{i}"),
                wires::HSR_HDISTR_SLR[i],
            );
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            self.builder
                .xtile_id(tcls::CLK_HSR_SSIT2_H, "CLK_HSR_SSIT2_H", xy)
                .switchbox(bslots::HSR_CLK)
                .optin_muxes(&wires::HSR_HROUTE_W[..])
                .optin_muxes(&wires::HSR_HDISTR_W[..])
                .optin_muxes(&wires::HSR_HROUTE_SLR[..])
                .optin_muxes(&wires::HSR_HDISTR_SLR[..])
                .extract();
        }
    }

    fn fill_tiles_hsr_clk_me(&mut self) {
        let fabric = get_fabric(self.rd);
        for (tkn, is_me2) in [
            ("CLK_REBUF_HSR_ME_TNOC_CORE_MX", false),
            ("CLK_REBUF_HSR_ME_TNOC_ULVT_CORE_MX", true),
        ] {
            let ulvt = if is_me2 { "_ULVT" } else { "" };
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_0_{i}"),
                    wires::HSR_HDISTR_W[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_HDISTR_1_{i}"),
                    wires::HSR_HDISTR[i],
                );
            }
            for i in 0..6 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_DRVR_TRI{ulvt}_{ii}_CLK_IN", ii = i + 48),
                    wires::HSR_OUT_BUFG_TEST[i],
                );
            }
            for (wire, name) in [
                (wires::HSR_CLK_TEST_IN[0], "NPI_CLK"),
                (wires::HSR_CLK_TEST_IN[1], "CLK_FROM_ME0"),
                (wires::HSR_CLK_TEST_IN[2], "NOC_CLK"),
                (wires::HSR_CLK_TEST_IN[3], "GND_WIRE0"),
                (wires::HSR_CLK_TEST_IN[3], "GND_WIRE"),
                (wires::HSR_CLK_TEST_IN[3], "MCLK"),
                (wires::HSR_CLK_TEST_IN[4], "SCAN_CLK"),
                (wires::HSR_CLK_TEST_IN[5], "CLK_FROM_ME1"),
                (wires::HSR_IMUX_TO_ME, "CLK_TO_ME"),
                (wires::HSR_IMUX_HSM_TO_ME, "HSM_REF_CLK_TO_ME"),
                (wires::OUT_PS_HSM_REF_CLK[0], "HSM_REF_CLK0"),
                (wires::OUT_PS_HSM_REF_CLK[1], "HSM_REF_CLK1"),
            ] {
                self.builder.extra_name_tile(tkn, name, wire);
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let mut bels = vec![];
                for i in 0..6 {
                    bels.push(self.builder.bel_xy(
                        bslots::BUFG_HSR_TEST[i],
                        if is_me2 {
                            "BUFG_HSR_TEST_ULVT"
                        } else {
                            "BUFG_HSR_TEST"
                        },
                        0,
                        i,
                    ));
                }
                self.builder
                    .xtile_id(
                        tcls::CLK_ME,
                        if is_me2 {
                            "CLK_ME_ME2"
                        } else {
                            match fabric {
                                FabricKind::Orig => "CLK_ME",
                                FabricKind::Ulvt => "CLK_ME_ULVT",
                                _ => "CLK_ME_REV2",
                            }
                        },
                        xy,
                    )
                    .bels(bels)
                    .switchbox(bslots::HSR_CLK)
                    .optin_muxes(&wires::HSR_HDISTR[..])
                    .optin_muxes(&wires::HSR_HDISTR_W[..])
                    .optin_muxes(&[wires::HSR_IMUX_TO_ME, wires::HSR_IMUX_HSM_TO_ME])
                    .extract();
            }
        }
    }

    fn fill_tiles_hsr_clk_hbm(&mut self) {
        let tkn = "CLKE2_REBUF_HBM_CORE_MX";
        for i in 0..24 {
            self.builder
                .extra_name_tile(tkn, format!("CLK_HROUTE_L{i}"), wires::HSR_HROUTE_W[i]);
            self.builder
                .extra_name_tile(tkn, format!("CLK_HROUTE_R{i}"), wires::HSR_HROUTE[i]);
            self.builder.extra_name_tile(
                tkn,
                format!(
                    "CLK_CMT_MUX_2TO1_{ii}_CLK_OUT",
                    ii = [
                        0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12,
                        13, 14, 15,
                    ][i]
                ),
                wires::HSR_HROUTE_MUX[i],
            );
        }
        for i in 0..6 {
            self.builder.extra_name_tile(
                tkn,
                format!("CLK_CMT_DRVR_TRI_{ii}_CLK_IN", ii = i + 72),
                wires::HSR_OUT_BUFG_TEST[i],
            );
        }
        for (wire, name) in [
            (wires::HSR_CLK_TEST_IN[0], "NPI_CLK"),
            (wires::HSR_CLK_TEST_IN[1], "CLK_IN_TEST0"),
            (wires::HSR_CLK_TEST_IN[2], "NOC_CLK"),
            (wires::HSR_CLK_TEST_IN[3], "GND_WIRE0"),
            (wires::HSR_CLK_TEST_IN[3], "GND_WIRE"),
            (wires::HSR_CLK_TEST_IN[3], "MCLK"),
            (wires::HSR_CLK_TEST_IN[4], "SCAN_CLK"),
            (wires::HSR_CLK_TEST_IN[5], "CLK_IN_TEST1"),
            (wires::HSR_IMUX_TO_HBM[0], "CLK_TO_HBM0"),
            (wires::HSR_IMUX_TO_HBM[1], "CLK_TO_HBM1"),
            (wires::OUT_PS_HSM_REF_CLK[0], "HSM_REF_CLK0"),
            (wires::OUT_PS_HSM_REF_CLK[1], "HSM_REF_CLK1"),
            (
                wires::HSR_IMUX_TO_HBM_HROUTE[0],
                "CLK_CMT_MUX_24_ENC_0_CLK_OUT",
            ),
            (
                wires::HSR_IMUX_TO_HBM_HROUTE[1],
                "CLK_CMT_MUX_24_ENC_1_CLK_OUT",
            ),
        ] {
            self.builder.extra_name_tile(tkn, name, wire);
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            let mut bels = vec![];
            for i in 0..6 {
                bels.push(
                    self.builder
                        .bel_xy(bslots::BUFG_HSR_TEST[i], "BUFG_HSR_TEST", 0, i),
                );
            }
            for i in 0..2 {
                for (pin, wire) in [
                    ("I", wires::HSR_IN_BUFG_HBM[i].cell(0)),
                    ("O", wires::HSR_OUT_BUFG_HBM[i].cell(0)),
                ] {
                    self.builder
                        .extra_name_pin_xy(tkn, "BUFG_PS", 0, i, pin, wire, true);
                }
                bels.push(self.builder.bel_xy(bslots::BUFG_HBM[i], "BUFG_PS", 0, i));
            }
            self.builder
                .xtile_id(tcls::CLK_HBM, "CLK_HBM", xy)
                .bels(bels)
                .switchbox(bslots::HSR_CLK)
                .optin_muxes(&wires::HSR_HROUTE[..])
                .optin_muxes(&wires::HSR_HROUTE_W[..])
                .optin_muxes(&wires::HSR_HROUTE_MUX[..])
                .optin_muxes(&wires::HSR_IMUX_TO_HBM[..])
                .optin_muxes(&wires::HSR_IMUX_TO_HBM_HROUTE[..])
                .extract();
        }
    }

    fn fill_tiles_hsr_hbm_intf(&mut self) {
        let tkn = "NOC_HBM_BLI_SCAN_TOP_MX";
        let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() else {
            return;
        };
        let bli_xy = [
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN0_DFX_SCAN_CSSD_TRIG_N",
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN0_DFX_SCAN_CLK_N",
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN1_DFX_SCAN_CLK_N",
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN1_DFX_SCAN_CSSD_TRIG_N",
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN2_DFX_SCAN_CSSD_TRIG_N",
            "NOC_HBM_BLI_SCAN_ATOM_0_IF_HBM_BLI_DFX_IN2_DFX_SCAN_CLK_N",
        ]
        .map(|wn| self.xlat_bli(xy, wn).unwrap());

        let nmu_xy = [0, 3].map(|idx| {
            self.follow_wire(bli_xy[idx], "BLI_GRP0_A_BLI_IMUX0")
                .unwrap()
        });

        let mut bels =
            vec![
                self.builder
                    .bel_xy(bslots::NOC_HBM_BLI_SCAN, "NOC_HBM_BLI_SCAN", 0, 0),
            ];

        for i in 0..2 {
            bels.push(
                self.builder
                    .bel_xy(bslots::NOC_NMU_HBM2E[i], "NOC_NMU_HBM2E", 0, 0)
                    .raw_tile(1 + i)
                    .pin_name_only("FROM_NOC", 1)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("TO_PHY", 1)
                    .pin_name_only("IF_BLI_PHY_DIR_IN_BLI_CLK", 1),
            );
        }

        let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
        let mut xt = self
            .builder
            .xtile_id(tcls::HBM_INTF, "HBM_INTF", xy)
            .raw_tile(nmu_xy[0])
            .raw_tile(nmu_xy[1])
            .bels(bels);
        for (i, bli_xy) in bli_xy.into_iter().enumerate() {
            xt = xt.ref_single(bli_xy, i, bli)
        }

        xt.extract();
    }

    fn fill_tiles_hsr_hbm_chnl(&mut self) {
        let Some(&hbmmc_xy) = self.rd.tiles_by_kind_name("HBMMC_CORE").iter().next() else {
            return;
        };
        let Some(&phy_xy) = self.rd.tiles_by_kind_name("HBM_PHY_CHNL_TOP").iter().next() else {
            return;
        };
        let Some(&io_xy) = self.rd.tiles_by_kind_name("HBM_IO_CHNL_TOP").iter().next() else {
            return;
        };

        let mut bel = self.builder.bel_xy(bslots::HBM_CHNL, "HBM_MC", 0, 0);
        for pin in self.bel_pins(hbmmc_xy, "HBM_MC", 0, 0) {
            let npin = &format!("MC_{pin}");
            bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
        }

        for (pin, wire) in [
            (
                "CH0_PHY_CHNL_CORE_DFI_CLK_MC2PHY_BLI",
                wires::HSR_OUT_HBM_CLK_MC2PHY.cell(0),
            ),
            (
                "CH0_PHY_CHNL_CORE_DFI_CLK_NOC2PHY_BLI",
                wires::HSR_OUT_HBM_CLK_NOC2PHY.cell(0),
            ),
        ] {
            self.builder.extra_name_pin_xy(
                "HBM_PHY_CHNL_TOP",
                "HBM_PHY_CHNL",
                0,
                0,
                pin,
                wire,
                true,
            );
        }
        bel = bel
            .sub_xy("HBM_PHY_CHNL", 0, 0)
            .raw_tile(1)
            .pin_rename("CH0_PHY_CHNL_CORE_DFI_CLK_MC2PHY_BLI", "DFI_CLK_MC2PHY")
            .pin_rename("CH0_PHY_CHNL_CORE_DFI_CLK_NOC2PHY_BLI", "DFI_CLK_NOC2PHY");
        for pin in self.bel_pins(phy_xy, "HBM_PHY_CHNL", 0, 0) {
            if !matches!(
                pin,
                "CH0_PHY_CHNL_CORE_DFI_CLK_MC2PHY_BLI" | "CH0_PHY_CHNL_CORE_DFI_CLK_NOC2PHY_BLI"
            ) {
                let npin = &format!("PHY_{pin}");
                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
            }
        }

        bel = bel.sub_xy("HBM_IO_CHNL", 0, 0).raw_tile(2);
        for pin in self.bel_pins(io_xy, "HBM_IO_CHNL", 0, 0) {
            let npin = &format!("IO_{pin}");
            bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
        }

        self.builder
            .xtile_id(tcls::HBM_CHNL, "HBM_CHNL", hbmmc_xy)
            .raw_tile(phy_xy)
            .raw_tile(io_xy)
            .bel(bel)
            .extract();
    }

    fn fill_tiles_hsr_hbm_ms(&mut self) {
        let Some(&phy_xy) = self.rd.tiles_by_kind_name("HBM_PHY_MS_TOP").iter().next() else {
            return;
        };
        let Some(&io_xy) = self.rd.tiles_by_kind_name("HBM_IO_MS_TOP").iter().next() else {
            return;
        };
        let Some(&hpll_xy) = self.rd.tiles_by_kind_name("CMT_HPLL").iter().next() else {
            return;
        };

        let bli0_xy = self
            .xlat_bli(phy_xy, "HBM_PHY_MS_CORE_0_PHY_MS_CORE_CLKOUTPHY_EN_MISC")
            .unwrap();

        let sm_xy = ["BLI_GRP0_B_BLI_IMUX0", "BLI_GRP0_A_BLI_IMUX0"].map(|wn| {
            self.follow_wire_filter(bli0_xy, wn, |n| n.starts_with("HSM_AMS_SAT_ME"))
                .unwrap()
        });

        let bli1_xy = self
            .xlat_bli(sm_xy[0], "AMS_SATELLITE_ATOM_0_CSSDTRIGN")
            .unwrap();

        let mut bels = vec![];

        for (pin, wire) in [
            (
                "PHY_MS_CORE_CLKOUT_CCIO_MISC",
                wires::HSR_OUT_HBM_CCIO.cell(0),
            ),
            (
                "PHY_MS_CORE_CLKOUTPHY_MISC",
                wires::HSR_CLK_TEST_IN[1].cell(0),
            ),
            ("PHY_MS_CORE_CLKOUT_MISC", wires::HSR_CLK_TEST_IN[5].cell(0)),
            (
                "PHY_MS_CORE_TMUXOUT_MISC",
                wires::HSR_CLK_TEST_IN[1].cell(1),
            ),
        ] {
            self.builder
                .extra_name_pin_xy("HBM_PHY_MS_TOP", "HBM_PHY_MS", 0, 0, pin, wire, true);
        }
        self.builder.extra_name_pin_xy(
            "HBM_IO_MS_TOP",
            "HBM_IO_MS",
            0,
            0,
            "HBM_IO_MS_CORE_PHY2CCIO_REF_CLK",
            wires::HSR_IMUX_TO_HBM[0].cell(0),
            true,
        );

        let mut bel = self.builder.bel_xy(bslots::HBM_MS, "HBM_PHY_MS", 0, 0);
        for pin in self.bel_pins(phy_xy, "HBM_PHY_MS", 0, 0) {
            if let Some(spin) = pin.strip_prefix("PHY_MS_CORE_")
                && (spin.starts_with("PLL_TEST")
                    || spin.starts_with("DLL_TEST")
                    || matches!(
                        spin,
                        "PWRDWN_MISC"
                            | "CLKOUTPHY_EN_MISC"
                            | "CLKOUT_CCIO_MISC"
                            | "CLKOUT_MISC"
                            | "CLKOUTPHY_MISC"
                            | "TMUXOUT_MISC"
                    ))
            {
                let spin = spin.replace("_MISC", "");
                bel = bel.pin_rename(pin, spin);
            } else {
                let npin = &format!("PHY_{pin}");
                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
            }
        }

        bel = bel.sub_xy("HBM_IO_MS", 0, 0).raw_tile(1);
        for pin in self.bel_pins(io_xy, "HBM_IO_MS", 0, 0) {
            if pin == "HBM_IO_MS_CORE_PHY2CCIO_REF_CLK" {
                bel = bel.pin_rename(pin, "CCIO_REF_CLK")
            } else {
                let npin = &format!("IO_{pin}");
                bel = bel.pin_rename(pin, npin).pin_name_only(npin, 1);
            }
        }

        bels.push(bel);

        for i in 0..2 {
            let mut bel = self
                .builder
                .bel_xy(bslots::SYSMON_SAT_HBM[i], "SYSMON_SAT", 0, 0)
                .raw_tile(2 + i)
                .pin_name_only("DATAREADY", 1);
            for i in 0..16 {
                bel = bel
                    .pin_name_only(&format!("PVALUE_{i}_"), 1)
                    .pin_name_only(&format!("VVALUE_{i}_"), 1)
                    .pin_name_only(&format!("TVALUE_{i}_"), 1);
            }
            for i in 0..8 {
                bel = bel.pin_name_only(&format!("ALARM_{i}_"), 1);
            }
            bels.push(bel);
        }
        bels.push(
            self.bel_name_only(hpll_xy, bslots::HPLL, "HPLL", 0, 0)
                .raw_tile(4),
        );

        let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
        self.builder
            .xtile_id(tcls::HBM_MS, "HBM_MS", phy_xy)
            .raw_tile(io_xy)
            .raw_tile(sm_xy[0])
            .raw_tile(sm_xy[1])
            .raw_tile(hpll_xy)
            .ref_single(bli0_xy, 2, bli)
            .ref_single(bli1_xy, 3, bli)
            .bels(bels)
            .extract();
    }

    #[allow(clippy::type_complexity)]
    pub fn fill_tile_noc2(
        &mut self,
        tcls: TileClassId,
        naming: &str,
        tiles: &[(Coord, BelSlotId, &str, Option<(usize, usize)>)],
        bli_refs: &[Option<(Coord, &str)>],
    ) {
        let mut bels = vec![];
        for (idx, &(xy, bslot, slot, scan)) in tiles.iter().enumerate() {
            if let Some((n, nm)) = scan {
                let mut bel = self.builder.bel_xy(bslot, slot, 0, 0).raw_tile(idx);
                for i in n..15 {
                    bel = bel
                        .pin_name_only(&format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"), 1)
                        .pin_name_only(&format!("NOC2_SCAN_CHNL_TO_PL_{i}_"), 1);
                }
                for i in nm..14 {
                    bel = bel.pin_name_only(&format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"), 1);
                }
                bels.push(bel);
            } else if bslots::HNOC_NMU512.contains(bslot) || bslot == bslots::HNOC_NSU512 {
                bels.push(
                    self.builder
                        .bel_xy(bslot, slot, 0, 0)
                        .raw_tile(idx)
                        .pin_name_only("TO_NOC", 1)
                        .pin_name_only("FROM_NOC", 1),
                );
            } else if bslot == bslots::HNOC_ME2_NCRB[0] {
                bels.push(self.bel_name_only(xy, bslot, slot, 0, 0).raw_tile(idx));
                bels.push(
                    self.bel_name_only(xy, bslots::HNOC_ME2_NCRB[1], slot, 0, 1)
                        .raw_tile(idx),
                );
            } else {
                bels.push(self.bel_name_only(xy, bslot, slot, 0, 0).raw_tile(idx));
            }
        }
        let mut bli_xy = vec![];
        let mut is_mux = false;
        for (idx, &br) in bli_refs.iter().enumerate() {
            if let Some((xy, w)) = br {
                let xy = self.xlat_bli(xy, w).unwrap();
                if self.rd.tiles[&xy].name.starts_with("BLI_LS_MUX") {
                    is_mux = true;
                }
                bli_xy.push((idx, xy));
            }
        }
        let mut xt = self.builder.xtile_id(tcls, naming, tiles[0].0).bels(bels);
        for &(xy, _, _, _) in &tiles[1..] {
            xt = xt.raw_tile(xy);
        }
        for (idx, xy) in bli_xy {
            let bli = xt.builder.ndb.get_tile_class_naming(if is_mux {
                "BLI_MUX_NOC"
            } else {
                "BLI_MUX"
            });
            xt = xt.ref_single(xy, idx, bli);
        }
        xt.extract();
    }

    fn fill_tiles_hsr_nic(&mut self) {
        if !self.rd.tile_kinds.contains_key("NOC2_NMU256_TOP") {
            return;
        }
        let mut tile_rows = vec![];
        let null = self.rd.tile_kinds.get("NULL").unwrap().0;
        for y in (self.rd.height - 6)..self.rd.height {
            let mut row = vec![];
            for x in 0..self.rd.width {
                let xy = Coord { x, y };
                if self.rd.tiles[&xy].kind != null {
                    row.push(xy);
                }
            }
            tile_rows.push(row);
        }

        {
            let xy = tile_rows[1][0];
            let scan_xy = tile_rows[1][1];
            let bli_xy = self
                .xlat_bli(scan_xy, "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N")
                .unwrap();

            let mut bels = vec![];

            for i in 0..2 {
                bels.push(self.bel_name_only(xy, bslots::HNOC_NCRB[i], "NOC_NCRB", 0, i));
            }

            let mut bel = self
                .builder
                .bel_xy(bslots::HNOC_NCRB_SCAN, "NOC2_SCAN", 0, 0)
                .raw_tile(1);
            for i in 4..15 {
                bel = bel
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"), 1)
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_TO_PL_{i}_"), 1);
            }
            for i in 3..14 {
                bel = bel.pin_name_only(&format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"), 1);
            }
            bels.push(bel);

            let bli = self.builder.ndb.get_tile_class_naming("BLI_MUX");
            self.builder
                .xtile_id(tcls::HNOC2_NCRB, "HNOC2_NCRB", xy)
                .raw_tile(scan_xy)
                .bels(bels)
                .ref_single(bli_xy, 0, bli)
                .extract();
        }

        self.fill_tile_noc2(
            tcls::HNOC_NIC_CPM_CDX,
            "HNOC_NIC_CPM_CDX",
            &[
                (
                    tile_rows[0][0],
                    bslots::HNOC_NIC_AXIS_NMU[0],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][1],
                    bslots::HNOC_NIC_AXIS_NMU[1],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][2],
                    bslots::HNOC_NIC_AXIS_NMU[2],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][3],
                    bslots::HNOC_NIC_AXIS_NMU[3],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][4],
                    bslots::HNOC_NIC_AXI_NMU,
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][5],
                    bslots::HNOC_NIC_XBR2X4,
                    "NOC2_XBR2X4",
                    None,
                ),
                (tile_rows[0][6], bslots::HNOC_NPS[0], "NOC2_NPS5555", None),
                (tile_rows[0][7], bslots::HNOC_NPS[1], "NOC2_NPS5555", None),
                (
                    tile_rows[2][0],
                    bslots::HNOC_NIC_AXIS_NSU[0],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][1],
                    bslots::HNOC_NIC_AXIS_NSU[1],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][2],
                    bslots::HNOC_NIC_AXIS_NSU[2],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][3],
                    bslots::HNOC_NIC_AXIS_NSU[3],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][4],
                    bslots::HNOC_NIC_AXI_NSU,
                    "NOC2_NSU256",
                    None,
                ),
                (tile_rows[2][5], bslots::HNOC_SCAN[0], "NOC2_SCAN", None),
                (tile_rows[2][6], bslots::HNOC_NPS[2], "NOC2_NPS5555", None),
                (tile_rows[2][7], bslots::HNOC_NPS[3], "NOC2_NPS5555", None),
            ],
            &[],
        );
        self.fill_tile_noc2(
            tcls::HNOC_NIC_CPM_DPU,
            "HNOC_NIC_CPM_DPU",
            &[
                (
                    tile_rows[0][8],
                    bslots::HNOC_NIC_AXIS_NMU[0],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][9],
                    bslots::HNOC_NIC_AXIS_NMU[1],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][10],
                    bslots::HNOC_NIC_AXIS_NMU[2],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[0][11],
                    bslots::HNOC_NIC_AXIS_NMU[3],
                    "NOC2_NMU256",
                    None,
                ),
                (tile_rows[0][12], bslots::HNOC_NPS[0], "NOC2_NPS7575", None),
                (tile_rows[0][13], bslots::HNOC_NPS[1], "NOC2_NPS7575", None),
                (
                    tile_rows[0][14],
                    bslots::HNOC_SCAN[0],
                    "NOC2_SCAN",
                    Some((13, 12)),
                ),
                (
                    tile_rows[2][8],
                    bslots::HNOC_NIC_AXIS_NSU[0],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][9],
                    bslots::HNOC_NIC_AXIS_NSU[1],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][10],
                    bslots::HNOC_NIC_AXIS_NSU[2],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[2][11],
                    bslots::HNOC_NIC_AXIS_NSU[3],
                    "NOC2_NSU256",
                    None,
                ),
                (tile_rows[2][12], bslots::HNOC_NPS[2], "NOC2_NPS7575", None),
                (tile_rows[2][13], bslots::HNOC_NPS[3], "NOC2_NPS7575", None),
                (
                    tile_rows[2][14],
                    bslots::HNOC_SCAN[1],
                    "NOC2_SCAN",
                    Some((15, 14)),
                ),
            ],
            &[
                None,
                Some((
                    tile_rows[0][14],
                    "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                )),
            ],
        );

        self.fill_tile_noc2(
            tcls::HNOC_NIC_HNICX,
            "HNOC_NIC_HNICX",
            &[
                (
                    tile_rows[3][0],
                    bslots::HNOC_NIC_AXIS_NMU[0],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[3][1],
                    bslots::HNOC_NIC_AXIS_NMU[1],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[3][2],
                    bslots::HNOC_NIC_AXIS_NMU[2],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[3][3],
                    bslots::HNOC_NIC_AXIS_NMU[3],
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[3][4],
                    bslots::HNOC_NIC_AXI_NMU,
                    "NOC2_NMU256",
                    None,
                ),
                (
                    tile_rows[3][5],
                    bslots::HNOC_NIC_XBR2X4,
                    "NOC2_XBR2X4",
                    None,
                ),
                (tile_rows[3][6], bslots::HNOC_NPS[0], "NOC2_NPS7575", None),
                (tile_rows[3][7], bslots::HNOC_NPS[1], "NOC2_NPS7575", None),
                (
                    tile_rows[4][0],
                    bslots::HNOC_NIC_AXIS_NSU[0],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[4][1],
                    bslots::HNOC_NIC_AXIS_NSU[1],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[4][2],
                    bslots::HNOC_NIC_AXIS_NSU[2],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[4][3],
                    bslots::HNOC_NIC_AXIS_NSU[3],
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[4][4],
                    bslots::HNOC_NIC_AXI_NSU,
                    "NOC2_NSU256",
                    None,
                ),
                (
                    tile_rows[4][5],
                    bslots::HNOC_SCAN[0],
                    "NOC2_SCAN",
                    Some((15, 14)),
                ),
                (tile_rows[4][6], bslots::HNOC_NPS[2], "NOC2_NPS7575", None),
                (tile_rows[4][7], bslots::HNOC_NPS[3], "NOC2_NPS7575", None),
            ],
            &[
                None,
                Some((
                    tile_rows[4][5],
                    "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                )),
            ],
        );

        let mut pos = 0;
        while pos < tile_rows[5].len() {
            if self.rd.tiles[&tile_rows[5][pos]]
                .name
                .starts_with("NOC2_NMU")
            {
                self.fill_tile_noc2(
                    tcls::HNOC_NIC_BLI,
                    "HNOC_NIC_BLI",
                    &[
                        (
                            tile_rows[5][pos],
                            bslots::HNOC_NMU512[0],
                            "NOC2_NMU512",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 1],
                            bslots::HNOC_NSU512,
                            "NOC2_NSU512",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 2],
                            bslots::HNOC_NPS[0],
                            "NOC2_NPS7575",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 3],
                            bslots::HNOC_NPS[1],
                            "NOC2_NPS7575",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 4],
                            bslots::HNOC_NPP_RPTR[0],
                            "NOC_NPP_RPTR",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 5],
                            bslots::HNOC_NPP_RPTR[1],
                            "NOC_NPP_RPTR",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 6],
                            bslots::HNOC_SCAN[0],
                            "NOC2_SCAN",
                            Some((6, 5)),
                        ),
                    ],
                    &[
                        Some((
                            tile_rows[5][pos],
                            "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_2_",
                        )),
                        Some((tile_rows[5][pos], "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WID_3_")),
                        Some((
                            tile_rows[5][pos],
                            "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_18_",
                        )),
                        Some((tile_rows[5][pos], "NOC2_NMU512_ATOM_0_IF_NOC_AXI_ARID_0_")),
                        Some((
                            tile_rows[5][pos],
                            "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_PAR_34_",
                        )),
                        Some((
                            tile_rows[5][pos],
                            "NOC2_NMU512_ATOM_0_IF_NOC_AXI_WDATA_387_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_PAR_2_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_147_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_386_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_RDATA_419_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_ARREADY",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_ARUSER_11_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_WDATA_407_",
                        )),
                        Some((
                            tile_rows[5][pos + 1],
                            "NOC2_NSU512_ATOM_0_IF_NOC_AXI_WDATA_442_",
                        )),
                        Some((
                            tile_rows[5][pos + 6],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                        None,
                    ],
                );
                pos += 7;
            } else {
                self.fill_tile_noc2(
                    tcls::HNOC_NIC_VNOC,
                    "HNOC_NIC_VNOC",
                    &[
                        (tile_rows[5][pos], bslots::HNOC_NPS[0], "NOC2_NPS7575", None),
                        (
                            tile_rows[5][pos + 1],
                            bslots::HNOC_NPS[1],
                            "NOC2_NPS7575",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 2],
                            bslots::HNOC_NPS[2],
                            "NOC2_NPS7575",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 3],
                            bslots::HNOC_NPS[3],
                            "NOC2_NPS7575",
                            None,
                        ),
                        (
                            tile_rows[5][pos + 4],
                            bslots::HNOC_SCAN[0],
                            "NOC2_SCAN",
                            Some((6, 5)),
                        ),
                    ],
                    &[
                        Some((
                            tile_rows[5][pos + 4],
                            "NOC2_SCAN_ATOM_0_IF_NOC2_SCAN_CNTL_IN_SCAN_CLK_N",
                        )),
                        None,
                    ],
                );
                pos += 5;
            }
        }
    }

    pub fn fill_tiles_hsr(&mut self) {
        self.fill_tiles_hnoc_ncrb();
        self.fill_tiles_hnoc_ssit();
        self.fill_tiles_hnoc_nidb_h();
        self.fill_tiles_hnoc_xpio();
        self.fill_tiles_hnoc_me();
        self.fill_tiles_hnoc_me2();
        self.fill_tiles_hnoc_hbm();
        self.fill_tiles_hnoc_x5io();
        self.fill_tiles_hsr_sysmon_sat();
        self.fill_tiles_hsr_mmcm();
        self.fill_tiles_hsr_dpll();
        self.fill_tiles_hsr_clk_xpio();
        self.fill_tiles_hsr_clk_ssit2_h();
        self.fill_tiles_hsr_xpio();
        self.fill_tiles_hsr_xpio_dci();
        self.fill_tiles_hsr_xpio_scan();
        self.fill_tiles_hsr_xpll();
        self.fill_tiles_hsr_ddrmc();
        self.fill_tiles_hsr_x5io();
        self.fill_tiles_hsr_x5pll();
        self.fill_tiles_hsr_x5io_bank();
        self.fill_tiles_hsr_x5io_scan();
        self.fill_tiles_hsr_ddrmc5();
        self.fill_tiles_hsr_clk_me();
        self.fill_tiles_hsr_clk_hbm();
        self.fill_tiles_hsr_hbm_intf();
        self.fill_tiles_hsr_hbm_chnl();
        self.fill_tiles_hsr_hbm_ms();
        self.fill_tiles_hsr_nic();
    }
}
