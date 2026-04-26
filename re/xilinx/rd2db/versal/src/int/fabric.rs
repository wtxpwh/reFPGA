use prjcombine_interconnect::{
    db::{BelInfo, BelPin, LegacyBel, WireSlotIdExt},
    dir::DirH,
};
use prjcombine_re_xilinx_naming::db::BelNaming;
use prjcombine_versal::defs::{bslots, tcls, wires};

use crate::int::{IntMaker, XTileInfoExt};

impl<'rd> IntMaker<'rd> {
    pub fn fill_tiles_cle(&mut self) {
        for (tkn, tcid, naming, side) in [
            ("CLE_W_CORE", tcls::CLE, "CLE", DirH::E),
            ("CLE_E_CORE", tcls::CLE, "CLE", DirH::W),
            ("CLE_W_VR_CORE", tcls::CLE_VR, "CLE_VR", DirH::E),
            ("CLE_E_VR_CORE", tcls::CLE_VR, "CLE_VR", DirH::W),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let int_xy = self.builder.walk_to_int(xy, (!side).into(), false).unwrap();
                let bel_slicel = self
                    .builder
                    .bel_xy(bslots::SLICE[0], "SLICE", 0, 0)
                    .pin_rename("CKEN1", "CE_0_")
                    .pin_rename("CKEN2", "CE_1_")
                    .pin_rename("CKEN3", "CE_2_")
                    .pin_rename("CKEN4", "CE_3_")
                    .pin_name_only("CIN", 1)
                    .pin_name_only("COUT", 1);
                let bel_slicem = self
                    .builder
                    .bel_xy(bslots::SLICE[1], "SLICE", 1, 0)
                    .pin_rename("CKEN1", "CE_0_")
                    .pin_rename("CKEN2", "CE_1_")
                    .pin_rename("CKEN3", "CE_2_")
                    .pin_rename("CKEN4", "CE_3_")
                    .pin_name_only("SRL_IN_B", 1)
                    .pin_name_only("SRL_OUT_B", 1)
                    .pin_name_only("CIN", 1)
                    .pin_name_only("COUT", 1);
                let cle_bc_xy = self
                    .builder
                    .delta(xy, if side == DirH::E { 1 } else { -1 }, 0);
                let cle_bc_kind = self.rd.tiles[&cle_bc_xy].kind;
                let cle_bc_naming = match &self.rd.tile_kinds.key(cle_bc_kind)[..] {
                    "CLE_BC_CORE" | "CLE_BC_CORE_MX" => "CLE_BC",
                    "SLL" => "CLE_BC_SLL",
                    "SLL2" => "CLE_BC_SLL2",
                    _ => unreachable!(),
                };
                let cle_bc_naming = self.builder.ndb.get_tile_class_naming(cle_bc_naming);
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .bel(bel_slicel)
                    .bel(bel_slicem)
                    .ref_int_side(int_xy, side, 0)
                    .ref_xlat(cle_bc_xy, &[Some(0), None], cle_bc_naming)
                    .extract();
            }
        }
    }

    pub fn fill_tiles_bram(&mut self) {
        for (side, tkn) in [
            (DirH::E, "BRAM_LOCF_BR_TILE"),
            (DirH::E, "BRAM_LOCF_TR_TILE"),
            (DirH::W, "BRAM_LOCF_BL_TILE"),
            (DirH::W, "BRAM_LOCF_TL_TILE"),
            (DirH::E, "BRAM_ROCF_BR_TILE"),
            (DirH::E, "BRAM_ROCF_TR_TILE"),
            (DirH::W, "BRAM_ROCF_BL_TILE"),
            (DirH::W, "BRAM_ROCF_TL_TILE"),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let intf = match side {
                    DirH::E => "INTF_E",
                    DirH::W => "INTF_W",
                };
                let intf = self.builder.ndb.get_tile_class_naming(intf);
                let intf_xy = if side == DirH::E {
                    self.builder.delta(xy, -1, 0)
                } else {
                    self.builder.delta(xy, 1, 0)
                };
                let mut bel = self
                    .builder
                    .bel_xy(bslots::BRAM, "RAMB36", 0, 0)
                    .pin_rename("CLKARDCLKL", "CLKAL")
                    .pin_rename("CLKARDCLKU", "CLKAU")
                    .pin_rename("CLKBWRCLKL", "CLKBL")
                    .pin_rename("CLKBWRCLKU", "CLKBU")
                    .pin_rename("ENARDENL", "ENAL")
                    .pin_rename("ENARDENU", "ENAU")
                    .pin_rename("ENBWRENL", "ENBL")
                    .pin_rename("ENBWRENU", "ENBU")
                    .pin_rename("REGCEAREGCEL", "REGCEAL")
                    .pin_rename("REGCEAREGCEU", "REGCEAU")
                    .pin_rename("RSTRAMARSTRAML", "RSTRAMAL")
                    .pin_rename("RSTRAMARSTRAMU", "RSTRAMAU")
                    .pin_rename("RSTREGARSTREGL", "RSTREGAL")
                    .pin_rename("RSTREGARSTREGU", "RSTREGAU")
                    .pin_name_only("CASINSBITERR", 1)
                    .pin_name_only("CASINDBITERR", 1)
                    .pin_name_only("CASOUTSBITERR", 1)
                    .pin_name_only("CASOUTDBITERR", 1);
                for ab in ['A', 'B'] {
                    for i in 0..32 {
                        bel = bel
                            .pin_name_only(&format!("CASDIN{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CASDOUT{ab}_{i}_"), 1);
                    }
                    for i in 0..4 {
                        bel = bel
                            .pin_name_only(&format!("CASDINP{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CASDOUTP{ab}_{i}_"), 1);
                    }
                }

                for i in 0..12 {
                    bel = bel
                        .pin_rename(format!("ADDRARDADDRL_{i}_"), format!("ADDRAL_{i}_"))
                        .pin_rename(format!("ADDRARDADDRU_{i}_"), format!("ADDRAU_{i}_"))
                        .pin_rename(format!("ADDRBWRADDRL_{i}_"), format!("ADDRBL_{i}_"))
                        .pin_rename(format!("ADDRBWRADDRU_{i}_"), format!("ADDRBU_{i}_"));
                }

                for i in 0..16 {
                    let i0 = 2 * i;
                    let i1 = 2 * i + 1;
                    bel = bel
                        .pin_rename(format!("DINADIN_{i0}_"), format!("DIAL_{i}_"))
                        .pin_rename(format!("DINADIN_{i1}_"), format!("DIAU_{i}_"))
                        .pin_rename(format!("DINBDIN_{i0}_"), format!("DIBL_{i}_"))
                        .pin_rename(format!("DINBDIN_{i1}_"), format!("DIBU_{i}_"))
                        .pin_rename(format!("DOUTADOUT_{i0}_"), format!("DOAL_{i}_"))
                        .pin_rename(format!("DOUTADOUT_{i1}_"), format!("DOAU_{i}_"))
                        .pin_rename(format!("DOUTBDOUT_{i0}_"), format!("DOBL_{i}_"))
                        .pin_rename(format!("DOUTBDOUT_{i1}_"), format!("DOBU_{i}_"));
                }

                for i in 0..2 {
                    let i0 = 2 * i;
                    let i1 = 2 * i + 1;
                    bel = bel
                        .pin_rename(format!("DINPADINP_{i0}_"), format!("DIPAL_{i}_"))
                        .pin_rename(format!("DINPADINP_{i1}_"), format!("DIPAU_{i}_"))
                        .pin_rename(format!("DINPBDINP_{i0}_"), format!("DIPBL_{i}_"))
                        .pin_rename(format!("DINPBDINP_{i1}_"), format!("DIPBU_{i}_"))
                        .pin_rename(format!("DOUTPADOUTP_{i0}_"), format!("DOPAL_{i}_"))
                        .pin_rename(format!("DOUTPADOUTP_{i1}_"), format!("DOPAU_{i}_"))
                        .pin_rename(format!("DOUTPBDOUTP_{i0}_"), format!("DOPBL_{i}_"))
                        .pin_rename(format!("DOUTPBDOUTP_{i1}_"), format!("DOPBU_{i}_"));
                }
                for i in 0..4 {
                    bel = bel
                        .pin_rename(format!("WEBWEL_{i}_"), format!("WEBL_{i}_"))
                        .pin_rename(format!("WEBWEU_{i}_"), format!("WEBU_{i}_"))
                }

                let tile = &self.rd.tiles[&xy];
                let tk = &self.rd.tile_kinds[tile.kind];
                for i in 0..2 {
                    bel = bel.sub_xy("RAMB18", 0, i);
                    let tksite = tk
                        .sites
                        .get(&self.rd.slot_xy("RAMB18", 0, i).unwrap())
                        .unwrap()
                        .1;
                    for pin in tksite.pins.keys() {
                        bel = bel
                            .pin_rename(pin, format!("H{i}_{pin}"))
                            .pin_name_only(&format!("H{i}_{pin}"), 0);
                    }
                }

                let mut xn = self.builder.xtile_id(tcls::BRAM, "BRAM", xy);
                for i in 0..4 {
                    let cur_intf_xy = xn.builder.delta(intf_xy, 0, i as i32);
                    xn = xn.ref_single(cur_intf_xy, i, intf)
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_uram(&mut self) {
        for (tkn, is_delay) in [
            ("URAM_LOCF_BL_TILE", false),
            ("URAM_LOCF_TL_TILE", false),
            ("URAM_ROCF_BL_TILE", false),
            ("URAM_ROCF_TL_TILE", false),
            ("URAM_DELAY_LOCF_TL_TILE", true),
            ("URAM_DELAY_ROCF_TL_TILE", true),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let intf = self.builder.ndb.get_tile_class_naming("INTF_W");
                let intf_xy = self.builder.delta(xy, 1, 0);
                let mut bel = self.builder.bel_xy(bslots::URAM, "URAM288", 0, 0);
                for ab in ['A', 'B'] {
                    bel = bel
                        .pin_name_only(&format!("CAS_IN_EN_{ab}"), 1)
                        .pin_name_only(&format!("CAS_OUT_EN_{ab}"), 1)
                        .pin_name_only(&format!("CAS_IN_SBITERR_{ab}"), 1)
                        .pin_name_only(&format!("CAS_OUT_SBITERR_{ab}"), 1)
                        .pin_name_only(&format!("CAS_IN_DBITERR_{ab}"), 1)
                        .pin_name_only(&format!("CAS_OUT_DBITERR_{ab}"), 1)
                        .pin_name_only(&format!("CAS_IN_RDACCESS_{ab}"), 1)
                        .pin_name_only(&format!("CAS_OUT_RDACCESS_{ab}"), 1)
                        .pin_name_only(&format!("CAS_IN_RDB_WR_{ab}"), 1)
                        .pin_name_only(&format!("CAS_OUT_RDB_WR_{ab}"), 1);
                    for i in 0..72 {
                        bel = bel
                            .pin_name_only(&format!("CAS_IN_DIN_{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CAS_IN_DOUT_{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CAS_OUT_DIN_{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CAS_OUT_DOUT_{ab}_{i}_"), 1);
                    }
                    for i in 0..26 {
                        bel = bel
                            .pin_name_only(&format!("CAS_IN_ADDR_{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CAS_OUT_ADDR_{ab}_{i}_"), 1);
                    }
                    for i in 0..9 {
                        bel = bel
                            .pin_name_only(&format!("CAS_IN_BWE_{ab}_{i}_"), 1)
                            .pin_name_only(&format!("CAS_OUT_BWE_{ab}_{i}_"), 1);
                    }
                }
                if is_delay {
                    let tile = &self.rd.tiles[&xy];
                    let tk = &self.rd.tile_kinds[tile.kind];
                    let slot = self.rd.slot_xy("URAM_CAS_DLY", 0, 0).unwrap();
                    let tksite = tk.sites.get(&slot).unwrap().1;
                    bel = bel.sub_xy("URAM_CAS_DLY", 0, 0);
                    for pin in tksite.pins.keys() {
                        bel = bel
                            .pin_rename(pin, format!("CAS_DLY_{pin}"))
                            .pin_name_only(&format!("CAS_DLY_{pin}"), 1);
                    }
                }
                let mut xn = self.builder.xtile_id(
                    tcls::URAM,
                    if is_delay { "URAM_DELAY" } else { "URAM" },
                    xy,
                );
                for i in 0..4 {
                    let cur_intf_xy = xn.builder.delta(intf_xy, 0, i as i32);
                    xn = xn.ref_single(cur_intf_xy, i, intf)
                }
                xn.bel(bel).extract();
            }
        }
    }

    pub fn fill_tiles_dsp(&mut self) {
        for tkn in [
            "DSP_LOCF_B_TILE",
            "DSP_LOCF_T_TILE",
            "DSP_ROCF_B_TILE",
            "DSP_ROCF_T_TILE",
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                for i in 0..2 {
                    let mut bel = self
                        .builder
                        .bel_xy(bslots::DSP[i], "DSP", i, 0)
                        .pin_name_only("MULTSIGNIN", 1)
                        .pin_name_only("MULTSIGNOUT", 1)
                        .pin_name_only("CARRYCASCIN", 1)
                        .pin_name_only("CARRYCASCOUT", 1)
                        .pin_name_only("CONJ_CPLX_OUT", 1)
                        .pin_name_only("CONJ_CPLX_MULT_IN", 0)
                        .pin_name_only("CONJ_CPLX_PREADD_IN", 0);
                    for i in 0..34 {
                        bel = bel
                            .pin_name_only(&format!("ACIN_{i}_"), 1)
                            .pin_name_only(&format!("ACOUT_{i}_"), 1);
                    }
                    for i in 0..32 {
                        bel = bel
                            .pin_name_only(&format!("BCIN_{i}_"), 1)
                            .pin_name_only(&format!("BCOUT_{i}_"), 1);
                    }
                    for i in 0..58 {
                        bel = bel
                            .pin_name_only(&format!("PCIN_{i}_"), 1)
                            .pin_name_only(&format!("PCOUT_{i}_"), 1);
                    }
                    for i in 0..10 {
                        bel = bel
                            .pin_name_only(&format!("AD_CPLX_{i}_"), 0)
                            .pin_name_only(&format!("AD_DATA_CPLX_{i}_"), 1);
                    }
                    for i in 0..18 {
                        bel = bel
                            .pin_name_only(&format!("A_TO_D_CPLX_{i}_"), 1)
                            .pin_name_only(&format!("D_FROM_A_CPLX_{i}_"), 1)
                            .pin_name_only(&format!("A_CPLX_{i}_"), 1)
                            .pin_name_only(&format!("B2B1_CPLX_{i}_"), 1);
                    }
                    for i in 0..37 {
                        bel = bel
                            .pin_name_only(&format!("U_CPLX_{i}_"), 0)
                            .pin_name_only(&format!("V_CPLX_{i}_"), 0);
                    }
                    let tile = &self.rd.tiles[&xy];
                    let tk = &self.rd.tile_kinds[tile.kind];
                    let slot = self.rd.slot_xy("DSP58_CPLX", 0, 0).unwrap();
                    let tksite = tk.sites.get(&slot).unwrap().1;
                    bel = bel.sub_xy("DSP58_CPLX", 0, 0);
                    for pin in tksite.pins.keys() {
                        bel = bel
                            .pin_rename(pin, format!("CPLX_{pin}"))
                            .pin_name_only(&format!("CPLX_{pin}"), 1);
                    }
                    bels.push(bel);
                }
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E");
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W");
                let naming = if self.dev_naming.has_dsp_v2 {
                    "DSP_V2"
                } else {
                    "DSP_V1"
                };
                let intf0_xy = self.builder.delta(xy, -1, 0);
                let intf1_xy = self.builder.delta(xy, -1, 1);
                let intf2_xy = self.builder.delta(xy, 2, 0);
                let intf3_xy = self.builder.delta(xy, 2, 1);
                self.builder
                    .xtile_id(tcls::DSP, naming, xy)
                    .ref_single(intf0_xy, 0, intf_e)
                    .ref_single(intf1_xy, 1, intf_e)
                    .ref_single(intf2_xy, 2, intf_w)
                    .ref_single(intf3_xy, 3, intf_w)
                    .bels(bels)
                    .extract();
            }
        }
    }

    pub fn fill_tiles_hard(&mut self) {
        for (slot, tcid, naming, tkn, sk, is_large) in [
            (
                bslots::PCIE4,
                tcls::PCIE4,
                "PCIE4",
                "PCIEB_BOT_TILE",
                "PCIE40",
                false,
            ),
            (
                bslots::PCIE4,
                tcls::PCIE4,
                "PCIE4",
                "PCIEB_TOP_TILE",
                "PCIE40",
                false,
            ),
            (
                bslots::PCIE5,
                tcls::PCIE5,
                "PCIE5",
                "PCIEB5_BOT_TILE",
                "PCIE50",
                false,
            ),
            (
                bslots::PCIE5,
                tcls::PCIE5,
                "PCIE5",
                "PCIEB5_TOP_TILE",
                "PCIE50",
                false,
            ),
            (
                bslots::MRMAC,
                tcls::MRMAC,
                "MRMAC",
                "MRMAC_BOT_TILE",
                "MRMAC",
                false,
            ),
            (
                bslots::MRMAC,
                tcls::MRMAC,
                "MRMAC",
                "MRMAC_TOP_TILE",
                "MRMAC",
                false,
            ),
            (
                bslots::DFE_CFC,
                tcls::DFE_CFC,
                "DFE_CFC_S",
                "DFE_CFC_BOT_TILE",
                "DFE_CFC_BOT",
                false,
            ),
            (
                bslots::DFE_CFC,
                tcls::DFE_CFC,
                "DFE_CFC_N",
                "DFE_CFC_TOP_TILE",
                "DFE_CFC_TOP",
                false,
            ),
            (
                bslots::SDFEC,
                tcls::SDFEC,
                "SDFEC",
                "SDFECA_TOP_TILE",
                "SDFEC_A",
                false,
            ),
            (
                bslots::DCMAC,
                tcls::DCMAC,
                "DCMAC",
                "DCMAC_TILE",
                "DCMAC",
                true,
            ),
            (bslots::ILKN, tcls::ILKN, "ILKN", "ILKN_TILE", "ILKNF", true),
            (bslots::HSC, tcls::HSC, "HSC", "HSC_TILE", "HSC", true),
        ] {
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bel = self.builder.bel_xy(slot, sk, 0, 0);
                if slot == bslots::PCIE4 {
                    for i in 0..57 {
                        bel = bel.pin_name_only(&format!("XILUNCONNOUT_{i}_"), 1);
                    }
                }
                if slot == bslots::PCIE5 {
                    for i in 0..22 {
                        bel = bel.pin_name_only(&format!("XILUNCONNOUT_{i}_"), 1);
                    }
                }
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_HB");
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W_HB");
                let height = if is_large { 96 } else { 48 };
                let mut xn = self.builder.xtile_id(tcid, naming, xy);
                for i in 0..height {
                    let intf_w_xy = xn.builder.delta(xy, -1, (i + i / 4) as i32);
                    let intf_e_xy = xn.builder.delta(xy, 1, (i + i / 4) as i32);
                    xn = xn.ref_single(intf_w_xy, i, intf_e).ref_single(
                        intf_e_xy,
                        i + height,
                        intf_w,
                    )
                }
                xn.bel(bel).extract();
            }
        }
    }

    fn classify_hdiologic_pin(&self, pname: &str) -> Option<(String, usize)> {
        if let Some(suffix) = pname.strip_prefix("IDLYM_") {
            Some((format!("IDLY_{suffix}"), 0))
        } else if let Some(suffix) = pname.strip_prefix("IDLYS_") {
            Some((format!("IDLY_{suffix}"), 1))
        } else if let Some(suffix) = pname.strip_prefix("ODLYM_") {
            Some((format!("ODLY_{suffix}"), 0))
        } else if let Some(suffix) = pname.strip_prefix("ODLYS_") {
            Some((format!("ODLY_{suffix}"), 1))
        } else if let Some(suffix) = pname.strip_prefix("IPFFM_") {
            Some((format!("IPFF_{suffix}"), 0))
        } else if let Some(suffix) = pname.strip_prefix("IPFFS_") {
            Some((format!("IPFF_{suffix}"), 1))
        } else if let Some(suffix) = pname.strip_prefix("OPFFM_") {
            Some((format!("OPFF_{suffix}"), 0))
        } else if let Some(suffix) = pname.strip_prefix("OPFFS_") {
            Some((format!("OPFF_{suffix}"), 1))
        } else if let Some(suffix) = pname.strip_prefix("TFFM_") {
            Some((format!("TFF_{suffix}"), 0))
        } else if let Some(suffix) = pname.strip_prefix("TFFS_") {
            Some((format!("TFF_{suffix}"), 1))
        } else {
            None
        }
    }

    pub fn fill_tiles_hdio(&mut self) {
        for tkn in ["HDIO_TILE", "HDIO_BOT_TILE"] {
            for i in 0..4 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "BUFGCE_HDIO",
                    0,
                    i,
                    "O",
                    wires::OUT_BUFGCE_HDIO[i].cell(0),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "BUFGCE_HDIO",
                    0,
                    i,
                    "I",
                    wires::IMUX_BUFGCE_HDIO[i].cell(0),
                    true,
                );
                for (j, w) in [
                    wires::IMUX_BUFGCE_HDIO_DUMMY0,
                    wires::IMUX_BUFGCE_HDIO_DUMMY1,
                    wires::IMUX_BUFGCE_HDIO_DUMMY2,
                    wires::IMUX_BUFGCE_HDIO_DUMMY3,
                    wires::IMUX_BUFGCE_HDIO_DUMMY4,
                    wires::IMUX_BUFGCE_HDIO_DUMMY5,
                    wires::IMUX_BUFGCE_HDIO_DUMMY6,
                    wires::IMUX_BUFGCE_HDIO_DUMMY7,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder
                        .extra_name_tile(tkn, format!("VCC_WIRE{k}", k = i * 8 + j), w[i]);
                }
            }
            for (w, pin) in wires::OUT_DPLL
                .into_iter()
                .zip(["CLKOUT0", "CLKOUT1", "CLKOUT2", "CLKOUT3", "TMUXOUT"])
            {
                self.builder
                    .extra_name_pin_xy(tkn, "DPLL", 0, 0, pin, w.cell(0), true);
            }
            self.builder.extra_name_pin_xy(
                tkn,
                "DPLL",
                0,
                0,
                "CLKIN",
                wires::IMUX_DPLL_CLKIN.cell(0),
                true,
            );
            self.builder.extra_name_pin_xy(
                tkn,
                "DPLL",
                0,
                0,
                "CLKIN_DESKEW",
                wires::IMUX_DPLL_CLKIN_DESKEW.cell(0),
                true,
            );
            self.builder.extra_name_pin_xy(
                tkn,
                "IOB",
                0,
                5,
                "RXOUT_M",
                wires::HDIO_CCIO[0].cell(0),
                true,
            );
            self.builder.extra_name_pin_xy(
                tkn,
                "IOB",
                0,
                6,
                "RXOUT_M",
                wires::HDIO_CCIO[1].cell(0),
                true,
            );
            self.builder
                .extra_name_tile(tkn, "IF_RCLK_CLK_TO_DPLL", wires::IMUX_DPLL_HDIO_RCLK);
            self.builder
                .extra_name_tile(tkn, "VCC_WIRE32", wires::IMUX_DPLL_CLKIN_DESKEW_DUMMY[0]);
            self.builder
                .extra_name_tile(tkn, "VCC_WIRE33", wires::IMUX_DPLL_CLKIN_DESKEW_DUMMY[1]);
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                let mut bels = vec![];
                for i in 0..11 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDIOLOGIC[i], "HDIOLOGIC", 0, i)
                            .manual()
                            .pin_name_only("TFFM_Q", 1)
                            .pin_name_only("TFFS_Q", 1)
                            .pin_name_only("OPFFM_Q", 1)
                            .pin_name_only("OPFFS_Q", 1)
                            .pin_name_only("IPFFM_D", 0)
                            .pin_name_only("IPFFS_D", 0),
                    );
                }
                for i in 0..11 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::HDIOB[i], "IOB", 0, i)
                            .manual()
                            .pin_name_only("RXOUT_M", 1)
                            .pin_name_only("RXOUT_S", 1)
                            .pin_name_only("OP_M", 0)
                            .pin_name_only("OP_S", 0)
                            .pin_name_only("TRISTATE_M", 0)
                            .pin_name_only("TRISTATE_S", 0),
                    );
                }
                for i in 0..4 {
                    bels.push(
                        self.builder
                            .bel_xy(bslots::BUFGCE_HDIO[i], "BUFGCE_HDIO", 0, i),
                    );
                }
                bels.push(self.builder.bel_xy(bslots::DPLL_HDIO, "DPLL", 0, 0));
                bels.push(self.builder.bel_xy(bslots::HDIO_BIAS, "HDIO_BIAS", 0, 0));
                bels.push(self.builder.bel_xy(bslots::RPI_HD_APB, "RPI_HD_APB", 0, 0));
                bels.push(
                    self.builder
                        .bel_xy(bslots::HDLOGIC_APB, "HDLOGIC_APB", 0, 0),
                );
                let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E_HB");
                let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W_HB");
                let mut xn = self
                    .builder
                    .xtile_id(tcls::HDIO, "HDIO", xy)
                    .switchbox(bslots::BEL_INT)
                    .optin_muxes(&[wires::IMUX_DPLL_CLKIN, wires::IMUX_DPLL_CLKIN_DESKEW])
                    .optin_muxes(&wires::IMUX_BUFGCE_HDIO[..])
                    .bels(bels);
                for i in 0..48 {
                    let intf_w_xy = xn.builder.delta(xy, -1, (i + i / 4) as i32);
                    let intf_e_xy = xn.builder.delta(xy, 1, (i + i / 4) as i32);
                    xn = xn
                        .ref_single(intf_w_xy, i, intf_e)
                        .ref_single(intf_e_xy, i + 48, intf_w)
                }
                let mut xt = xn.extract();
                let orig_iob = xt.bels.split_off(11);
                let orig_iol = xt.bels;
                for (i, (bel, mut bn)) in orig_iob.into_iter().enumerate() {
                    let bslot0 = bslots::HDIOB[i * 2];
                    let bslot1 = bslots::HDIOB[i * 2 + 1];
                    let mut bel0 = LegacyBel::default();
                    let mut bn0 = BelNaming::default();
                    let mut bel1 = LegacyBel::default();
                    let mut bn1 = BelNaming::default();
                    bn0.tiles = bn.tiles.clone();
                    bn1.tiles = bn.tiles.clone();
                    for (pname, pin) in bel.pins {
                        let pn = bn.pins.remove(&pname).unwrap();
                        if let Some(npname) = pname.strip_suffix("_M") {
                            bel0.pins.insert(npname.to_string(), pin);
                            bn0.pins.insert(npname.to_string(), pn);
                        } else if let Some(npname) = pname.strip_suffix("_S") {
                            bn0.pins.insert(pname.to_string(), pn.clone());
                            bel1.pins.insert(npname.to_string(), pin);
                            bn1.pins.insert(npname.to_string(), pn);
                        } else {
                            println!("UMMM HDIOB PIN {pname}");
                        }
                    }
                    bn0.pins.extend(bn.pins);
                    if matches!(i, 5 | 6) {
                        bel0.pins.insert(
                            "CLKPAD".into(),
                            BelPin::new_out(wires::HDIO_CCIO[i - 5].cell(0)),
                        );
                        let pn = bn0.pins["RXOUT_M"].clone();
                        bn0.pins.insert("CLKPAD".into(), pn);
                    }
                    self.builder
                        .insert_tcls_bel(tcls::HDIO, bslot0, BelInfo::Legacy(bel0));
                    self.builder.insert_bel_naming("HDIO", bslot0, bn0);
                    self.builder
                        .insert_tcls_bel(tcls::HDIO, bslot1, BelInfo::Legacy(bel1));
                    self.builder.insert_bel_naming("HDIO", bslot1, bn1);
                }
                for (i, (bel, mut bn)) in orig_iol.into_iter().enumerate() {
                    let bslot0 = bslots::HDIOLOGIC[i * 2];
                    let bslot1 = bslots::HDIOLOGIC[i * 2 + 1];
                    let bslot_scan = bslots::HDIOLOGIC_SCAN[i];
                    let mut bel0 = LegacyBel::default();
                    let mut bn0 = BelNaming::default();
                    let mut bel1 = LegacyBel::default();
                    let mut bn1 = BelNaming::default();
                    let mut bel_scan = LegacyBel::default();
                    let mut bn_scan = BelNaming::default();
                    bn0.tiles = bn.tiles.clone();
                    bn1.tiles = bn.tiles.clone();
                    bn_scan.tiles = bn.tiles.clone();
                    for (pname, pin) in bel.pins {
                        let pn = bn.pins.remove(&pname).unwrap();
                        if let Some((npname, which)) = self.classify_hdiologic_pin(&pname) {
                            [&mut bel0, &mut bel1][which]
                                .pins
                                .insert(npname.to_string(), pin);
                            [&mut bn0, &mut bn1][which]
                                .pins
                                .insert(npname.to_string(), pn.clone());
                        } else {
                            bel_scan.pins.insert(pname.clone(), pin);
                        }
                        bn_scan.pins.insert(pname, pn);
                    }
                    bn_scan.pins.extend(bn.pins);
                    self.builder
                        .insert_tcls_bel(tcls::HDIO, bslot0, BelInfo::Legacy(bel0));
                    self.builder.insert_bel_naming("HDIO", bslot0, bn0);
                    self.builder
                        .insert_tcls_bel(tcls::HDIO, bslot1, BelInfo::Legacy(bel1));
                    self.builder.insert_bel_naming("HDIO", bslot1, bn1);
                    self.builder
                        .insert_tcls_bel(tcls::HDIO, bslot_scan, BelInfo::Legacy(bel_scan));
                    self.builder.insert_bel_naming("HDIO", bslot_scan, bn_scan);
                }
            }
        }
    }

    pub fn fill_tiles_rclk_hdio(&mut self) {
        self.builder
            .extra_name("IF_RCLK_BOT_CLK_TO_DPLL", wires::IMUX_DPLL_HDIO_RCLK_S);
        self.builder
            .extra_name("IF_RCLK_TOP_CLK_TO_DPLL", wires::IMUX_DPLL_HDIO_RCLK);
        for i in 0..4 {
            self.builder.extra_name(
                format!("IF_RCLK_BOT_CLK_FROM_BUFG{i}"),
                wires::OUT_BUFGCE_HDIO_S[i],
            );
            self.builder.extra_name(
                format!("IF_RCLK_TOP_CLK_FROM_BUFG{i}"),
                wires::OUT_BUFGCE_HDIO[i],
            );
        }
        for (tcid, tkn, naming) in [
            (tcls::RCLK_HDIO, "RCLK_HDIO_CORE", "RCLK_HDIO"),
            (tcls::RCLK_HDIO_VR, "RCLK_HDIO_VR_CORE", "RCLK_HDIO_VR"),
        ] {
            for i in 0..12 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HROUTE{i}"),
                    wires::CLK_HROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_{si}_CLK_OUT", si = 24 + i),
                    wires::CLK_HROUTE_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_ULVT_{si}_CLK_OUT", si = 24 + i),
                    wires::CLK_HROUTE_MUX[i],
                );
            }
            let swz = [
                0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2, 12, 15, 16, 17, 18, 19, 20, 21, 22, 23, 13,
                14,
            ];
            for (i, si) in swz.into_iter().enumerate() {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR{i}"),
                    wires::CLK_HDISTR[i],
                );
                match i {
                    0..8 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_HDISTR_LSB{i}"),
                            wires::CLK_HDISTR[i],
                        );
                    }
                    12..20 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_HDISTR_MSB{ii}", ii = i - 12),
                            wires::CLK_HDISTR[i],
                        );
                    }
                    8..12 | 20..24 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_CMT_DRVR_TRI_ULVT_{si}_CLK_OUT_B"),
                            wires::CLK_HDISTR_FAKE[i % 8],
                        );
                    }
                    _ => (),
                }
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_{si}_CLK_OUT"),
                    wires::CLK_HDISTR_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_ULVT_{si}_CLK_OUT"),
                    wires::CLK_HDISTR_MUX[i],
                );
            }
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR_LOC{i}"),
                    wires::CLK_HDISTR_LOC[i],
                );
            }
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&[wires::IMUX_DPLL_HDIO_RCLK, wires::IMUX_DPLL_HDIO_RCLK_S])
                    .optin_muxes(&wires::CLK_HROUTE[..])
                    .optin_muxes(&wires::CLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::CLK_HDISTR[..])
                    .optin_muxes(&wires::CLK_HDISTR_MUX[..])
                    .optin_muxes(&wires::CLK_HDISTR_FAKE[..])
                    .extract();
            }
        }

        for (tcid, tkn, naming) in [
            (tcls::RCLK_HB_HDIO, "RCLK_HB_HDIO_CORE", "RCLK_HB_HDIO"),
            (
                tcls::RCLK_HB_HDIO_VR,
                "RCLK_HB_HDIO_VR_CORE",
                "RCLK_HB_HDIO_VR",
            ),
        ] {
            for i in 0..12 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HROUTE{i}"),
                    wires::CLK_HROUTE[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_{si}_CLK_OUT", si = 24 + i),
                    wires::CLK_HROUTE_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_ULVT_{si}_CLK_OUT", si = 24 + i),
                    wires::CLK_HROUTE_MUX[i],
                );
                let b = [68, 72, 76, 80, 84, 88, 96, 100, 104, 108, 112, 116][i];
                for (j, w) in [
                    wires::CLK_HROUTE_MUX_DUMMY0,
                    wires::CLK_HROUTE_MUX_DUMMY1,
                    wires::CLK_HROUTE_MUX_DUMMY2,
                    wires::CLK_HROUTE_MUX_DUMMY3,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder
                        .extra_name_tile(tkn, format!("GND_WIRE{k}", k = b + j), w[i]);
                }
            }
            let swz = [
                0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2, 12, 15, 16, 17, 18, 19, 20, 21, 22, 23, 13,
                14,
            ];
            for (i, si) in swz.into_iter().enumerate() {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR{i}"),
                    wires::CLK_HDISTR[i],
                );
                match i {
                    0..8 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_HDISTR_LSB{i}"),
                            wires::CLK_HDISTR[i],
                        );
                    }
                    12..20 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_HDISTR_MSB{ii}", ii = i - 12),
                            wires::CLK_HDISTR[i],
                        );
                    }
                    8..12 | 20..24 => {
                        self.builder.extra_name_tile(
                            tkn,
                            format!("CLK_CMT_DRVR_TRI_ULVT_{si}_CLK_OUT_B"),
                            wires::CLK_HDISTR_FAKE[i % 8],
                        );
                    }
                    _ => (),
                }
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_{si}_CLK_OUT"),
                    wires::CLK_HDISTR_MUX[i],
                );
                self.builder.extra_name_tile(
                    tkn,
                    format!("CLK_CMT_MUX_8TO1_ULVT_{si}_CLK_OUT"),
                    wires::CLK_HDISTR_MUX[i],
                );

                let b = [
                    0, 92, 120, 124, 128, 132, 136, 140, 8, 12, 4, 48, 16, 28, 32, 36, 40, 44, 52,
                    56, 60, 64, 20, 24,
                ][i];
                for (j, w) in [
                    wires::CLK_HDISTR_MUX_DUMMY0,
                    wires::CLK_HDISTR_MUX_DUMMY1,
                    wires::CLK_HDISTR_MUX_DUMMY2,
                    wires::CLK_HDISTR_MUX_DUMMY3,
                ]
                .into_iter()
                .enumerate()
                {
                    self.builder
                        .extra_name_tile(tkn, format!("GND_WIRE{k}", k = b + j), w[i]);
                }
            }
            for i in 0..24 {
                self.builder.extra_name_tile(
                    tkn,
                    format!("IF_HCLK_CLK_HDISTR_LOC{i}"),
                    wires::CLK_HDISTR_LOC[i],
                );
            }
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_24_ENC_1_CLK_OUT",
                wires::IMUX_DPLL_HDIO_RCLK,
            );
            self.builder.extra_name_tile(
                tkn,
                "CLK_CMT_MUX_24_ENC_ULVT_1_CLK_OUT",
                wires::IMUX_DPLL_HDIO_RCLK,
            );
            if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
                self.builder
                    .xtile_id(tcid, naming, xy)
                    .switchbox(bslots::RCLK_BEL_INT)
                    .optin_muxes(&[wires::IMUX_DPLL_HDIO_RCLK, wires::IMUX_DPLL_HDIO_RCLK_S])
                    .optin_muxes(&wires::CLK_HROUTE[..])
                    .optin_muxes(&wires::CLK_HROUTE_MUX[..])
                    .optin_muxes(&wires::CLK_HDISTR[..])
                    .optin_muxes(&wires::CLK_HDISTR_MUX[..])
                    .optin_muxes(&wires::CLK_HDISTR_FAKE[..])
                    .extract();
            }
        }
    }

    pub fn fill_tiles_vnoc(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("AMS_SAT_VNOC_TILE")
            .iter()
            .next()
            && self.rd.slot_kinds.contains("SYSMON_SAT")
        {
            let bel = self
                .builder
                .bel_xy(bslots::SYSMON_SAT_VNOC, "SYSMON_SAT", 0, 0);
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E");
            let mut xn = self
                .builder
                .xtile_id(tcls::SYSMON_SAT_VNOC, "SYSMON_SAT_VNOC", xy);
            for i in 0..48 {
                let intf_xy = xn.builder.delta(xy, -1, -49 + (i + i / 4) as i32);
                xn = xn.ref_single(intf_xy, i, intf_e)
            }
            xn.bel(bel).extract();
        }

        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("MISR_TILE")
            .iter()
            .next()
        {
            let bel = self.builder.bel_xy(bslots::MISR, "MISR", 0, 0);
            let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W");
            let mut xn = self.builder.xtile_id(tcls::MISR, "MISR", xy);
            for i in 0..48 {
                let intf_xy = xn.builder.delta(xy, 1, 1 + (i + i / 4) as i32);
                xn = xn.ref_single(intf_xy, i + 48, intf_w)
            }
            xn.bel(bel).extract();
        }

        if let Some(&nsu_xy) = self
            .builder
            .rd
            .tiles_by_kind_name("NOC_NSU512_TOP")
            .iter()
            .next()
        {
            let nps_a_xy = self.builder.delta(nsu_xy, 0, 9);
            let nps_b_xy = self.builder.delta(nsu_xy, 0, 19);
            let nmu_xy = self.builder.delta(nsu_xy, 0, 29);
            let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E");
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W");
            let bels = [
                self.builder
                    .bel_xy(bslots::VNOC_NSU512, "NOC_NSU512", 0, 0)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS_NSU, "NOC_NPS_VNOC", 0, 0)
                    .raw_tile(1)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS_NMU, "NOC_NPS_VNOC", 0, 0)
                    .raw_tile(2)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NMU512, "NOC_NMU512", 0, 0)
                    .raw_tile(3)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
            ];
            let mut xn = self
                .builder
                .xtile_id(tcls::VNOC, "VNOC", nsu_xy)
                .raw_tile(nps_a_xy)
                .raw_tile(nps_b_xy)
                .raw_tile(nmu_xy);
            for i in 0..48 {
                let intf_w_xy = xn.builder.delta(nsu_xy, -1, -9 + (i + i / 4) as i32);
                let intf_e_xy = xn.builder.delta(nsu_xy, 2, -9 + (i + i / 4) as i32);
                xn = xn
                    .ref_single(intf_w_xy, i, intf_w)
                    .ref_single(intf_e_xy, i + 48, intf_e)
            }
            xn.bels(bels).extract();
        }

        if let Some(&nsu_xy) = self
            .builder
            .rd
            .tiles_by_kind_name("NOC2_NSU512_VNOC_TILE")
            .iter()
            .next()
        {
            let mut nps_a_xy = self.builder.delta(nsu_xy, 0, 4);
            if self
                .builder
                .rd
                .tile_kinds
                .key(self.rd.tiles[&nps_a_xy].kind)
                == "NULL"
            {
                nps_a_xy = self.builder.delta(nps_a_xy, -1, 0);
            }
            let nps_b_xy = self.builder.delta(nps_a_xy, 0, 4);
            let nmu_xy = self.builder.delta(nps_a_xy, 0, 7);
            let scan_xy = self.builder.delta(nsu_xy, 1, 0);
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_E");
            let intf_w = self.builder.ndb.get_tile_class_naming("INTF_W");
            let mut bel_scan = self
                .builder
                .bel_xy(bslots::VNOC_SCAN, "NOC2_SCAN", 0, 0)
                .raw_tile(4);
            for i in 6..15 {
                bel_scan = bel_scan
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"), 1)
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_TO_PL_{i}_"), 1);
            }
            for i in 5..14 {
                bel_scan = bel_scan.pin_name_only(&format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"), 1);
            }
            let bels = [
                self.builder
                    .bel_xy(bslots::VNOC_NSU512, "NOC2_NSU512", 0, 0)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS_NSU, "NOC2_NPS5555", 0, 0)
                    .raw_tile(1)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS_NMU, "NOC2_NPS5555", 0, 0)
                    .raw_tile(2)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NMU512, "NOC2_NMU512", 0, 0)
                    .raw_tile(3)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
                bel_scan,
            ];
            let mut xn = self
                .builder
                .xtile_id(tcls::VNOC2, "VNOC2", nsu_xy)
                .raw_tile(nps_a_xy)
                .raw_tile(nps_b_xy)
                .raw_tile(nmu_xy)
                .raw_tile(scan_xy);
            for i in 0..48 {
                let intf_l_xy = xn.builder.delta(nps_a_xy, -1, -13 + (i + i / 4) as i32);
                let intf_r_xy = xn.builder.delta(nps_a_xy, 3, -13 + (i + i / 4) as i32);
                xn = xn
                    .ref_single(intf_l_xy, i, intf_e)
                    .ref_single(intf_r_xy, i + 48, intf_w)
            }
            xn.bels(bels).extract();
        }

        if let Some(&nsu_xy) = self
            .builder
            .rd
            .tiles_by_kind_name("NOC2_NSU512_VNOC4_TILE")
            .iter()
            .next()
        {
            let nps_a_xy = self.builder.delta(nsu_xy, -1, 4);
            let nps_b_xy = self.builder.delta(nps_a_xy, 0, 4);
            let nmu_xy = self.builder.delta(nps_a_xy, 0, 7);
            let scan_xy = self.builder.delta(nsu_xy, 1, 0);
            let intf_w = self.builder.ndb.get_tile_class_naming("INTF_E");
            let intf_e = self.builder.ndb.get_tile_class_naming("INTF_W");
            let mut bel_scan = self
                .builder
                .bel_xy(bslots::VNOC_SCAN, "NOC2_SCAN", 0, 0)
                .raw_tile(4);
            for i in 7..15 {
                bel_scan = bel_scan
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_FROM_PL_{i}_"), 1)
                    .pin_name_only(&format!("NOC2_SCAN_CHNL_TO_PL_{i}_"), 1);
            }
            for i in 7..14 {
                bel_scan = bel_scan.pin_name_only(&format!("NOC2_SCAN_CHNL_MASK_FROM_PL_{i}_"), 1);
            }
            let bels = [
                self.builder
                    .bel_xy(bslots::VNOC_NSU512, "NOC2_NSU512", 0, 0)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS6X_NSU, "NOC2_NPS6X", 0, 0)
                    .raw_tile(1)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("IN_4", 1)
                    .pin_name_only("IN_5", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1)
                    .pin_name_only("OUT_4", 1)
                    .pin_name_only("OUT_5", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NPS6X_NMU, "NOC2_NPS6X", 0, 0)
                    .raw_tile(2)
                    .pin_name_only("IN_0", 1)
                    .pin_name_only("IN_1", 1)
                    .pin_name_only("IN_2", 1)
                    .pin_name_only("IN_3", 1)
                    .pin_name_only("IN_4", 1)
                    .pin_name_only("IN_5", 1)
                    .pin_name_only("OUT_0", 1)
                    .pin_name_only("OUT_1", 1)
                    .pin_name_only("OUT_2", 1)
                    .pin_name_only("OUT_3", 1)
                    .pin_name_only("OUT_4", 1)
                    .pin_name_only("OUT_5", 1),
                self.builder
                    .bel_xy(bslots::VNOC_NMU512, "NOC2_NMU512", 0, 0)
                    .raw_tile(3)
                    .pin_name_only("TO_NOC", 1)
                    .pin_name_only("FROM_NOC", 1),
                bel_scan,
            ];
            let mut xn = self
                .builder
                .xtile_id(tcls::VNOC4, "VNOC4", nsu_xy)
                .raw_tile(nps_a_xy)
                .raw_tile(nps_b_xy)
                .raw_tile(nmu_xy)
                .raw_tile(scan_xy);
            for i in 0..48 {
                let intf_w_xy = xn.builder.delta(nps_a_xy, -1, -13 + (i + i / 4) as i32);
                let intf_e_xy = xn.builder.delta(nps_a_xy, 5, -13 + (i + i / 4) as i32);

                xn = xn
                    .ref_single(intf_w_xy, i, intf_w)
                    .ref_single(intf_e_xy, i + 48, intf_e)
            }
            xn.bels(bels).extract();
        }
    }
}
