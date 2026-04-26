use prjcombine_interconnect::{
    db::{BelInfo, WireSlotIdExt},
    dir::{Dir, DirH},
};
use prjcombine_re_xilinx_naming_versal::{
    BUFDIV_LEAF_SWZ_A, BUFDIV_LEAF_SWZ_AH, BUFDIV_LEAF_SWZ_B, BUFDIV_LEAF_SWZ_BH,
};
use prjcombine_versal::{
    chip::FabricKind,
    defs::{bslots, tcls, wires},
};

use crate::{
    grid::get_fabric,
    int::{IntMaker, SWZ12, XTileInfoExt},
};

impl<'rd> IntMaker<'rd> {
    fn fill_tiles_rclk(&mut self) {
        for tkn in [
            "RCLK_INT_L_FT",
            "RCLK_INT_R_FT",
            "RCLK_INT_L_VR_FT",
            "RCLK_INT_R_VR_FT",
        ] {
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let int_xy = self.builder.delta(xy, 0, 1);
                let mut int_xy_b = self.builder.delta(xy, 0, -1);
                if self
                    .builder
                    .rd
                    .tile_kinds
                    .key(self.rd.tiles[&int_xy_b].kind)
                    != "INT"
                {
                    int_xy_b = self.builder.delta(int_xy_b, 0, -1);
                    if self
                        .builder
                        .rd
                        .tile_kinds
                        .key(self.rd.tiles[&int_xy_b].kind)
                        != "INT"
                    {
                        continue;
                    }
                }
                self.builder
                    .xtile_id(tcls::RCLK, "RCLK", xy)
                    .ref_int_side(int_xy, DirH::W, 0)
                    .ref_int_side(int_xy, DirH::E, 1)
                    .extract_muxes(bslots::RCLK_INT)
                    .extract();
                break;
            }
        }
    }

    fn fill_tiles_rclk_cle(&mut self) {
        let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK");
        for (tkn, naming, bkind, swz) in [
            (
                "RCLK_CLE_CORE",
                "RCLK_CLE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
            ),
            (
                "RCLK_CLE_VR_CORE",
                "RCLK_CLE_VR",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
            ),
            (
                "RCLK_CLE_LAG_CORE",
                "RCLK_CLE_LAG",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
            ),
        ] {
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR_LOC{i}"),
                    wires::CLK_HDISTR_LOC[i],
                );
            }
            for (i, y) in swz.into_iter().enumerate() {
                self.builder.extra_name_pin_xy(
                    tkn,
                    bkind,
                    0,
                    y as usize,
                    "I",
                    wires::IMUX_BUFDIV_LEAF[i].cell(0),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    bkind,
                    0,
                    y as usize,
                    "O",
                    if i < 16 {
                        wires::GCLK_CLE_S[i].cell(0)
                    } else {
                        wires::GCLK_CLE[i - 16].cell(0)
                    },
                    true,
                );
            }

            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let mut bels = vec![];
                for (i, y) in swz.into_iter().enumerate() {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BUFDIV_LEAF[i], bkind, 0, y as usize)
                        .pin_name_only("O_CASC", 1);
                    if i != 0 {
                        bel = bel.pin_name_only("I_CASC", 0);
                    }
                    bels.push(bel);
                }
                let int_e_xy = self.builder.delta(
                    self.builder
                        .walk_to_int(self.builder.delta(xy, 0, 1), Dir::E, false)
                        .unwrap(),
                    0,
                    -1,
                );
                self.builder
                    .xtile_id(tcls::RCLK_CLE, naming, xy)
                    .ref_xlat(int_e_xy, &[Some(0), None], rclk_int)
                    .bels(bels)
                    .switchbox(bslots::RCLK_INTF_INT)
                    .optin_muxes(&wires::IMUX_BUFDIV_LEAF[..])
                    .extract();
            }
        }
    }

    fn fill_tiles_rclk_intf(&mut self) {
        let rclk_int = self.builder.ndb.get_tile_class_naming("RCLK");

        for (side, naming, tkn, bkind, swz, has_dfx) in [
            (
                DirH::E,
                "DSP",
                "RCLK_DSP_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::W,
                "DSP",
                "RCLK_DSP_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_AH,
                true,
            ),
            (
                DirH::E,
                "DSP_VR",
                "RCLK_DSP_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "DSP_VR",
                "RCLK_DSP_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_BH,
                true,
            ),
            (
                DirH::E,
                "HB",
                "RCLK_HB_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::E,
                "HB_VR",
                "RCLK_HB_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "HB",
                "RCLK_HB_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_AH,
                false,
            ),
            (
                DirH::W,
                "HB_VR",
                "RCLK_HB_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_BH,
                false,
            ),
            (
                DirH::E,
                "SDFEC",
                "RCLK_SDFEC_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "SDFEC",
                "RCLK_SDFEC_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_BH,
                false,
            ),
            (
                DirH::E,
                "HDIO",
                "RCLK_HDIO_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::E,
                "HDIO_VR",
                "RCLK_HDIO_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "HDIO",
                "RCLK_HDIO_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_AH,
                false,
            ),
            (
                DirH::W,
                "HDIO_VR",
                "RCLK_HDIO_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_BH,
                false,
            ),
            (
                DirH::E,
                "HB_HDIO",
                "RCLK_HB_HDIO_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "HB_HDIO_VR",
                "RCLK_HB_HDIO_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "HB_HDIO",
                "RCLK_HB_HDIO_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_BH,
                false,
            ),
            (
                DirH::W,
                "HB_HDIO_VR",
                "RCLK_HB_HDIO_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_BH,
                false,
            ),
            (
                DirH::W,
                "VNOC",
                "RCLK_INTF_L_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "VNOC_VR",
                "RCLK_INTF_L_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "VNOC",
                "RCLK_INTF_R_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "VNOC_VR",
                "RCLK_INTF_R_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "CFRM",
                "RCLK_INTF_OPT_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::W,
                "CFRM_VR",
                "RCLK_INTF_OPT_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "GT",
                "RCLK_INTF_TERM_LEFT_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::W,
                "GT_VR",
                "RCLK_INTF_TERM_LEFT_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "GT",
                "RCLK_INTF_TERM_RIGHT_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                false,
            ),
            (
                DirH::E,
                "GT_VR",
                "RCLK_INTF_TERM_RIGHT_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "GT_ALT",
                "RCLK_INTF_TERM2_RIGHT_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "BRAM",
                "RCLK_BRAM_CORE_MY",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                true,
            ),
            (
                DirH::W,
                "BRAM_VR",
                "RCLK_BRAM_VR_CORE_MY",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::W,
                "URAM",
                "RCLK_URAM_CORE_MY",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                true,
            ),
            (
                DirH::W,
                "URAM_VR",
                "RCLK_URAM_VR_CORE_MY",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::E,
                "BRAM",
                "RCLK_BRAM_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                true,
            ),
            (
                DirH::E,
                "BRAM_VR",
                "RCLK_BRAM_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::E,
                "BRAM_CLKBUF",
                "RCLK_BRAM_CLKBUF_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_A,
                true,
            ),
            (
                DirH::E,
                "BRAM_CLKBUF_VR",
                "RCLK_BRAM_CLKBUF_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::E,
                "BRAM_CLKBUF_NOPD",
                "RCLK_BRAM_CLKBUF_NOPD_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::E,
                "BRAM_CLKBUF_NOPD_VR",
                "RCLK_BRAM_CLKBUF_NOPD_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                true,
            ),
            (
                DirH::W,
                "HB_FULL",
                "RCLK_HB_FULL_R_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "HB_FULL",
                "RCLK_HB_FULL_L_CORE",
                "BUFDIV_LEAF",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::W,
                "HB_FULL_VR",
                "RCLK_HB_FULL_R_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
            (
                DirH::E,
                "HB_FULL_VR",
                "RCLK_HB_FULL_L_VR_CORE",
                "BUFDIV_LEAF_ULVT",
                BUFDIV_LEAF_SWZ_B,
                false,
            ),
        ] {
            self.builder.clear_extra_name_tile(tkn);
            for (i, y) in swz.into_iter().enumerate() {
                self.builder.extra_name_pin_xy(
                    tkn,
                    bkind,
                    0,
                    y as usize,
                    "I",
                    wires::IMUX_BUFDIV_LEAF[i].cell(0),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    bkind,
                    0,
                    y as usize,
                    "O",
                    if i < 16 {
                        wires::GCLK_INTF_S[i].cell(0)
                    } else {
                        wires::GCLK_INTF[i - 16].cell(0)
                    },
                    true,
                );
            }
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR_LOC{i}"),
                    wires::CLK_HDISTR_LOC[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_L_CLK_HDISTR_LOC{i}"),
                    wires::CLK_HDISTR_LOC[i],
                );
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let int_xy = self.builder.delta(
                    self.builder
                        .walk_to_int(self.builder.delta(xy, 0, 1), Dir::H(!side), false)
                        .unwrap(),
                    0,
                    -1,
                );
                if int_xy.x.abs_diff(xy.x) > 5 {
                    continue;
                }
                if self
                    .builder
                    .rd
                    .tile_kinds
                    .key(self.rd.tiles[&self.builder.delta(int_xy, 0, 1)].kind)
                    != "INT"
                {
                    continue;
                }
                let mut bels = vec![];
                for (i, y) in swz.into_iter().enumerate() {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::BUFDIV_LEAF[i], bkind, 0, y as usize)
                        .pin_name_only("O_CASC", 1);
                    if i != 0 {
                        bel = bel.pin_name_only("I_CASC", 0);
                    }
                    bels.push(bel);
                }
                self.builder
                    .xtile_id(tcls::RCLK_INTF, format!("RCLK_INTF_{side}_{naming}"), xy)
                    .ref_xlat(
                        int_xy,
                        &if side == DirH::W {
                            [Some(0), None]
                        } else {
                            [None, Some(0)]
                        },
                        rclk_int,
                    )
                    .bels(bels)
                    .switchbox(bslots::RCLK_INTF_INT)
                    .optin_muxes(&wires::IMUX_BUFDIV_LEAF[..])
                    .extract();
                if has_dfx {
                    let bel = self.builder.bel_xy(bslots::RCLK_DFX_TEST, "RCLK", 0, 0);
                    self.builder
                        .xtile_id(tcls::RCLK_DFX, format!("RCLK_DFX_{naming}"), xy)
                        .ref_xlat(
                            int_xy,
                            &if side == DirH::W {
                                [Some(0), None]
                            } else {
                                [None, Some(0)]
                            },
                            rclk_int,
                        )
                        .bel(bel)
                        .extract();
                }
            }
        }
    }

    fn fill_tiles_rclk_clkbuf(&mut self) {
        for (tkn, tcid, naming, is_pd) in [
            (
                "RCLK_BRAM_CLKBUF_CORE",
                tcls::RCLK_SPLITTER_PD,
                "RCLK_SPLITTER_PD",
                true,
            ),
            (
                "RCLK_BRAM_CLKBUF_VR_CORE",
                tcls::RCLK_SPLITTER_PD_VR,
                "RCLK_SPLITTER_PD_VR",
                true,
            ),
            (
                "RCLK_BRAM_CLKBUF_NOPD_CORE",
                tcls::RCLK_SPLITTER,
                "RCLK_SPLITTER",
                false,
            ),
            (
                "RCLK_BRAM_CLKBUF_NOPD_VR_CORE",
                tcls::RCLK_SPLITTER_VR,
                "RCLK_SPLITTER_VR",
                false,
            ),
        ] {
            let swz = [
                0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2, 12, 15, 16, 17, 18, 19, 20, 21, 22, 23, 13,
                14,
            ];
            for i in 0..24 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("IF_HCLK_L_CLK_HDISTR{i}"),
                    0,
                    wires::CLK_HDISTR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("IF_HCLK_R_CLK_HDISTR{i}"),
                    1,
                    wires::CLK_HDISTR[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_DRVR_TRI_ULVT_{ii}_CLK_OUT_B", ii = swz[i] * 2),
                    0,
                    wires::CLK_HDISTR_VOID[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_DRVR_TRI_ULVT_{ii}_CLK_OUT_B", ii = swz[i] * 2 + 1),
                    1,
                    wires::CLK_HDISTR_VOID[i],
                );
            }
            for i in 0..12 {
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("IF_HCLK_L_CLK_HROUTE{i}"),
                    0,
                    wires::CLK_HROUTE[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("IF_HCLK_R_CLK_HROUTE{i}"),
                    1,
                    wires::CLK_HROUTE[i],
                );
            }
            if is_pd {
                for i in 0..24 {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("IF_HCLK_L_CLK_HDISTR_LOC{i}"),
                        0,
                        wires::CLK_HDISTR_LOC[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("IF_HCLK_R_CLK_HDISTR_LOC{i}"),
                        1,
                        wires::CLK_HDISTR_LOC[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("IF_HCLK_L_CLK_PD{i}"),
                        0,
                        wires::CLK_PD_H_SPLITTER[i],
                    );
                }
            }
            for &xy in self.rd.tiles_by_kind_name(tkn) {
                let mut bels = vec![];
                for i in 0..24 {
                    let mut bel = self.builder.bel_xy(bslots::GCLK_PD_H[i], "GCLK_PD", 0, i);
                    if !is_pd {
                        bel = bel
                            .pin_name_only("CLK_IN0", 1)
                            .pin_name_only("CLK_IN1", 1)
                            .pin_name_only("PD_OUT", 1);
                    }
                    bels.push(bel);
                }
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .bels(bels)
                    .switchbox(bslots::CLK_INT)
                    .optin_muxes(&wires::CLK_HROUTE[..])
                    .optin_muxes(&wires::CLK_HDISTR[..])
                    .optin_muxes(&wires::CLK_HDISTR_VOID[..])
                    .extract();
            }
        }
    }

    pub fn fill_tiles_rclk_vnoc(&mut self) {
        let fabric = get_fabric(self.rd);
        let ulvt = if fabric == FabricKind::Ulvt {
            "_ULVT"
        } else {
            ""
        };
        let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W");
        let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E");
        let misr = self.rd.tile_kinds.get("MISR_TILE").unwrap().0;
        for &tkn in crate::clk::CLK_VNOC_TILE_KINDS {
            let is_ps = tkn.contains("_PSS");
            for i in 0..24 {
                let ii = [
                    0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13,
                    14, 15,
                ][i];
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{ii}_CLK_IN24"),
                    0,
                    wires::CLK_HDISTR_TEST[i],
                );
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{ii}_CLK_IN25"),
                    48,
                    wires::CLK_HDISTR_TEST[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{ii}_CLK_OUT"),
                    wires::CLK_VROUTE_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                    wires::CLK_VROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                    wires::CLK_VDISTR[i],
                );
                let ii = [
                    0, 3, 8, 9, 10, 11, 13, 14, 15, 16, 1, 12, 17, 18, 19, 20, 21, 22, 23, 2, 4, 5,
                    6, 7,
                ][i];
                self.builder.extra_name_tile(
                    tkn,
                    format!("VCC_WIRE{ii}"),
                    wires::CLK_VROUTE_MUX_DUMMY[i],
                );
            }
            for i in 0..12 {
                if is_ps {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_BUFG_PS{ulvt}_{ii}_I", ii = 25 + i),
                        0,
                        wires::CLK_HROUTE[i],
                    );
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_BUFG_PS{ulvt}_{ii}_O", ii = 25 + i),
                        0,
                        wires::OUT_BUFG_PS[i],
                    );
                } else {
                    self.builder.extra_name_tile_cell(
                        tkn,
                        format!("CLK_HROUTE_L{i}"),
                        0,
                        wires::CLK_HROUTE[i],
                    );
                }
                self.builder.extra_name_tile_cell(
                    tkn,
                    format!("CLK_HROUTE_R{i}"),
                    48,
                    wires::CLK_HROUTE[i],
                );
            }
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut misr_xy = xy;
                while self.rd.tiles[&misr_xy].kind != misr {
                    misr_xy = self.builder.delta(misr_xy, 1, 0);
                }
                let mut bels = vec![];
                let tcid = if is_ps {
                    tcls::RCLK_VNOC_PS
                } else {
                    tcls::RCLK_VNOC
                };
                let naming = match fabric {
                    FabricKind::Orig | FabricKind::Rev2 => {
                        if is_ps {
                            "RCLK_VNOC_PS"
                        } else {
                            "RCLK_VNOC"
                        }
                    }
                    FabricKind::Ssit => {
                        if is_ps {
                            "RCLK_VNOC_PS_SSIT"
                        } else {
                            "RCLK_VNOC_SSIT"
                        }
                    }
                    FabricKind::Ssit2 => {
                        if is_ps {
                            "RCLK_VNOC_PS_SSIT2"
                        } else {
                            "RCLK_VNOC_SSIT2"
                        }
                    }
                    FabricKind::Ulvt => {
                        if is_ps {
                            "RCLK_VNOC_PS_ULVT"
                        } else {
                            "RCLK_VNOC_ULVT"
                        }
                    }
                };
                for i in 0..24 {
                    let slot = if fabric == FabricKind::Ulvt {
                        "BUFG_FABRIC_ULVT"
                    } else {
                        "BUFG_FABRIC"
                    };
                    self.builder.extra_name_pin_xy(
                        tkn,
                        slot,
                        0,
                        i,
                        "O",
                        wires::OUT_BUFG_FABRIC[i].cell(0),
                        true,
                    );
                    bels.push(self.builder.bel_xy(bslots::BUFG_FABRIC[i], slot, 0, i));
                }
                for i in 0..48 {
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::GCLK_TEST[i],
                                if fabric == FabricKind::Ulvt {
                                    "GCLK_TEST_ULVT"
                                } else {
                                    "GCLK_TEST"
                                },
                                0,
                                (i % 24) * 2 + (i / 24),
                            )
                            .pin_rename("CLKIN", "I")
                            .pin_rename("CLKOUT", "O"),
                    );
                }
                for i in 0..24 {
                    self.builder.extra_name_pin_xy(
                        tkn,
                        "GCLK_TEST_RING",
                        0,
                        i,
                        "CLKOUT",
                        wires::CLK_HDISTR_TEST_RING[i].cell(0),
                        false,
                    );
                    bels.push(
                        self.builder
                            .bel_xy(bslots::GCLK_TEST_RING_HDISTR[i], "GCLK_TEST_RING", 0, i)
                            .pin_rename("CLKIN", "I")
                            .pin_rename("CLKOUT", "O"),
                    );
                }
                for i in 0..12 {
                    self.builder.extra_name_pin_xy(
                        tkn,
                        "GCLK_TEST_RING",
                        0,
                        i + 24,
                        "CLKOUT",
                        wires::CLK_HROUTE_TEST_RING[i].cell(0),
                        false,
                    );
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::GCLK_TEST_RING_HROUTE[i],
                                "GCLK_TEST_RING",
                                0,
                                i + 24,
                            )
                            .pin_rename("CLKIN", "I")
                            .pin_rename("CLKOUT", "O"),
                    );
                }
                for i in 0..24 {
                    bels.push(self.builder.bel_xy(bslots::GCLK_PD_H[i], "GCLK_PD", 0, i));
                }

                for i in 0..48 {
                    let slot = match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "GCLK_DELAY",
                        FabricKind::Ssit => "GCLK_DELAY_SSIT",
                        FabricKind::Ssit2 | FabricKind::Ulvt => "GCLK_DELAY_SSIT2",
                    };
                    for (pin, wire) in [
                        ("I", wires::CLK_HDISTR[i % 24].cell(i / 24 * 48)),
                        ("O", wires::CLK_HDISTR_LOC_DRV[i % 24].cell(i / 24 * 48)),
                        ("PD_BOT", wires::CLK_PD_V_S[i % 24].cell(i / 24 * 48)),
                        ("PD_TOP", wires::CLK_PD_V[i % 24].cell(i / 24 * 48)),
                        (
                            "PD_LEFT",
                            if i < 24 {
                                wires::CLK_PD_H_SPLITTER[i % 24].cell(0)
                            } else {
                                wires::CLK_PD_H_VNOC[i % 24].cell(0)
                            },
                        ),
                        (
                            "PD_RIGHT",
                            if i < 24 {
                                wires::CLK_PD_H_VNOC[i % 24].cell(0)
                            } else {
                                wires::CLK_PD_H_SPLITTER[i % 24].cell(48)
                            },
                        ),
                    ] {
                        self.builder
                            .extra_name_pin_xy(tkn, slot, 0, i, pin, wire, true);
                    }
                    let is_ssit2 = matches!(fabric, FabricKind::Ssit2 | FabricKind::Ulvt);
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::GCLK_DELAY[i], slot, 0, i)
                        .manual()
                        .pin_name_only("EN_TAPS", 0)
                        .pin_name_only("LOAD", 0);
                    let num_taps = if is_ssit2 { 12 } else { 9 };
                    for j in 0..num_taps {
                        if i % 24 == 0 {
                            bel = bel.pin_name_only(&format!("TAPS_IN_{j}_"), 1);
                        } else {
                            bel = bel.pin_name_only(&format!("TAPS_IN_{j}_"), 0);
                        }
                        if i % 24 != 23 || j >= 9 {
                            bel = bel.pin_name_only(&format!("TAPS_{j}_"), 1);
                        }
                    }
                    if i % 24 == 0 {
                        bel = bel.pin_name_only("LOCK_IN", 1);
                    } else {
                        bel = bel.pin_name_only("LOCK_IN", 0);
                    }
                    if i % 24 != 23 {
                        bel = bel.pin_name_only("LOCK", 1);
                    }
                    bels.push(bel);
                }

                let slot = if is_ps {
                    "GCLK_TAPS_DECODE_VNOC_PS"
                } else {
                    "GCLK_TAPS_DECODE_VNOC"
                };
                let mut bel = self
                    .builder
                    .bel_xy(bslots::GCLK_TAPS_DECODE_VNOC, slot, 0, 0)
                    .manual()
                    .pin_rename("LOAD_EN_INT", "LOAD")
                    .pin_name_only("LOAD_EN_L", 1)
                    .pin_name_only("LOAD_EN_R", 1);
                for i in 0..24 {
                    bel = bel
                        .pin_name_only(&format!("EN_TAPS_L_{i}_"), 1)
                        .pin_name_only(&format!("EN_TAPS_R_{i}_"), 1);
                }
                bels.push(bel);

                if is_ps {
                    let slot = if fabric == FabricKind::Ulvt {
                        "BUFG_PS_ULVT"
                    } else {
                        "BUFG_PS"
                    };
                    for i in 0..12 {
                        bels.push(self.builder.bel_xy(bslots::BUFG_PS[i], slot, 0, i));
                    }
                }

                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .force_ext_pips()
                    .switchbox(bslots::CLK_INT)
                    .optin_muxes(&wires::CLK_VROUTE[..])
                    .optin_muxes(&wires::CLK_VROUTE_MUX[..])
                    .optin_muxes(&wires::CLK_HROUTE[..])
                    .optin_muxes(&wires::CLK_HDISTR[..])
                    .bels(bels);
                for i in 0..48 {
                    let intf_w_xy = xn.builder.delta(xy, -1, 1 + (i + i / 4) as i32);
                    let intf_e_xy = xn.builder.delta(misr_xy, 1, 1 + (i + i / 4) as i32);

                    xn = xn
                        .ref_single(intf_w_xy, i, intf_w)
                        .ref_single(intf_e_xy, i + 48, intf_e)
                }
                let mut xt = xn.extract();
                let (mut bel_dec, mut bn_dec) = xt.bels.pop().unwrap();
                for i in 0..9 {
                    let pname = &format!("TAPS_{i}_");
                    for (j, dpname) in
                        [(23, &format!("TAPS_L_{i}_")), (47, &format!("TAPS_R_{i}_"))]
                    {
                        let pin = xt.bels[j].0.pins.remove(pname).unwrap();
                        bel_dec.pins.insert(dpname.clone(), pin);
                        let pn = xt.bels[j].1.pins[pname].clone();
                        bn_dec.pins.insert(dpname.clone(), pn);
                    }
                }
                for (j, dpname) in [(23, "LOCK_L"), (47, "LOCK_R")] {
                    let pin = xt.bels[j].0.pins.remove("LOCK").unwrap();
                    bel_dec.pins.insert(dpname.into(), pin);
                    let pn = xt.bels[j].1.pins["LOCK"].clone();
                    bn_dec.pins.insert(dpname.into(), pn);
                }
                for pname in ["INC", "DEC"] {
                    for i in 0..48 {
                        let pin = xt.bels[i].0.pins.remove(pname).unwrap();
                        let mut pn = xt.bels[i].1.pins[pname].clone();
                        pn.pips.clear();
                        pn.name = pn.name_far.clone();
                        if i == 0 {
                            bel_dec.pins.insert(pname.into(), pin);
                            bn_dec.pins.insert(pname.into(), pn);
                        } else {
                            assert_eq!(bel_dec.pins[pname], pin);
                            assert_eq!(bn_dec.pins[pname], pn);
                        }
                    }
                }

                self.builder.insert_tcls_bel(
                    tcid,
                    bslots::GCLK_TAPS_DECODE_VNOC,
                    BelInfo::Legacy(bel_dec),
                );
                self.builder
                    .insert_bel_naming(naming, bslots::GCLK_TAPS_DECODE_VNOC, bn_dec);
                for (i, (bel, bn)) in xt.bels.into_iter().enumerate() {
                    self.builder
                        .insert_tcls_bel(tcid, bslots::GCLK_DELAY[i], BelInfo::Legacy(bel));
                    self.builder
                        .insert_bel_naming(naming, bslots::GCLK_DELAY[i], bn);
                }
            }
        }
    }

    pub fn fill_tiles_rclk_gt(&mut self) {
        let fabric = get_fabric(self.rd);
        let ulvt = if fabric == FabricKind::Ulvt {
            "_ULVT"
        } else {
            ""
        };
        for &tkn in crate::clk::CLK_GT_TILE_KINDS {
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_36_ENC_ULVT_0_CLK_OUT",
                wires::IMUX_DPLL_CLKIN_S,
            );
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_36_ENC_ULVT_1_CLK_OUT",
                wires::IMUX_DPLL_CLKIN,
            );
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_8TO1_ULVT_0_CLK_OUT",
                wires::IMUX_DPLL_CLKIN_DESKEW_S,
            );
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_8TO1_ULVT_1_CLK_OUT",
                wires::IMUX_DPLL_CLKIN_DESKEW,
            );
            self.builder
                .extra_name_tile(tkn, "DPLL_CLKIN_BOT", wires::IMUX_DPLL_CLKIN_S);
            self.builder
                .extra_name_tile(tkn, "DPLL_CLKIN_TOP", wires::IMUX_DPLL_CLKIN);
            self.builder.extra_name_tile(
                tkn,
                "DPLL_CLKIN_DESKEW_BOT",
                wires::IMUX_DPLL_CLKIN_DESKEW_S,
            );
            self.builder.extra_name_tile(
                tkn,
                "DPLL_CLKIN_DESKEW_TOP",
                wires::IMUX_DPLL_CLKIN_DESKEW,
            );
            self.builder
                .extra_name_tile(tkn, "CPM_CLK_IN0", wires::OUT_CPM_CLK[0]);
            self.builder
                .extra_name_tile(tkn, "CPM_CLK_IN1", wires::OUT_CPM_CLK[1]);
            for i in 0..12 {
                self.builder
                    .extra_name_tile(tkn, format!("CLK_HROUTE{i}"), wires::CLK_HROUTE[i]);
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{i}_CLK_OUT"),
                    wires::CLK_HROUTE_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{i}_CLK_IN26"),
                    wires::CLK_HDISTR_TEST[2 * i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_28_ENC{ulvt}_{i}_CLK_IN27"),
                    wires::CLK_HDISTR_TEST[2 * i + 1],
                );
            }
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                    wires::CLK_VROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                    wires::CLK_VDISTR_S[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR{i}"),
                    wires::CLK_VDISTR[i],
                );

                let ii = [
                    0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2, 12, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                    13, 14,
                ][i];
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_OUT"),
                    wires::CLK_VROUTE_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_DRVR_TRI_ULVT_{iii}_CLK_OUT_B", iii = 2 * i + 1),
                    wires::CLK_VROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_IN1"),
                    wires::CLK_HROUTE_TEST[i],
                );

                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_2TO1{ulvt}_{ii}_CLK_OUT"),
                    wires::CLK_HDISTR_MUX[i],
                );
            }
            let side = if tkn.ends_with("_MY") {
                DirH::W
            } else {
                DirH::E
            };
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self
                    .builder
                    .walk_to_int(self.builder.delta(xy, 0, 1), (!side).into(), false)
                    .unwrap();
                let intf = self.builder.ndb.get_tile_class_naming(match side {
                    DirH::W => "INTF_W_TERM_GT",
                    DirH::E => "INTF_E_TERM_GT",
                });
                let intf_xy = self.builder.delta(
                    int_xy,
                    match side {
                        DirH::W => -1,
                        DirH::E => 1,
                    },
                    0,
                );
                let (tcid, naming) = if tkn.contains("NO_LOC") {
                    (tcls::RCLK_GT, "RCLK_GT_NO_LOC_ULVT")
                } else if tkn == "CLKE2_GT_ULVT_TILE_MY" {
                    (tcls::RCLK_GT_CPM6, "RCLK_GT_CPM6_ULVT")
                } else {
                    match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => (tcls::RCLK_GT, "RCLK_GT"),
                        FabricKind::Ssit => (tcls::RCLK_GT, "RCLK_GT_SSIT"),
                        FabricKind::Ssit2 => (tcls::RCLK_GT, "RCLK_GT_SSIT2"),
                        FabricKind::Ulvt => (tcls::RCLK_GT, "RCLK_GT_ULVT"),
                    }
                };
                let mut bels = vec![];
                for i in 0..24 {
                    let slot = match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "GCLK_DELAY",
                        FabricKind::Ssit => "GCLK_DELAY_SSIT",
                        FabricKind::Ssit2 | FabricKind::Ulvt => "GCLK_DELAY_SSIT2",
                    };
                    for (pin, wire) in [
                        (
                            "I",
                            if matches!(tcid, tcls::RCLK_GT_CPM6) {
                                wires::CLK_HDISTR_VOID[i].cell(0)
                            } else {
                                wires::CLK_HDISTR[i].cell(0)
                            },
                        ),
                        ("O", wires::CLK_HDISTR_LOC_DRV[i].cell(0)),
                        ("PD_BOT", wires::CLK_PD_V_S[i].cell(0)),
                        ("PD_TOP", wires::CLK_PD_V[i].cell(0)),
                        ("PD_RIGHT", wires::CLK_PD_H_SPLITTER[i].cell(0)),
                    ] {
                        self.builder
                            .extra_name_pin_xy(tkn, slot, 0, i, pin, wire, true);
                    }
                    let is_ssit2 = matches!(fabric, FabricKind::Ssit2 | FabricKind::Ulvt);
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::GCLK_DELAY[i], slot, 0, i)
                        .manual()
                        .pin_name_only("EN_TAPS", 0)
                        .pin_name_only("LOAD", 0)
                        .pin_name_only("PD_LEFT", 1);
                    if tcid == tcls::RCLK_GT_CPM6 {
                        bel = bel
                            .pin_name_only("PD_RIGHT", 1)
                            .pin_name_only("PD_BOT", 1)
                            .pin_name_only("PD_TOP", 1);
                    }
                    let num_taps = if is_ssit2 { 12 } else { 9 };
                    for j in 0..num_taps {
                        if i == 0 {
                            bel = bel.pin_name_only(&format!("TAPS_IN_{j}_"), 1);
                        } else {
                            bel = bel.pin_name_only(&format!("TAPS_IN_{j}_"), 0);
                        }
                        if i != 23 || j >= 9 {
                            bel = bel.pin_name_only(&format!("TAPS_{j}_"), 1);
                        }
                    }
                    if i == 0 {
                        bel = bel.pin_name_only("LOCK_IN", 1);
                    } else {
                        bel = bel.pin_name_only("LOCK_IN", 0);
                    }
                    if i != 23 {
                        bel = bel.pin_name_only("LOCK", 1);
                    }
                    bels.push(bel);
                }
                let mut bel = self
                    .builder
                    .bel_xy(bslots::GCLK_TAPS_DECODE_GT, "GCLK_TAPS_DECODE_GT", 0, 0)
                    .manual()
                    .pin_rename("LOAD_EN_INT", "LOAD")
                    .pin_name_only("LOAD_EN", 1);
                for i in 0..24 {
                    bel = bel.pin_name_only(&format!("EN_TAPS_{i}_"), 1);
                }
                bels.push(bel);

                for i in 0..48 {
                    bels.push(
                        self.builder
                            .bel_xy(
                                bslots::GCLK_TEST[i],
                                if fabric == FabricKind::Ulvt {
                                    "GCLK_TEST_ULVT"
                                } else {
                                    "GCLK_TEST"
                                },
                                0,
                                (i % 24) * 2 + (i / 24),
                            )
                            .pin_rename("CLKIN", "I")
                            .pin_rename("CLKOUT", "O"),
                    );
                }

                for i in 0..24 {
                    let slot = if fabric == FabricKind::Ulvt {
                        "BUFG_GT_ULVT"
                    } else {
                        "BUFG_GT"
                    };
                    for (pin, wire) in [
                        ("I", wires::IMUX_BUFG_GT_I[i]),
                        ("CE", wires::IMUX_BUFG_GT_CE[i]),
                        ("CLR", wires::IMUX_BUFG_GT_CLR[i]),
                        ("O", wires::OUT_BUFG_GT[i]),
                    ] {
                        self.builder
                            .extra_name_pin_xy(tkn, slot, 0, i, pin, wire.cell(0), true);
                    }
                    bels.push(self.builder.bel_xy(bslots::BUFG_GT[i], slot, 0, i));
                }
                for i in 0..41 {
                    if i != 40 {
                        let wire = match i {
                            0..5 => wires::OUT_DPLL_GT_S[i],
                            5..20 => wires::OUT_GT_CLK_S[i - 5],
                            20..25 => wires::OUT_DPLL_GT[i - 20],
                            25..40 => wires::OUT_GT_CLK[i - 25],
                            _ => unreachable!(),
                        };
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "BUFG_GT_SYNC",
                            0,
                            i,
                            "CLK",
                            wire.cell(0),
                            true,
                        );
                    }
                    for (pin, wire) in [
                        ("CESYNC", wires::OUT_BUFG_GT_SYNC_CE[i]),
                        ("CLRSYNC", wires::OUT_BUFG_GT_SYNC_CLR[i]),
                    ] {
                        self.builder.extra_name_pin_xy(
                            tkn,
                            "BUFG_GT_SYNC",
                            0,
                            i,
                            pin,
                            wire.cell(0),
                            true,
                        );
                    }
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFG_GT_SYNC[i], "BUFG_GT_SYNC", 0, i),
                    );
                }
                let mut d = self.builder.add_dummies(tkn);

                d.insert(
                    wires::IMUX_DPLL_CLKIN.cell(0),
                    true,
                    &Vec::from_iter(wires::IMUX_DPLL_CLKIN_DUMMY.into_iter().map(|w| w.cell(0))),
                )
                .insert(
                    wires::IMUX_DPLL_CLKIN_S.cell(0),
                    true,
                    &Vec::from_iter(
                        wires::IMUX_DPLL_CLKIN_S_DUMMY
                            .into_iter()
                            .map(|w| w.cell(0)),
                    ),
                )
                .insert(
                    wires::IMUX_DPLL_CLKIN_DESKEW.cell(0),
                    true,
                    &Vec::from_iter(
                        wires::IMUX_DPLL_CLKIN_DESKEW_DUMMY
                            .into_iter()
                            .map(|w| w.cell(0)),
                    ),
                )
                .insert(
                    wires::IMUX_DPLL_CLKIN_DESKEW_S.cell(0),
                    true,
                    &Vec::from_iter(
                        wires::IMUX_DPLL_CLKIN_DESKEW_S_DUMMY
                            .into_iter()
                            .map(|w| w.cell(0)),
                    ),
                );
                for i in 0..24 {
                    d.insert(
                        wires::IMUX_BUFG_GT_CE[i].cell(0),
                        true,
                        &[wires::IMUX_BUFG_GT_CE_DUMMY[i].cell(0)],
                    );
                    d.insert(
                        wires::IMUX_BUFG_GT_CLR[i].cell(0),
                        false,
                        &[wires::IMUX_BUFG_GT_CLR_DUMMY[i].cell(0)],
                    );
                }
                if tcid == tcls::RCLK_GT_CPM6 {
                    d.insert(
                        wires::IMUX_DPLL_CLKIN.cell(0),
                        false,
                        &Vec::from_iter(
                            wires::IMUX_DPLL_CLKIN_DUMMY_GND
                                .into_iter()
                                .map(|w| w.cell(0)),
                        ),
                    )
                    .insert(
                        wires::IMUX_DPLL_CLKIN_S.cell(0),
                        false,
                        &Vec::from_iter(
                            wires::IMUX_DPLL_CLKIN_S_DUMMY_GND
                                .into_iter()
                                .map(|w| w.cell(0)),
                        ),
                    );
                    for i in 0..24 {
                        d.insert(
                            wires::CLK_HDISTR_MUX[i].cell(0),
                            false,
                            &[
                                wires::CLK_HDISTR_MUX_DUMMY0[i].cell(0),
                                wires::CLK_HDISTR_MUX_DUMMY1[i].cell(0),
                            ],
                        );
                    }
                }

                let mut xn = self
                    .builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::CLK_INT)
                    .optin_muxes(&wires::CLK_HDISTR[..])
                    .optin_muxes(&wires::CLK_HDISTR_VOID[..])
                    .optin_muxes(&wires::CLK_HDISTR_MUX[..])
                    .optin_muxes(&wires::CLK_HROUTE[..])
                    .optin_muxes(&wires::CLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::CLK_VROUTE[..])
                    .optin_muxes(&wires::CLK_VROUTE_MUX[..])
                    .optin_muxes(&wires::IMUX_BUFG_GT_I[..])
                    .optin_muxes(&wires::IMUX_BUFG_GT_CE[..])
                    .optin_muxes(&wires::IMUX_BUFG_GT_CLR[..])
                    .optin_muxes(&[
                        wires::IMUX_DPLL_CLKIN,
                        wires::IMUX_DPLL_CLKIN_S,
                        wires::IMUX_DPLL_CLKIN_DESKEW,
                        wires::IMUX_DPLL_CLKIN_DESKEW_S,
                    ])
                    .bels(bels);
                for i in 0..48 {
                    let cxy = xn.builder.delta(intf_xy, 0, (i + i / 4) as i32);
                    xn = xn.ref_single(cxy, i, intf);
                }
                let mut xt = xn.extract();
                let (mut bel_dec, mut bn_dec) = xt.bels.pop().unwrap();
                for i in 0..9 {
                    let pname = &format!("TAPS_{i}_");
                    let pin = xt.bels[23].0.pins.remove(pname).unwrap();
                    bel_dec.pins.insert(pname.clone(), pin);
                    let pn = xt.bels[23].1.pins[pname].clone();
                    bn_dec.pins.insert(pname.clone(), pn);
                }
                let pin = xt.bels[23].0.pins.remove("LOCK").unwrap();
                bel_dec.pins.insert("LOCK".into(), pin);
                let pn = xt.bels[23].1.pins["LOCK"].clone();
                bn_dec.pins.insert("LOCK".into(), pn);
                for pname in ["INC", "DEC"] {
                    for i in 0..24 {
                        let pin = xt.bels[i].0.pins.remove(pname).unwrap();
                        let mut pn = xt.bels[i].1.pins[pname].clone();
                        pn.pips.clear();
                        pn.name = pn.name_far.clone();
                        if i == 0 {
                            bel_dec.pins.insert(pname.into(), pin);
                            bn_dec.pins.insert(pname.into(), pn);
                        } else {
                            assert_eq!(bel_dec.pins[pname], pin);
                            assert_eq!(bn_dec.pins[pname], pn);
                        }
                    }
                }

                self.builder.insert_tcls_bel(
                    tcid,
                    bslots::GCLK_TAPS_DECODE_GT,
                    BelInfo::Legacy(bel_dec),
                );
                self.builder
                    .insert_bel_naming(naming, bslots::GCLK_TAPS_DECODE_GT, bn_dec);
                for (i, (bel, bn)) in xt.bels.into_iter().enumerate() {
                    self.builder
                        .insert_tcls_bel(tcid, bslots::GCLK_DELAY[i], BelInfo::Legacy(bel));
                    self.builder
                        .insert_bel_naming(naming, bslots::GCLK_DELAY[i], bn);
                }
            }
        }
    }

    pub fn fill_tiles_rclk_ssit(&mut self) {
        let tkn = "RCLK_CIDB_CORE";
        let tcid = tcls::RCLK_SPLITTER_SSIT;
        let naming = "RCLK_SPLITTER_SSIT";
        for i in 0..24 {
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_L_CLK_HDISTR{i}"),
                wires::CLK_HDISTR[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_SLR_CLK_HDISTR{i}"),
                wires::CLK_HDISTR_SSIT2_H[i],
            );
        }
        for i in 0..12 {
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_L_CLK_HROUTE{i}"),
                wires::CLK_HROUTE[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_SLR_CLK_HROUTE{i}"),
                wires::CLK_HROUTE_SSIT2_H[i],
            );
        }
        for i in 0..24 {
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_L_CLK_HDISTR_LOC{i}"),
                wires::CLK_HDISTR_LOC[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_SLR_CLK_HDISTR_LOC{i}"),
                wires::CLK_HDISTR_LOC_SSIT2_H[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("IF_HCLK_L_CLK_PD{i}"),
                wires::CLK_PD_H_SPLITTER[i],
            );
            let ii = [
                0, 9, 12, 15, 18, 21, 24, 27, 30, 33, 3, 6, 36, 45, 48, 51, 54, 57, 60, 63, 66, 69,
                39, 42,
            ][i];
            self.builder.extra_name_tile(
                tkn,
                format!("CLKE2_DRIVER_ESD_{ii}_CLK_IN"),
                wires::OUT_GCLK_PD[i],
            );
            self.builder.extra_name_tile(
                tkn,
                format!("CLKE2_DRIVER_ESD_{ii}_CLK_B_UBUMP", ii = ii + 1),
                wires::CLK_HDISTR_LOC_SSIT2_LOOP[i],
            );
        }
        for &xy in self.rd.tiles_by_kind_name(tkn) {
            let mut bels = vec![];
            for i in 0..24 {
                bels.push(self.builder.bel_xy(bslots::GCLK_PD_H[i], "GCLK_PD", 0, i));
            }
            self.builder
                .xtile_id(tcid, naming, xy)
                .bels(bels)
                .switchbox(bslots::CLK_INT)
                .optin_muxes(&wires::CLK_HROUTE[..])
                .optin_muxes(&wires::CLK_HDISTR[..])
                .optin_muxes(&wires::CLK_HROUTE_SSIT2_H[..])
                .optin_muxes(&wires::CLK_HDISTR_SSIT2_H[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_H[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_LOOP[..])
                .optin_muxes(&wires::CLK_PD_H_SPLITTER[..])
                .extract();
        }
    }

    pub fn fill_tiles_clk_v_buf_vnoc(&mut self) {
        let fabric = get_fabric(self.rd);
        let ulvt = if fabric == FabricKind::Ulvt {
            "_ULVT"
        } else {
            ""
        };
        for &tkn in crate::clk::CLK_VNOC_REBUF_TILE_KINDS {
            let (tcid, naming) = if tkn.contains("_EDGE") {
                (tcls::CLK_V_BUF_VNOC_SSIT2_EDGE, "CLK_V_BUF_VNOC_SSIT2_EDGE")
            } else if tkn.contains("_SSIT_HSR") {
                (
                    tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S,
                    "CLK_V_BUF_VNOC_SSIT_HNOC_S",
                )
            } else if tkn.contains("_SSIT_BOT") {
                (tcls::CLK_V_BUF_VNOC_SSIT_S, "CLK_V_BUF_VNOC_SSIT_S")
            } else if tkn.contains("_SSIT_MB") {
                (tcls::CLK_V_BUF_VNOC_SSIT_MID_S, "CLK_V_BUF_VNOC_SSIT_MID_S")
            } else if tkn.contains("_SSIT_MT") {
                (tcls::CLK_V_BUF_VNOC_SSIT_MID_N, "CLK_V_BUF_VNOC_SSIT_MID_N")
            } else if tkn.contains("_SSIT_TOP") {
                (tcls::CLK_V_BUF_VNOC_SSIT_N, "CLK_V_BUF_VNOC_SSIT_N")
            } else if tkn.contains("_SSIT_HBM") {
                (tcls::CLK_V_BUF_VNOC_SSIT_N, "CLK_V_BUF_VNOC_SSIT_N_HBM")
            } else if tkn.contains("_SSIT_AIE") {
                (tcls::CLK_V_BUF_VNOC_SSIT_N, "CLK_V_BUF_VNOC_SSIT_N_AIE")
            } else if tkn.contains("_TOP") {
                (
                    tcls::CLK_V_BUF_VNOC,
                    match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "CLK_V_BUF_VNOC_TOP",
                        FabricKind::Ssit => "CLK_V_BUF_VNOC_TOP_SSIT",
                        FabricKind::Ssit2 => "CLK_V_BUF_VNOC_TOP_SSIT2",
                        FabricKind::Ulvt => "CLK_V_BUF_VNOC_TOP_ULVT",
                    },
                )
            } else {
                (
                    tcls::CLK_V_BUF_VNOC,
                    match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "CLK_V_BUF_VNOC",
                        FabricKind::Ssit => "CLK_V_BUF_VNOC_SSIT",
                        FabricKind::Ssit2 => "CLK_V_BUF_VNOC_SSIT2",
                        FabricKind::Ulvt => "CLK_V_BUF_VNOC_ULVT",
                    },
                )
            };
            if tkn.contains("_EDGE") {
                continue;
            }
            const SWZ: [usize; 24] = [
                0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14,
                15,
            ];
            for i in 0..24 {
                for (name, wire) in [
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                        wires::CLK_VDISTR[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV1_{i}"),
                        wires::CLK_VDISTR_LEV1[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV2_{i}"),
                        wires::CLK_VDISTR_LEV2[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV21_{i}"),
                        wires::CLK_VDISTR_LEV1[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV3_{i}"),
                        wires::CLK_VDISTR_LEV2[i].cell(0),
                    ),
                    (
                        format!("CLK_VDISTR_LEV3_BOT{i}"),
                        wires::CLK_VDISTR_LEV3[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                        wires::CLK_VROUTE[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR{i}"),
                        wires::CLK_VDISTR_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV1_{i}"),
                        wires::CLK_VDISTR_LEV1_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV2_{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV21_{i}"),
                        wires::CLK_VDISTR_LEV1_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV3_{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    ),
                    (
                        format!("CLK_VDISTR_LEV3_TOP{i}"),
                        wires::CLK_VDISTR_LEV3_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VROUTE{i}"),
                        wires::CLK_VROUTE_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_L{i}"),
                        wires::CLK_HDISTR_LOC_DRV[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_R{i}"),
                        wires::CLK_HDISTR_LOC_DRV[i].cell(1),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_HDISTR_LOC_L{i}"),
                        wires::CLK_HDISTR_LOC_DRV_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_HDISTR_LOC_R{i}"),
                        wires::CLK_HDISTR_LOC_DRV_N[i].cell(1),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_PD_L{i}"),
                        wires::CLK_PD_V[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_PD_R{i}"),
                        wires::CLK_PD_V[i].cell(1),
                    ),
                    (
                        format!("IF_CLK2HSR_HSR_CLK_HDISTR{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    ),
                    (
                        format!("IF_CLK2HSR_HSR_CLK_HROUTE{i}"),
                        wires::CLK_VROUTE_N[i].cell(0),
                    ),
                ] {
                    self.builder.extra_name_tile_tw(tkn, name, wire);
                }
                if tcid == tcls::CLK_V_BUF_VNOC {
                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i]),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i] + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 48 + 2 * SWZ[i]),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!(
                                "CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT",
                                ii = 48 + 2 * SWZ[i] + 1
                            ),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i]),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i] + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_DRVR_TRI{ulvt}_{ii}_CLK_OUT_B", ii = 2 * SWZ[i] + 1),
                            wires::CLK_VDISTR_N[i].cell(0),
                        ),
                        (
                            format!(
                                "CLK_CMT_DRVR_TRI{ulvt}_{ii}_CLK_OUT_B",
                                ii = 48 + 2 * SWZ[i] + 1
                            ),
                            wires::CLK_VDISTR_LEV1_N[i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if tcid == tcls::CLK_V_BUF_VNOC_SSIT_N {
                    for (name, wire) in [
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE{i}"),
                            wires::CLK_VROUTE_N[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED{i}"),
                            wires::CLK_VDISTR_LEV2_N[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_SHARED{i}"),
                            wires::CLK_VDISTR_LEV2_N[i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if tcid == tcls::CLK_V_BUF_VNOC_SSIT_S {
                    for (name, wire) in [
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE{i}"),
                            wires::CLK_VROUTE[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED{i}"),
                            wires::CLK_VDISTR_LEV2[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_L{i}"),
                            wires::CLK_HDISTR_LOC_DRV_S[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_R{i}"),
                            wires::CLK_HDISTR_LOC_DRV_S[i].cell(1),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_PD_L{i}"),
                            wires::CLK_PD_V[i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_PD_R{i}"),
                            wires::CLK_PD_V[i].cell(1),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                const SWZ12: [usize; 12] = [0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2];
                if matches!(
                    tcid,
                    tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S
                        | tcls::CLK_V_BUF_VNOC_SSIT_MID_S
                        | tcls::CLK_V_BUF_VNOC_SSIT_MID_N
                ) {
                    let vdi = (SWZ12[i / 2] + i % 2 * 12) * 4;
                    let vd1i = 96 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd2i = 144 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd1mi = 48 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd2mi = 96 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_PRE_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 1),
                            wires::CLK_VDISTR_N_PRE_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 2),
                            wires::CLK_VDISTR_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 3),
                            wires::CLK_VDISTR_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 2),
                            wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 3),
                            wires::CLK_VDISTR_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd1i),
                            wires::CLK_VDISTR_LEV1_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd1i + 1),
                            wires::CLK_VDISTR_LEV1_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd1i),
                            wires::CLK_VDISTR_LEV1_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd1i + 1),
                            wires::CLK_VDISTR_LEV1_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd2i),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd2i + 1),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd2i),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd2i + 1),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd1mi),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd1mi + 1),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd2mi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd2mi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_VNOC_SSIT_N) {
                    let vdi = SWZ12[i / 2] + i % 2 * 12;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;

                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi),
                            wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi + 1),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_IN3", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_VNOC_SSIT_S) {
                    let vdi = SWZ12[i / 2] + i % 2 * 12;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;

                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi),
                            wires::CLK_VDISTR_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi + 1),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_IN3", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
            }
            const SWZ2: [usize; 48] = [
                0, 1, 12, 23, 34, 43, 44, 45, 46, 47, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15,
                16, 17, 18, 19, 20, 21, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 35, 36, 37, 38,
                39, 40, 41, 42,
            ];
            if tkn.contains("_TOP") && !tkn.contains("_SSIT") {
                for i in 0..48 {
                    self.builder.extra_name_tile_tw(
                        tkn,
                        format!("GND_WIRE{ii}", ii = SWZ2[i]),
                        wires::CLK_HDISTR_LOC_DRV_N[i % 24].cell(i / 24),
                    );
                }
            }
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                let gclk_test = if fabric == FabricKind::Ulvt {
                    "GCLK_TEST_ULVT"
                } else {
                    "GCLK_TEST"
                };
                if matches!(
                    tcid,
                    tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S
                        | tcls::CLK_V_BUF_VNOC_SSIT_MID_S
                        | tcls::CLK_V_BUF_VNOC_SSIT_MID_N
                ) {
                    let mut dummy = self.builder.add_dummies(tkn);
                    for i in 0..12 {
                        dummy.insert(
                            wires::CLK_VDISTR_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_TMUX_DUMMY[2 * i].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV1_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV1_TMUX_DUMMY[2 * i].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV2_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV2_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV2_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV2_TMUX_DUMMY[2 * i].cell(0)],
                        );
                    }
                    if tcid == tcls::CLK_V_BUF_VNOC_SSIT_HNOC_S {
                        for i in 0..24 {
                            dummy.insert(
                                wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                                false,
                                &[wires::CLK_VDISTR_LEV2_N_MUX_DUMMY[i].cell(0)],
                            );
                        }
                    }
                    for i in 0..12 {
                        bels.push(
                            self.builder
                                .bel_xy(bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * i], gclk_test, 0, i)
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    12 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1[2 * i],
                                    gclk_test,
                                    0,
                                    24 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    36 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(bslots::GCLK_TEST_RING_VDISTR[2 * i], gclk_test, 0, 48 + i)
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    60 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_VNOC_SSIT_N) {
                    for i in 0..24 {
                        bels.extend([
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    24 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    48 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    72 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_VNOC_SSIT_S) {
                    for i in 0..24 {
                        bels.extend([
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    24 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    48 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    72 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                }
                if !matches!(tcid, tcls::CLK_V_BUF_VNOC_SSIT_N) {
                    for i in 0..48 {
                        bels.push(self.builder.bel_xy(bslots::GCLK_PD_V[i], "GCLK_PD", 0, i));
                    }
                }
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::CLK_V_BUF)
                    .optin_muxes(&wires::CLK_VDISTR_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_PRE_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_PRE_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR[..])
                    .optin_muxes(&wires::CLK_VDISTR_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV3[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV3_N[..])
                    .optin_muxes(&wires::CLK_VROUTE[..])
                    .optin_muxes(&wires::CLK_VROUTE_N[..])
                    .bels(bels)
                    .extract();
            }
        }
    }

    pub fn fill_tiles_clk_v_buf_vnoc_ssit2(&mut self) {
        let tkn = "CLKE2_REBUF_VERT_EDGE_AOO_TILE";
        let tcid = tcls::CLK_V_BUF_VNOC_SSIT2_EDGE;
        let naming = "CLK_V_BUF_VNOC_SSIT2_EDGE";
        for i in 0..24 {
            let ii = [
                0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14,
                15,
            ][i] * 3;
            for (name, wire) in [
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                    wires::CLK_VDISTR[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV1_{i}"),
                    wires::CLK_VDISTR_LEV1[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV2_{i}"),
                    wires::CLK_VDISTR_LEV2[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                    wires::CLK_VROUTE[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE{i}"),
                    wires::CLK_VROUTE_SSIT2_V[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED{i}"),
                    wires::CLK_VDISTR_SSIT2_V[i].cell(0),
                ),
                (format!("PD_L_SLR{i}"), wires::CLK_PD_V[i].cell(0)),
                (
                    format!("IF_WRAP_CLK_V_BOT_PD_R{i}"),
                    wires::CLK_PD_V[i].cell(1),
                ),
                (
                    format!("CLKE2_DRIVER_ESD_{ii}_CLK_IN"),
                    wires::OUT_GCLK_PD[i].cell(0),
                ),
                (
                    format!("CLKE2_DRIVER_ESD_{iii}_CLK_IN", iii = ii + 72),
                    wires::OUT_GCLK_PD[i].cell(1),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_L{i}"),
                    wires::CLK_HDISTR_LOC[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_R{i}"),
                    wires::CLK_HDISTR_LOC[i].cell(1),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_L{i}"),
                    wires::CLK_HDISTR_LOC_SSIT2_V[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_R{i}"),
                    wires::CLK_HDISTR_LOC_SSIT2_V[i].cell(1),
                ),
                (
                    format!("CLKE2_DRIVER_ESD_{iii}_CLK_B_UBUMP", iii = ii + 1),
                    wires::CLK_HDISTR_LOC_SSIT2_LOOP[i].cell(0),
                ),
                (
                    format!("CLKE2_DRIVER_ESD_{iii}_CLK_B_UBUMP", iii = ii + 72 + 1),
                    wires::CLK_HDISTR_LOC_SSIT2_LOOP[i].cell(1),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] * 2 + i % 2
                    ),
                    wires::CLK_VDISTR_TMUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_IN1",
                        ii = SWZ12[i / 2] * 2 + i % 2
                    ),
                    wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_4TO1_{ii}_CLK_OUT",
                        ii = (SWZ12[i / 2] * 2 + i % 2) * 2
                    ),
                    wires::CLK_VDISTR_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_4TO1_{ii}_CLK_OUT",
                        ii = (SWZ12[i / 2] * 2 + i % 2) * 2 + 1
                    ),
                    wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_3TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] * 2 + i % 2
                    ),
                    wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_8TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] * 2 + i % 2
                    ),
                    wires::CLK_VDISTR_N_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_8TO1_{ii}_CLK_IN3",
                        ii = SWZ12[i / 2] * 2 + i % 2
                    ),
                    wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
        }
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let mut dummy = self.builder.add_dummies(tkn);
            for i in 0..24 {
                dummy.insert(
                    wires::CLK_VDISTR_N_MUX[i].cell(0),
                    false,
                    &[
                        wires::CLK_VDISTR_N_MUX_DUMMY0[i].cell(0),
                        wires::CLK_VDISTR_N_MUX_DUMMY1[i].cell(0),
                        wires::CLK_VDISTR_N_MUX_DUMMY2[i].cell(0),
                    ],
                );
            }
            let mut bels = vec![];
            for i in 0..48 {
                bels.push(
                    self.builder
                        .bel_xy(
                            match i % 4 {
                                0 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[i / 2],
                                1 => bslots::GCLK_TEST_RING_VDISTR[i / 2],
                                2 => bslots::GCLK_TEST_RING_VDISTR[i / 2],
                                3 => bslots::GCLK_TEST_RING_VDISTR_LEV2_N[i / 2],
                                _ => unreachable!(),
                            },
                            "GCLK_TEST",
                            0,
                            i,
                        )
                        .pin_rename("CLKIN", "I")
                        .pin_rename("CLKOUT", "O"),
                );
                bels.push(self.builder.bel_xy(bslots::GCLK_PD_V[i], "GCLK_PD", 0, i));
            }
            self.builder
                .xtile_id(tcid, naming, xy)
                .switchbox(bslots::CLK_V_BUF)
                .optin_muxes(&wires::CLK_PD_V[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_LOOP[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VROUTE[..])
                .optin_muxes(&wires::CLK_VROUTE_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VDISTR[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV1[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV2[..])
                .optin_muxes(&wires::CLK_VDISTR_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VDISTR_TMUX[..])
                .optin_muxes(&wires::CLK_VDISTR_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV1_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV2_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_N_MUX[..])
                .bels(bels)
                .extract();
        }
    }

    pub fn fill_tiles_clk_v_buf_gt(&mut self) {
        let fabric = get_fabric(self.rd);
        let ulvt = if fabric == FabricKind::Ulvt {
            "_ULVT"
        } else {
            ""
        };
        for &tkn in crate::clk::CLK_GT_REBUF_TILE_KINDS {
            let (tcid, naming) = if tkn.contains("_EDGE") {
                (tcls::CLK_V_BUF_GT_SSIT2_EDGE, "CLK_V_BUF_GT_SSIT2_EDGE")
            } else if tkn.contains("_SSIT_HSR") {
                (tcls::CLK_V_BUF_GT_SSIT_HNOC_S, "CLK_V_BUF_GT_SSIT_HNOC_S")
            } else if tkn.contains("_SSIT_BOT") {
                (tcls::CLK_V_BUF_GT_SSIT_S, "CLK_V_BUF_GT_SSIT_S")
            } else if tkn.contains("_SSIT_MB") {
                (tcls::CLK_V_BUF_GT_SSIT_MID_S, "CLK_V_BUF_GT_SSIT_MID_S")
            } else if tkn.contains("_SSIT_MT") {
                (tcls::CLK_V_BUF_GT_SSIT_MID_N, "CLK_V_BUF_GT_SSIT_MID_N")
            } else if tkn.contains("_SSIT_TOP") {
                (tcls::CLK_V_BUF_GT_SSIT_N, "CLK_V_BUF_GT_SSIT_N")
            } else if tkn.contains("_SSIT_HBM") {
                (tcls::CLK_V_BUF_GT_SSIT_N, "CLK_V_BUF_GT_SSIT_N_HBM")
            } else {
                (
                    tcls::CLK_V_BUF_GT,
                    match fabric {
                        FabricKind::Orig | FabricKind::Rev2 => "CLK_V_BUF_GT",
                        FabricKind::Ssit => "CLK_V_BUF_GT_SSIT",
                        FabricKind::Ssit2 => "CLK_V_BUF_GT_SSIT2",
                        FabricKind::Ulvt => "CLK_V_BUF_GT_ULVT",
                    },
                )
            };
            if tkn.contains("_EDGE") {
                continue;
            }
            const SWZ: [usize; 24] = [
                0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14,
                15,
            ];
            for i in 0..24 {
                for (name, wire) in [
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                        wires::CLK_VDISTR[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV1_{i}"),
                        wires::CLK_VDISTR_LEV1[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV2_{i}"),
                        wires::CLK_VDISTR_LEV2[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV21_{i}"),
                        wires::CLK_VDISTR_LEV1[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV3_{i}"),
                        wires::CLK_VDISTR_LEV2[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                        wires::CLK_VROUTE[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR{i}"),
                        wires::CLK_VDISTR_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV1_{i}"),
                        wires::CLK_VDISTR_LEV1_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV2_{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV21_{i}"),
                        wires::CLK_VDISTR_LEV1_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV3_{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VROUTE{i}"),
                        wires::CLK_VROUTE_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_R{i}"),
                        wires::CLK_HDISTR_LOC_DRV[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_HDISTR_LOC_R{i}"),
                        wires::CLK_HDISTR_LOC_DRV_N[i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_BOT_PD_R{i}"),
                        wires::CLK_PD_V[i].cell(0),
                    ),
                ] {
                    self.builder.extra_name_tile_tw(tkn, name, wire);
                }
                if tcid == tcls::CLK_V_BUF_GT {
                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i]),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i] + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT", ii = 48 + 2 * SWZ[i]),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!(
                                "CLK_CMT_MUX_4TO1{ulvt}_{ii}_CLK_OUT",
                                ii = 48 + 2 * SWZ[i] + 1
                            ),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i]),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1{ulvt}_{ii}_CLK_OUT", ii = 2 * SWZ[i] + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if matches!(
                    tcid,
                    tcls::CLK_V_BUF_GT_SSIT_HNOC_S
                        | tcls::CLK_V_BUF_GT_SSIT_MID_S
                        | tcls::CLK_V_BUF_GT_SSIT_MID_N
                ) {
                    let vdi = (SWZ12[i / 2] + i % 2 * 12) * 4;
                    let vd1i = 96 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd2i = 144 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd1mi = 48 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    let vd2mi = 96 + (SWZ12[i / 2] + i % 2 * 12) * 2;
                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_PRE_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 1),
                            wires::CLK_VDISTR_N_PRE_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 2),
                            wires::CLK_VDISTR_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 3),
                            wires::CLK_VDISTR_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 2),
                            wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 3),
                            wires::CLK_VDISTR_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd1i),
                            wires::CLK_VDISTR_LEV1_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd1i + 1),
                            wires::CLK_VDISTR_LEV1_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd1i),
                            wires::CLK_VDISTR_LEV1_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd1i + 1),
                            wires::CLK_VDISTR_LEV1_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd2i),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vd2i + 1),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd2i),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vd2i + 1),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd1mi),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd1mi + 1),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd2mi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vd2mi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!(
                                "CLK_CMT_DRVR_TRI_{ii}_CLK_OUT_B",
                                ii = 120 + SWZ12[i / 2] * 2 + i % 2
                            ),
                            wires::CLK_VDISTR_LEV2_N[i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_GT_SSIT_N) {
                    let vdi = SWZ12[i / 2] + i % 2 * 12;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;

                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi),
                            wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi + 1),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_IN3", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if matches!(tcid, tcls::CLK_V_BUF_GT_SSIT_S) {
                    let vdi = SWZ12[i / 2] + i % 2 * 12;
                    let vdmi = (SWZ12[i / 2] + i % 2 * 12) * 2;

                    for (name, wire) in [
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi),
                            wires::CLK_VDISTR_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi),
                            wires::CLK_VDISTR_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 24),
                            wires::CLK_VDISTR_LEV1_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_OUT", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_N_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_2TO1_{ii}_CLK_IN1", ii = vdi + 48),
                            wires::CLK_VDISTR_LEV2_N_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_3TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi + 1),
                            wires::CLK_VDISTR_LEV1_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 48 + vdmi),
                            wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi + 1),
                            wires::CLK_VDISTR_N_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = vdmi),
                            wires::CLK_VDISTR_MUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_OUT", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                        ),
                        (
                            format!("CLK_CMT_MUX_4TO1_{ii}_CLK_IN3", ii = 96 + vdi),
                            wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_R{i}"),
                            wires::CLK_HDISTR_LOC_DRV_S[i].cell(1),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_PD_R{i}"),
                            wires::CLK_PD_V[i].cell(1),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
            }
            for i in 0..12 {
                for (name, wire) in [
                    (
                        format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV3_GT{i}"),
                        wires::CLK_VDISTR_LEV2[2 * i].cell(0),
                    ),
                    (
                        format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_LEV3_GT{i}"),
                        wires::CLK_VDISTR_LEV2_N[2 * i].cell(0),
                    ),
                    (
                        format!("CLK_VDISTR_LEV3_TOP_SLR{i}"),
                        wires::CLK_VDISTR_LEV3_N[2 * i].cell(0),
                    ),
                    (
                        format!("CLK_VDISTR_LEV3_BOT_SLR{i}"),
                        wires::CLK_VDISTR_LEV3[2 * i].cell(0),
                    ),
                ] {
                    self.builder.extra_name_tile_tw(tkn, name, wire);
                }
                if tcid == tcls::CLK_V_BUF_GT_SSIT_N {
                    for (name, wire) in [
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE_GT{i}"),
                            wires::CLK_VROUTE_N[2 * i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED_GT{i}"),
                            wires::CLK_VDISTR_LEV2_N[2 * i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_CLK_VROUTE_GT{i}"),
                            wires::CLK_VROUTE_N[2 * i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_TOP_CLK_VDISTR_SHARED_GT{i}"),
                            wires::CLK_VDISTR_LEV2_N[2 * i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
                if tcid == tcls::CLK_V_BUF_GT_SSIT_S {
                    for (name, wire) in [
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE_GT{i}"),
                            wires::CLK_VROUTE[2 * i].cell(0),
                        ),
                        (
                            format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED_GT{i}"),
                            wires::CLK_VDISTR_LEV2[2 * i].cell(0),
                        ),
                    ] {
                        self.builder.extra_name_tile_tw(tkn, name, wire);
                    }
                }
            }
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                let gclk_test = if fabric == FabricKind::Ulvt {
                    "GCLK_TEST_ULVT"
                } else {
                    "GCLK_TEST"
                };
                let mut dummy = self.builder.add_dummies(tkn);
                if matches!(tcid, tcls::CLK_V_BUF_GT_SSIT_S | tcls::CLK_V_BUF_GT_SSIT_N) {
                    for i in 0..12 {
                        dummy.insert(
                            wires::CLK_VDISTR_MUX[2 * i + 1].cell(0),
                            false,
                            &[
                                wires::CLK_VDISTR_MUX_DUMMY0[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_MUX_DUMMY1[2 * i + 1].cell(0),
                            ],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_N_MUX[2 * i + 1].cell(0),
                            false,
                            &[
                                wires::CLK_VDISTR_N_MUX_DUMMY0[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_N_MUX_DUMMY1[2 * i + 1].cell(0),
                            ],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_MUX[2 * i + 1].cell(0),
                            false,
                            &[
                                wires::CLK_VDISTR_LEV1_MUX_DUMMY0[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_LEV1_MUX_DUMMY1[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_LEV1_MUX_DUMMY2[2 * i + 1].cell(0),
                            ],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_N_MUX[2 * i + 1].cell(0),
                            false,
                            &[
                                wires::CLK_VDISTR_LEV1_N_MUX_DUMMY0[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_LEV1_N_MUX_DUMMY1[2 * i + 1].cell(0),
                                wires::CLK_VDISTR_LEV1_N_MUX_DUMMY2[2 * i + 1].cell(0),
                            ],
                        );
                    }
                }

                if matches!(
                    tcid,
                    tcls::CLK_V_BUF_GT_SSIT_HNOC_S
                        | tcls::CLK_V_BUF_GT_SSIT_MID_S
                        | tcls::CLK_V_BUF_GT_SSIT_MID_N
                ) {
                    for i in 0..12 {
                        dummy.insert(
                            wires::CLK_VDISTR_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_TMUX_DUMMY[2 * i].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV1_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV1_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV1_TMUX_DUMMY[2 * i].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV2_N_TMUX[2 * i + 1].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV2_N_TMUX_DUMMY[2 * i + 1].cell(0)],
                        );
                        dummy.insert(
                            wires::CLK_VDISTR_LEV2_TMUX[2 * i].cell(0),
                            false,
                            &[wires::CLK_VDISTR_LEV2_TMUX_DUMMY[2 * i].cell(0)],
                        );
                    }
                    if tcid == tcls::CLK_V_BUF_GT_SSIT_HNOC_S {
                        for i in 0..24 {
                            dummy.insert(
                                wires::CLK_VDISTR_LEV2_N_MUX[i].cell(0),
                                false,
                                &[wires::CLK_VDISTR_LEV2_N_MUX_DUMMY[i].cell(0)],
                            );
                        }
                    }
                    for i in 0..12 {
                        bels.push(
                            self.builder
                                .bel_xy(bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * i], gclk_test, 0, i)
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    12 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1[2 * i],
                                    gclk_test,
                                    0,
                                    24 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    36 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(bslots::GCLK_TEST_RING_VDISTR[2 * i], gclk_test, 0, 48 + i)
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                        bels.push(
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_N[2 * i + 1],
                                    gclk_test,
                                    0,
                                    60 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        );
                    }
                }
                if tcid == tcls::CLK_V_BUF_GT_SSIT_N {
                    for i in 0..12 {
                        bels.extend([
                            self.builder
                                .bel_xy(bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * i], gclk_test, 0, i)
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * i],
                                    gclk_test,
                                    0,
                                    60 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                    for i in 0..24 {
                        bels.extend([
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    12 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    36 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                }
                if tcid == tcls::CLK_V_BUF_GT_SSIT_S {
                    for i in 0..12 {
                        bels.extend([
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N[2 * i],
                                    gclk_test,
                                    0,
                                    i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2[2 * i],
                                    gclk_test,
                                    0,
                                    60 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                    for i in 0..24 {
                        bels.extend([
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_LEV1_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    12 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                            self.builder
                                .bel_xy(
                                    bslots::GCLK_TEST_RING_VDISTR_N[2 * (i % 12) + i / 12],
                                    gclk_test,
                                    0,
                                    36 + i,
                                )
                                .pin_rename("CLKIN", "I")
                                .pin_rename("CLKOUT", "O"),
                        ]);
                    }
                }
                if !matches!(tcid, tcls::CLK_V_BUF_GT_SSIT_N) {
                    for i in 0..24 {
                        bels.push(self.builder.bel_xy(bslots::GCLK_PD_V[i], "GCLK_PD", 0, i));
                    }
                }
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::CLK_V_BUF)
                    .optin_muxes(&wires::CLK_VDISTR_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N_TMUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_PRE_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR_N_PRE_MUX[..])
                    .optin_muxes(&wires::CLK_VDISTR[..])
                    .optin_muxes(&wires::CLK_VDISTR_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV1_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV2_N[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV3[..])
                    .optin_muxes(&wires::CLK_VDISTR_LEV3_N[..])
                    .optin_muxes(&wires::CLK_VROUTE[..])
                    .optin_muxes(&wires::CLK_VROUTE_N[..])
                    .bels(bels)
                    .extract();
            }
        }
    }

    pub fn fill_tiles_clk_v_buf_gt_ssit2(&mut self) {
        let tkn = "CLKE2_REBUF_VERT_GT_EDGE_AOO_TILE_MY";
        let tcid = tcls::CLK_V_BUF_GT_SSIT2_EDGE;
        let naming = "CLK_V_BUF_GT_SSIT2_EDGE";
        for i in 0..24 {
            let ii = [
                0, 11, 16, 17, 18, 19, 20, 21, 22, 23, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14,
                15,
            ][i] * 3;
            for (name, wire) in [
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR{i}"),
                    wires::CLK_VDISTR[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV1_{i}"),
                    wires::CLK_VDISTR_LEV1[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VDISTR_LEV2_{i}"),
                    wires::CLK_VDISTR_LEV2[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_VROUTE{i}"),
                    wires::CLK_VROUTE[i].cell(0),
                ),
                (format!("PD_R_SLR{i}"), wires::CLK_PD_V[i].cell(0)),
                (
                    format!("CLKE2_DRIVER_ESD_{ii}_CLK_IN"),
                    wires::OUT_GCLK_PD[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_BOT_CLK_HDISTR_LOC_R{i}"),
                    wires::CLK_HDISTR_LOC[i].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_HDISTR_LOC_R{i}"),
                    wires::CLK_HDISTR_LOC_SSIT2_V[i].cell(0),
                ),
                (
                    format!("CLKE2_DRIVER_ESD_{iii}_CLK_B_UBUMP", iii = ii + 1),
                    wires::CLK_HDISTR_LOC_SSIT2_LOOP[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_TMUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_IN1",
                        ii = SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_TEST[i ^ 1].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_4TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_OUT",
                        ii = 24 + SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV1_TMUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_IN1",
                        ii = 24 + SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV1_TEST[i ^ 1].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_4TO1_{ii}_CLK_OUT",
                        ii = 24 + SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV1_MUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_OUT",
                        ii = 48 + SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV2_TMUX[i].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_2TO1_{ii}_CLK_IN1",
                        ii = 48 + SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV2_TEST[i ^ 1].cell(0),
                ),
                (
                    format!(
                        "CLK_CMT_MUX_3TO1_{ii}_CLK_OUT",
                        ii = SWZ12[i / 2] + i % 2 * 12
                    ),
                    wires::CLK_VDISTR_LEV2_MUX[i].cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
        }
        for i in 0..12 {
            for (name, wire) in [
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_VROUTE_GT{i}"),
                    wires::CLK_VROUTE_SSIT2_V[i * 2].cell(0),
                ),
                (
                    format!("IF_WRAP_CLK_V_SLR_CLK_VDISTR_SHARED_GT{i}"),
                    wires::CLK_VDISTR_SSIT2_V[i * 2].cell(0),
                ),
                (
                    format!("CLK_CMT_MUX_8TO1_{ii}_CLK_OUT", ii = i / 2 + i % 2 * 6),
                    wires::CLK_VDISTR_N_MUX[i * 2].cell(0),
                ),
                (
                    format!("CLK_CMT_MUX_8TO1_{ii}_CLK_IN3", ii = i / 2 + i % 2 * 6),
                    wires::CLK_VDISTR_LEV2_N_TEST[(i * 2) ^ 2].cell(0),
                ),
            ] {
                self.builder.extra_name_tile_tw(tkn, name, wire);
            }
        }
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let mut dummy = self.builder.add_dummies(tkn);
            for i in 0..12 {
                dummy.insert(
                    wires::CLK_VDISTR_MUX[i * 2 + 1].cell(0),
                    false,
                    &[
                        wires::CLK_VDISTR_MUX_DUMMY0[i * 2 + 1].cell(0),
                        wires::CLK_VDISTR_MUX_DUMMY1[i * 2 + 1].cell(0),
                    ],
                );
                dummy.insert(
                    wires::CLK_VDISTR_LEV1_MUX[i * 2 + 1].cell(0),
                    false,
                    &[
                        wires::CLK_VDISTR_LEV1_MUX_DUMMY0[i * 2 + 1].cell(0),
                        wires::CLK_VDISTR_LEV1_MUX_DUMMY1[i * 2 + 1].cell(0),
                    ],
                );
                dummy.insert(
                    wires::CLK_VDISTR_LEV2_MUX[i * 2 + 1].cell(0),
                    false,
                    &[
                        wires::CLK_VDISTR_LEV2_MUX_DUMMY0[i * 2 + 1].cell(0),
                        wires::CLK_VDISTR_LEV2_MUX_DUMMY1[i * 2 + 1].cell(0),
                    ],
                );
                dummy.insert(
                    wires::CLK_VDISTR_N_MUX[i * 2].cell(0),
                    false,
                    &[
                        wires::CLK_VDISTR_N_MUX_DUMMY0[i * 2].cell(0),
                        wires::CLK_VDISTR_N_MUX_DUMMY1[i * 2].cell(0),
                        wires::CLK_VDISTR_N_MUX_DUMMY2[i * 2].cell(0),
                    ],
                );
            }
            let mut bels = vec![];
            for i in 0..84 {
                bels.push(
                    self.builder
                        .bel_xy(
                            match i / 24 {
                                0 => bslots::GCLK_TEST_RING_VDISTR_LEV2[i % 12 * 2 + i / 12 % 2],
                                1 => bslots::GCLK_TEST_RING_VDISTR_LEV1[i % 12 * 2 + i / 12 % 2],
                                2 => bslots::GCLK_TEST_RING_VDISTR[i % 12 * 2 + i / 12 % 2],
                                3 => {
                                    bslots::GCLK_TEST_RING_VDISTR_LEV2_N
                                        [(i % 6 * 2 + i / 6 % 2) * 2]
                                }
                                _ => unreachable!(),
                            },
                            "GCLK_TEST",
                            0,
                            i,
                        )
                        .pin_rename("CLKIN", "I")
                        .pin_rename("CLKOUT", "O"),
                );
            }
            for i in 0..24 {
                bels.push(self.builder.bel_xy(bslots::GCLK_PD_V[i], "GCLK_PD", 0, i));
            }
            self.builder
                .xtile_id(tcid, naming, xy)
                .switchbox(bslots::CLK_V_BUF)
                .optin_muxes(&wires::CLK_PD_V[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_LOOP[..])
                .optin_muxes(&wires::CLK_HDISTR_LOC_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VROUTE[..])
                .optin_muxes(&wires::CLK_VROUTE_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VDISTR[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV1[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV2[..])
                .optin_muxes(&wires::CLK_VDISTR_SSIT2_V[..])
                .optin_muxes(&wires::CLK_VDISTR_TMUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV1_TMUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV2_TMUX[..])
                .optin_muxes(&wires::CLK_VDISTR_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV1_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_LEV2_MUX[..])
                .optin_muxes(&wires::CLK_VDISTR_N_MUX[..])
                .bels(bels)
                .extract();
        }
    }

    pub fn fill_tiles_clk_s_buf(&mut self) {
        for (tcid, naming, tkns) in [
            (
                tcls::CLK_S_BUF_VNOC,
                "CLK_S_BUF_VNOC",
                crate::clk::CLK_VNOC_BOT_TILE_KINDS,
            ),
            (
                tcls::CLK_S_BUF_GT,
                "CLK_S_BUF_GT",
                crate::clk::CLK_GT_BOT_TILE_KINDS,
            ),
        ] {
            for tkn in tkns {
                for i in 0..24 {
                    self.builder.extra_name_tile_tw(
                        tkn,
                        format!("IF_WRAP_CLK_V_CLK_VDISTR_LEV2_{i}"),
                        wires::CLK_VDISTR_LEV2_N[i].cell(0),
                    );
                    self.builder.extra_name_tile_tw(
                        tkn,
                        format!("IF_WRAP_CLK_V_CLK_VROUTE{i}"),
                        wires::CLK_VROUTE_N[i].cell(0),
                    );
                    self.builder.extra_name_tile_tw(
                        tkn,
                        format!("CLK_HDISTR{i}"),
                        wires::HSR_HDISTR[i].cell(2),
                    );
                    self.builder.extra_name_tile_tw(
                        tkn,
                        format!("CLK_HROUTE{i}"),
                        wires::HSR_HROUTE[i].cell(1),
                    );
                }
                if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                    self.builder
                        .xtile_id(tcid, naming, xy)
                        .switchbox(bslots::CLK_V_BUF)
                        .optin_muxes(&wires::HSR_HROUTE[..])
                        .optin_muxes(&wires::HSR_HDISTR[..])
                        .optin_muxes(&wires::CLK_VROUTE_N[..])
                        .optin_muxes(&wires::CLK_VDISTR_LEV2_N[..])
                        .extract();
                }
            }
        }
    }

    pub fn fill_tiles_clk(&mut self) {
        self.fill_tiles_rclk();
        self.fill_tiles_rclk_cle();
        self.fill_tiles_rclk_intf();
        self.fill_tiles_rclk_clkbuf();
        self.fill_tiles_rclk_vnoc();
        self.fill_tiles_rclk_gt();
        self.fill_tiles_rclk_ssit();
        self.fill_tiles_clk_v_buf_vnoc();
        self.fill_tiles_clk_v_buf_vnoc_ssit2();
        self.fill_tiles_clk_v_buf_gt();
        self.fill_tiles_clk_v_buf_gt_ssit2();
        self.fill_tiles_clk_s_buf();
    }
}
