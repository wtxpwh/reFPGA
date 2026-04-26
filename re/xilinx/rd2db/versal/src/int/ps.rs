use prjcombine_interconnect::{db::WireSlotIdExt, dir::Dir};
use prjcombine_versal::{
    chip::FabricKind,
    defs::{bslots, tcls, wires},
};

use crate::{grid::get_fabric, int::IntMaker};

impl<'rd> IntMaker<'rd> {
    fn fill_tiles_ps9(&mut self) {
        let fabric = get_fabric(self.rd);
        let tkn = "PSS_BASE_CORE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let naming = if self
                .builder
                .rd
                .wires
                .contains("PSS_BASE_L80_ATOM_0_NOC_PMC_0_PIN")
            {
                "PS9_2VP"
            } else if fabric != FabricKind::Orig {
                "PS9_REV2"
            } else {
                "PS9"
            };
            let mut bel = self.builder.bel_xy(bslots::PS9, "PS9", 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("PS9", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for i in 0..6 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PS9",
                    0,
                    0,
                    format!("LPDRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i].cell(48),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PS9",
                    0,
                    0,
                    format!("PMCRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i + 6].cell(48),
                    true,
                );
            }
            for (pin, wire) in [
                ("HSMREFCLK_0_", wires::OUT_PS_HSM_REF_CLK[0].cell(96)),
                ("HSMREFCLK_1_", wires::OUT_PS_HSM_REF_CLK[1].cell(96)),
                ("NOCCLK", wires::OUT_PS_NOC_CLK.cell(96)),
                ("NPICLK", wires::OUT_PS_NPI_CLK.cell(96)),
                ("NTISCANCLKIN", wires::OUT_PS_SCAN_CLK.cell(96)),
                ("MCLK", wires::OUT_PS_MCLK.cell(96)),
                (
                    "PMCNOCPPLLTESTCLKOUT_1_",
                    wires::HSR_CLK_TEST_IN[0].cell(96),
                ),
                (
                    "PMCNOCPPLLTESTCLKOUT_0_",
                    wires::HSR_CLK_TEST_IN[1].cell(96),
                ),
                (
                    "PMCNOCNPLLTESTCLKOUT_1_",
                    wires::HSR_CLK_TEST_IN[2].cell(96),
                ),
                (
                    "PMCNOCNPLLTESTCLKOUT_0_",
                    wires::HSR_CLK_TEST_IN[3].cell(96),
                ),
                (
                    "FPDNOCLPDPLLTESTCLKOUT_0_",
                    wires::HSR_CLK_TEST_IN[0].cell(97),
                ),
                (
                    "FPDNOCFPDPLLTESTCLKOUT_1_",
                    wires::HSR_CLK_TEST_IN[1].cell(97),
                ),
                (
                    "FPDNOCFPDPLLTESTCLKOUT_0_",
                    wires::HSR_CLK_TEST_IN[2].cell(97),
                ),
                (
                    "FPDNOCCPMPLLTESTCLKOUT_1_",
                    wires::HSR_CLK_TEST_IN[3].cell(97),
                ),
                (
                    "FPDNOCCPMPLLTESTCLKOUT_0_",
                    wires::HSR_CLK_TEST_IN[4].cell(97),
                ),
                (
                    "FPDNOCLPDPLLTESTCLKOUT_1_",
                    wires::HSR_CLK_TEST_IN[5].cell(97),
                ),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "PS9", 0, 0, pin, wire, true);
            }
            self.builder
                .extra_name_tile_tw(tkn, "GND_WIRE", wires::OUT_PS_MCLK.cell(96));
            for pin in tksite.pins.keys() {
                // dummy VCC_WIRE
                if pin == "NOCPORN" {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NPI
                if pin == "NPIRSTN"
                    || pin.starts_with("NPIINTERRUPTOUT")
                    || pin == "PSPMCNOCNPIAXICLK"
                    || pin.starts_with("IFPMCNOCNPI")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NOC
                if matches!(
                    pin.as_str(),
                    "PCIE_NOC_0"
                        | "PCIE_NOC_1"
                        | "PMC_NOC_0"
                        | "RPU_NOC_0"
                        | "NCI_NOC_0"
                        | "NCI_NOC_1"
                        | "CCI_NOC_0"
                        | "CCI_NOC_1"
                        | "CCI_NOC_2"
                        | "CCI_NOC_3"
                        | "NOC_PCIE_0"
                        | "NOC_PMC_0"
                        | "NOC_CCI_0"
                        | "NOC_CCI_1"
                        | "NOC_NCI_0"
                        | "NOC_NCI_1"
                ) {
                    bel = bel.pin_name_only(pin, 1);
                }
                // XRAM
                if pin.starts_with("CPUR5")
                    || pin.contains("OCM2")
                    || pin.starts_with("LPDLSBUS")
                    || pin.starts_with("LPDXRAM")
                    || pin.starts_with("OCMRAWRSTN")
                    || pin == "PSXRAMSUPPLYSTATUSVCCINTLPD"
                    || pin.starts_with("SCANOCM2")
                    || pin.starts_with("XRAMLPD")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // CPM
                if pin.starts_with("CPM")
                    || pin == "EFUSECPMCCIXDIS"
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xn = self.builder.xtile_id(tcls::PS9, naming, xy);
            let intf_w = xn.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xn
                .builder
                .walk_to_int(xn.builder.delta(xy, 0, -21), Dir::E, false)
                .unwrap();
            for i in 0..96 {
                let cxy = xn.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_w);
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_cpm4(&mut self) {
        if let Some(&core_xy) = self.rd.tiles_by_kind_name("CPM_CORE").iter().next()
            && let Some(&ext_xy) = self
                .builder
                .rd
                .tiles_by_kind_name("CPM_EXT_TILE")
                .iter()
                .next()
        {
            for (pin, wire) in [
                ("CPMDPLLPCIE0USERCLK", wires::OUT_CPM_CLK[0].cell(0)),
                ("CPMDPLLPCIE1USERCLK", wires::OUT_CPM_CLK[1].cell(0)),
            ] {
                self.builder
                    .extra_name_pin_xy("CPM_CORE", "CPM_MAIN", 0, 0, pin, wire, true);
            }

            let mut bel = self.builder.bel_xy(bslots::CPM4, "CPM_MAIN", 0, 0);

            let tile = &self.rd.tiles[&core_xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("CPM_MAIN", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                // PS9
                if pin.starts_with("CPMPLPLL")
                    || pin == "CPMOSCCLKDIV2"
                    || pin == "EFUSECPMCCIXDIS"
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // EXT
                if pin.starts_with("DFXE2M")
                    || pin.starts_with("IFCPMEXT")
                    || pin.starts_with("IFEXTCPM")
                    || pin.starts_with("PCIEAEXT")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // XPIPE
                if pin.starts_with("IFCPMXPIPE") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            bel = bel.sub_xy("CPM_EXT", 0, 0).raw_tile(1);

            let tile = &self.rd.tiles[&ext_xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("CPM_EXT", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                if pin.starts_with("DFXE2M")
                    || pin.starts_with("IFCPMEXT")
                    || pin.starts_with("IFEXTCPM")
                    || pin.starts_with("PCIEAEXT")
                {
                    let rpin = &format!("_EXT_{pin}");
                    bel = bel.pin_rename(pin, rpin).pin_name_only(rpin, 1);
                }
            }

            let mut xt = self
                .builder
                .xtile_id(tcls::CPM4, "CPM4", core_xy)
                .raw_tile(ext_xy)
                .bel(bel);

            let intf_w = xt.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xt
                .builder
                .walk_to_int(xt.builder.delta(core_xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..48 {
                let cxy = xt.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xt = xt.ref_single(cxy, i, intf_w);
            }

            let intf_e = xt.builder.ndb.get_tile_class_naming("INTF_E_HB");
            let intf_w = xt.builder.ndb.get_tile_class_naming("INTF_W_HB");
            for i in 0..48 {
                let intf_w_xy = xt.builder.delta(ext_xy, -1, (i + i / 4) as i32);
                let intf_e_xy = xt.builder.delta(ext_xy, 1, (i + i / 4) as i32);
                xt = xt
                    .ref_single(intf_w_xy, 48 + i, intf_e)
                    .ref_single(intf_e_xy, 96 + i, intf_w)
            }

            xt.extract();
        }
    }

    fn fill_tiles_cpm5(&mut self) {
        let tkn = "CPM_G5_TILE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let bname = "CPM5";

            for i in 0..2 {
                for j in 0..12 {
                    let pin = format!("CPM5RCLK{i}_{j}_");
                    let wire = wires::CLK_HROUTE[j].cell(48 + 96 * i);
                    self.builder
                        .extra_name_pin_xy(tkn, bname, 0, 0, pin, wire, true);
                }
            }

            let mut bel = self.builder.bel_xy(bslots::CPM5, bname, 0, 0);

            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy(bname, 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                // PS9
                if pin.starts_with("CPMPLPLL")
                    || pin == "CPMOSCCLKDIV2"
                    || pin == "EFUSECPMCCIXDIS"
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // GTYP
                if pin.starts_with("IFBUFGT") || pin.starts_with("IFF") || pin.starts_with("SCAN") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // dummy
                if pin.starts_with("Q") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xt = self.builder.xtile_id(tcls::CPM5, "CPM5", xy).bel(bel);

            let intf_w = xt.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xt
                .builder
                .walk_to_int(xt.builder.delta(xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..192 {
                let cxy = xt.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xt = xt.ref_single(cxy, i, intf_w);
            }

            xt.extract();
        }
    }

    fn fill_tiles_cpm6(&mut self) {
        let tkn = "CPM6_TILE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let bname = "CPM6";
            for i in 0..4 {
                for j in 0..15 {
                    let pin = format!("CPMCLKBUFGT{a}_{b}_", a = i / 2, b = i % 2 * 15 + j);
                    let wire = wires::OUT_GT_CLK[j].cell(i * 48);
                    self.builder
                        .extra_name_pin_xy(tkn, bname, 0, 0, pin, wire, true);
                }
                for j in 0..5 {
                    let pin = format!("CPMCLKDPLL{a}_{b}_", a = i / 2, b = i % 2 * 5 + j);
                    let wire = wires::OUT_DPLL_GT[j].cell(i * 48);
                    self.builder
                        .extra_name_pin_xy(tkn, bname, 0, 0, pin, wire, true);
                }
            }

            let mut bel = self.builder.bel_xy(bslots::CPM6, bname, 0, 0);

            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy(bname, 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                // PS9
                if pin.starts_with("CPMPLPLL")
                    || pin == "CPMOSCCLKDIV2"
                    || pin == "EFUSECPMCCIXDIS"
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // GTM2
                if pin.starts_with("BOTGTOCT")
                    || pin.starts_with("IFBOTGT")
                    || pin.starts_with("TOPGTOCT")
                    || pin.starts_with("IFTOPGT")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xt = self.builder.xtile_id(tcls::CPM6, "CPM6", xy).bel(bel);

            let intf_w = xt.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xt
                .builder
                .walk_to_int(xt.builder.delta(xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..192 {
                let cxy = xt.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xt = xt.ref_single(cxy, i, intf_w);
            }

            xt.extract();
        }
    }

    fn fill_tiles_psxl(&mut self) {
        let tkn = "PSXL_CORE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let mut bel = self.builder.bel_xy(bslots::PSXL, "PSX", 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("PSX", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for i in 0..6 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PSX",
                    0,
                    0,
                    format!("LPDRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i].cell(48),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PSX",
                    0,
                    0,
                    format!("PMCRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i + 6].cell(48),
                    true,
                );
            }
            for i in 0..12 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PSX",
                    0,
                    0,
                    format!("PSPLFPDRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i].cell(144),
                    true,
                );
            }
            for (pin, wire) in [
                ("HSMREFCLK_0_", wires::OUT_PS_HSM_REF_CLK[0].cell(240)),
                ("HSMREFCLK_1_", wires::OUT_PS_HSM_REF_CLK[1].cell(240)),
                ("NPICLK", wires::OUT_PS_NPI_CLK.cell(240)),
                ("NTISCANCLKIN", wires::OUT_PS_SCAN_CLK.cell(240)),
                ("MCLK", wires::OUT_PS_MCLK.cell(240)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "PSX", 0, 0, pin, wire, true);
            }
            self.builder
                .extra_name_tile_tw(tkn, "GND_WIRE", wires::OUT_PS_MCLK.cell(96));
            for pin in tksite.pins.keys() {
                // dummy wires
                if pin.starts_with("NOCLPXSPARE")
                    || pin == "NOCPORN"
                    || pin == "NOCRSTN"
                    || pin.starts_with("SYSRST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NPI
                if pin == "NPIRSTN"
                    || pin.starts_with("NPIINTERRUPTOUT")
                    || pin == "PSXLPMXNOCNPIAXICLK"
                    || pin.starts_with("IFPMXNOCNPI")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NOC
                if matches!(
                    pin.as_str(),
                    "PSXL_NOC_PCIE_0"
                        | "PSXL_NOC_PCIE_1"
                        | "PSXL_NOC_PCIE_2"
                        | "PSXL_NOC_PCIE_3"
                        | "PMC_NOC_0"
                        | "RPU_NOC_0"
                        | "CMN_NOC_0"
                        | "CMN_NOC_1"
                        | "CMN_NOC_2"
                        | "CMN_NOC_3"
                        | "CMN_NOC_4"
                        | "CMN_NOC_5"
                        | "CMN_NOC_6"
                        | "CMN_NOC_7"
                        | "CMN_NOC_8"
                        | "NOC_PCIE_0"
                        | "NOC_PMC_0"
                        | "CMN_NOC_CCI_0"
                        | "CMN_NOC_CCI_1"
                        | "CMN_NOC_CCI_2"
                        | "CMN_NOC_CCI_3"
                ) {
                    bel = bel.pin_name_only(pin, 1);
                }
                // CPM
                if matches!(
                    pin.as_str(),
                    "CPM5ISPRESENT" | "CPMOSCCLKDIV2" | "CPMPSXCXSCLK" | "DBGLPDRAWRSTN"
                ) || pin.starts_with("EFUSE")
                    || pin.starts_with("IFCMNCML256")
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFCPM5NPSX")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("IFPSXCPM5N")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xn = self.builder.xtile_id(tcls::PSXL, "PSXL", xy);
            let intf_w = xn.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xn
                .builder
                .walk_to_int(xn.builder.delta(xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..240 {
                let cxy = xn.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_w);
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_cpm5n(&mut self) {
        let tkn = "CPM_G5N2X_TILE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let bname = "CPM5N";

            for i in 0..2 {
                for j in 0..12 {
                    let pin = format!("CPMRCLK{i}_{j}_");
                    let wire = wires::CLK_HROUTE[j].cell(96 * i);
                    self.builder
                        .extra_name_pin_xy(tkn, bname, 0, 0, pin, wire, true);
                }
            }

            let mut bel = self.builder.bel_xy(bslots::CPM5N, bname, 0, 0);

            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy(bname, 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                // PSXL
                if matches!(
                    pin.as_str(),
                    "CPM5ISPRESENT" | "CPMOSCCLKDIV2" | "CPMPSXCXSCLK" | "DBGLPDRAWRSTN"
                ) || pin.starts_with("EFUSE")
                    || pin.starts_with("IFCMNCML256")
                    || pin.starts_with("IFCPMPS")
                    || pin.starts_with("IFCPM5NPSX")
                    || pin.starts_with("IFPSCPM")
                    || pin.starts_with("IFPSXCPM5N")
                    || pin.starts_with("LPDCPM")
                    || pin.starts_with("PERST")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NOC
                if pin.starts_with("CDX_") || pin.starts_with("DPU_") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // GTYP
                if pin.starts_with("IFBUFGT") || pin.starts_with("IFF") || pin.starts_with("SCAN") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // dummy
                if pin.starts_with("Q") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xt = self.builder.xtile_id(tcls::CPM5N, "CPM5N", xy).bel(bel);

            let intf_w = xt.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xt
                .builder
                .walk_to_int(xt.builder.delta(xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..192 {
                let cxy = xt.builder.delta(int_xy, -1, (i + i / 4 + i / 144) as i32);
                xt = xt.ref_single(cxy, i, intf_w);
            }

            xt.extract();
        }
    }

    fn fill_tiles_psxc(&mut self) {
        let tkn = "PSXC_TILE";
        if let Some(&xy) = self.rd.tiles_by_kind_name(tkn).iter().next() {
            let mut bel = self.builder.bel_xy(bslots::PSXC, "PS11", 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("PS11", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for i in 0..6 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PS11",
                    0,
                    0,
                    format!("LPDRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i].cell(48),
                    true,
                );
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PS11",
                    0,
                    0,
                    format!("PMCRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i + 6].cell(48),
                    true,
                );
            }
            for i in 0..12 {
                self.builder.extra_name_pin_xy(
                    tkn,
                    "PS11",
                    0,
                    0,
                    format!("PSPLFPDRCLKCLK_{i}_"),
                    wires::CLK_HROUTE[i].cell(144),
                    true,
                );
            }
            for (pin, wire) in [
                ("HSMREFCLK_0_", wires::OUT_PS_HSM_REF_CLK[0].cell(192)),
                ("HSMREFCLK_1_", wires::OUT_PS_HSM_REF_CLK[1].cell(192)),
                ("NPICLK", wires::OUT_PS_NPI_CLK.cell(192)),
                ("NTISCANCLKIN", wires::OUT_PS_SCAN_CLK.cell(192)),
                ("MCLK", wires::OUT_PS_MCLK.cell(192)),
            ] {
                self.builder
                    .extra_name_pin_xy(tkn, "PS11", 0, 0, pin, wire, true);
            }
            self.builder
                .extra_name_tile_tw(tkn, "GND_WIRE", wires::OUT_PS_MCLK.cell(96));
            for pin in tksite.pins.keys() {
                // dummy wires
                if pin == "NOCPORN"
                    || pin == "NOCRSTN"
                    || pin.starts_with("SYSRST")
                    || pin.starts_with("PSPLFPDRCLKCLKAPEX")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NPI
                if pin == "NPIRSTN"
                    || pin.starts_with("NPIINTERRUPTOUT")
                    || pin == "PSXLPMXNOCNPIAXICLK"
                    || pin.starts_with("IFPMXNOCNPI")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // NOC
                if matches!(
                    pin.as_str(),
                    "PMC_NOC_0"
                        | "RPU_NOC_0"
                        | "CMN_NOC_0"
                        | "CMN_NOC_1"
                        | "CMN_NOC_2"
                        | "CMN_NOC_3"
                        | "CMN_NOC_4"
                        | "CMN_NOC_5"
                        | "CMN_NOC_6"
                        | "CMN_NOC_7"
                        | "CMN_NOC_8"
                        | "MMI_NOC_0"
                        | "NOC_PMC_0"
                        | "CMN_NOC_CCI_0"
                        | "CMN_NOC_CCI_1"
                        | "CMN_NOC_CCI_2"
                        | "CMN_NOC_CCI_3"
                ) {
                    bel = bel.pin_name_only(pin, 1);
                }
                // MMI
                if matches!(
                    pin.as_str(),
                    "EFUSEBCFMTDIS"
                        | "FPDINITVCCAUXLVLSHB"
                        | "FPDMMIPORN"
                        | "FPXMMITOPSWCLK"
                        | "GTMMIPSPLFTIDMONOUTCLK"
                        | "GTPSHSDPRXOUTCLK"
                        | "GTPSHSDPTXOUTCLK"
                        | "HSDPPERSTN"
                        | "MMIISPRESENT"
                        | "MMIPSMDC"
                        | "MMIPSMDIOEN"
                        | "MMIPSMDIOOUT"
                        | "PSAXICFGCLK"
                        | "PSGEMTSURSTN"
                        | "PSGTHSDPPIPECLK"
                        | "PSGTHSDPRXUSRCLK"
                        | "PSTSUCLK"
                ) || pin.starts_with("IFMMIPS")
                    || pin.starts_with("IFPSMMI")
                    || pin.starts_with("IFPSPLMMI")
                    || pin.starts_with("PSPLMMI")
                    || pin.starts_with("IFPSGTYP")
                    || pin.starts_with("MMIRCLK")
                    || pin.starts_with("PSMMI")
                    || pin.starts_with("PCIE")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }

            let mut xn = self.builder.xtile_id(tcls::PSXC, "PSXC", xy);
            let intf_w = xn.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
            let int_xy = xn
                .builder
                .walk_to_int(xn.builder.delta(xy, 0, 1), Dir::E, false)
                .unwrap();
            for i in 0..192 {
                let cxy = xn.builder.delta(int_xy, -1, (i + i / 4) as i32);
                xn = xn.ref_single(cxy, i, intf_w);
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_mmi(&mut self) {
        if let Some(&xy) = self.rd.tiles_by_kind_name("MMI_TILE").iter().next() {
            let has_int = self.rd.slot_kinds.contains("MMI");

            let bk = if has_int { "MMI" } else { "MMI2" };
            let mut bel = self.builder.bel_xy(bslots::MMI, bk, 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy(bk, 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                if !has_int {
                    bel = bel.pin_name_only(pin, 1);
                    continue;
                }
                // PSXC
                if matches!(
                    pin.as_str(),
                    "EFUSEBCFMTDIS"
                        | "FPDINITVCCAUXLVLSHB"
                        | "FPDMMIPORN"
                        | "FPXMMITOPSWCLK"
                        | "GTMMIPSPLFTIDMONOUTCLK"
                        | "GTPSHSDPRXOUTCLK"
                        | "GTPSHSDPTXOUTCLK"
                        | "HSDPPERSTN"
                        | "MMIISPRESENT"
                        | "MMIPSMDC"
                        | "MMIPSMDIOEN"
                        | "MMIPSMDIOOUT"
                        | "PSAXICFGCLK"
                        | "PSGEMTSURSTN"
                        | "PSGTHSDPPIPECLK"
                        | "PSGTHSDPRXUSRCLK"
                        | "PSTSUCLK"
                ) || pin.starts_with("IFMMIPS")
                    || pin.starts_with("IFPSMMI")
                    || pin.starts_with("IFPSPLMMI")
                    || pin.starts_with("PSPLMMI")
                    || pin.starts_with("IFPSGTYP")
                    || pin.starts_with("MMIRCLK")
                    || pin.starts_with("PSMMI")
                    || pin.starts_with("PCIE")
                {
                    bel = bel.pin_name_only(pin, 1);
                }
                // GTYP
                if pin.starts_with("IFMMIGTYP") {
                    bel = bel.pin_name_only(pin, 1);
                }
                // dummy
                if matches!(
                    pin.as_str(),
                    "SYSRST1N" | "SYSRST2N" | "SYSRST3N" | "NPICLKEPXPIPE" | "PORRESETN"
                ) {
                    bel = bel.pin_name_only(pin, 1);
                }
                // discard
                if pin.starts_with("UNUSEDTIEHIGH") {
                    bel = bel.pin_name_only(pin, 1);
                }
            }
            let mut xn = self.builder.xtile_id(
                if has_int { tcls::MMI } else { tcls::MMI_NC },
                if has_int { "MMI" } else { "MMI_NC" },
                xy,
            );
            if has_int {
                let intf_w = xn.builder.ndb.get_tile_class_naming("INTF_W_TERM_PSS");
                let int_xy = xn
                    .builder
                    .walk_to_int(xn.builder.delta(xy, 0, 1), Dir::E, false)
                    .unwrap();
                for i in 0..96 {
                    let cxy = xn.builder.delta(int_xy, -1, (i + i / 4) as i32);
                    xn = xn.ref_single(cxy, i, intf_w);
                }
            }
            xn.bel(bel).extract();
        }
    }

    fn fill_tiles_xram(&mut self) {
        if let Some(&xy) = self
            .builder
            .rd
            .tiles_by_kind_name("XRAM_CORE")
            .iter()
            .next()
        {
            let has_int = self
                .builder
                .rd
                .tiles_by_kind_name("INT")
                .iter()
                .any(|&int_xy| int_xy.y == xy.y);

            let mut bel = self.builder.bel_xy(bslots::XRAM, "XRAM", 0, 0);
            let tile = &self.rd.tiles[&xy];
            let tk = &self.rd.tile_kinds[tile.kind];
            let slot = self.rd.slot_xy("XRAM", 0, 0).unwrap();
            let tksite = tk.sites.get(&slot).unwrap().1;
            for pin in tksite.pins.keys() {
                if !has_int || !(pin.contains("PLXRAM") || pin.contains("XRAMPL")) {
                    bel = bel.pin_name_only(pin, 1);
                }
            }
            let mut xn =
                self.builder
                    .xtile_id(if has_int { tcls::XRAM } else { tcls::XRAM_NC }, "XRAM", xy);
            if has_int {
                let intf_w = xn.builder.ndb.get_tile_class_naming("INTF_W_TERM_GT");
                let int_xy = xn.builder.walk_to_int(xy, Dir::E, false).unwrap();
                for i in 0..48 {
                    let cxy = xn.builder.delta(int_xy, -1, (i + i / 4) as i32);
                    xn = xn.ref_single(cxy, i, intf_w);
                }
            }
            xn.bel(bel).extract();
        }
    }

    pub fn fill_tiles_ps(&mut self) {
        self.fill_tiles_ps9();
        self.fill_tiles_cpm4();
        self.fill_tiles_cpm5();
        self.fill_tiles_cpm6();
        self.fill_tiles_xram();
        self.fill_tiles_psxl();
        self.fill_tiles_cpm5n();
        self.fill_tiles_psxc();
        self.fill_tiles_mmi();
    }
}
