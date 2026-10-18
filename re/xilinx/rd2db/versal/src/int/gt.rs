use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH},
};
use prjcombine_re_xilinx_rawdump::Coord;
use prjcombine_versal::defs::{bslots, tcls, wires};

use crate::int::IntMaker;

impl<'rd> IntMaker<'rd> {
    pub fn fill_tiles_gt_misc(&mut self) {
        for (tkn, intf_kind, int_dir) in [
            ("AMS_SAT_GT_BOT_TILE_MY", "INTF_W_TERM_GT", Dir::E),
            ("AMS_SAT_GT_TOP_TILE_MY", "INTF_W_TERM_GT", Dir::E),
            ("AMS_SAT_GT_BOT_TILE", "INTF_E_TERM_GT", Dir::W),
            ("AMS_SAT_GT_TOP_TILE", "INTF_E_TERM_GT", Dir::W),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let base_xy = self.builder.delta(xy, 0, -24);
                let intf = self.builder.ndb.get_tile_class_naming(intf_kind);
                let int_xy = self.builder.walk_to_int(base_xy, int_dir, true).unwrap();
                let intf_xy = self
                    .builder
                    .delta(int_xy, if int_dir == Dir::E { -1 } else { 1 }, 0);

                if self.rd.slot_kinds.contains("SYSMON_SAT") {
                    let bel = self
                        .builder
                        .bel_xy(bslots::SYSMON_SAT_GT, "SYSMON_SAT", 0, 0);
                    let mut xn = self
                        .builder
                        .xtile_id(tcls::SYSMON_SAT_GT, "SYSMON_SAT_GT", xy);
                    for i in 0..48 {
                        let intf_xy = xn.builder.delta(intf_xy, 0, (i + i / 4) as i32);
                        xn = xn.ref_single(intf_xy, i, intf)
                    }
                    xn.bel(bel).extract();
                }

                let bel = self.builder.bel_xy(bslots::DPLL_GT, "DPLL", 0, 0);
                let dpll_xy = self.builder.delta(xy, 0, -15);
                let tkn_dpll = self.rd.tile_kinds.key(self.rd.tiles[&dpll_xy].kind);
                self.builder.extra_name_pin_xy(
                    tkn_dpll,
                    "DPLL",
                    0,
                    0,
                    "CLKIN",
                    wires::IMUX_DPLL_CLKIN.cell(0),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn_dpll,
                    "DPLL",
                    0,
                    0,
                    "CLKIN_DESKEW",
                    wires::IMUX_DPLL_CLKIN_DESKEW.cell(0),
                    true,
                );
                for (i, pin) in ["CLKOUT0", "CLKOUT1", "CLKOUT2", "CLKOUT3", "TMUXOUT"]
                    .into_iter()
                    .enumerate()
                {
                    self.builder.extra_name_pin_xy(
                        tkn_dpll,
                        "DPLL",
                        0,
                        0,
                        pin,
                        wires::OUT_DPLL[i].cell(0),
                        true,
                    )
                }
                let mut xn = self.builder.xtile_id(tcls::DPLL_GT, "DPLL_GT", dpll_xy);
                for i in 0..48 {
                    let intf_xy = xn.builder.delta(intf_xy, 0, (i + i / 4) as i32);
                    xn = xn.ref_single(intf_xy, i, intf)
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_rfadc_rfdac(&mut self) {
        for (tkn_s, tkn_n, tcid, bslots, bk) in [
            (
                "RFADC_BOT_CORE",
                "RFADC_TOP_CORE",
                tcls::RFADC,
                bslots::RFADC,
                "RFADC",
            ),
            (
                "RFDAC_BOT_CORE",
                "RFDAC_TOP_CORE",
                tcls::RFDAC,
                bslots::RFDAC,
                "RFDAC",
            ),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn_s).iter().next() {
                let clk_pins = if tcid == tcls::RFADC {
                    vec![
                        (0, "CLK_ADC", wires::OUT_GT_CLK[0].cell(0)),
                        (1, "CLK_ADC_SPARE", wires::OUT_GT_CLK[1].cell(0)),
                        (0, "PLL_DMON_OUT", wires::OUT_GT_CLK[14].cell(0)),
                        (1, "PLL_DMON_OUT_SPARE", wires::OUT_GT_CLK[0].cell(48)),
                    ]
                } else {
                    vec![
                        (0, "CLK_DAC", wires::OUT_GT_CLK[0].cell(0)),
                        (1, "CLK_DAC_SPARE", wires::OUT_GT_CLK[1].cell(0)),
                        (0, "CLK_CAL", wires::OUT_GT_CLK[14].cell(0)),
                        (1, "CLK_CAL_SPARE", wires::OUT_GT_CLK[0].cell(48)),
                        (0, "PLL_DMON_OUT", wires::OUT_GT_CLK[1].cell(48)),
                        (1, "PLL_DMON_OUT_SPARE", wires::OUT_GT_CLK[14].cell(48)),
                    ]
                };
                for (i, pin, wire) in clk_pins {
                    self.builder
                        .extra_name_pin_xy([tkn_s, tkn_n][i], bk, 0, 0, pin, wire, true);
                }

                let mut bels = vec![];
                for i in 0..2 {
                    let mut bel = self.builder.bel_xy(bslots[i], bk, 0, 0).raw_tile(i);
                    for pin in [
                        "CLK_DISTR_IN_LEFT_P",
                        "CLK_DISTR_IN_LEFT_PLL_P",
                        "CLK_DISTR_IN_LEFT_T1_P",
                        "CLK_DISTR_IN_RIGHT_P",
                        "CLK_DISTR_OUT_LEFT_P",
                        "CLK_DISTR_OUT_RIGHT_P",
                        "CLK_DISTR_OUT_RIGHT_PLL_P",
                        "CLK_DISTR_OUT_RIGHT_T1_P",
                        "SYSREF_IN1_L_P",
                        "SYSREF_IN2_L_P",
                        "SYSREF_IN_R_P",
                        "SYSREF_OUT1_R_P",
                        "SYSREF_OUT2_R_P",
                        "SYSREF_OUT_L_P",
                    ] {
                        bel = bel.pin_name_only(pin, 1);
                    }
                    if tcid == tcls::RFADC {
                        if i == 0 {
                            bel = bel
                                .pin_name_only("CLK_ADC_SPARE", 1)
                                .pin_name_only("PLL_DMON_OUT_SPARE", 1);
                        } else {
                            bel = bel
                                .pin_name_only("CLK_ADC", 1)
                                .pin_name_only("PLL_DMON_OUT", 1);
                        }
                    } else {
                        if i == 0 {
                            bel = bel
                                .pin_name_only("CLK_DAC_SPARE", 1)
                                .pin_name_only("CLK_CAL_SPARE", 1)
                                .pin_name_only("PLL_DMON_OUT_SPARE", 1);
                        } else {
                            bel = bel
                                .pin_name_only("CLK_DAC", 1)
                                .pin_name_only("CLK_CAL", 1)
                                .pin_name_only("PLL_DMON_OUT", 1);
                        }
                    }
                    bels.push(bel)
                }

                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
                let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let top_xy = self.builder.delta(xy, 0, 60);
                let mut xn = self.builder.xtile_id(tcid, bk, xy).raw_tile(top_xy);
                for i in 0..96 {
                    let cxy = xn.builder.delta(int_xy, 1, (i + i / 4) as i32);
                    xn = xn.ref_single(cxy, i, intf_e);
                }
                xn.bels(bels).extract();
            }
        }
    }

    pub fn fill_tiles_gt(&mut self) {
        for (tcid, tkn, side) in [
            (tcls::GTY, "GTY_QUAD_SINGLE_MY", DirH::W),
            (tcls::GTYP, "GTYP_QUAD_SINGLE_MY", DirH::W),
            (tcls::GTM, "GTM_QUAD_SINGLE_MY", DirH::W),
            (tcls::GTM2_S, "GTMPW_QUAD_SINGLE_MY", DirH::W),
            (tcls::GTY, "GTY_QUAD_SINGLE", DirH::E),
            (tcls::GTYP, "GTYP_QUAD_SINGLE", DirH::E),
            (tcls::GTM, "GTM_QUAD_SINGLE", DirH::E),
            (tcls::GTM2_S, "GTMPW_QUAD_SINGLE", DirH::E),
        ] {
            let (bslot, bn, cbn) = match tcid {
                tcls::GTY => (bslots::GTY, "GTY_QUAD", "GTY_REFCLK"),
                tcls::GTYP => (bslots::GTYP, "GTYP_QUAD", "GTYP_REFCLK"),
                tcls::GTM => (bslots::GTM, "GTM_QUAD", "GTM_REFCLK"),
                tcls::GTM2_S => (bslots::GTM2, "GTM2_QUAD", "GTM2_REFCLK"),
                _ => unreachable!(),
            };
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let mut has_int = self
                    .builder
                    .rd
                    .tiles_by_kind_name("INT")
                    .iter()
                    .any(|&int_xy| int_xy.y == xy.y);
                let mut intf_xy = vec![];
                let mut xpipe_xy = None;
                let mut hnicpipe_xy = None;
                let mut intf = None;
                let mut is_n = false;
                if has_int {
                    let int_xy = self.builder.walk_to_int(xy, (!side).into(), false).unwrap();
                    let rclk_xy = self.builder.delta(int_xy, 0, -1);
                    is_n = self
                        .builder
                        .rd
                        .tile_kinds
                        .key(self.rd.tiles[&rclk_xy].kind)
                        .starts_with("RCLK");
                    let dx = match side {
                        DirH::W => -1,
                        DirH::E => 1,
                    };
                    let x_range = if xy.x < int_xy.x {
                        xy.x..int_xy.x
                    } else {
                        int_xy.x..xy.x
                    };
                    let mut has_dpll = false;
                    for x in x_range {
                        let pipe_xy = Coord { x, ..xy };
                        let pipe_tile = &self.rd.tiles[&pipe_xy];
                        match self.rd.tile_kinds.key(pipe_tile.kind).as_str() {
                            "HNICPIPE_QUAD_SINGLE" => hnicpipe_xy = Some(pipe_xy),
                            "XPIPE_QUAD_SINGLE" => xpipe_xy = Some(pipe_xy),
                            _ => (),
                        }
                        let dpll_xy = self.builder.delta(pipe_xy, 0, 9);
                        let dpll_tile = &self.rd.tiles[&dpll_xy];
                        if self
                            .builder
                            .rd
                            .tile_kinds
                            .key(dpll_tile.kind)
                            .starts_with("CMT_DPLL")
                        {
                            has_dpll = true;
                        }
                    }
                    if has_dpll {
                        for i in 0..48 {
                            intf_xy.push(self.builder.delta(int_xy, dx, i + i / 4));
                        }
                        intf = Some(self.builder.ndb.get_tile_class_naming(match side {
                            DirH::W => "INTF_W_TERM_GT",
                            DirH::E => "INTF_E_TERM_GT",
                        }));
                    } else {
                        has_int = false;
                    }
                }
                let mut above_xram = false;
                if tcid == tcls::GTYP && self.rd.tile_kinds.contains_key("XRAM_CORE") {
                    let xram_xy = self.builder.delta(xy, 0, -60);
                    let xram_tile = &self.rd.tiles[&xram_xy];
                    if self.rd.tile_kinds.key(xram_tile.kind) == "XRAM_CORE" {
                        above_xram = true;
                    }
                }
                let tcid = if hnicpipe_xy.is_some() {
                    tcls::GTM_HNIC
                } else if above_xram {
                    tcls::GTYP_XRAM
                } else if !has_int {
                    match tcid {
                        tcls::GTYP => tcls::GTYP_PS,
                        tcls::GTM2_S => tcls::GTM2_PS,
                        _ => unreachable!(),
                    }
                } else if tcid == tcls::GTM2_S && is_n {
                    tcls::GTM2_N
                } else {
                    tcid
                };
                let naming = if xpipe_xy.is_some() {
                    "GTY_XPIPE"
                } else if hnicpipe_xy.is_some() {
                    "GTM_HNIC"
                } else {
                    match bslot {
                        bslots::GTY => "GTY",
                        bslots::GTYP => "GTYP",
                        bslots::GTM => "GTM",
                        bslots::GTM2 => "GTM2",
                        _ => unreachable!(),
                    }
                };
                if has_int {
                    for (idx, slot, sy, pin) in [
                        if bslot == bslots::GTM2 {
                            (0, bn, 0, "REFCLK0_ODIV2")
                        } else {
                            (0, cbn, 0, "REFCLK_ODIV2")
                        },
                        (1, bn, 0, "CH0_TXOUTCLK"),
                        (2, bn, 0, "CH1_TXOUTCLK"),
                        (3, bn, 0, "CH0_RXOUTCLK"),
                        (4, bn, 0, "CH1_RXOUTCLK"),
                        (5, bn, 0, "CH0_DMONITOROUTCLK"),
                        (6, bn, 0, "CH1_DMONITOROUTCLK"),
                        if bslot == bslots::GTM2 {
                            (7, bn, 0, "REFCLK1_ODIV2")
                        } else {
                            (7, cbn, 1, "REFCLK_ODIV2")
                        },
                        (8, bn, 0, "CH2_TXOUTCLK"),
                        (9, bn, 0, "CH3_TXOUTCLK"),
                        (10, bn, 0, "CH2_RXOUTCLK"),
                        (11, bn, 0, "CH3_RXOUTCLK"),
                        (12, bn, 0, "CH2_DMONITOROUTCLK"),
                        (13, bn, 0, "CH3_DMONITOROUTCLK"),
                        (14, bn, 0, "BUFG_GT_RSVD"),
                    ] {
                        if idx == 14 && bslot != bslots::GTM2 {
                            continue;
                        }
                        self.builder.extra_name_pin_xy(
                            tkn,
                            slot,
                            0,
                            sy,
                            pin,
                            wires::OUT_GT_CLK[idx].cell(0),
                            true,
                        );
                    }
                }

                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                let slot = self.rd.slot_xy(bn, 0, 0).unwrap();
                let tksite = tk.sites.get(&slot).unwrap().1;
                let mut bel = self.builder.bel_xy(bslot, bn, 0, 0);
                for i in 0..2 {
                    bel = bel
                        .extra_wire(
                            format!("REFCLKOUTSOUTH{i}"),
                            &[format!("IF_CASC_REFCLK_REFCLKOUTSOUTH{i}")],
                        )
                        .extra_wire(
                            format!("REFCLKOUTNORTH{i}"),
                            &[format!("IF_CASC_REFCLK_REFCLKOUTNORTH{i}")],
                        )
                        .extra_wire(
                            format!("REFCLKOUTSOUTH{i}_DUMMY"),
                            &[format!("VCC_WIRE{i}")],
                        )
                        .extra_wire(
                            format!("REFCLKOUTNORTH{i}_DUMMY"),
                            &[format!("VCC_WIRE{ii}", ii = i + 2)],
                        );
                }
                let has_pipe = xpipe_xy.is_some() || hnicpipe_xy.is_some();
                if !has_int {
                    for pin in tksite.pins.keys() {
                        bel = bel.pin_name_only(pin, 1);
                    }
                } else if has_pipe {
                    for pin in tksite.pins.keys() {
                        if pin.ends_with("_RXOUTCLK")
                            || pin.ends_with("_TXOUTCLK")
                            || pin.ends_with("_DMONITOROUTCLK")
                        {
                            // keep it
                        } else if pin.starts_with("PIPE")
                            || pin.starts_with("RXPI")
                            || pin.starts_with("TXPI")
                            || pin.starts_with("RESETDONE")
                            || (pin.starts_with("REFCLK") && pin.ends_with("INT"))
                            || pin.contains("RXRECCLK")
                            || pin.ends_with("REFCLK0")
                            || pin.ends_with("REFCLK1")
                        {
                            bel = bel.pin_name_only(pin, 1);
                        } else {
                            let rpin = &format!("_GT_{pin}");
                            bel = bel.pin_rename(pin, rpin).pin_name_only(rpin, 1);
                        }
                    }
                } else {
                    for i in 0..6 {
                        bel = bel
                            .pin_name_only(&format!("PIPESOUTHIN_{i}_"), 1)
                            .pin_name_only(&format!("PIPESOUTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("PIPENORTHIN_{i}_"), 1)
                            .pin_name_only(&format!("PIPENORTHOUT_{i}_"), 1);
                    }
                    for i in 0..4 {
                        bel = bel
                            .pin_name_only(&format!("RXPISOUTHIN_{i}_"), 1)
                            .pin_name_only(&format!("RXPISOUTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("RXPINORTHIN_{i}_"), 1)
                            .pin_name_only(&format!("RXPINORTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("TXPISOUTHIN_{i}_"), 1)
                            .pin_name_only(&format!("TXPISOUTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("TXPINORTHIN_{i}_"), 1)
                            .pin_name_only(&format!("TXPINORTHOUT_{i}_"), 1);
                    }
                    for i in 0..2 {
                        bel = bel
                            .pin_name_only(&format!("LCPLLSOUTHREFCLK{i}"), 1)
                            .pin_name_only(&format!("LCPLLNORTHREFCLK{i}"), 1)
                            .pin_name_only(&format!("RESETDONE_SOUTHIN_{i}_"), 1)
                            .pin_name_only(&format!("RESETDONE_SOUTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("RESETDONE_NORTHIN_{i}_"), 1)
                            .pin_name_only(&format!("RESETDONE_NORTHOUT_{i}_"), 1)
                            .pin_name_only(&format!("REFCLK{i}_GTREFCLKPDINT"), 1)
                            .pin_name_only(&format!("REFCLK{i}_CLKTESTSIGINT"), 1)
                            .pin_name_only(&format!("REFCLK{i}_ODIV2INT"), 1);
                        for j in 0..2 {
                            bel = bel
                                .pin_name_only(&format!("HSCLK{i}_LCPLLGTREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_LCPLLSOUTHREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_LCPLLNORTHREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_RPLLGTREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_RPLLSOUTHREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_RPLLNORTHREFCLK{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_RXRECCLKOUT{j}"), 1)
                                .pin_name_only(&format!("HSCLK{i}_RXRECCLKSEL_{j}_"), 1);
                        }
                    }
                }
                for i in 0..2 {
                    bel = bel.sub_xy(cbn, 0, i);
                    if bslot == bslots::GTM2 {
                        let rpin = &format!("_REFCLK{i}_REFCLK_ODIV2");
                        bel = bel.pin_rename("REFCLK_ODIV2", rpin).pin_name_only(rpin, 1);
                    } else {
                        bel = bel.pin_rename("REFCLK_ODIV2", format!("REFCLK{i}_ODIV2"));
                        if !has_int {
                            bel = bel.pin_name_only(&format!("REFCLK{i}_ODIV2"), 1);
                        }
                    }
                    for pin in [
                        "REFCLK_CEB",
                        "REFCLK_CLKTESTSIG",
                        "REFCLK_GTREFCLK",
                        "REFCLK_RXRECCLK0",
                        "REFCLK_RXRECCLK1",
                        "REFCLK_RXRECCLK2",
                        "REFCLK_RXRECCLK3",
                        "RXRECCLKSEL_0_",
                        "RXRECCLKSEL_1_",
                    ] {
                        let rpin = &format!("_REFCLK{i}_{pin}");
                        bel = bel.pin_rename(pin, rpin).pin_name_only(rpin, 1);
                    }
                }
                if xpipe_xy.is_some() {
                    let tk = self
                        .builder
                        .rd
                        .tile_kinds
                        .get("XPIPE_QUAD_SINGLE")
                        .unwrap()
                        .1;
                    let slot = self.rd.slot_xy("XPIPE_QUAD", 0, 0).unwrap();
                    let xtksite = tk.sites.get(&slot).unwrap().1;
                    bel = bel.sub_xy("XPIPE_QUAD", 0, 0).raw_tile(1);
                    for pin in xtksite.pins.keys() {
                        if !tksite.pins.contains_key(pin) {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                }
                let mut bel_hnic = None;
                if hnicpipe_xy.is_some() {
                    let tk = self
                        .builder
                        .rd
                        .tile_kinds
                        .get("HNICPIPE_QUAD_SINGLE")
                        .unwrap()
                        .1;
                    let slot = self.rd.slot_xy("HNICPIPE_QUAD", 0, 0).unwrap();
                    let xtksite = tk.sites.get(&slot).unwrap().1;
                    bel = bel
                        .sub_xy("HNICPIPE_QUAD", 0, 0)
                        .raw_tile(1)
                        .extra_wire("HNICPIPE_GND", &["GND_WIRE"]);
                    let mut bel_h = self
                        .builder
                        .bel_xy(bslots::HNICPIPE, "HNICPIPE_QUAD", 0, 0)
                        .raw_tile(1);
                    for pin in xtksite.pins.keys() {
                        if !tksite.pins.contains_key(pin) {
                            bel = bel.pin_name_only(pin, 1);
                        }
                        if pin.starts_with("HNICPIPE_SCAN")
                            || matches!(
                                pin.as_str(),
                                "HNICPIPE_CSSD_TRIG_N" | "HNICPIPE_EDT_UPDT_N"
                            )
                        {
                            let rpin = pin.strip_prefix("HNICPIPE_").unwrap();
                            bel_h = bel_h.pin_rename(pin, rpin);
                        } else {
                            bel_h = bel_h.pin_name_only(pin, 1);
                        }
                    }
                    bel_hnic = Some(bel_h);
                }
                let mut xt = self.builder.xtile_id(tcid, naming, xy).bel(bel);
                if let Some(bel_hnic) = bel_hnic {
                    xt = xt.bel(bel_hnic);
                }
                if let Some(pipe_xy) = xpipe_xy.or(hnicpipe_xy) {
                    xt = xt.raw_tile(pipe_xy);
                }
                for (i, intf_xy) in intf_xy.into_iter().enumerate() {
                    xt = xt.ref_single(intf_xy, i, intf.unwrap());
                }
                xt.extract();
                if let Some(xpipe_xy) = xpipe_xy {
                    let pin_txoutclkdbg = if is_n {
                        "TXOUTCLKDBG_SOUTH"
                    } else {
                        "TXOUTCLKDBG_NORTH"
                    };
                    self.builder.extra_name_pin_xy(
                        "XPIPE_QUAD_SINGLE",
                        "XPIPE_QUAD",
                        0,
                        0,
                        pin_txoutclkdbg,
                        wires::OUT_GT_CLK[14].cell(0),
                        true,
                    );
                    let mut bel = self.builder.bel_xy(bslots::XPIPE, "XPIPE_QUAD", 0, 0);
                    let tk = self
                        .builder
                        .rd
                        .tile_kinds
                        .get("XPIPE_QUAD_SINGLE")
                        .unwrap()
                        .1;
                    let slot = self.rd.slot_xy("XPIPE_QUAD", 0, 0).unwrap();
                    let xtksite = tk.sites.get(&slot).unwrap().1;
                    for pin in xtksite.pins.keys() {
                        if pin == pin_txoutclkdbg {
                            bel = bel.pin_rename(pin, "TXOUTCLKDBG");
                        } else {
                            bel = bel.pin_name_only(pin, 1);
                        }
                    }
                    self.builder
                        .xtile_id(
                            tcls::XPIPE,
                            if is_n { "XPIPE_N" } else { "XPIPE_S" },
                            xpipe_xy,
                        )
                        .bel(bel)
                        .extract();
                }
            }
        }
    }

    pub fn fill_tiles_vdu(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("VDU_CORE_MY")
            .iter()
            .next()
        {
            for (pin, wire) in [
                ("VDUCORECLK", wires::OUT_DPLL[2].cell(0)),
                ("VDUMCUCLK", wires::OUT_DPLL[3].cell(0)),
            ] {
                self.builder
                    .extra_name_pin_xy("VDU_CORE_MY", "VDU", 0, 0, pin, wire, true);
            }

            let bel = self.builder.bel_xy(bslots::VDU, "VDU", 0, 0);
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let mut xn = self.builder.xtile_id(tcls::VDU, "VDU", xy);
            for i in 0..48 {
                let cxy = xn.builder.delta(int_xy, 1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_e);
            }
            xn.bel(bel).extract();
        }
    }

    pub fn fill_tiles_bfr_b(&mut self) {
        for tkn in ["BFR_TILE_B_BOT_CORE", "BFR_TILE_B_TOP_CORE"] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let bel = self.builder.bel_xy(bslots::BFR_B, "BFR_B", 0, 0);
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
                let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
                let mut xn = self.builder.xtile_id(tcls::BFR_B, "BFR_B", xy);
                for i in 0..48 {
                    let cxy = xn.builder.delta(int_xy, 1, (i + i / 4) as i32);
                    xn = xn.ref_single(cxy, i, intf_e);
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_vcu2(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("VCU2_TILE")
            .iter()
            .next()
        {
            let mut noc_xy = vec![];
            let mut tmp_xy = xy;
            loop {
                if self
                    .builder
                    .rd
                    .tile_kinds
                    .key(self.rd.tiles[&tmp_xy].kind)
                    .starts_with("NOC")
                {
                    noc_xy.push(tmp_xy);
                    if noc_xy.len() == 2 {
                        break;
                    }
                }
                tmp_xy = self.builder.delta(tmp_xy, 1, 0);
            }
            for (pin, wire) in [
                ("VCU2ENCCLK", wires::OUT_DPLL[2].cell(0)),
                ("VCU2DECCLK", wires::OUT_DPLL[2].cell(48)),
                ("VCU2AXISLAVECLK", wires::OUT_DPLL[3].cell(48)),
            ] {
                self.builder
                    .extra_name_pin_xy("VCU2_TILE", "VCU2", 0, 0, pin, wire, true);
            }
            let mut bels = vec![];
            let mut bel = self
                .builder
                .bel_xy(bslots::VCU2, "VCU2", 0, 0)
                .pin_name_only("NMU_VCU2_0", 1)
                .pin_name_only("NMU_VCU2_1", 1)
                .pin_name_only("NMU_VCU2_2", 1)
                .pin_name_only("NMU_VCU2_3", 1)
                .pin_name_only("NSU_VCU2", 1)
                .pin_name_only("VCU2NOCSCANCLKN", 1)
                .pin_name_only("VCU2NOCSCANCSSDTRIGN", 1)
                .pin_name_only("VCU2NOCSCANEDTUPDTN", 1)
                .pin_name_only("VCU2NOCSCANENN", 1)
                .pin_name_only("VCU2NOCSCANMODERSTN", 1);
            for i in 0..12 {
                bel = bel
                    .pin_name_only(&format!("NOCVCU2SCANCHNLIN_{i}_"), 1)
                    .pin_name_only(&format!("VCU2NOCSCANCHNLMASKOUT_{i}_"), 1)
                    .pin_name_only(&format!("VCU2NOCSCANCHNLOUT_{i}_"), 1);
            }
            for (bslot, rt, kind, dx, dy) in [
                (bslots::VCU2_NMU[2], 1, "NOC2_NMU128", 0, 0),
                (bslots::VCU2_NMU[3], 2, "NOC2_NMU128", 1, 0),
                (bslots::VCU2_NMU[0], 3, "NOC2_NMU128", 0, 3),
                (bslots::VCU2_NMU[1], 4, "NOC2_NMU128", 1, 3),
                (bslots::VCU2_NSU, 5, "NOC2_NSU128", 0, 7),
                (bslots::VCU2_NOC_SCAN, 6, "NOC2_SCAN", 1, 7),
                (bslots::VCU2_NPS6X[2], 7, "NOC2_NPS6X", 0, 11),
                (bslots::VCU2_NPS6X[3], 8, "NOC2_NPS6X", 1, 11),
                (bslots::VCU2_NPS6X[0], 9, "NOC2_NPS6X", 0, 13),
                (bslots::VCU2_NPS6X[1], 10, "NOC2_NPS6X", 1, 13),
            ] {
                let ncrd = self.builder.delta(noc_xy[dx], 0, dy);
                bels.push(self.bel_name_only(ncrd, bslot, kind, 0, 0).raw_tile(rt));
            }
            bels.push(bel);
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let mut xn = self.builder.xtile_id(tcls::VCU2, "VCU2", xy);
            for dy in [0, 3, 7, 11, 13] {
                let xy_a = xn.builder.delta(noc_xy[0], 0, dy);
                let xy_b = xn.builder.delta(noc_xy[1], 0, dy);
                xn = xn.raw_tile(xy_a).raw_tile(xy_b);
            }
            for i in 0..96 {
                let cxy = xn.builder.delta(int_xy, 1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_e);
            }
            xn.bels(bels).extract();
        }
    }

    pub fn fill_tiles_isp2(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("ISP2_CORE")
            .iter()
            .next()
        {
            let mut noc_xy = xy;
            while !self
                .builder
                .rd
                .tile_kinds
                .key(self.rd.tiles[&noc_xy].kind)
                .starts_with("NOC")
            {
                noc_xy = self.builder.delta(noc_xy, 1, 0);
            }
            for (pin, wire) in [
                ("DPLLISPCLK0", wires::OUT_DPLL[2].cell(0)),
                ("DPLLISPCLK1", wires::OUT_DPLL[3].cell(0)),
            ] {
                self.builder
                    .extra_name_pin_xy("ISP2_CORE", "ISP2", 0, 0, pin, wire, true);
            }
            let mut bels = vec![];
            let mut bel = self
                .builder
                .bel_xy(bslots::ISP2, "ISP2", 0, 0)
                .pin_name_only("NMU_ISP_0", 1)
                .pin_name_only("NMU_ISP_1", 1)
                .pin_name_only("NSU_ISP", 1)
                .pin_name_only("ISP2NOCSCANCLKN", 1)
                .pin_name_only("ISP2NOCSCANCSSDTRIGN", 1)
                .pin_name_only("ISP2NOCSCANEDTUPDTN", 1)
                .pin_name_only("ISP2NOCSCANENN", 1)
                .pin_name_only("ISP2NOCSCANMODERSTN", 1);
            for i in 0..7 {
                bel = bel
                    .pin_name_only(&format!("NOCISP2SCANCHNLIN_{i}_"), 1)
                    .pin_name_only(&format!("ISP2NOCSCANCHNLMASKOUT_{i}_"), 1)
                    .pin_name_only(&format!("ISP2NOCSCANCHNLOUT_{i}_"), 1);
            }
            for (bslot, rt, kind, dx, dy) in [
                (bslots::VCU2_NMU[0], 1, "NOC2_NMU128", 0, 0),
                (bslots::VCU2_NMU[1], 2, "NOC2_NMU128", 1, 0),
                (bslots::VCU2_NSU, 3, "NOC2_NSU128", 0, 3),
                (bslots::VCU2_NOC_SCAN, 4, "NOC2_SCAN", 1, 3),
                (bslots::VCU2_NPS6X[1], 5, "NOC2_NPS6X", 0, 7),
                (bslots::VCU2_NPS6X[0], 6, "NOC2_NPS6X", 1, 7),
            ] {
                let ncrd = self.builder.delta(noc_xy, dx, dy);
                bels.push(self.bel_name_only(ncrd, bslot, kind, 0, 0).raw_tile(rt));
            }
            bels.push(bel);
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let mut xn = self.builder.xtile_id(tcls::ISP2, "ISP2", xy);
            for dy in [0, 3, 7] {
                let xy_a = xn.builder.delta(noc_xy, 0, dy);
                let xy_b = xn.builder.delta(noc_xy, 1, dy);
                xn = xn.raw_tile(xy_a).raw_tile(xy_b);
            }
            for i in 0..48 {
                let cxy = xn.builder.delta(int_xy, 1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_e);
            }
            xn.bels(bels).extract();
        }
    }

    pub fn fill_tiles_hnicx(&mut self) {
        let tkn = "HNICX_TILE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            for (pin, wire) in [
                ("FHX_MAC_STATUS_CLK_OUT", wires::OUT_GT_CLK[0].cell(0)),
                ("TEST_PLL_CLK_OUT_0_", wires::OUT_GT_CLK_S[0].cell(288)),
                ("TEST_PLL_CLK_OUT_1_", wires::OUT_GT_CLK_S[1].cell(288)),
                ("SLV_CLK_HNIC", wires::OUT_GT_CLK_S[2].cell(288)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "HNICX", 0, 0, pin, wire, true);
            }

            let mut bel = self.builder.bel_xy(bslots::HNICX, "HNICX", 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("HNICX", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                if !pin.starts_with("FHX_")
                    && pin != "SLV_CLK_HNIC"
                    && !pin.starts_with("TEST_PLL_")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_TERM_GT");
            let int_xy = self.builder.walk_to_int(xy, Dir::W, false).unwrap();
            let mut xn = self.builder.xtile_id(tcls::HNICX, "HNICX", xy);
            for i in 0..336 {
                let cxy = xn.builder.delta(int_xy, 1, (i + i / 4 + i / 288) as i32);
                xn = xn.ref_single(cxy, i, intf_e);
            }
            xn.bel(bel).extract();
        }
    }
}
