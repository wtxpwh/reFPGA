use std::collections::BTreeSet;

use prjcombine_interconnect::db::{BelInfo, SwitchBoxItem, WireSlotIdExt};
use prjcombine_re_collector::diff::{xlat_bit, xlat_bit_wide_bi, xlat_bitvec, xlat_enum_attr};
use prjcombine_re_hammer::Session;
use prjcombine_re_xilinx_geom::ExpandedDevice;
use prjcombine_types::{bits, bsdata::TileBit};
use prjcombine_virtex4::{
    chip::ChipKind,
    defs::{
        bcls::BRAM_V5 as BRAM,
        bslots, enums,
        virtex6::tcls as tcls_v6,
        virtex7::{tcls as tcls_v7, wires as wires_v7},
    },
};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::{
        fbuild::{FuzzBuilderBase, FuzzCtx},
        int::{BaseIntPip, FuzzIntPip},
        props::mutex::WireMutexExclusive,
    },
    virtex4::specials,
};

pub fn add_fuzzers<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let ExpandedDevice::Virtex4(edev) = backend.edev else {
        unreachable!()
    };
    let tcid = match edev.kind {
        ChipKind::Virtex6 => tcls_v6::BRAM,
        ChipKind::Virtex7 => tcls_v7::BRAM,
        _ => unreachable!(),
    };
    // TODO: globals: RSR[BT] RSR[BT]P EN_TSTEFUSEDLYCTRL
    let mut ctx = FuzzCtx::new(session, backend, tcid);
    {
        let mut bctx = ctx.bel(bslots::BRAM);
        let mode = "RAMB36E1";
        bctx.build()
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_special(specials::BRAM_RAMB36)
            .mode(mode)
            .commit();

        for (pin, pname) in [
            (BRAM::CLKAL, "CLKARDCLKL"),
            (BRAM::CLKAU, "CLKARDCLKU"),
            (BRAM::CLKBL, "CLKBWRCLKL"),
            (BRAM::CLKBU, "CLKBWRCLKU"),
            (BRAM::REGCLKAL, "REGCLKARDRCLKL"),
            (BRAM::REGCLKAU, "REGCLKARDRCLKU"),
            (BRAM::REGCLKBL, "REGCLKBL"),
            (BRAM::REGCLKBU, "REGCLKBU"),
            (BRAM::ENAL, "ENARDENL"),
            (BRAM::ENAU, "ENARDENU"),
            (BRAM::ENBL, "ENBWRENL"),
            (BRAM::ENBU, "ENBWRENU"),
            (BRAM::RSTREGAL, "RSTREGARSTREGL"),
            (BRAM::RSTREGAU, "RSTREGARSTREGU"),
            (BRAM::RSTREGBL, "RSTREGBL"),
            (BRAM::RSTREGBU, "RSTREGBU"),
            (BRAM::RSTRAMAL, "RSTRAMARSTRAML"),
            (BRAM::RSTRAMAU, "RSTRAMARSTRAMU"),
            (BRAM::RSTRAMBL, "RSTRAMBL"),
            (BRAM::RSTRAMBU, "RSTRAMBU"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .pin(pname)
                .test_bel_input_inv_enum(format!("{pname}INV"), pin, pname, format!("{pname}_B"));
        }

        for (attr, aname) in [
            (BRAM::INIT_A_L, "INIT_A"),
            (BRAM::INIT_B_L, "INIT_B"),
            (BRAM::SRVAL_A_L, "SRVAL_A"),
            (BRAM::SRVAL_B_L, "SRVAL_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("READ_WIDTH_A", "36")
                .attr("READ_WIDTH_B", "36")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_special_bits(attr, specials::BRAM_RAMB36, 0)
                .multi_attr(aname, MultiValue::Hex(0), 36);
        }
        for i in 0..0x80 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("READ_WIDTH_A", "36")
                .attr("READ_WIDTH_B", "36")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_special_bits(BRAM::DATA_L, specials::BRAM_RAMB36, i * 0x100)
                .multi_attr(format!("INIT_{i:02X}"), MultiValue::Hex(0), 256);
        }
        for i in 0..0x10 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("READ_WIDTH_A", "36")
                .attr("READ_WIDTH_B", "36")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_special_bits(BRAM::DATAP_L, specials::BRAM_RAMB36, i * 0x100)
                .multi_attr(format!("INITP_{i:02X}"), MultiValue::Hex(0), 256);
        }
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_attr_bool_auto(BRAM::SAVEDATA, "FALSE", "TRUE");

        for (spec, val) in [(specials::BRAM_TDP, "TDP"), (specials::BRAM_SDP, "SDP")] {
            bctx.mode(mode)
                .null_bits()
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_special(spec)
                .attr("RAM_MODE", val)
                .commit();
        }

        for (attr, aname) in [
            (BRAM::WRITE_MODE_A_L, "WRITE_MODE_A"),
            (BRAM::WRITE_MODE_B_L, "WRITE_MODE_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_special_rename(aname, attr, specials::BRAM_RAMB36);
        }
        for (attr, aname) in [(BRAM::DOA_REG_L, "DOA_REG"), (BRAM::DOB_REG_L, "DOB_REG")] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_bool_special_rename(aname, attr, specials::BRAM_RAMB36, "0", "1");
        }
        for (attr, aname) in [
            (BRAM::RAM_EXTENSION_A_LOWER, "RAM_EXTENSION_A"),
            (BRAM::RAM_EXTENSION_B_LOWER, "RAM_EXTENSION_B"),
        ] {
            for (val, vname) in [(false, "NONE"), (false, "UPPER"), (true, "LOWER")] {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "FULL")
                    .test_bel_attr_bits_bi(attr, val)
                    .attr(aname, vname)
                    .commit();
            }
        }
        for (attr, aname, sdp_ok) in [
            (BRAM::READ_MUX_UL_A, "READ_WIDTH_A", true),
            (BRAM::READ_MUX_UL_B, "READ_WIDTH_B", false),
            (BRAM::WRITE_MUX_UL_A, "WRITE_WIDTH_A", false),
            (BRAM::WRITE_MUX_UL_B, "WRITE_WIDTH_B", true),
        ] {
            for ram_mode in ["TDP", "SDP"] {
                if ram_mode == "SDP" && !sdp_ok {
                    continue;
                }
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "FULL")
                    .attr("DOA_REG", "0")
                    .attr("DOB_REG", "0")
                    .attr("RAM_MODE", ram_mode)
                    .test_bel_attr_bits(attr)
                    .attr(aname, "1")
                    .commit();
            }
        }
        for (attr, aname, sdp_ok) in [
            (BRAM::READ_WIDTH_A_L, "READ_WIDTH_A", true),
            (BRAM::READ_WIDTH_B_L, "READ_WIDTH_B", false),
            (BRAM::WRITE_WIDTH_A_L, "WRITE_WIDTH_A", false),
            (BRAM::WRITE_WIDTH_B_L, "WRITE_WIDTH_B", true),
        ] {
            for ram_mode in ["TDP", "SDP"] {
                if ram_mode == "SDP" && !sdp_ok {
                    continue;
                }
                for (val, vname) in [
                    (enums::BRAM_V5_DATA_WIDTH::_1, "0"),
                    (enums::BRAM_V5_DATA_WIDTH::_1, "2"),
                    (enums::BRAM_V5_DATA_WIDTH::_2, "4"),
                    (enums::BRAM_V5_DATA_WIDTH::_4, "9"),
                    (enums::BRAM_V5_DATA_WIDTH::_9, "18"),
                    (enums::BRAM_V5_DATA_WIDTH::_18, "36"),
                ] {
                    bctx.mode(mode)
                        .global_mutex("BRAM_OPT", "NONE")
                        .tile_mutex("MODE", "FULL")
                        .attr("DOA_REG", "0")
                        .attr("DOB_REG", "0")
                        .attr("RAM_MODE", ram_mode)
                        .test_bel_attr_special_val(attr, specials::BRAM_RAMB36, val)
                        .attr(aname, vname)
                        .commit();
                }
            }
        }
        for (attr, aname) in [
            (BRAM::READ_SDP_L, "READ_WIDTH_A"),
            (BRAM::WRITE_SDP_L, "WRITE_WIDTH_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("DOA_REG", "0")
                .attr("DOB_REG", "0")
                .attr("RAM_MODE", "SDP")
                .test_bel_attr_special(attr, specials::BRAM_RAMB36)
                .attr(aname, "72")
                .commit();
        }

        for (attr, aname) in [
            (BRAM::RSTREG_PRIORITY_A_L, "RSTREG_PRIORITY_A"),
            (BRAM::RSTREG_PRIORITY_B_L, "RSTREG_PRIORITY_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_special_rename(aname, attr, specials::BRAM_RAMB36);
        }

        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_attr_special_rename(
                "RDADDR_COLLISION_HWCONFIG",
                BRAM::RDADDR_COLLISION_HWCONFIG_L,
                specials::BRAM_RAMB36,
            );
        if edev.kind == ChipKind::Virtex7 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_special_rename(
                    "EN_PWRGATE",
                    BRAM::EN_PWRGATE_L,
                    specials::BRAM_RAMB36,
                );
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_bool_auto(BRAM::EN_SDBITERR_INIT_V6, "FALSE", "TRUE");
        }

        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_WRITE", "FALSE")
            .test_bel_attr_bool_auto(BRAM::EN_ECC_READ, "FALSE", "TRUE");
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_READ", "FALSE")
            .test_bel_attr_bool_rename(
                "EN_ECC_WRITE",
                if edev.kind == ChipKind::Virtex6 {
                    BRAM::EN_ECC_WRITE_NO_READ
                } else {
                    BRAM::EN_ECC_WRITE
                },
                "FALSE",
                "TRUE",
            );
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_READ", "TRUE")
            .test_bel_attr_bool_auto(BRAM::EN_ECC_WRITE, "FALSE", "TRUE");

        for (attr, opt) in [
            (BRAM::BYPASS_RSR, "BYPASS_RSR"),
            (BRAM::SWAP_CFGPORT, "SWAP_CFGPORT"),
        ] {
            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_bel_attr_bits(attr)
                .global(opt, "ENABLED")
                .commit();
        }
        for (attr, opt) in [
            (BRAM::EN_TSTBRAMRST, "EN_TSTBRAMRST"),
            (BRAM::DIS_TSTFIFORST, "DIS_TSTFIFORST"),
        ] {
            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_bel_attr_bits(attr)
                .global(opt, "1")
                .commit();
        }
        for (val, vname) in [
            (enums::BRAM_WW_VALUE::NONE, "NO_WW"),
            (enums::BRAM_WW_VALUE::_0, "WW0"),
            (enums::BRAM_WW_VALUE::_1, "WW1"),
        ] {
            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_bel_attr_val(BRAM::WW_VALUE, val)
                .global("WEAK_WRITE", vname)
                .commit();
        }
        for (val, vname) in ["0", "1", "10", "11", "100", "101", "110", "111"]
            .into_iter()
            .enumerate()
        {
            for (attr, opt) in [
                (BRAM::TRD_DLY_L, "TRD_DLY_L"),
                (BRAM::TRD_DLY_U, "TRD_DLY_U"),
            ] {
                bctx.mode(mode)
                    .global_mutex_here("BRAM_OPT")
                    .test_bel_attr_bitvec_u32(attr, val as u32)
                    .global(opt, vname)
                    .commit();
            }
        }

        if edev.kind == ChipKind::Virtex6 {
            for (val, vname) in ["0", "1", "10", "11", "100", "101", "110", "111"]
                .into_iter()
                .enumerate()
            {
                for (attr, opt) in [
                    (BRAM::V6_TWR_DLY_L, "TWR_DLY_L"),
                    (BRAM::V6_TWR_DLY_U, "TWR_DLY_U"),
                ] {
                    bctx.mode(mode)
                        .global_mutex_here("BRAM_OPT")
                        .test_bel_attr_bitvec_u32(attr, val as u32)
                        .global(opt, vname)
                        .commit();
                }
            }
            for (val, vname) in ["0", "1", "10", "11"].into_iter().enumerate() {
                for (attr, opt) in [
                    (BRAM::EN_TSTREFBL, "EN_TSTREFBL"),
                    (BRAM::EN_TSTRSRW, "EN_TSTRSRW"),
                ] {
                    bctx.mode(mode)
                        .global_mutex_here("BRAM_OPT")
                        .test_bel_attr_bitvec_u32(attr, val as u32)
                        .global(opt, vname)
                        .commit();
                }
            }
            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_global_attr_bool_rename("EN_TSTBLCLAMP", BRAM::EN_TSTBLCLAMP, "0", "1");
        } else {
            for (val, vname) in [
                "0", "1", "10", "11", "100", "101", "110", "111", "1000", "1001", "1010", "1011",
                "1100", "1101", "1110", "1111",
            ]
            .into_iter()
            .enumerate()
            {
                for (attr, opt) in [
                    (BRAM::V7_TWR_DLY_A_L, "TWR_DLY_A_L"),
                    (BRAM::V7_TWR_DLY_A_U, "TWR_DLY_A_U"),
                    (BRAM::V7_TWR_DLY_B_L, "TWR_DLY_B_L"),
                    (BRAM::V7_TWR_DLY_B_U, "TWR_DLY_B_U"),
                ] {
                    bctx.mode(mode)
                        .global_mutex_here("BRAM_OPT")
                        .test_bel_attr_bitvec_u32(attr, val as u32)
                        .global(opt, vname)
                        .commit();
                }
            }

            for (val, vname) in ["0", "1", "10", "11"].into_iter().enumerate() {
                for (attr, opt) in [
                    (BRAM::TSTREFBL_CTRL, "TSTREFBL_CTRL"),
                    (BRAM::TSTRSR_RWCTRL, "TSTRSR_RWCTRL"),
                    (BRAM::EN_TSTRFMODE_DLY, "EN_TSTRFMODE_DLY"),
                    (BRAM::EN_TSTPULSEPU_DLY, "EN_TSTPULSEPU_DLY"),
                    (BRAM::EN_TSTEXTCLK, "EN_TSTEXTCLK"),
                    (BRAM::EN_TSTRSTC_PW, "EN_TSTRSTC_PW"),
                    (BRAM::EN_TSTBLPC_DLY, "EN_TSTBLPC_DLY"),
                    (BRAM::EN_TST_REGOUT_DLY_SEL, "EN_TST_REGOUT_DLY_SEL"),
                    (BRAM::TST_RNG_OSC, "TST_RNG_OSC"),
                ] {
                    bctx.mode(mode)
                        .global_mutex_here("BRAM_OPT")
                        .test_bel_attr_bitvec_u32(attr, val as u32)
                        .global(opt, vname)
                        .commit();
                }
            }
            for (attr, opt) in [
                (BRAM::DIS_TSTBLCLAMP, "DIS_TSTBLCLAMP"),
                (BRAM::TST_SSRLAT_WF, "TST_SSRLAT_WF"),
            ] {
                bctx.mode(mode)
                    .global_mutex_here("BRAM_OPT")
                    .test_global_attr_bool_rename(opt, attr, "0", "1");
            }

            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_global_attr_bool_rename(
                    "EN_TSTBLCLAMP_RD",
                    BRAM::EN_TSTBLCLAMP_RD,
                    "NO",
                    "YES",
                );

            for (attr, opt) in [
                (BRAM::EN_TSTBLCLMP_WW, "EN_TSTBLCLMP_WW"),
                (BRAM::EN_TSTSNM, "EN_TSTSNM"),
                (BRAM::LAT_RST_DLYEN, "LAT_RST_DLYEN"),
                (BRAM::STUCK_DET_EN, "STUCK_DET_EN"),
                (BRAM::TST_PULSEPU_SFT, "TST_PULSEPU_SFT"),
                (BRAM::TST_BIST_CTL, "TST_BIST_CTL"),
            ] {
                bctx.mode(mode)
                    .global_mutex_here("BRAM_OPT")
                    .test_global_attr_bool_rename(opt, attr, "DISABLE", "ENABLE");
            }
        }

        let mode = "FIFO36E1";

        bctx.build()
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_special(specials::BRAM_FIFO36)
            .mode(mode)
            .commit();

        for (pin, pname) in [
            (BRAM::CLKAL, "RDCLKL"),
            (BRAM::CLKAU, "RDCLKU"),
            (BRAM::CLKBL, "WRCLKL"),
            (BRAM::CLKBU, "WRCLKU"),
            (BRAM::REGCLKAL, "RDRCLKL"),
            (BRAM::REGCLKAU, "RDRCLKU"),
            (BRAM::ENAL, "RDENL"),
            (BRAM::ENAU, "RDENU"),
            (BRAM::ENBL, "WRENL"),
            (BRAM::ENBU, "WRENU"),
            (BRAM::RSTREGAL, "RSTREGL"),
            (BRAM::RSTREGAU, "RSTREGU"),
            (BRAM::RSTRAMAL, "RST"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .pin(pname)
                .test_bel_input_inv_enum(format!("{pname}INV"), pin, pname, format!("{pname}_B"));
        }

        for (attr, aname) in [(BRAM::INIT_A_L, "INIT"), (BRAM::SRVAL_A_L, "SRVAL")] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("DATA_WIDTH", "72")
                .attr("FIFO_MODE", "FIFO36_72")
                .test_bel_attr_special_bits(attr, specials::BRAM_FIFO36, 0)
                .multi_attr(aname, MultiValue::Hex(0), 72);
        }

        for (spec, val) in [
            (specials::BRAM_TDP, "FIFO36"),
            (specials::BRAM_SDP, "FIFO36_72"),
        ] {
            bctx.mode(mode)
                .null_bits()
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_special(spec)
                .attr("FIFO_MODE", val)
                .commit();
        }
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_attr_bool_auto(BRAM::EN_SYN, "FALSE", "TRUE");
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_attr_bool_auto(BRAM::FIRST_WORD_FALL_THROUGH, "FALSE", "TRUE");
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_SYN", "TRUE")
            .test_bel_attr_multi(BRAM::ALMOST_FULL_OFFSET, MultiValue::Hex(0));
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_SYN", "TRUE")
            .test_bel_attr_multi(BRAM::ALMOST_EMPTY_OFFSET, MultiValue::Hex(0));

        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .test_bel_attr_bool_special_rename(
                "DO_REG",
                BRAM::DOA_REG_L,
                specials::BRAM_FIFO36,
                "0",
                "1",
            );

        for (val, vname) in [
            (enums::BRAM_V5_FIFO_WIDTH::_2, "4"),
            (enums::BRAM_V5_FIFO_WIDTH::_4, "9"),
            (enums::BRAM_V5_FIFO_WIDTH::_9, "18"),
            (enums::BRAM_V5_FIFO_WIDTH::_18, "36"),
            (enums::BRAM_V5_FIFO_WIDTH::_36, "72"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("DO_REG", "0")
                .attr("FIFO_MODE", "FIFO36")
                .test_bel_attr_special_val(BRAM::FIFO_WIDTH, specials::BRAM_FIFO36, val)
                .attr("DATA_WIDTH", vname)
                .commit();
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .attr("DO_REG", "0")
                .attr("FIFO_MODE", "FIFO36_72")
                .test_bel_attr_special_val(BRAM::FIFO_WIDTH, specials::BRAM_FIFO36_72, val)
                .attr("DATA_WIDTH", vname)
                .commit();
        }

        if edev.kind == ChipKind::Virtex6 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_special_rename(
                    "RSTREG_PRIORITY",
                    BRAM::RSTREG_PRIORITY_A_L,
                    specials::BRAM_FIFO36,
                );
        } else {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_special_rename(
                    "EN_PWRGATE",
                    BRAM::EN_PWRGATE_L,
                    specials::BRAM_RAMB36,
                );
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_bool_auto(BRAM::EN_SDBITERR_INIT_V6, "FALSE", "TRUE");
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "FULL")
                .test_bel_attr_special_rename(
                    "RDADDR_COLLISION_HWCONFIG",
                    BRAM::RDADDR_COLLISION_HWCONFIG_L,
                    specials::BRAM_RAMB36,
                );
        }

        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_WRITE", "FALSE")
            .test_bel_attr_bool_auto(BRAM::EN_ECC_READ, "FALSE", "TRUE");
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_READ", "FALSE")
            .test_bel_attr_bool_rename(
                "EN_ECC_WRITE",
                if edev.kind == ChipKind::Virtex6 {
                    BRAM::EN_ECC_WRITE_NO_READ
                } else {
                    BRAM::EN_ECC_WRITE
                },
                "FALSE",
                "TRUE",
            );
        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "FULL")
            .attr("EN_ECC_READ", "TRUE")
            .test_bel_attr_bool_auto(BRAM::EN_ECC_WRITE, "FALSE", "TRUE");

        for (attr, opt) in [
            (BRAM::TEST_FIFO_FLAG, "TEST_FIFO_FLAG"),
            (BRAM::TEST_FIFO_OFFSET, "TEST_FIFO_OFFSET"),
            (BRAM::TEST_FIFO_CNT, "TEST_FIFO_CNT"),
        ] {
            bctx.mode(mode)
                .global_mutex_here("BRAM_OPT")
                .test_bel_attr_bits(attr)
                .global(opt, "ENABLED")
                .commit();
        }
    }
    for i in 0..2 {
        let mut bctx = ctx.bel(bslots::BRAM).sub(i + 1);
        let mode = "RAMB18E1";
        bctx.build()
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "HALF")
            .test_bel_special([specials::BRAM_RAMB18_L, specials::BRAM_RAMB18_U][i])
            .mode(mode)
            .commit();

        for (pin, pname) in [
            ([BRAM::CLKAL, BRAM::CLKAU][i], "CLKARDCLK"),
            ([BRAM::CLKBL, BRAM::CLKBU][i], "CLKBWRCLK"),
            ([BRAM::REGCLKAL, BRAM::REGCLKAU][i], "REGCLKARDRCLK"),
            ([BRAM::REGCLKBL, BRAM::REGCLKBU][i], "REGCLKB"),
            ([BRAM::ENAL, BRAM::ENAU][i], "ENARDEN"),
            ([BRAM::ENBL, BRAM::ENBU][i], "ENBWREN"),
            ([BRAM::RSTREGAL, BRAM::RSTREGAU][i], "RSTREGARSTREG"),
            ([BRAM::RSTREGBL, BRAM::RSTREGBU][i], "RSTREGB"),
            ([BRAM::RSTRAMAL, BRAM::RSTRAMAU][i], "RSTRAMARSTRAM"),
            ([BRAM::RSTRAMBL, BRAM::RSTRAMBU][i], "RSTRAMB"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .pin(pname)
                .test_bel_input_inv_enum(format!("{pname}INV"), pin, pname, format!("{pname}_B"));
        }

        for (attr, aname) in [
            ([BRAM::INIT_A_L, BRAM::INIT_A_U][i], "INIT_A"),
            ([BRAM::INIT_B_L, BRAM::INIT_B_U][i], "INIT_B"),
            ([BRAM::SRVAL_A_L, BRAM::SRVAL_A_U][i], "SRVAL_A"),
            ([BRAM::SRVAL_B_L, BRAM::SRVAL_B_U][i], "SRVAL_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("READ_WIDTH_A", "18")
                .attr("READ_WIDTH_B", "18")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_bits(attr)
                .multi_attr(aname, MultiValue::Hex(0), 18);
        }
        for j in 0..0x40 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("READ_WIDTH_A", "18")
                .attr("READ_WIDTH_B", "18")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_bits_base([BRAM::DATA_L, BRAM::DATA_U][i], j * 0x100)
                .multi_attr(format!("INIT_{j:02X}"), MultiValue::Hex(0), 256);
        }
        for j in 0..8 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("READ_WIDTH_A", "18")
                .attr("READ_WIDTH_B", "18")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_bits_base([BRAM::DATAP_L, BRAM::DATAP_U][i], j * 0x100)
                .multi_attr(format!("INITP_{j:02X}"), MultiValue::Hex(0), 256);
        }

        for (spec, val) in [(specials::BRAM_TDP, "TDP"), (specials::BRAM_SDP, "SDP")] {
            bctx.mode(mode)
                .null_bits()
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_special(spec)
                .attr("RAM_MODE", val)
                .commit();
        }

        for (attr, aname) in [
            (
                [BRAM::WRITE_MODE_A_L, BRAM::WRITE_MODE_A_U][i],
                "WRITE_MODE_A",
            ),
            (
                [BRAM::WRITE_MODE_B_L, BRAM::WRITE_MODE_B_U][i],
                "WRITE_MODE_B",
            ),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("RAM_MODE", "TDP")
                .test_bel_attr_rename(aname, attr);
        }
        for (attr, aname) in [
            ([BRAM::DOA_REG_L, BRAM::DOA_REG_U][i], "DOA_REG"),
            ([BRAM::DOB_REG_L, BRAM::DOB_REG_U][i], "DOB_REG"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_bool_rename(aname, attr, "0", "1");
        }
        for (attr, aname, sdp_ok) in [
            (
                [BRAM::READ_WIDTH_A_L, BRAM::READ_WIDTH_A_U][i],
                "READ_WIDTH_A",
                true,
            ),
            (
                [BRAM::READ_WIDTH_B_L, BRAM::READ_WIDTH_B_U][i],
                "READ_WIDTH_B",
                false,
            ),
            (
                [BRAM::WRITE_WIDTH_A_L, BRAM::WRITE_WIDTH_A_U][i],
                "WRITE_WIDTH_A",
                false,
            ),
            (
                [BRAM::WRITE_WIDTH_B_L, BRAM::WRITE_WIDTH_B_U][i],
                "WRITE_WIDTH_B",
                true,
            ),
        ] {
            for ram_mode in ["TDP", "SDP"] {
                if ram_mode == "SDP" && !sdp_ok {
                    continue;
                }
                for (val, vname) in [
                    (enums::BRAM_V5_DATA_WIDTH::_1, "0"),
                    (enums::BRAM_V5_DATA_WIDTH::_1, "1"),
                    (enums::BRAM_V5_DATA_WIDTH::_2, "2"),
                    (enums::BRAM_V5_DATA_WIDTH::_4, "4"),
                    (enums::BRAM_V5_DATA_WIDTH::_9, "9"),
                    (enums::BRAM_V5_DATA_WIDTH::_18, "18"),
                ] {
                    bctx.mode(mode)
                        .global_mutex("BRAM_OPT", "NONE")
                        .tile_mutex("MODE", "HALF")
                        .attr("DOA_REG", "0")
                        .attr("DOB_REG", "0")
                        .pin("WEBWE0")
                        .pin("WEBWE1")
                        .pin("WEBWE2")
                        .pin("WEBWE3")
                        .pin("WEBWE4")
                        .pin("WEBWE5")
                        .pin("WEBWE6")
                        .pin("WEBWE7")
                        .pin("WEA0")
                        .pin("WEA1")
                        .pin("WEA2")
                        .pin("WEA3")
                        .attr("RAM_MODE", ram_mode)
                        .test_bel_attr_val(attr, val)
                        .attr(aname, vname)
                        .commit();
                }
            }
        }
        for (attr, aname) in [
            ([BRAM::READ_SDP_L, BRAM::READ_SDP_U][i], "READ_WIDTH_A"),
            ([BRAM::WRITE_SDP_L, BRAM::WRITE_SDP_U][i], "WRITE_WIDTH_B"),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("DOA_REG", "0")
                .attr("DOB_REG", "0")
                .pin("WEBWE0")
                .pin("WEBWE1")
                .pin("WEBWE2")
                .pin("WEBWE3")
                .pin("WEBWE4")
                .pin("WEBWE5")
                .pin("WEBWE6")
                .pin("WEBWE7")
                .attr("RAM_MODE", "SDP")
                .test_bel_attr_bits(attr)
                .attr(aname, "36")
                .commit();
        }

        for (attr, aname) in [
            (
                [BRAM::RSTREG_PRIORITY_A_L, BRAM::RSTREG_PRIORITY_A_U][i],
                "RSTREG_PRIORITY_A",
            ),
            (
                [BRAM::RSTREG_PRIORITY_B_L, BRAM::RSTREG_PRIORITY_B_U][i],
                "RSTREG_PRIORITY_B",
            ),
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_rename(aname, attr);
        }

        bctx.mode(mode)
            .global_mutex("BRAM_OPT", "NONE")
            .tile_mutex("MODE", "HALF")
            .test_bel_attr_rename(
                "RDADDR_COLLISION_HWCONFIG",
                [
                    BRAM::RDADDR_COLLISION_HWCONFIG_L,
                    BRAM::RDADDR_COLLISION_HWCONFIG_U,
                ][i],
            );

        if edev.kind == ChipKind::Virtex7 {
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_rename("EN_PWRGATE", [BRAM::EN_PWRGATE_L, BRAM::EN_PWRGATE_U][i]);
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .tile_mutex("SDBITERR", format!("HALF_{i}"))
                .test_bel_attr_bool_auto(BRAM::EN_SDBITERR_INIT_V6, "FALSE", "TRUE");
        }

        if i == 0 {
            let mode = "FIFO18E1";

            bctx.build()
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_special(specials::BRAM_FIFO18)
                .mode(mode)
                .commit();

            for (pin, pname) in [
                (BRAM::CLKAL, "RDCLK"),
                (BRAM::CLKBL, "WRCLK"),
                (BRAM::REGCLKAL, "RDRCLK"),
                (BRAM::ENAL, "RDEN"),
                (BRAM::ENBL, "WREN"),
                (BRAM::RSTREGAL, "RSTREG"),
                (BRAM::RSTRAMAL, "RST"),
            ] {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .pin(pname)
                    .test_bel_input_inv_enum(
                        format!("{pname}INV"),
                        pin,
                        pname,
                        format!("{pname}_B"),
                    );
            }

            for (attr, aname) in [(BRAM::INIT_A_L, "INIT"), (BRAM::SRVAL_A_L, "SRVAL")] {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .attr("DATA_WIDTH", "36")
                    .attr("FIFO_MODE", "FIFO18_36")
                    .test_bel_attr_special_bits(attr, specials::BRAM_FIFO18, 0)
                    .multi_attr(aname, MultiValue::Hex(0), 36);
            }

            for (spec, val) in [
                (specials::BRAM_TDP, "FIFO18"),
                (specials::BRAM_SDP, "FIFO18_36"),
            ] {
                bctx.mode(mode)
                    .null_bits()
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .test_bel_special(spec)
                    .attr("FIFO_MODE", val)
                    .commit();
            }
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_bool_auto(BRAM::EN_SYN, "FALSE", "TRUE");
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_bool_auto(BRAM::FIRST_WORD_FALL_THROUGH, "FALSE", "TRUE");

            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("EN_SYN", "TRUE")
                .test_bel_attr_multi(BRAM::ALMOST_FULL_OFFSET, MultiValue::Hex(0));
            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .attr("EN_SYN", "TRUE")
                .test_bel_attr_multi(BRAM::ALMOST_EMPTY_OFFSET, MultiValue::Hex(0));

            bctx.mode(mode)
                .global_mutex("BRAM_OPT", "NONE")
                .tile_mutex("MODE", "HALF")
                .test_bel_attr_bool_special_rename(
                    "DO_REG",
                    BRAM::DOA_REG_L,
                    specials::BRAM_FIFO18,
                    "0",
                    "1",
                );

            for (val, vname) in [
                (enums::BRAM_V5_FIFO_WIDTH::_4, "4"),
                (enums::BRAM_V5_FIFO_WIDTH::_9, "9"),
                (enums::BRAM_V5_FIFO_WIDTH::_18, "18"),
                (enums::BRAM_V5_FIFO_WIDTH::_36, "36"),
            ] {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .attr("DO_REG", "0")
                    .attr("FIFO_MODE", "FIFO18")
                    .test_bel_attr_val(BRAM::FIFO_WIDTH, val)
                    .attr("DATA_WIDTH", vname)
                    .commit();
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .attr("DO_REG", "0")
                    .attr("FIFO_MODE", "FIFO18_36")
                    .test_bel_attr_special_val(BRAM::FIFO_WIDTH, specials::BRAM_FIFO18_36, val)
                    .attr("DATA_WIDTH", vname)
                    .commit();
            }

            if edev.kind == ChipKind::Virtex6 {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .test_bel_attr_special_rename(
                        "RSTREG_PRIORITY",
                        BRAM::RSTREG_PRIORITY_A_L,
                        specials::BRAM_FIFO18,
                    );
            } else {
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .test_bel_attr_rename("EN_PWRGATE", BRAM::EN_PWRGATE_L);
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .tile_mutex("SDBITERR", format!("HALF_{i}"))
                    .test_bel_attr_bool_auto(BRAM::EN_SDBITERR_INIT_V6, "FALSE", "TRUE");
                bctx.mode(mode)
                    .global_mutex("BRAM_OPT", "NONE")
                    .tile_mutex("MODE", "HALF")
                    .test_bel_attr_rename(
                        "RDADDR_COLLISION_HWCONFIG",
                        BRAM::RDADDR_COLLISION_HWCONFIG_L,
                    );
            }
        }
    }
    if edev.kind == ChipKind::Virtex7 {
        let BelInfo::SwitchBox(ref sb) = edev.db[tcls_v7::BRAM].bels[bslots::SPEC_INT] else {
            unreachable!()
        };
        for item in &sb.items {
            match item {
                SwitchBoxItem::Mux(mux) => {
                    for &src in mux.src.keys() {
                        if wires_v7::IMUX_IMUX.contains(src.wire) {
                            ctx.build()
                                .prop(WireMutexExclusive::new(mux.dst))
                                .test_routing(mux.dst, src)
                                .prop(FuzzIntPip::new(mux.dst, src.tw))
                                .commit();
                        } else {
                            let odst = backend.edev.db_index[tcls_v7::BRAM].pips_fwd[&src]
                                .iter()
                                .copied()
                                .find(|&w| w.tw != mux.dst)
                                .unwrap()
                                .tw;
                            ctx.build()
                                .global_mutex("BRAM_ADDR_CASCADE", "USE")
                                .prop(WireMutexExclusive::new(mux.dst))
                                .prop(WireMutexExclusive::new(odst))
                                .prop(BaseIntPip::new(odst, src.tw))
                                .test_routing(mux.dst, src)
                                .prop(FuzzIntPip::new(mux.dst, src.tw))
                                .commit();
                        }
                    }
                }
                SwitchBoxItem::PermaBuf(buf) if !wires_v7::IMUX_BRAM.contains(buf.dst.wire) => {
                    let mutex = if wires_v7::BRAM_ADDRA_CASC.contains(buf.dst.wire) {
                        "CASCADE_OUT_A"
                    } else if wires_v7::BRAM_ADDRB_CASC.contains(buf.dst.wire) {
                        "CASCADE_OUT_B"
                    } else {
                        unreachable!()
                    };
                    ctx.build()
                        .global_mutex("BRAM_ADDR_CASCADE", "TEST")
                        .tile_mutex(mutex, buf.dst)
                        .test_routing_special(buf.dst, specials::PRESENT)
                        .prop(FuzzIntPip::new(buf.dst, buf.src.tw))
                        .commit();
                }
                _ => (),
            }
        }
    }
    let mut ctx = FuzzCtx::new_null(session, backend);
    ctx.build()
        .extra_tiles_by_bel_attr_bits(
            bslots::BRAM,
            if edev.kind == ChipKind::Virtex6 {
                BRAM::V6_TEST_ATTRIBUTES
            } else {
                BRAM::V7_TEST_ATTRIBUTES
            },
        )
        .test_global_special(specials::BRAM_TEST_ATTRIBUTES)
        .multi_global(
            "TEST_ATTRIBUTES",
            MultiValue::Hex(0),
            if edev.kind == ChipKind::Virtex6 {
                20
            } else {
                19
            },
        );
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx) {
    let ExpandedDevice::Virtex4(edev) = ctx.edev else {
        unreachable!()
    };
    let tcid = match edev.kind {
        ChipKind::Virtex6 => tcls_v6::BRAM,
        ChipKind::Virtex7 => tcls_v7::BRAM,
        _ => unreachable!(),
    };
    let bslot = bslots::BRAM;
    let mut present_ramb36 = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_RAMB36);
    let mut present_fifo36 = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_FIFO36);
    let mut present_ramb18_l = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_RAMB18_L);
    let mut present_ramb18_u = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_RAMB18_U);
    let mut present_fifo18 = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_FIFO18);

    for pin in [
        BRAM::CLKAL,
        BRAM::CLKAU,
        BRAM::CLKBL,
        BRAM::CLKBU,
        BRAM::REGCLKAL,
        BRAM::REGCLKAU,
        BRAM::REGCLKBL,
        BRAM::REGCLKBU,
        BRAM::ENAL,
        BRAM::ENAU,
        BRAM::ENBL,
        BRAM::ENBU,
        BRAM::RSTREGAL,
        BRAM::RSTREGAU,
        BRAM::RSTREGBL,
        BRAM::RSTREGBU,
        BRAM::RSTRAMAL,
        BRAM::RSTRAMAU,
        BRAM::RSTRAMBL,
        BRAM::RSTRAMBU,
    ] {
        ctx.collect_bel_input_inv_bi(tcid, bslot, pin);
    }

    for (attr_al, attr_au, attr_bl, attr_bu) in [
        (
            BRAM::INIT_A_L,
            BRAM::INIT_A_U,
            BRAM::INIT_B_L,
            BRAM::INIT_B_U,
        ),
        (
            BRAM::SRVAL_A_L,
            BRAM::SRVAL_A_U,
            BRAM::SRVAL_B_L,
            BRAM::SRVAL_B_U,
        ),
    ] {
        let bits_al = xlat_bitvec(ctx.get_diffs_attr_bits(tcid, bslot, attr_al, 18));
        let bits_au = xlat_bitvec(ctx.get_diffs_attr_bits(tcid, bslot, attr_au, 18));
        let bits_bl = xlat_bitvec(ctx.get_diffs_attr_bits(tcid, bslot, attr_bl, 18));
        let bits_bu = xlat_bitvec(ctx.get_diffs_attr_bits(tcid, bslot, attr_bu, 18));
        let bits_a = xlat_bitvec(ctx.get_diffs_attr_special_bits(
            tcid,
            bslot,
            attr_al,
            specials::BRAM_RAMB36,
            36,
        ));
        let bits_b = xlat_bitvec(ctx.get_diffs_attr_special_bits(
            tcid,
            bslot,
            attr_bl,
            specials::BRAM_RAMB36,
            36,
        ));
        let bits = xlat_bitvec(ctx.get_diffs_attr_special_bits(
            tcid,
            bslot,
            attr_al,
            specials::BRAM_FIFO36,
            72,
        ));
        let bits_l = xlat_bitvec(ctx.get_diffs_attr_special_bits(
            tcid,
            bslot,
            attr_al,
            specials::BRAM_FIFO18,
            36,
        ));
        assert_eq!(&bits[..36], bits_a);
        assert_eq!(&bits[36..], bits_b);
        assert_eq!(&bits_l[..18], bits_al);
        assert_eq!(&bits_l[18..], bits_bl);

        for i in 0..18 {
            assert_eq!(bits_al[i], bits_a[2 * i]);
            assert_eq!(bits_au[i], bits_a[2 * i + 1]);
            assert_eq!(bits_bl[i], bits_b[2 * i]);
            assert_eq!(bits_bu[i], bits_b[2 * i + 1]);
        }

        for bits in [&bits_al, &bits_bl] {
            present_ramb36.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
            present_fifo36.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
            present_ramb18_l.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
            present_fifo18.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
        }
        for bits in [&bits_au, &bits_bu] {
            present_ramb36.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
            present_fifo36.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
            present_ramb18_u.apply_bitvec_diff(bits, &bits![0; 18], &bits![1; 18]);
        }

        ctx.insert_bel_attr_bitvec(tcid, bslot, attr_al, bits_al);
        ctx.insert_bel_attr_bitvec(tcid, bslot, attr_au, bits_au);
        ctx.insert_bel_attr_bitvec(tcid, bslot, attr_bl, bits_bl);
        ctx.insert_bel_attr_bitvec(tcid, bslot, attr_bu, bits_bu);
    }

    ctx.collect_bel_attr(tcid, bslot, BRAM::DATA_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::DATA_U);
    ctx.collect_bel_attr(tcid, bslot, BRAM::DATAP_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::DATAP_U);

    let data = xlat_bitvec(ctx.get_diffs_attr_special_bits(
        tcid,
        bslot,
        BRAM::DATA_L,
        specials::BRAM_RAMB36,
        0x8000,
    ));
    let datap = xlat_bitvec(ctx.get_diffs_attr_special_bits(
        tcid,
        bslot,
        BRAM::DATAP_L,
        specials::BRAM_RAMB36,
        0x1000,
    ));
    let mut data_l = vec![];
    let mut data_u = vec![];
    for (i, bit) in data.into_iter().enumerate() {
        if i.is_multiple_of(2) {
            data_l.push(bit);
        } else {
            data_u.push(bit);
        }
    }
    let mut datap_l = vec![];
    let mut datap_u = vec![];
    for (i, bit) in datap.into_iter().enumerate() {
        if i.is_multiple_of(2) {
            datap_l.push(bit);
        } else {
            datap_u.push(bit);
        }
    }
    ctx.insert_bel_attr_bitvec(tcid, bslot, BRAM::DATA_L, data_l);
    ctx.insert_bel_attr_bitvec(tcid, bslot, BRAM::DATA_U, data_u);
    ctx.insert_bel_attr_bitvec(tcid, bslot, BRAM::DATAP_L, datap_l);
    ctx.insert_bel_attr_bitvec(tcid, bslot, BRAM::DATAP_U, datap_u);

    let bits = xlat_bit_wide_bi(
        ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::SAVEDATA, false),
        ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::SAVEDATA, true),
    );
    ctx.insert_bel_attr_bitvec(tcid, bslot, BRAM::SAVEDATA, bits);

    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::EN_SYN);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::FIRST_WORD_FALL_THROUGH);
    ctx.collect_bel_attr(tcid, bslot, BRAM::ALMOST_EMPTY_OFFSET);
    ctx.collect_bel_attr(tcid, bslot, BRAM::ALMOST_FULL_OFFSET);
    for attr in [BRAM::ALMOST_FULL_OFFSET, BRAM::ALMOST_EMPTY_OFFSET] {
        let item = ctx.bel_attr_bitvec(tcid, bslot, attr);
        present_ramb36.apply_bitvec_diff(item, &bits![0; 13], &bits![1; 13]);
        present_fifo36.apply_bitvec_diff(item, &bits![0; 13], &bits![1; 13]);
        present_ramb18_l.apply_bitvec_diff(item, &bits![0; 13], &bits![1; 13]);
        present_fifo18.apply_bitvec_diff(item, &bits![0; 13], &bits![1; 13]);
    }

    for (attr_l, attr_u) in [
        (BRAM::WRITE_MODE_A_L, BRAM::WRITE_MODE_A_U),
        (BRAM::WRITE_MODE_B_L, BRAM::WRITE_MODE_B_U),
    ] {
        for val in [
            enums::BRAM_WRITE_MODE::READ_FIRST,
            enums::BRAM_WRITE_MODE::WRITE_FIRST,
            enums::BRAM_WRITE_MODE::NO_CHANGE,
        ] {
            let diff_f =
                ctx.get_diff_attr_special_val(tcid, bslot, attr_l, specials::BRAM_RAMB36, val);
            let diff_h0 = ctx.peek_diff_attr_val(tcid, bslot, attr_l, val);
            let diff_h1 = ctx.peek_diff_attr_val(tcid, bslot, attr_u, val);
            assert_eq!(diff_f, diff_h0.combine(diff_h1));
        }
        ctx.collect_bel_attr(tcid, bslot, attr_l);
        ctx.collect_bel_attr(tcid, bslot, attr_u);

        let enum_l = ctx.bel_attr_enum(tcid, bslot, attr_l);
        let enum_u = ctx.bel_attr_enum(tcid, bslot, attr_u);

        present_fifo36.apply_enum_diff(
            enum_l,
            enums::BRAM_WRITE_MODE::NO_CHANGE,
            enums::BRAM_WRITE_MODE::WRITE_FIRST,
        );
        present_fifo18.apply_enum_diff(
            enum_l,
            enums::BRAM_WRITE_MODE::NO_CHANGE,
            enums::BRAM_WRITE_MODE::WRITE_FIRST,
        );
        present_fifo36.apply_enum_diff(
            enum_u,
            enums::BRAM_WRITE_MODE::NO_CHANGE,
            enums::BRAM_WRITE_MODE::WRITE_FIRST,
        );
    }
    for val in [false, true] {
        let diff_fifo18 = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            BRAM::DOA_REG_L,
            specials::BRAM_FIFO18,
            0,
            val,
        );
        let diff_fifo36 = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            BRAM::DOA_REG_L,
            specials::BRAM_FIFO36,
            0,
            val,
        );
        let diff_a = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            BRAM::DOA_REG_L,
            specials::BRAM_RAMB36,
            0,
            val,
        );
        let diff_b = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            BRAM::DOB_REG_L,
            specials::BRAM_RAMB36,
            0,
            val,
        );
        let diff_al = ctx.peek_diff_attr_bool_bi(tcid, bslot, BRAM::DOA_REG_L, val);
        let diff_bl = ctx.peek_diff_attr_bool_bi(tcid, bslot, BRAM::DOB_REG_L, val);
        let diff_au = ctx.peek_diff_attr_bool_bi(tcid, bslot, BRAM::DOA_REG_U, val);
        let diff_bu = ctx.peek_diff_attr_bool_bi(tcid, bslot, BRAM::DOB_REG_U, val);
        assert_eq!(diff_fifo18, diff_al.combine(diff_bl));
        assert_eq!(diff_fifo36, diff_a.combine(&diff_b));
        assert_eq!(diff_a, diff_al.combine(diff_au));
        assert_eq!(diff_b, diff_bl.combine(diff_bu));
    }
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::DOA_REG_L);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::DOA_REG_U);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::DOB_REG_L);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::DOB_REG_U);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::RAM_EXTENSION_A_LOWER);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::RAM_EXTENSION_B_LOWER);

    // extract SDP bits
    for (attr, attr_a, attr_b) in [
        (BRAM::READ_SDP_L, BRAM::READ_WIDTH_A_L, BRAM::READ_WIDTH_B_L),
        (BRAM::READ_SDP_U, BRAM::READ_WIDTH_A_U, BRAM::READ_WIDTH_B_U),
        (
            BRAM::WRITE_SDP_L,
            BRAM::WRITE_WIDTH_A_L,
            BRAM::WRITE_WIDTH_B_L,
        ),
        (
            BRAM::WRITE_SDP_U,
            BRAM::WRITE_WIDTH_A_U,
            BRAM::WRITE_WIDTH_B_U,
        ),
    ] {
        let mut diff = ctx.get_diff_attr_bool(tcid, bslot, attr);
        // ???
        if attr != BRAM::READ_SDP_L {
            diff = diff.combine(&!ctx.peek_diff_attr_val(
                tcid,
                bslot,
                attr_a,
                enums::BRAM_V5_DATA_WIDTH::_18,
            ));
        }
        diff = diff.combine(&!ctx.peek_diff_attr_val(
            tcid,
            bslot,
            attr_b,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ));
        ctx.insert_bel_attr_bool(tcid, bslot, attr, xlat_bit(diff));
    }
    // clear RAMB36 version
    for (attr, attr_u, attr_a, attr_b) in [
        (
            BRAM::READ_SDP_L,
            BRAM::READ_SDP_U,
            BRAM::READ_WIDTH_A_L,
            BRAM::READ_WIDTH_B_L,
        ),
        (
            BRAM::WRITE_SDP_L,
            BRAM::WRITE_SDP_U,
            BRAM::WRITE_WIDTH_A_L,
            BRAM::WRITE_WIDTH_B_L,
        ),
    ] {
        let mut diff = ctx.get_diff_attr_special(tcid, bslot, attr, specials::BRAM_RAMB36);
        diff = diff.combine(&!ctx.peek_diff_attr_special_val(
            tcid,
            bslot,
            attr_a,
            specials::BRAM_RAMB36,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ));
        diff = diff.combine(&!ctx.peek_diff_attr_special_val(
            tcid,
            bslot,
            attr_b,
            specials::BRAM_RAMB36,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ));
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, attr), true, false);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, attr_u), true, false);
        diff.assert_empty();
    }

    // extract MUX_UL bits
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_MUX_UL_A);
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_MUX_UL_B);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_MUX_UL_A);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_MUX_UL_B);
    for (attr_mux, attr_l, attr_u) in [
        (
            BRAM::READ_MUX_UL_A,
            BRAM::READ_WIDTH_A_L,
            BRAM::READ_WIDTH_A_U,
        ),
        (
            BRAM::READ_MUX_UL_B,
            BRAM::READ_WIDTH_B_L,
            BRAM::READ_WIDTH_B_U,
        ),
        (
            BRAM::WRITE_MUX_UL_A,
            BRAM::WRITE_WIDTH_A_L,
            BRAM::WRITE_WIDTH_A_U,
        ),
        (
            BRAM::WRITE_MUX_UL_B,
            BRAM::WRITE_WIDTH_B_L,
            BRAM::WRITE_WIDTH_B_U,
        ),
    ] {
        for val in [
            enums::BRAM_V5_DATA_WIDTH::_1,
            enums::BRAM_V5_DATA_WIDTH::_2,
            enums::BRAM_V5_DATA_WIDTH::_4,
            enums::BRAM_V5_DATA_WIDTH::_9,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ] {
            let mut diff =
                ctx.get_diff_attr_special_val(tcid, bslot, attr_l, specials::BRAM_RAMB36, val);
            diff = diff.combine(&!ctx.peek_diff_attr_val(tcid, bslot, attr_l, val));
            diff = diff.combine(&!ctx.peek_diff_attr_val(tcid, bslot, attr_u, val));
            if val == enums::BRAM_V5_DATA_WIDTH::_4 {
                diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, attr_mux), true, false);
            }
            diff.assert_empty();
        }
    }

    // and collect the main attrs
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_WIDTH_A_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_WIDTH_A_U);
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_WIDTH_B_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::READ_WIDTH_B_U);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_WIDTH_A_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_WIDTH_A_U);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_WIDTH_B_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::WRITE_WIDTH_B_U);

    let mut diffs = vec![];
    for (val, width) in [
        (enums::BRAM_V5_FIFO_WIDTH::_4, enums::BRAM_V5_DATA_WIDTH::_4),
        (enums::BRAM_V5_FIFO_WIDTH::_9, enums::BRAM_V5_DATA_WIDTH::_9),
        (
            enums::BRAM_V5_FIFO_WIDTH::_18,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ),
        (
            enums::BRAM_V5_FIFO_WIDTH::_36,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ),
    ] {
        let mut diff = ctx.get_diff_attr_val(tcid, bslot, BRAM::FIFO_WIDTH, val);
        let mut diff_sdp = ctx.get_diff_attr_special_val(
            tcid,
            bslot,
            BRAM::FIFO_WIDTH,
            specials::BRAM_FIFO18_36,
            val,
        );
        if width == enums::BRAM_V5_DATA_WIDTH::_18 {
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_B_L),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_A_L),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
        }
        if val == enums::BRAM_V5_FIFO_WIDTH::_36 {
            diff_sdp.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, BRAM::READ_SDP_L), true, false);
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_SDP_L),
                true,
                false,
            );
        }
        assert_eq!(diff, diff_sdp);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_A_L),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_B_L),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        diffs.push((val, diff));
    }
    for (val, width) in [
        (enums::BRAM_V5_FIFO_WIDTH::_2, enums::BRAM_V5_DATA_WIDTH::_2),
        (enums::BRAM_V5_FIFO_WIDTH::_4, enums::BRAM_V5_DATA_WIDTH::_4),
        (enums::BRAM_V5_FIFO_WIDTH::_9, enums::BRAM_V5_DATA_WIDTH::_9),
        (
            enums::BRAM_V5_FIFO_WIDTH::_18,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ),
        (
            enums::BRAM_V5_FIFO_WIDTH::_36,
            enums::BRAM_V5_DATA_WIDTH::_18,
        ),
    ] {
        let mut diff = ctx.get_diff_attr_special_val(
            tcid,
            bslot,
            BRAM::FIFO_WIDTH,
            specials::BRAM_FIFO36,
            val,
        );
        let mut diff_sdp = ctx.get_diff_attr_special_val(
            tcid,
            bslot,
            BRAM::FIFO_WIDTH,
            specials::BRAM_FIFO36_72,
            val,
        );
        if val == enums::BRAM_V5_FIFO_WIDTH::_36 {
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_B_L),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_B_U),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_A_L),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff_sdp.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_A_U),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, BRAM::READ_SDP_L), true, false);
            diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, BRAM::READ_SDP_U), true, false);
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_SDP_L),
                true,
                false,
            );
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_SDP_U),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, BRAM::READ_SDP_L), true, false);
            diff_sdp.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, BRAM::READ_SDP_U), true, false);
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_SDP_L),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_SDP_U),
                true,
                false,
            );
        }
        if val == enums::BRAM_V5_FIFO_WIDTH::_4 {
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::READ_MUX_UL_A),
                true,
                false,
            );
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::READ_MUX_UL_B),
                true,
                false,
            );
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_MUX_UL_A),
                true,
                false,
            );
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_MUX_UL_B),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::READ_MUX_UL_A),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::READ_MUX_UL_B),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_MUX_UL_A),
                true,
                false,
            );
            diff_sdp.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, BRAM::WRITE_MUX_UL_B),
                true,
                false,
            );
        }

        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_A_L),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        if val != enums::BRAM_V5_FIFO_WIDTH::_36 {
            diff.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_B_L),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_A_U),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
            diff.apply_enum_diff(
                ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_B_U),
                width,
                enums::BRAM_V5_DATA_WIDTH::_1,
            );
        }
        diff_sdp.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_A_L),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        diff_sdp.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_B_L),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        diff_sdp.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::READ_WIDTH_A_U),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );
        diff_sdp.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, BRAM::WRITE_WIDTH_B_U),
            width,
            enums::BRAM_V5_DATA_WIDTH::_1,
        );

        assert_eq!(diff, diff_sdp);
        diffs.push((val, diff));
    }
    ctx.insert_bel_attr_enum(tcid, bslot, BRAM::FIFO_WIDTH, xlat_enum_attr(diffs));

    if edev.kind == ChipKind::Virtex6 {
        for val in [
            enums::BRAM_V5_RSTREG_PRIORITY::REGCE,
            enums::BRAM_V5_RSTREG_PRIORITY::RSTREG,
        ] {
            let diff_fifo = ctx.get_diff_attr_special_val(
                tcid,
                bslot,
                BRAM::RSTREG_PRIORITY_A_L,
                specials::BRAM_FIFO36,
                val,
            );
            let diff_a = ctx.peek_diff_attr_special_val(
                tcid,
                bslot,
                BRAM::RSTREG_PRIORITY_A_L,
                specials::BRAM_RAMB36,
                val,
            );
            let diff_b = ctx.peek_diff_attr_special_val(
                tcid,
                bslot,
                BRAM::RSTREG_PRIORITY_B_L,
                specials::BRAM_RAMB36,
                val,
            );
            assert_eq!(diff_fifo, diff_a.combine(diff_b));

            let diff_fifo_l = ctx.get_diff_attr_special_val(
                tcid,
                bslot,
                BRAM::RSTREG_PRIORITY_A_L,
                specials::BRAM_FIFO18,
                val,
            );
            let diff_al = ctx.peek_diff_attr_val(tcid, bslot, BRAM::RSTREG_PRIORITY_A_L, val);
            let diff_bl = ctx.peek_diff_attr_val(tcid, bslot, BRAM::RSTREG_PRIORITY_B_L, val);
            assert_eq!(diff_fifo_l, diff_al.combine(diff_bl));
        }
    }
    for (attr_l, attr_u) in [
        (BRAM::RSTREG_PRIORITY_A_L, BRAM::RSTREG_PRIORITY_A_U),
        (BRAM::RSTREG_PRIORITY_B_L, BRAM::RSTREG_PRIORITY_B_U),
    ] {
        for val in [
            enums::BRAM_V5_RSTREG_PRIORITY::REGCE,
            enums::BRAM_V5_RSTREG_PRIORITY::RSTREG,
        ] {
            let diff_f =
                ctx.get_diff_attr_special_val(tcid, bslot, attr_l, specials::BRAM_RAMB36, val);
            let diff_h0 = ctx.peek_diff_attr_val(tcid, bslot, attr_l, val);
            let diff_h1 = ctx.peek_diff_attr_val(tcid, bslot, attr_u, val);
            assert_eq!(diff_f, diff_h0.combine(diff_h1));
        }
        ctx.collect_bel_attr(tcid, bslot, attr_l);
        ctx.collect_bel_attr(tcid, bslot, attr_u);
    }

    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::EN_ECC_READ);
    ctx.collect_bel_attr_bi(tcid, bslot, BRAM::EN_ECC_WRITE);
    if edev.kind == ChipKind::Virtex6 {
        ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::EN_ECC_WRITE_NO_READ, false)
            .assert_empty();
        let mut diff = ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::EN_ECC_WRITE_NO_READ, true);
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, BRAM::EN_ECC_WRITE),
            true,
            false,
        );
        ctx.insert_bel_attr_bool(tcid, bslot, BRAM::EN_ECC_WRITE_NO_READ, xlat_bit(diff));
    }

    for val in [
        enums::BRAM_V5_RDADDR_COLLISION_HWCONFIG::PERFORMANCE,
        enums::BRAM_V5_RDADDR_COLLISION_HWCONFIG::DELAYED_WRITE,
    ] {
        let diff_f = ctx.get_diff_attr_special_val(
            tcid,
            bslot,
            BRAM::RDADDR_COLLISION_HWCONFIG_L,
            specials::BRAM_RAMB36,
            val,
        );
        let diff_h0 = ctx.peek_diff_attr_val(tcid, bslot, BRAM::RDADDR_COLLISION_HWCONFIG_L, val);
        let diff_h1 = ctx.peek_diff_attr_val(tcid, bslot, BRAM::RDADDR_COLLISION_HWCONFIG_U, val);
        assert_eq!(diff_f, diff_h0.combine(diff_h1));
    }
    ctx.collect_bel_attr(tcid, bslot, BRAM::RDADDR_COLLISION_HWCONFIG_L);
    ctx.collect_bel_attr(tcid, bslot, BRAM::RDADDR_COLLISION_HWCONFIG_U);

    if edev.kind == ChipKind::Virtex7 {
        for val in [
            enums::BRAM_V5_EN_PWRGATE::NONE,
            enums::BRAM_V5_EN_PWRGATE::LEFT,
            enums::BRAM_V5_EN_PWRGATE::RIGHT,
            enums::BRAM_V5_EN_PWRGATE::BOTH,
        ] {
            let diff_f = ctx.get_diff_attr_special_val(
                tcid,
                bslot,
                BRAM::EN_PWRGATE_L,
                specials::BRAM_RAMB36,
                val,
            );
            let diff_h0 = ctx.peek_diff_attr_val(tcid, bslot, BRAM::EN_PWRGATE_L, val);
            let diff_h1 = ctx.peek_diff_attr_val(tcid, bslot, BRAM::EN_PWRGATE_U, val);
            assert_eq!(diff_f, diff_h0.combine(diff_h1));
        }
        ctx.collect_bel_attr(tcid, bslot, BRAM::EN_PWRGATE_L);
        ctx.collect_bel_attr(tcid, bslot, BRAM::EN_PWRGATE_U);

        ctx.collect_bel_attr_bi(tcid, bslot, BRAM::EN_SDBITERR_INIT_V6);
    }

    present_ramb36.assert_empty();
    present_ramb18_l.assert_empty();
    present_ramb18_u.assert_empty();
    let is_fifo_u = present_fifo36.combine(&!&present_fifo18);
    ctx.insert_bel_attr_bool(tcid, bslot, BRAM::FIFO_ENABLE_L, xlat_bit(present_fifo18));
    ctx.insert_bel_attr_bool(tcid, bslot, BRAM::FIFO_ENABLE_U, xlat_bit(is_fifo_u));

    ctx.collect_bel_attr(tcid, bslot, BRAM::BYPASS_RSR);
    ctx.collect_bel_attr(tcid, bslot, BRAM::SWAP_CFGPORT);
    ctx.collect_bel_attr(tcid, bslot, BRAM::TEST_FIFO_FLAG);
    ctx.collect_bel_attr(tcid, bslot, BRAM::TEST_FIFO_OFFSET);
    ctx.collect_bel_attr(tcid, bslot, BRAM::TEST_FIFO_CNT);

    for attr in [BRAM::WW_VALUE, BRAM::EN_TSTBRAMRST, BRAM::DIS_TSTFIFORST] {
        ctx.collect_bel_attr(tcid, bslot, attr);
    }
    for attr in [BRAM::TRD_DLY_L, BRAM::TRD_DLY_U] {
        ctx.collect_bel_attr_sparse(tcid, bslot, attr, 0..8);
    }
    if edev.kind == ChipKind::Virtex6 {
        ctx.collect_bel_attr(tcid, bslot, BRAM::V6_TEST_ATTRIBUTES);
        for attr in [BRAM::V6_TWR_DLY_L, BRAM::V6_TWR_DLY_U] {
            ctx.collect_bel_attr_sparse(tcid, bslot, attr, 0..8);
        }
        for attr in [BRAM::EN_TSTREFBL, BRAM::EN_TSTRSRW] {
            ctx.collect_bel_attr_sparse(tcid, bslot, attr, 0..4);
        }
        ctx.collect_bel_attr_bi(tcid, bslot, BRAM::EN_TSTBLCLAMP);
    } else {
        ctx.collect_bel_attr(tcid, bslot, BRAM::V7_TEST_ATTRIBUTES);
        for attr in [
            BRAM::V7_TWR_DLY_A_L,
            BRAM::V7_TWR_DLY_A_U,
            BRAM::V7_TWR_DLY_B_L,
            BRAM::V7_TWR_DLY_B_U,
        ] {
            ctx.collect_bel_attr_sparse(tcid, bslot, attr, 0..16);
        }

        for attr in [
            BRAM::TSTREFBL_CTRL,
            BRAM::TSTRSR_RWCTRL,
            BRAM::EN_TSTRFMODE_DLY,
            BRAM::EN_TSTPULSEPU_DLY,
            BRAM::EN_TSTEXTCLK,
            BRAM::EN_TSTRSTC_PW,
            BRAM::EN_TSTBLPC_DLY,
            BRAM::EN_TST_REGOUT_DLY_SEL,
            BRAM::TST_RNG_OSC,
        ] {
            ctx.collect_bel_attr_sparse(tcid, bslot, attr, 0..4);
        }
        for attr in [
            BRAM::DIS_TSTBLCLAMP,
            BRAM::TST_SSRLAT_WF,
            BRAM::EN_TSTBLCLMP_WW,
            BRAM::EN_TSTSNM,
            BRAM::LAT_RST_DLYEN,
            BRAM::STUCK_DET_EN,
            BRAM::TST_PULSEPU_SFT,
            BRAM::EN_TSTBLCLAMP_RD,
        ] {
            ctx.collect_bel_attr_bi(tcid, bslot, attr);
        }

        // hm. bug? bug.
        ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::TST_BIST_CTL, false)
            .assert_empty();
        ctx.get_diff_attr_bool_bi(tcid, bslot, BRAM::TST_BIST_CTL, true)
            .assert_empty();
        ctx.insert_bel_attr_bool(
            tcid,
            bslot,
            BRAM::TST_BIST_CTL,
            TileBit::new(0, 26, 27).pos(),
        );
    }

    if edev.kind == ChipKind::Virtex7 {
        let BelInfo::SwitchBox(ref sb) = edev.db[tcls_v7::BRAM].bels[bslots::SPEC_INT] else {
            unreachable!()
        };
        for item in &sb.items {
            match item {
                SwitchBoxItem::Mux(mux) => {
                    ctx.collect_mux(tcid, mux.dst);
                }
                SwitchBoxItem::PermaBuf(buf) if !wires_v7::IMUX_BRAM.contains(buf.dst.wire) => {
                    let wires = if wires_v7::BRAM_ADDRA_CASC.contains(buf.dst.wire) {
                        wires_v7::BRAM_ADDRA_CASC
                    } else if wires_v7::BRAM_ADDRB_CASC.contains(buf.dst.wire) {
                        wires_v7::BRAM_ADDRB_CASC
                    } else {
                        unreachable!()
                    };
                    let bit =
                        xlat_bit(ctx.get_diff_routing_special(tcid, buf.dst, specials::PRESENT));
                    ctx.insert_support(
                        tcid,
                        BTreeSet::from_iter(wires.into_iter().map(|w| w.cell(0))),
                        vec![bit],
                    );
                }
                _ => (),
            }
        }
    }
}
