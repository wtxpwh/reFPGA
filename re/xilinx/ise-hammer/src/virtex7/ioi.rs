use std::collections::BTreeSet;

use prjcombine_entity::EntityId;
use prjcombine_interconnect::db::WireSlotIdExt;
use prjcombine_re_collector::diff::{
    Diff, OcdMode, extract_common_diff, xlat_bit, xlat_bit_bi, xlat_bit_wide_bi, xlat_bitvec,
    xlat_enum_attr, xlat_enum_attr_ocd, xlat_enum_raw,
};
use prjcombine_re_hammer::Session;
use prjcombine_types::bitrect::TileBit;
use prjcombine_virtex4::defs::{
    bcls::{IDELAY, ILOGIC, ODELAY, OLOGIC},
    bslots, enums,
    virtex7::{tcls, wires},
};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::{
        fbuild::{FuzzBuilderBase, FuzzCtx},
        int::{BaseIntPip, FuzzIntPip},
        props::{
            DynProp,
            bel::{BaseBelAttr, BaseBelMode},
            mutex::{TileMutex, WireMutexExclusive},
            relation::Related,
        },
    },
    virtex4::specials,
    virtex5::io::HclkIoi,
};

fn add_fuzzers_routing<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    for (tcid, num_io) in [
        (tcls::IO_HR_PAIR, 2),
        (tcls::IO_HR_S, 1),
        (tcls::IO_HR_N, 1),
        (tcls::IO_HP_PAIR, 2),
        (tcls::IO_HP_S, 1),
        (tcls::IO_HP_N, 1),
    ] {
        let Some(mut ctx) = FuzzCtx::try_new(session, backend, tcid) else {
            continue;
        };
        for io in 0..num_io {
            for w in [wires::IMUX_IOI_ICLK, wires::IMUX_IOI_OCLK] {
                let dst0 = w[0].cell(io);
                let dst1 = w[1].cell(io);
                let mux = &backend.edev.db_index[tcid].muxes[&dst0];
                for &src in mux.src.keys() {
                    if src.wire == wires::PHASER_OCLK90 {
                        ctx.build()
                            .prop(WireMutexExclusive::new(dst0))
                            .prop(WireMutexExclusive::new(dst1))
                            .prop(BaseIntPip::new(dst1, wires::PHASER_OCLK.cell(io)))
                            .test_routing(dst0, src)
                            .prop(FuzzIntPip::new(dst0, src.tw))
                            .commit();

                        ctx.build()
                            .prop(WireMutexExclusive::new(dst0))
                            .prop(WireMutexExclusive::new(dst1))
                            .test_routing_pair_special(dst1, src, specials::IOI_OCLK90_BOTH)
                            .prop(FuzzIntPip::new(dst0, src.tw))
                            .commit();
                    } else {
                        ctx.build()
                            .prop(WireMutexExclusive::new(dst0))
                            .prop(WireMutexExclusive::new(dst1))
                            .prop(BaseIntPip::new(dst1, src.tw))
                            .test_routing(dst0, src)
                            .prop(FuzzIntPip::new(dst0, src.tw))
                            .commit();
                        ctx.build()
                            .prop(WireMutexExclusive::new(dst0))
                            .prop(WireMutexExclusive::new(dst1))
                            .test_routing(dst1, src)
                            .prop(FuzzIntPip::new(dst1, src.tw))
                            .commit();
                    }
                }
            }

            let dst0 = wires::IMUX_IOI_OCLKDIV[0].cell(io);
            let dst1 = wires::IMUX_IOI_OCLKDIV[1].cell(io);
            let dst0f = wires::IMUX_IOI_OCLKDIVF[0].cell(io);
            let dst1f = wires::IMUX_IOI_OCLKDIVF[1].cell(io);
            let src_p = wires::PHASER_OCLKDIV.cell(io);
            ctx.build()
                .prop(WireMutexExclusive::new(dst0))
                .prop(WireMutexExclusive::new(dst1))
                .prop(BaseIntPip::new(dst1, src_p))
                .test_routing(dst0, src_p.pos())
                .prop(FuzzIntPip::new(dst0, src_p))
                .commit();
            ctx.build()
                .prop(WireMutexExclusive::new(dst0))
                .prop(WireMutexExclusive::new(dst1))
                .test_routing(dst1, src_p.pos())
                .prop(FuzzIntPip::new(dst1, src_p))
                .commit();
            let mux = &backend.edev.db_index[tcid].muxes[&dst0f];
            for &src in mux.src.keys() {
                ctx.build()
                    .prop(WireMutexExclusive::new(dst0))
                    .prop(WireMutexExclusive::new(dst1))
                    .prop(WireMutexExclusive::new(dst0f))
                    .prop(WireMutexExclusive::new(dst1f))
                    .prop(BaseIntPip::new(dst1, src.tw))
                    .test_routing(dst0, src)
                    .prop(FuzzIntPip::new(dst0, src.tw))
                    .commit();
                ctx.build()
                    .prop(WireMutexExclusive::new(dst0))
                    .prop(WireMutexExclusive::new(dst1))
                    .prop(WireMutexExclusive::new(dst0f))
                    .prop(WireMutexExclusive::new(dst1f))
                    .test_routing(dst1, src)
                    .prop(FuzzIntPip::new(dst1, src.tw))
                    .commit();
            }
        }
    }
}

fn add_fuzzers_ilogic<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    for (tcid, is_hp, num_io) in [
        (tcls::IO_HR_PAIR, false, 2),
        (tcls::IO_HR_S, false, 1),
        (tcls::IO_HR_N, false, 1),
        (tcls::IO_HP_PAIR, true, 2),
        (tcls::IO_HP_S, true, 1),
        (tcls::IO_HP_N, true, 1),
    ] {
        let Some(mut ctx) = FuzzCtx::try_new(session, backend, tcid) else {
            continue;
        };
        for i in 0..num_io {
            let mut bctx = ctx.bel(bslots::ILOGIC[i]);

            bctx.build()
                .null_bits()
                .test_bel_special(specials::ILOGIC)
                .mode("ILOGICE2")
                .commit();
            bctx.build()
                .test_bel_special(specials::ISERDES)
                .mode("ISERDESE2")
                .commit();

            bctx.mode("ISERDESE2").pin("D").test_bel_attr_bool_rename(
                "DINV",
                ILOGIC::D_INV,
                "D",
                "D_B",
            );
            bctx.mode("ISERDESE2").pin("CLK").test_bel_attr_bool_rename(
                "CLKINV",
                ILOGIC::CLK_INV,
                "CLK",
                "CLK_B",
            );
            for (attr, val) in [(ILOGIC::OCLK1_INV, "OCLK"), (ILOGIC::OCLK2_INV, "OCLK_B")] {
                bctx.mode("ISERDESE2")
                    .attr("DATA_RATE", "SDR")
                    .pin("OCLK")
                    .test_bel_attr_bits(attr)
                    .attr("OCLKINV", val)
                    .commit();
            }
            bctx.mode("ISERDESE2")
                .attr("DYN_CLKDIV_INV_EN", "FALSE")
                .test_bel_input_inv_auto(ILOGIC::CLKDIV);
            bctx.mode("ISERDESE2")
                .attr("DYN_CLKDIVP_INV_EN", "FALSE")
                .test_bel_input_inv_auto(ILOGIC::CLKDIVP);
            bctx.mode("ISERDESE2")
                .test_bel_attr_bool_auto(ILOGIC::DYN_CLK_INV_EN, "FALSE", "TRUE");
            bctx.mode("ISERDESE2").test_bel_attr_bool_auto(
                ILOGIC::DYN_CLKDIV_INV_EN,
                "FALSE",
                "TRUE",
            );
            bctx.mode("ISERDESE2").test_bel_attr_bool_auto(
                ILOGIC::DYN_CLKDIVP_INV_EN,
                "FALSE",
                "TRUE",
            );

            bctx.mode("ILOGICE2")
                .attr("IFFTYPE", "#FF")
                .pin("SR")
                .test_bel_attr_bits(ILOGIC::FFI_SR_ENABLE)
                .attr("SRUSED", "0")
                .commit();
            bctx.mode("ISERDESE2")
                .attr("DATA_WIDTH", "2")
                .attr("DATA_RATE", "SDR")
                .test_bel_attr_bool_auto(ILOGIC::SERDES, "FALSE", "TRUE");
            bctx.mode("ISERDESE2")
                .test_bel_attr_auto(ILOGIC::SERDES_MODE);
            bctx.mode("ISERDESE2")
                .attr("SERDES", "FALSE")
                .test_bel_attr_subset_auto(
                    ILOGIC::DATA_WIDTH,
                    &[
                        enums::IO_DATA_WIDTH::_2,
                        enums::IO_DATA_WIDTH::_3,
                        enums::IO_DATA_WIDTH::_4,
                        enums::IO_DATA_WIDTH::_5,
                        enums::IO_DATA_WIDTH::_6,
                        enums::IO_DATA_WIDTH::_7,
                        enums::IO_DATA_WIDTH::_8,
                        enums::IO_DATA_WIDTH::_10,
                        enums::IO_DATA_WIDTH::_14,
                    ],
                );
            bctx.mode("ISERDESE2").test_bel_attr_auto(ILOGIC::NUM_CE);

            for (attr, aname) in [
                (ILOGIC::FFI1_INIT, "INIT_Q1"),
                (ILOGIC::FFI2_INIT, "INIT_Q2"),
                (ILOGIC::FFI3_INIT, "INIT_Q3"),
                (ILOGIC::FFI4_INIT, "INIT_Q4"),
                (ILOGIC::FFI1_SRVAL, "SRVAL_Q1"),
                (ILOGIC::FFI2_SRVAL, "SRVAL_Q2"),
                (ILOGIC::FFI3_SRVAL, "SRVAL_Q3"),
                (ILOGIC::FFI4_SRVAL, "SRVAL_Q4"),
            ] {
                bctx.mode("ISERDESE2")
                    .test_bel_attr_bool_rename(aname, attr, "0", "1");
            }

            bctx.mode("ILOGICE2")
                .attr("IFFTYPE", "#FF")
                .test_bel_attr_bool_rename("SRTYPE", ILOGIC::FFI_SR_SYNC, "ASYNC", "SYNC");
            bctx.mode("ISERDESE2").test_bel_attr_bool_rename(
                "SRTYPE",
                ILOGIC::FFI_SR_SYNC,
                "ASYNC",
                "SYNC",
            );

            bctx.mode("ISERDESE2")
                .test_bel_attr_bool_auto(ILOGIC::D_EMU1, "FALSE", "TRUE");
            bctx.mode("ISERDESE2")
                .test_bel_attr_bool_auto(ILOGIC::D_EMU2, "FALSE", "TRUE");
            bctx.mode("ISERDESE2")
                .test_bel_attr_bool_auto(ILOGIC::RANK23_DLY, "FALSE", "TRUE");

            for (val, vname) in [
                (enums::ILOGIC_INTERFACE_TYPE::NETWORKING, "NETWORKING"),
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY, "MEMORY"),
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V7, "MEMORY_DDR3"),
                (enums::ILOGIC_INTERFACE_TYPE::OVERSAMPLE, "OVERSAMPLE"),
            ] {
                bctx.mode("ISERDESE2")
                    .test_bel_attr_val(ILOGIC::INTERFACE_TYPE, val)
                    .attr("INTERFACE_TYPE", vname)
                    .commit();
            }
            bctx.mode("ISERDESE2")
                .test_bel_attr_special(ILOGIC::INTERFACE_TYPE, specials::ILOGIC_MEMORY_QDR)
                .attr("INTERFACE_TYPE", "MEMORY_QDR")
                .commit();

            bctx.mode("ISERDESE2")
                .test_bel_attr_val(
                    ILOGIC::INTERFACE_TYPE,
                    enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6,
                )
                .attr("INTERFACE_TYPE", "MEMORY_DDR3")
                .attr("DDR3_V6", "TRUE")
                .commit();
            bctx.mode("ISERDESE2").test_bel_attr_auto(ILOGIC::DATA_RATE);
            bctx.mode("ISERDESE2")
                .test_bel_attr_auto(ILOGIC::DDR_CLK_EDGE);
            bctx.mode("ILOGICE2")
                .attr("IFFTYPE", "DDR")
                .test_bel_attr_auto(ILOGIC::DDR_CLK_EDGE);
            for (spec, val) in [
                (specials::ILOGIC_IFFTYPE_FF, "#FF"),
                (specials::ILOGIC_IFFTYPE_LATCH, "#LATCH"),
                (specials::ILOGIC_IFFTYPE_DDR, "DDR"),
            ] {
                bctx.mode("ILOGICE2")
                    .test_bel_special(spec)
                    .attr("IFFTYPE", val)
                    .commit();
            }

            bctx.mode("ISERDESE2")
                .pin("OFB")
                .test_bel_attr_bool_special_rename(
                    "OFB_USED",
                    ILOGIC::I_TSBYPASS_ENABLE,
                    specials::ISERDES,
                    "FALSE",
                    "TRUE",
                );
            for (val, vname) in [
                (enums::ILOGIC_MUX_TSBYPASS::GND, "FALSE"),
                (enums::ILOGIC_MUX_TSBYPASS::T, "TRUE"),
            ] {
                bctx.mode("ISERDESE2")
                    .pin("TFB")
                    .test_bel_attr_special_val(ILOGIC::MUX_TSBYPASS, specials::ISERDES, val)
                    .attr("TFB_USED", vname)
                    .commit();
            }
            for (spec, val) in [
                (specials::ILOGIC_IOBDELAY_NONE, "NONE"),
                (specials::ILOGIC_IOBDELAY_IFD, "IFD"),
                (specials::ILOGIC_IOBDELAY_IBUF, "IBUF"),
                (specials::ILOGIC_IOBDELAY_BOTH, "BOTH"),
            ] {
                bctx.mode("ISERDESE2")
                    .test_bel_special(spec)
                    .attr("IOBDELAY", val)
                    .commit();
            }

            bctx.mode("ILOGICE2")
                .attr("IMUX", "0")
                .attr("IDELMUX", "1")
                .attr("IFFMUX", "#OFF")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("TFB")
                .pin("OFB")
                .pin("O")
                .test_bel_attr_rename("D2OBYP_SEL", ILOGIC::MUX_TSBYPASS);
            bctx.mode("ILOGICE2")
                .attr("IFFMUX", "0")
                .attr("IFFTYPE", "#FF")
                .attr("IFFDELMUX", "1")
                .attr("IMUX", "#OFF")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("TFB")
                .pin("OFB")
                .test_bel_attr_rename("D2OFFBYP_SEL", ILOGIC::MUX_TSBYPASS);
            bctx.mode("ILOGICE2")
                .attr("IDELMUX", "1")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("O")
                .pin("TFB")
                .pin("OFB")
                .test_bel_attr_bool_rename("IMUX", ILOGIC::I_TSBYPASS_ENABLE, "1", "0");
            bctx.mode("ILOGICE2")
                .attr("IFFDELMUX", "1")
                .attr("IFFTYPE", "#FF")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("TFB")
                .pin("OFB")
                .test_bel_attr_bits(ILOGIC::FFI_TSBYPASS_ENABLE)
                .attr("IFFMUX", "0")
                .commit();
            bctx.mode("ILOGICE2")
                .attr("IMUX", "1")
                .attr("IFFMUX", "1")
                .attr("IFFTYPE", "#FF")
                .attr("IFFDELMUX", "0")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("O")
                .pin("Q1")
                .pin("TFB")
                .pin("OFB")
                .test_bel_attr_bool_rename("IDELMUX", ILOGIC::I_DELAY_ENABLE, "1", "0");
            bctx.mode("ILOGICE2")
                .attr("IMUX", "1")
                .attr("IFFMUX", "0")
                .attr("IFFTYPE", "#FF")
                .attr("IDELMUX", "0")
                .attr("D2OFFBYP_SEL", "T")
                .attr("DINV", "")
                .pin("D")
                .pin("DDLY")
                .pin("O")
                .pin("Q1")
                .pin("TFB")
                .pin("OFB")
                .test_bel_attr_bool_rename("IFFDELMUX", ILOGIC::FFI_DELAY_ENABLE, "1", "0");

            if !is_hp {
                bctx.build()
                    .null_bits()
                    .test_bel_special(specials::ILOGIC)
                    .mode("ILOGICE3")
                    .commit();
                for (val, vname) in [(false, "D"), (true, "D_B")] {
                    bctx.mode("ILOGICE3")
                        .attr("ZHOLD_IFF", "TRUE")
                        .attr("IFFTYPE", "#FF")
                        .pin("Q1")
                        .test_bel_attr_bits_bi(ILOGIC::FFI_ZHOLD_INV, val)
                        .attr("IFFDELMUXE3", "2")
                        .attr("IFFMUX", "1")
                        .attr("ZHOLD_IFF_INV", vname)
                        .commit();
                }
                bctx.mode("ILOGICE3")
                    .attr("ZHOLD_FABRIC", "TRUE")
                    .attr("IDELMUXE3", "2")
                    .attr("IMUX", "1")
                    .pin("O")
                    .test_bel_attr_bool_rename("ZHOLD_FABRIC_INV", ILOGIC::I_ZHOLD_INV, "D", "D_B");
                bctx.mode("ILOGICE3")
                    .attr("ZHOLD_IFF", "")
                    .test_bel_attr_bool_rename(
                        "ZHOLD_FABRIC",
                        ILOGIC::ZHOLD_ENABLE,
                        "FALSE",
                        "TRUE",
                    );
                bctx.mode("ILOGICE3")
                    .attr("ZHOLD_FABRIC", "")
                    .test_bel_attr_bool_rename("ZHOLD_IFF", ILOGIC::ZHOLD_ENABLE, "FALSE", "TRUE");
                bctx.mode("ILOGICE3")
                    .test_bel_attr_bits(ILOGIC::I_DELAY)
                    .multi_attr("IDELAY_VALUE", MultiValue::Dec(0), 5);
                bctx.mode("ILOGICE3")
                    .test_bel_attr_bits(ILOGIC::FFI_DELAY)
                    .multi_attr("IFFDELAY_VALUE", MultiValue::Dec(0), 5);
            }
        }
    }
}

fn add_fuzzers_ologic<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    for (tcid, num_io) in [
        (tcls::IO_HR_PAIR, 2),
        (tcls::IO_HR_S, 1),
        (tcls::IO_HR_N, 1),
        (tcls::IO_HP_PAIR, 2),
        (tcls::IO_HP_S, 1),
        (tcls::IO_HP_N, 1),
    ] {
        let Some(mut ctx) = FuzzCtx::try_new(session, backend, tcid) else {
            continue;
        };
        for i in 0..num_io {
            let mut bctx = ctx.bel(bslots::OLOGIC[i]);

            bctx.build()
                .test_bel_special(specials::OLOGIC)
                .mode("OLOGICE2")
                .commit();
            bctx.build()
                .test_bel_special(specials::OSERDES)
                .mode("OSERDESE2")
                .commit();

            for pin in [
                OLOGIC::D1,
                OLOGIC::D2,
                OLOGIC::D3,
                OLOGIC::D4,
                OLOGIC::D5,
                OLOGIC::D6,
                OLOGIC::D7,
                OLOGIC::D8,
                OLOGIC::T1,
                OLOGIC::T2,
                OLOGIC::T3,
                OLOGIC::T4,
                OLOGIC::CLKDIV,
                OLOGIC::CLKDIVF,
            ] {
                bctx.mode("OSERDESE2").test_bel_input_inv_auto(pin);
            }
            bctx.mode("OSERDESE2")
                .attr("DATA_RATE_OQ", "DDR")
                .attr("DDR_CLK_EDGE", "SAME_EDGE")
                .pin("OCE")
                .pin("CLK")
                .test_bel_attr_bool_special_rename(
                    "CLKINV",
                    OLOGIC::CLK1_INV,
                    specials::OSERDES_SAME_EDGE,
                    "CLK",
                    "CLK_B",
                );
            bctx.mode("OSERDESE2")
                .attr("DATA_RATE_OQ", "DDR")
                .attr("DDR_CLK_EDGE", "OPPOSITE_EDGE")
                .pin("OCE")
                .pin("CLK")
                .test_bel_attr_bool_special_rename(
                    "CLKINV",
                    OLOGIC::CLK1_INV,
                    specials::OSERDES_OPPOSITE_EDGE,
                    "CLK",
                    "CLK_B",
                );

            bctx.mode("OLOGICE2")
                .attr("OUTFFTYPE", "#FF")
                .test_bel_attr_bool_rename("SRTYPE_OQ", OLOGIC::FFO_SR_SYNC, "ASYNC", "SYNC");
            bctx.mode("OLOGICE2")
                .attr("TFFTYPE", "#FF")
                .test_bel_attr_bool_rename("SRTYPE_TQ", OLOGIC::FFT_SR_SYNC, "ASYNC", "SYNC");
            bctx.mode("OSERDESE2").test_bel_attr_bool_special_rename(
                "SRTYPE",
                OLOGIC::FFO_SR_SYNC,
                specials::OSERDES,
                "ASYNC",
                "SYNC",
            );

            bctx.mode("OLOGICE2")
                .test_bel_attr_bool_rename("INIT_OQ", OLOGIC::FFO_INIT, "0", "1");
            bctx.mode("OLOGICE2")
                .test_bel_attr_bool_rename("INIT_TQ", OLOGIC::FFT_INIT, "0", "1");
            bctx.mode("OSERDESE2").test_bel_attr_bool_special_rename(
                "INIT_OQ",
                OLOGIC::FFO_INIT,
                specials::OSERDES,
                "0",
                "1",
            );
            bctx.mode("OSERDESE2").test_bel_attr_bool_special_rename(
                "INIT_TQ",
                OLOGIC::FFT_INIT,
                specials::OSERDES,
                "0",
                "1",
            );
            bctx.mode("OLOGICE2").test_bel_attr_bool_rename(
                "SRVAL_OQ",
                OLOGIC::FFO_SRVAL,
                "0",
                "1",
            );
            bctx.mode("OLOGICE2").test_bel_attr_bool_rename(
                "SRVAL_TQ",
                OLOGIC::FFT_SRVAL,
                "0",
                "1",
            );
            bctx.mode("OSERDESE2").test_bel_attr_bool_special_rename(
                "SRVAL_OQ",
                OLOGIC::FFO_SRVAL,
                specials::OSERDES,
                "0",
                "1",
            );
            bctx.mode("OSERDESE2").test_bel_attr_bool_special_rename(
                "SRVAL_TQ",
                OLOGIC::FFT_SRVAL,
                specials::OSERDES,
                "0",
                "1",
            );

            for (attr, aname) in [
                (OLOGIC::FFO_SR_ENABLE, "OSRUSED"),
                (OLOGIC::FFT_SR_ENABLE, "TSRUSED"),
            ] {
                bctx.mode("OLOGICE2")
                    .attr("OUTFFTYPE", "#FF")
                    .attr("TFFTYPE", "#FF")
                    .pin("OCE")
                    .pin("TCE")
                    .pin("REV")
                    .pin("SR")
                    .test_bel_attr_bits(attr)
                    .attr(aname, "0")
                    .commit();
            }

            for (val, vname) in [
                (enums::OLOGIC_MUX_O::FF, "#FF"),
                (enums::OLOGIC_MUX_O::LATCH, "#LATCH"),
                (enums::OLOGIC_MUX_O::DDR, "DDR"),
            ] {
                bctx.mode("OLOGICE2")
                    .pin("OQ")
                    .test_bel_attr_val(OLOGIC::MUX_O, val)
                    .attr("OUTFFTYPE", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::OLOGIC_MUX_T::FF, "#FF"),
                (enums::OLOGIC_MUX_T::LATCH, "#LATCH"),
                (enums::OLOGIC_MUX_T::DDR, "DDR"),
            ] {
                bctx.mode("OLOGICE2")
                    .pin("TQ")
                    .test_bel_attr_val(OLOGIC::MUX_T, val)
                    .attr("TFFTYPE", vname)
                    .commit();
            }
            bctx.mode("OLOGICE2")
                .test_bel_attr_val(OLOGIC::MUX_O, enums::OLOGIC_MUX_O::D1)
                .attr("OQUSED", "0")
                .attr("O1USED", "0")
                .attr("D1INV", "D1")
                .attr("OMUX", "D1")
                .pin("OQ")
                .pin("D1")
                .commit();

            for (val, vname) in [
                (enums::OLOGIC_MUX_O::SERDES_SDR, "SDR"),
                (enums::OLOGIC_MUX_O::SERDES_DDR, "DDR"),
            ] {
                bctx.mode("OSERDESE2")
                    .test_bel_attr_val(OLOGIC::MUX_O, val)
                    .attr("DATA_RATE_OQ", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::OLOGIC_MUX_T::T1, "BUF"),
                (enums::OLOGIC_MUX_T::SERDES_SDR, "SDR"),
                (enums::OLOGIC_MUX_T::SERDES_DDR, "DDR"),
            ] {
                bctx.mode("OSERDESE2")
                    .test_bel_attr_val(OLOGIC::MUX_T, val)
                    .attr("DATA_RATE_TQ", vname)
                    .commit();
            }

            bctx.mode("OLOGICE2")
                .global("ENABLEMISR", "Y")
                .test_bel_attr_bool_auto(OLOGIC::MISR_ENABLE, "FALSE", "TRUE");
            bctx.mode("OLOGICE2")
                .global("ENABLEMISR", "Y")
                .test_bel_attr_bool_auto(OLOGIC::MISR_ENABLE_FDBK, "FALSE", "TRUE");
            bctx.mode("OLOGICE2")
                .global("ENABLEMISR", "Y")
                .test_bel_attr_auto_default(
                    OLOGIC::MISR_CLK_SELECT,
                    enums::OLOGIC_MISR_CLK_SELECT::NONE,
                );

            bctx.mode("OSERDESE2")
                .test_bel_attr_bool_auto(OLOGIC::SERDES, "FALSE", "TRUE");
            bctx.mode("OSERDESE2")
                .test_bel_attr_auto(OLOGIC::SERDES_MODE);
            bctx.mode("OSERDESE2")
                .test_bel_attr_bool_auto(OLOGIC::SELFHEAL, "FALSE", "TRUE");
            bctx.mode("OSERDESE2")
                .test_bel_attr_bool_auto(OLOGIC::RANK3_USED, "FALSE", "TRUE");
            bctx.mode("OSERDESE2")
                .test_bel_attr_bool_auto(OLOGIC::TBYTE_CTL, "FALSE", "TRUE");
            bctx.mode("OSERDESE2")
                .test_bel_attr_bool_auto(OLOGIC::TBYTE_SRC, "FALSE", "TRUE");
            bctx.mode("OSERDESE2").test_bel_attr_subset_auto(
                OLOGIC::TRISTATE_WIDTH,
                &[
                    enums::OLOGIC_TRISTATE_WIDTH::_1,
                    enums::OLOGIC_TRISTATE_WIDTH::_4,
                ],
            );
            for (val, vname) in [
                (enums::IO_DATA_WIDTH::_2, "2"),
                (enums::IO_DATA_WIDTH::_3, "3"),
                (enums::IO_DATA_WIDTH::_4, "4"),
                (enums::IO_DATA_WIDTH::_5, "5"),
                (enums::IO_DATA_WIDTH::_6, "6"),
                (enums::IO_DATA_WIDTH::_7, "7"),
                (enums::IO_DATA_WIDTH::_8, "8"),
            ] {
                bctx.mode("OSERDESE2")
                    .attr("DATA_RATE_OQ", "SDR")
                    .test_bel_attr_special_val(OLOGIC::DATA_WIDTH, specials::OSERDES_SDR, val)
                    .attr("DATA_WIDTH", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::IO_DATA_WIDTH::_4, "4"),
                (enums::IO_DATA_WIDTH::_6, "6"),
                (enums::IO_DATA_WIDTH::_8, "8"),
                (enums::IO_DATA_WIDTH::_10, "10"),
                (enums::IO_DATA_WIDTH::_14, "14"),
            ] {
                bctx.mode("OSERDESE2")
                    .attr("DATA_RATE_OQ", "DDR")
                    .test_bel_attr_special_val(OLOGIC::DATA_WIDTH, specials::OSERDES_DDR, val)
                    .attr("DATA_WIDTH", vname)
                    .commit();
            }
        }
    }
    let mut ctx = FuzzCtx::new_null(session, backend);
    ctx.build()
        .global("ENABLEMISR", "Y")
        .extra_tiles_by_bel_attr_bits(bslots::OLOGIC[0], OLOGIC::MISR_RESET)
        .test_global_special(specials::MISR_RESET)
        .global_diff("MISRRESET", "N", "Y")
        .commit();
}

fn add_fuzzers_iodelay<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    for (tcid, is_hp, num_io) in [
        (tcls::IO_HR_PAIR, false, 2),
        (tcls::IO_HR_S, false, 1),
        (tcls::IO_HR_N, false, 1),
        (tcls::IO_HP_PAIR, true, 2),
        (tcls::IO_HP_S, true, 1),
        (tcls::IO_HP_N, true, 1),
    ] {
        let Some(mut ctx) = FuzzCtx::try_new(session, backend, tcid) else {
            continue;
        };
        let setup_idelayctrl: [Box<DynProp>; 4] = [
            Box::new(Related::new(
                HclkIoi,
                TileMutex::new("IDELAYCTRL".into(), "USE".into()),
            )),
            Box::new(Related::new(
                HclkIoi,
                BaseBelMode::new(bslots::IDELAYCTRL, 0, "IDELAYCTRL".into()),
            )),
            Box::new(Related::new(
                HclkIoi,
                BaseBelAttr::new(
                    bslots::IDELAYCTRL,
                    0,
                    "IDELAYCTRL_EN".into(),
                    "ENABLE".into(),
                ),
            )),
            Box::new(Related::new(
                HclkIoi,
                BaseBelAttr::new(bslots::IDELAYCTRL, 0, "BIAS_MODE".into(), "0".into()),
            )),
        ];
        for i in 0..num_io {
            let mut bctx = ctx.bel(bslots::IDELAY[i]);
            let bel_ologic = bslots::OLOGIC[i];
            bctx.build()
                .props(setup_idelayctrl.clone())
                .test_bel_attr_bits(IDELAY::ENABLE)
                .mode("IDELAYE2")
                .commit();
            for pin in [IDELAY::C, IDELAY::DATAIN] {
                bctx.mode("IDELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("CINVCTRL_SEL", "FALSE")
                    .test_bel_input_inv_auto(pin);
            }
            bctx.mode("IDELAYE2")
                .props(setup_idelayctrl.clone())
                .attr("CINVCTRL_SEL", "FALSE")
                .pin("IDATAIN")
                .test_bel_attr_bool_rename(
                    "IDATAININV",
                    IDELAY::IDATAIN_INV,
                    "IDATAIN",
                    "IDATAIN_B",
                );
            for attr in [
                IDELAY::HIGH_PERFORMANCE_MODE,
                IDELAY::CINVCTRL_SEL,
                IDELAY::PIPE_SEL,
            ] {
                bctx.mode("IDELAYE2")
                    .props(setup_idelayctrl.clone())
                    .test_bel_attr_bool_auto(attr, "FALSE", "TRUE");
            }
            bctx.mode("IDELAYE2")
                .props(setup_idelayctrl.clone())
                .test_bel_attr_val(IDELAY::DELAY_SRC, enums::IDELAY_DELAY_SRC::NONE)
                .attr("DELAYCHAIN_OSC", "FALSE")
                .commit();
            bctx.mode("IDELAYE2")
                .props(setup_idelayctrl.clone())
                .test_bel_attr_val(IDELAY::DELAY_SRC, enums::IDELAY_DELAY_SRC::DELAYCHAIN_OSC)
                .attr("DELAYCHAIN_OSC", "TRUE")
                .commit();
            for (val, vname) in [
                (enums::IODELAY_DELAY_TYPE::FIXED, "FIXED"),
                (enums::IODELAY_DELAY_TYPE::VARIABLE, "VARIABLE"),
                (enums::IODELAY_DELAY_TYPE::VAR_LOAD, "VAR_LOAD"),
                (enums::IODELAY_DELAY_TYPE::VAR_LOAD, "VAR_LOAD_PIPE"),
            ] {
                bctx.mode("IDELAYE2")
                    .props(setup_idelayctrl.clone())
                    .test_bel_attr_val(IDELAY::DELAY_TYPE, val)
                    .attr("IDELAY_TYPE", vname)
                    .commit();
            }
            for (val, vname) in [
                (enums::IDELAY_DELAY_SRC::DATAIN, "DATAIN"),
                (enums::IDELAY_DELAY_SRC::IDATAIN, "IDATAIN"),
            ] {
                bctx.mode("IDELAYE2")
                    .props(setup_idelayctrl.clone())
                    .test_bel_attr_val(IDELAY::DELAY_SRC, val)
                    .attr("DELAY_SRC", vname)
                    .commit();
            }
            bctx.build()
                .attr("DELAY_SRC", "")
                .test_bel_attr_val(IDELAY::DELAY_SRC, enums::IDELAY_DELAY_SRC::OFB)
                .pip("IDATAIN", (bel_ologic, "OFB"))
                .commit();
            bctx.mode("IDELAYE2")
                .props(setup_idelayctrl.clone())
                .attr("DELAY_SRC", "IDATAIN")
                .attr("IDELAY_TYPE", "FIXED")
                .test_bel_attr_bits(IDELAY::IDELAY_VALUE_INIT)
                .multi_attr("IDELAY_VALUE", MultiValue::Dec(0), 5);
            if is_hp {
                bctx.mode("IDELAYE2_FINEDELAY")
                    .props(setup_idelayctrl.clone())
                    .test_bel_attr_bool_auto(IDELAY::FINEDELAY, "BYPASS", "ADD_DLY");
            }
        }
        if is_hp {
            for i in 0..num_io {
                let mut bctx = ctx.bel(bslots::ODELAY[i]);
                bctx.build()
                    .null_bits()
                    .props(setup_idelayctrl.clone())
                    .test_bel_special(specials::PRESENT)
                    .mode("ODELAYE2")
                    .commit();
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("CINVCTRL_SEL", "FALSE")
                    .test_bel_input_inv_auto(ODELAY::C);
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("CINVCTRL_SEL", "FALSE")
                    .pin("ODATAIN")
                    .test_bel_attr_bool_rename(
                        "ODATAININV",
                        ODELAY::ODATAIN_INV,
                        "ODATAIN",
                        "ODATAIN_B",
                    );
                for attr in [
                    ODELAY::HIGH_PERFORMANCE_MODE,
                    ODELAY::CINVCTRL_SEL,
                    ODELAY::PIPE_SEL,
                ] {
                    bctx.mode("ODELAYE2")
                        .props(setup_idelayctrl.clone())
                        .attr("DELAY_SRC", "")
                        .test_bel_attr_bool_auto(attr, "FALSE", "TRUE");
                }
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("DELAY_SRC", "")
                    .test_bel_attr_val(ODELAY::DELAY_SRC, enums::ODELAY_DELAY_SRC::NONE)
                    .attr("DELAYCHAIN_OSC", "FALSE")
                    .commit();
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("DELAY_SRC", "")
                    .test_bel_attr_val(ODELAY::DELAY_SRC, enums::ODELAY_DELAY_SRC::DELAYCHAIN_OSC)
                    .attr("DELAYCHAIN_OSC", "TRUE")
                    .commit();
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("DELAY_SRC", "ODATAIN")
                    .attr("PIPE_SEL", "FALSE")
                    .test_bel_attr_rename("ODELAY_TYPE", ODELAY::DELAY_TYPE);
                for (val, vname) in [
                    (enums::ODELAY_DELAY_SRC::ODATAIN, "ODATAIN"),
                    (enums::ODELAY_DELAY_SRC::CLKIN, "CLKIN"),
                ] {
                    bctx.mode("ODELAYE2")
                        .props(setup_idelayctrl.clone())
                        .attr("DELAYCHAIN_OSC", "")
                        .test_bel_attr_val(ODELAY::DELAY_SRC, val)
                        .attr("DELAY_SRC", vname)
                        .commit();
                }
                bctx.mode("ODELAYE2")
                    .props(setup_idelayctrl.clone())
                    .attr("DELAY_SRC", "ODATAIN")
                    .attr("ODELAY_TYPE", "FIXED")
                    .test_bel_attr_bits(ODELAY::ODELAY_VALUE_INIT)
                    .multi_attr("ODELAY_VALUE", MultiValue::Dec(0), 5);
                bctx.mode("ODELAYE2_FINEDELAY")
                    .props(setup_idelayctrl.clone())
                    .test_bel_attr_bool_auto(ODELAY::FINEDELAY, "BYPASS", "ADD_DLY");
            }
        }
    }
}

pub fn add_fuzzers<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    add_fuzzers_routing(session, backend);
    add_fuzzers_ilogic(session, backend);
    add_fuzzers_ologic(session, backend);
    add_fuzzers_iodelay(session, backend);
}
fn collect_fuzzers_routing(ctx: &mut CollectorCtx) {
    for (tcid, num_io) in [
        (tcls::IO_HR_PAIR, 2),
        (tcls::IO_HR_S, 1),
        (tcls::IO_HR_N, 1),
        (tcls::IO_HP_PAIR, 2),
        (tcls::IO_HP_S, 1),
        (tcls::IO_HP_N, 1),
    ] {
        if !ctx.has_tcls(tcid) {
            continue;
        };
        for io in 0..num_io {
            ctx.collect_mux(tcid, wires::IMUX_IOI_ICLK[0].cell(io));
            ctx.collect_mux(tcid, wires::IMUX_IOI_ICLK[1].cell(io));

            let dst0 = wires::IMUX_IOI_OCLK[0].cell(io);
            let dst1 = wires::IMUX_IOI_OCLK[1].cell(io);
            ctx.collect_mux(tcid, dst0);
            let mux = &ctx.edev.db_index[tcid].muxes[&dst0];
            let mut diffs = vec![(None, Diff::default())];
            for &src in mux.src.keys() {
                if src.wire == wires::PHASER_OCLK90 {
                    let mut diff = ctx.get_diff_routing_pair_special(
                        tcid,
                        dst1,
                        src,
                        specials::IOI_OCLK90_BOTH,
                    );
                    diff.apply_enum_diff_raw(ctx.sb_mux(tcid, dst0), &Some(src), &None);
                    diffs.push((Some(src), diff));
                } else {
                    diffs.push((Some(src), ctx.get_diff_routing(tcid, dst1, src)));
                }
            }
            ctx.insert_mux(tcid, dst1, xlat_enum_raw(diffs, OcdMode::Mux));

            for i in 0..2 {
                let dst = wires::IMUX_IOI_OCLKDIV[i].cell(io);
                let dst_f = wires::IMUX_IOI_OCLKDIVF[i].cell(io);

                let mut diffs_f = vec![];
                let mux = &ctx.edev.db_index[tcid].muxes[&dst_f];
                for &src in mux.src.keys() {
                    diffs_f.push((Some(src), ctx.get_diff_routing(tcid, dst, src)));
                }
                let diff_f = extract_common_diff(&mut diffs_f);
                diffs_f.push((None, Diff::default()));
                ctx.insert_mux(tcid, dst_f, xlat_enum_raw(diffs_f, OcdMode::Mux));

                let src_p = wires::PHASER_OCLKDIV.cell(io);
                let diffs = vec![
                    (None, Diff::default()),
                    (
                        Some(src_p.pos()),
                        ctx.get_diff_routing(tcid, dst, src_p.pos()),
                    ),
                    (Some(dst_f.pos()), diff_f),
                ];
                ctx.insert_mux(tcid, dst, xlat_enum_raw(diffs, OcdMode::Mux));
            }
        }
    }
}

fn collect_fuzzers_ilogic(ctx: &mut CollectorCtx) {
    for (tcid, bslot, is_hp) in [
        (tcls::IO_HR_PAIR, bslots::ILOGIC[0], false),
        (tcls::IO_HR_PAIR, bslots::ILOGIC[1], false),
        (tcls::IO_HR_S, bslots::ILOGIC[0], false),
        (tcls::IO_HR_N, bslots::ILOGIC[0], false),
        (tcls::IO_HP_PAIR, bslots::ILOGIC[0], true),
        (tcls::IO_HP_PAIR, bslots::ILOGIC[1], true),
        (tcls::IO_HP_S, bslots::ILOGIC[0], true),
        (tcls::IO_HP_N, bslots::ILOGIC[0], true),
    ] {
        if !ctx.has_tcls(tcid) {
            continue;
        }

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::D_INV);
        ctx.collect_bel_input_inv_bi(tcid, bslot, ILOGIC::CLKDIV);
        ctx.collect_bel_input_inv_bi(tcid, bslot, ILOGIC::CLKDIVP);
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::CLK_INV, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::CLK_INV, true),
        );
        ctx.insert_bel_attr_bitvec(tcid, bslot, ILOGIC::CLK_INV, bits);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::OCLK1_INV);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::OCLK2_INV);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::DYN_CLK_INV_EN);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::DYN_CLKDIV_INV_EN);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::DYN_CLKDIVP_INV_EN);

        ctx.collect_bel_attr(tcid, bslot, ILOGIC::FFI_SR_ENABLE);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::SERDES_MODE);

        let mut diffs = vec![(enums::IO_DATA_WIDTH::NONE, Diff::default())];
        for val in [
            enums::IO_DATA_WIDTH::_2,
            enums::IO_DATA_WIDTH::_3,
            enums::IO_DATA_WIDTH::_4,
            enums::IO_DATA_WIDTH::_5,
            enums::IO_DATA_WIDTH::_6,
            enums::IO_DATA_WIDTH::_7,
            enums::IO_DATA_WIDTH::_8,
            enums::IO_DATA_WIDTH::_10,
            enums::IO_DATA_WIDTH::_14,
        ] {
            diffs.push((
                val,
                ctx.get_diff_attr_val(tcid, bslot, ILOGIC::DATA_WIDTH, val),
            ));
        }
        let mut bits = xlat_enum_attr(diffs.clone()).bits;
        bits.swap(0, 1);
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            ILOGIC::DATA_WIDTH,
            xlat_enum_attr_ocd(diffs, OcdMode::FixedOrder(&bits)),
        );

        ctx.collect_bel_attr(tcid, bslot, ILOGIC::NUM_CE);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI1_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI2_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI3_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI4_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI1_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI2_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI3_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI4_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI_SR_SYNC);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::DATA_RATE);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::D_EMU1);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::D_EMU2);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::RANK23_DLY);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::DDR_CLK_EDGE);

        let diff_mem = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY,
        );
        let diff_qdr = ctx.get_diff_attr_special(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            specials::ILOGIC_MEMORY_QDR,
        );
        let diff_net = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::NETWORKING,
        );
        let diff_ddr3 = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V7,
        );
        let diff_ddr3_v6 = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6,
        );
        let diff_os = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::OVERSAMPLE,
        );
        let bitslip_en = diff_net.combine(&!&diff_qdr);
        let diff_net = diff_net.combine(&!&bitslip_en);
        let diff_os = diff_os.combine(&!&bitslip_en);
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::BITSLIP_ENABLE, xlat_bit(bitslip_en));
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            xlat_enum_attr(vec![
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY, diff_mem),
                (enums::ILOGIC_INTERFACE_TYPE::NETWORKING, diff_net),
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V7, diff_ddr3),
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6, diff_ddr3_v6),
                (enums::ILOGIC_INTERFACE_TYPE::OVERSAMPLE, diff_os),
            ]),
        );

        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IFFTYPE_LATCH);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::DDR_CLK_EDGE),
            enums::ILOGIC_DDR_CLK_EDGE::OPPOSITE_EDGE,
            enums::ILOGIC_DDR_CLK_EDGE::SAME_EDGE_PIPELINED,
        );
        diff.assert_empty();
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IFFTYPE_FF);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::DDR_CLK_EDGE),
            enums::ILOGIC_DDR_CLK_EDGE::OPPOSITE_EDGE,
            enums::ILOGIC_DDR_CLK_EDGE::SAME_EDGE_PIPELINED,
        );
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::INTERFACE_TYPE),
            enums::ILOGIC_INTERFACE_TYPE::NETWORKING,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY,
        );
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::FFI_LATCH, xlat_bit(!diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IFFTYPE_DDR);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::INTERFACE_TYPE),
            enums::ILOGIC_INTERFACE_TYPE::NETWORKING,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY,
        );
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::FFI_LATCH, xlat_bit(!diff));

        let diff_f = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::SERDES, false);
        let diff_t = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::SERDES, true);
        let (diff_f, diff_t, mut diff_serdes) = Diff::split(diff_f, diff_t);
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::SERDES, xlat_bit_bi(diff_f, diff_t));
        diff_serdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_SR_ENABLE),
            true,
            false,
        );
        diff_serdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_LATCH),
            false,
            true,
        );
        diff_serdes.assert_empty();

        ctx.collect_bel_attr(tcid, bslot, ILOGIC::MUX_TSBYPASS);
        let item = xlat_enum_attr(vec![
            (
                enums::ILOGIC_MUX_TSBYPASS::T,
                ctx.get_diff_attr_special_val(
                    tcid,
                    bslot,
                    ILOGIC::MUX_TSBYPASS,
                    specials::ISERDES,
                    enums::ILOGIC_MUX_TSBYPASS::T,
                ),
            ),
            (
                enums::ILOGIC_MUX_TSBYPASS::GND,
                ctx.get_diff_attr_special_val(
                    tcid,
                    bslot,
                    ILOGIC::MUX_TSBYPASS,
                    specials::ISERDES,
                    enums::ILOGIC_MUX_TSBYPASS::GND,
                ),
            ),
        ]);
        ctx.insert_bel_attr_enum(tcid, bslot, ILOGIC::MUX_TSBYPASS, item);

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::I_DELAY_ENABLE);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI_DELAY_ENABLE);

        ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IOBDELAY_NONE)
            .assert_empty();
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IOBDELAY_IBUF);
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::I_DELAY_ENABLE),
            true,
            false,
        );
        diff.assert_empty();
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IOBDELAY_IFD);
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_DELAY_ENABLE),
            true,
            false,
        );
        diff.assert_empty();
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IOBDELAY_BOTH);
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::I_DELAY_ENABLE),
            true,
            false,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_DELAY_ENABLE),
            true,
            false,
        );
        diff.assert_empty();

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::I_TSBYPASS_ENABLE);
        // the fuzzer is slightly fucked to work around some ridiculous ISE bug.
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::FFI_TSBYPASS_ENABLE);
        ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            ILOGIC::I_TSBYPASS_ENABLE,
            specials::ISERDES,
            0,
            false,
        )
        .assert_empty();
        let mut diff = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            ILOGIC::I_TSBYPASS_ENABLE,
            specials::ISERDES,
            0,
            true,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::I_TSBYPASS_ENABLE),
            true,
            false,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_TSBYPASS_ENABLE),
            true,
            false,
        );
        diff.assert_empty();

        let mut present_iserdes = ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES);
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI1_SRVAL),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI2_SRVAL),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI3_SRVAL),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI4_SRVAL),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI1_INIT),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI2_INIT),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI3_INIT),
            false,
            true,
        );
        present_iserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI4_INIT),
            false,
            true,
        );
        present_iserdes.assert_empty();

        if !is_hp {
            ctx.collect_bel_attr(tcid, bslot, ILOGIC::I_DELAY);
            ctx.collect_bel_attr(tcid, bslot, ILOGIC::FFI_DELAY);
            ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::ZHOLD_ENABLE);

            let diff0 = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::I_ZHOLD_INV, false);
            let diff1 = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::I_ZHOLD_INV, true);
            let (diff0, diff1, diff_en) = Diff::split(diff0, diff1);
            ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::I_ZHOLD_INV, xlat_bit_bi(diff0, diff1));
            ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::I_ZHOLD, xlat_bit(diff_en));

            let diff0 = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::FFI_ZHOLD_INV, false);
            let diff1 = ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::FFI_ZHOLD_INV, true);
            let (diff0, diff1, diff_en) = Diff::split(diff0, diff1);
            ctx.insert_bel_attr_bool(
                tcid,
                bslot,
                ILOGIC::FFI_ZHOLD_INV,
                xlat_bit_bi(diff0, diff1),
            );
            ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::FFI_ZHOLD, xlat_bit(diff_en));
        }
    }
}

fn collect_fuzzers_ologic(ctx: &mut CollectorCtx) {
    for (tcid, bslot, c, i) in [
        (tcls::IO_HR_PAIR, bslots::OLOGIC[0], 0, 0),
        (tcls::IO_HR_PAIR, bslots::OLOGIC[1], 1, 1),
        (tcls::IO_HR_S, bslots::OLOGIC[0], 0, 1),
        (tcls::IO_HR_N, bslots::OLOGIC[0], 0, 0),
        (tcls::IO_HP_PAIR, bslots::OLOGIC[0], 0, 0),
        (tcls::IO_HP_PAIR, bslots::OLOGIC[1], 1, 1),
        (tcls::IO_HP_S, bslots::OLOGIC[0], 0, 1),
        (tcls::IO_HP_N, bslots::OLOGIC[0], 0, 0),
    ] {
        if !ctx.has_tcls(tcid) {
            continue;
        }

        for pin in [
            OLOGIC::D1,
            OLOGIC::D2,
            OLOGIC::D3,
            OLOGIC::D4,
            OLOGIC::D5,
            OLOGIC::D6,
            OLOGIC::D7,
            OLOGIC::D8,
            OLOGIC::T1,
            OLOGIC::T2,
            OLOGIC::T3,
            OLOGIC::T4,
            OLOGIC::CLKDIV,
            OLOGIC::CLKDIVF,
        ] {
            ctx.collect_bel_input_inv_bi(tcid, bslot, pin);
        }

        ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::CLK1_INV,
            specials::OSERDES_SAME_EDGE,
            0,
            true,
        )
        .assert_empty();
        let diff_clk1 = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::CLK1_INV,
            specials::OSERDES_OPPOSITE_EDGE,
            0,
            false,
        );
        let diff_clk2 = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::CLK1_INV,
            specials::OSERDES_OPPOSITE_EDGE,
            0,
            true,
        );
        let diff_clk12 = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::CLK1_INV,
            specials::OSERDES_SAME_EDGE,
            0,
            false,
        );
        assert_eq!(diff_clk12, diff_clk1.combine(&diff_clk2));
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::CLK1_INV, xlat_bit(!diff_clk1));
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::CLK2_INV, xlat_bit(!diff_clk2));

        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::FFO_SR_SYNC);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::FFT_SR_SYNC);
        ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::FFO_SR_SYNC,
            specials::OSERDES,
            0,
            false,
        )
        .assert_empty();
        let mut diff = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            OLOGIC::FFO_SR_SYNC,
            specials::OSERDES,
            0,
            true,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_SR_SYNC),
            true,
            false,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFT_SR_SYNC),
            true,
            false,
        );
        diff.assert_empty();

        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::FFO_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::FFT_INIT);
        let bit = xlat_bit_bi(
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFO_INIT,
                specials::OSERDES,
                0,
                false,
            ),
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFO_INIT,
                specials::OSERDES,
                0,
                true,
            ),
        );
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFO_INIT, bit);
        let bit = xlat_bit_bi(
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFT_INIT,
                specials::OSERDES,
                0,
                false,
            ),
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFT_INIT,
                specials::OSERDES,
                0,
                true,
            ),
        );
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFT_INIT, bit);

        let ffo_srval = [
            TileBit::new(c, 32 + i, [32, 31][i]).neg(),
            TileBit::new(c, 32 + i, [20, 43][i]).neg(),
            TileBit::new(c, 33 - i, [19, 44][i]).neg(),
        ];
        ctx.insert_bel_attr_bitvec(tcid, bslot, OLOGIC::FFO_SRVAL, ffo_srval);
        let fft_srval = [
            TileBit::new(c, 32 + i, [52, 11][i]).neg(),
            TileBit::new(c, 32 + i, [46, 17][i]).neg(),
            TileBit::new(c, 33 - i, [45, 18][i]).neg(),
        ];
        ctx.insert_bel_attr_bitvec(tcid, bslot, OLOGIC::FFT_SRVAL, fft_srval);
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFO_SRVAL, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFO_SRVAL, true),
        );
        assert_eq!(BTreeSet::from_iter(bits), BTreeSet::from(ffo_srval));
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFT_SRVAL, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFT_SRVAL, true),
        );
        assert_eq!(BTreeSet::from_iter(bits), BTreeSet::from(fft_srval));
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFO_SRVAL,
                specials::OSERDES,
                0,
                false,
            ),
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFO_SRVAL,
                specials::OSERDES,
                0,
                true,
            ),
        );
        assert_eq!(BTreeSet::from_iter(bits), BTreeSet::from(ffo_srval));
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFT_SRVAL,
                specials::OSERDES,
                0,
                false,
            ),
            ctx.get_diff_attr_special_bit_bi(
                tcid,
                bslot,
                OLOGIC::FFT_SRVAL,
                specials::OSERDES,
                0,
                true,
            ),
        );
        assert_eq!(BTreeSet::from_iter(bits), BTreeSet::from(fft_srval));

        ctx.collect_bel_attr(tcid, bslot, OLOGIC::FFO_SR_ENABLE);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::FFT_SR_ENABLE);

        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::MISR_ENABLE);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::MISR_ENABLE_FDBK);
        ctx.collect_bel_attr_default(
            tcid,
            bslot,
            OLOGIC::MISR_CLK_SELECT,
            enums::OLOGIC_MISR_CLK_SELECT::NONE,
        );
        if !matches!(tcid, tcls::IO_HP_PAIR | tcls::IO_HR_PAIR) {
            ctx.collect_bel_attr(tcid, bslot, OLOGIC::MISR_RESET);
        }
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::SERDES);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::SERDES_MODE);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::SELFHEAL);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::RANK3_USED);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::TBYTE_CTL);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::TBYTE_SRC);
        ctx.collect_bel_attr_subset(
            tcid,
            bslot,
            OLOGIC::TRISTATE_WIDTH,
            &[
                enums::OLOGIC_TRISTATE_WIDTH::_1,
                enums::OLOGIC_TRISTATE_WIDTH::_4,
            ],
        );

        let mut diffs = vec![];
        for (val, ratio) in [
            (enums::IO_DATA_WIDTH::_2, enums::OLOGIC_CLOCK_RATIO::_2),
            (enums::IO_DATA_WIDTH::_3, enums::OLOGIC_CLOCK_RATIO::_3),
            (enums::IO_DATA_WIDTH::_4, enums::OLOGIC_CLOCK_RATIO::_4),
            (enums::IO_DATA_WIDTH::_5, enums::OLOGIC_CLOCK_RATIO::_5),
            (enums::IO_DATA_WIDTH::_6, enums::OLOGIC_CLOCK_RATIO::_6),
            (enums::IO_DATA_WIDTH::_7, enums::OLOGIC_CLOCK_RATIO::_7_8),
            (enums::IO_DATA_WIDTH::_8, enums::OLOGIC_CLOCK_RATIO::_7_8),
        ] {
            diffs.push((
                val,
                ratio,
                ctx.get_diff_attr_special_val(
                    tcid,
                    bslot,
                    OLOGIC::DATA_WIDTH,
                    specials::OSERDES_SDR,
                    val,
                ),
            ));
        }
        for (val, ratio) in [
            (enums::IO_DATA_WIDTH::_4, enums::OLOGIC_CLOCK_RATIO::_2),
            (enums::IO_DATA_WIDTH::_6, enums::OLOGIC_CLOCK_RATIO::_3),
            (enums::IO_DATA_WIDTH::_8, enums::OLOGIC_CLOCK_RATIO::_4),
            (enums::IO_DATA_WIDTH::_10, enums::OLOGIC_CLOCK_RATIO::_5),
            (enums::IO_DATA_WIDTH::_14, enums::OLOGIC_CLOCK_RATIO::_7_8),
        ] {
            diffs.push((
                val,
                ratio,
                ctx.get_diff_attr_special_val(
                    tcid,
                    bslot,
                    OLOGIC::DATA_WIDTH,
                    specials::OSERDES_DDR,
                    val,
                ),
            ));
        }
        let mut diffs_width = vec![(enums::IO_DATA_WIDTH::NONE, Diff::default())];
        let mut diffs_ratio = vec![(enums::OLOGIC_CLOCK_RATIO::NONE, Diff::default())];
        for &(width, ratio, ref diff) in &diffs {
            let mut diff_ratio = Diff::default();
            let mut diff_width = Diff::default();
            for (&bit, &val) in &diff.bits {
                if diffs
                    .iter()
                    .any(|&(owidth, _, ref odiff)| width != owidth && odiff.bits.contains_key(&bit))
                {
                    diff_ratio.bits.insert(bit, val);
                } else {
                    diff_width.bits.insert(bit, val);
                }
            }
            diffs_width.push((width, diff_width));
            diffs_ratio.push((ratio, diff_ratio));
        }
        ctx.insert_bel_attr_enum(tcid, bslot, OLOGIC::DATA_WIDTH, xlat_enum_attr(diffs_width));
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            OLOGIC::CLOCK_RATIO,
            xlat_enum_attr(diffs_ratio),
        );

        let mut diff_sdr =
            ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::SERDES_SDR);
        let mut diff_ddr =
            ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::SERDES_DDR);
        diff_sdr.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_SR_ENABLE),
            true,
            false,
        );
        diff_ddr.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_SR_ENABLE),
            true,
            false,
        );
        let item = xlat_enum_attr(vec![
            (enums::OLOGIC_MUX_O::NONE, Diff::default()),
            (
                enums::OLOGIC_MUX_O::D1,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::D1),
            ),
            (enums::OLOGIC_MUX_O::SERDES_SDR, diff_sdr),
            (enums::OLOGIC_MUX_O::DDR, diff_ddr),
            (
                enums::OLOGIC_MUX_O::FF,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::FF),
            ),
            (
                enums::OLOGIC_MUX_O::DDR,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::DDR),
            ),
            (
                enums::OLOGIC_MUX_O::LATCH,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::LATCH),
            ),
        ]);
        ctx.insert_bel_attr_enum(tcid, bslot, OLOGIC::MUX_O, item);

        let mut diff_sdr =
            ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::SERDES_SDR);
        let mut diff_ddr =
            ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::SERDES_DDR);
        diff_sdr.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFT_SR_ENABLE),
            true,
            false,
        );
        diff_ddr.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFT_SR_ENABLE),
            true,
            false,
        );
        let item = xlat_enum_attr(vec![
            (enums::OLOGIC_MUX_T::NONE, Diff::default()),
            (
                enums::OLOGIC_MUX_T::T1,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::T1),
            ),
            (enums::OLOGIC_MUX_T::SERDES_SDR, diff_sdr),
            (enums::OLOGIC_MUX_T::DDR, diff_ddr),
            (
                enums::OLOGIC_MUX_T::FF,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::FF),
            ),
            (
                enums::OLOGIC_MUX_T::DDR,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::DDR),
            ),
            (
                enums::OLOGIC_MUX_T::LATCH,
                ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_T, enums::OLOGIC_MUX_T::LATCH),
            ),
        ]);
        ctx.insert_bel_attr_enum(tcid, bslot, OLOGIC::MUX_T, item);

        let mut present_ologic = ctx.get_diff_bel_special(tcid, bslot, specials::OLOGIC);
        present_ologic.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::RANK3_USED),
            false,
            true,
        );
        present_ologic.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, OLOGIC::MUX_T),
            enums::OLOGIC_MUX_T::T1,
            enums::OLOGIC_MUX_T::NONE,
        );
        present_ologic.assert_empty();
        let mut present_oserdes = ctx.get_diff_bel_special(tcid, bslot, specials::OSERDES);
        present_oserdes.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, OLOGIC::FFO_SRVAL),
            0,
            7,
        );
        present_oserdes.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, OLOGIC::FFT_SRVAL),
            0,
            7,
        );
        present_oserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_INIT),
            false,
            true,
        );
        present_oserdes.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFT_INIT),
            false,
            true,
        );
        present_oserdes.assert_empty();
    }
    for tcid in [tcls::IO_HR_PAIR, tcls::IO_HP_PAIR] {
        if !ctx.has_tcls(tcid) {
            continue;
        }
        let mut diff = ctx.get_diff_attr_bool(tcid, bslots::OLOGIC[0], OLOGIC::MISR_RESET);
        let diff1 = diff.split_bits_by(|bit| bit.rect.to_idx() > 0);
        ctx.insert_bel_attr_bool(tcid, bslots::OLOGIC[0], OLOGIC::MISR_RESET, xlat_bit(diff));
        ctx.insert_bel_attr_bool(tcid, bslots::OLOGIC[1], OLOGIC::MISR_RESET, xlat_bit(diff1));
    }
}

fn collect_fuzzers_iodelay(ctx: &mut CollectorCtx) {
    for (tcid, bslot, is_hp) in [
        (tcls::IO_HR_PAIR, bslots::IDELAY[0], false),
        (tcls::IO_HR_PAIR, bslots::IDELAY[1], false),
        (tcls::IO_HR_S, bslots::IDELAY[0], false),
        (tcls::IO_HR_N, bslots::IDELAY[0], false),
        (tcls::IO_HP_PAIR, bslots::IDELAY[0], true),
        (tcls::IO_HP_PAIR, bslots::IDELAY[1], true),
        (tcls::IO_HP_S, bslots::IDELAY[0], true),
        (tcls::IO_HP_N, bslots::IDELAY[0], true),
    ] {
        if !ctx.has_tcls(tcid) {
            continue;
        }
        ctx.collect_bel_attr(tcid, bslot, IDELAY::ENABLE);
        ctx.collect_bel_input_inv_bi(tcid, bslot, IDELAY::C);
        ctx.collect_bel_input_inv_bi(tcid, bslot, IDELAY::DATAIN);
        ctx.collect_bel_attr_bi(tcid, bslot, IDELAY::IDATAIN_INV);
        ctx.collect_bel_attr_bi(tcid, bslot, IDELAY::HIGH_PERFORMANCE_MODE);
        ctx.collect_bel_attr_bi(tcid, bslot, IDELAY::CINVCTRL_SEL);
        ctx.collect_bel_attr_bi(tcid, bslot, IDELAY::PIPE_SEL);
        ctx.collect_bel_attr(tcid, bslot, IDELAY::DELAY_SRC);
        ctx.collect_bel_attr(tcid, bslot, IDELAY::DELAY_TYPE);
        let mut diffs_t = vec![];
        let mut diffs_f = vec![];
        for diff in ctx.get_diffs_attr_bits(tcid, bslot, IDELAY::IDELAY_VALUE_INIT, 5) {
            let mut diff_t = Diff::default();
            let mut diff_f = Diff::default();
            for (k, v) in diff.bits {
                if v {
                    diff_t.bits.insert(k, v);
                } else {
                    diff_f.bits.insert(k, v);
                }
            }
            diffs_t.push(diff_t);
            diffs_f.push(diff_f);
        }
        ctx.insert_bel_attr_bitvec(tcid, bslot, IDELAY::IDELAY_VALUE_INIT, xlat_bitvec(diffs_t));
        ctx.insert_bel_attr_bitvec(tcid, bslot, IDELAY::IDELAY_VALUE_CUR, xlat_bitvec(diffs_f));
        if is_hp {
            ctx.collect_bel_attr_bi(tcid, bslot, IDELAY::FINEDELAY);
        }
    }
    for (tcid, bslot) in [
        (tcls::IO_HP_PAIR, bslots::ODELAY[0]),
        (tcls::IO_HP_PAIR, bslots::ODELAY[1]),
        (tcls::IO_HP_S, bslots::ODELAY[0]),
        (tcls::IO_HP_N, bslots::ODELAY[0]),
    ] {
        if !ctx.has_tcls(tcid) {
            continue;
        }
        ctx.collect_bel_input_inv_bi(tcid, bslot, ODELAY::C);
        ctx.collect_bel_attr_bi(tcid, bslot, ODELAY::ODATAIN_INV);
        ctx.collect_bel_attr_bi(tcid, bslot, ODELAY::HIGH_PERFORMANCE_MODE);
        ctx.collect_bel_attr_bi(tcid, bslot, ODELAY::CINVCTRL_SEL);
        ctx.collect_bel_attr_bi(tcid, bslot, ODELAY::PIPE_SEL);
        ctx.collect_bel_attr(tcid, bslot, ODELAY::DELAY_SRC);

        let en = xlat_bit(ctx.get_diff_attr_val(
            tcid,
            bslot,
            ODELAY::DELAY_TYPE,
            enums::IODELAY_DELAY_TYPE::FIXED,
        ));
        let mut diff_var = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ODELAY::DELAY_TYPE,
            enums::IODELAY_DELAY_TYPE::VARIABLE,
        );
        diff_var.apply_bit_diff(en, true, false);
        let mut diff_vl = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ODELAY::DELAY_TYPE,
            enums::IODELAY_DELAY_TYPE::VAR_LOAD,
        );
        diff_vl.apply_bit_diff(en, true, false);
        ctx.insert_bel_attr_bool(tcid, bslot, ODELAY::ENABLE, en);
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            ODELAY::DELAY_TYPE,
            xlat_enum_attr(vec![
                (enums::IODELAY_DELAY_TYPE::FIXED, Diff::default()),
                (enums::IODELAY_DELAY_TYPE::VARIABLE, diff_var),
                (enums::IODELAY_DELAY_TYPE::VAR_LOAD, diff_vl),
            ]),
        );

        let mut diffs_t = vec![];
        let mut diffs_f = vec![];
        for diff in ctx.get_diffs_attr_bits(tcid, bslot, ODELAY::ODELAY_VALUE_INIT, 5) {
            let mut diff_t = Diff::default();
            let mut diff_f = Diff::default();
            for (k, v) in diff.bits {
                if v {
                    diff_t.bits.insert(k, v);
                } else {
                    diff_f.bits.insert(k, v);
                }
            }
            diffs_t.push(diff_t);
            diffs_f.push(diff_f);
        }
        ctx.insert_bel_attr_bitvec(tcid, bslot, ODELAY::ODELAY_VALUE_INIT, xlat_bitvec(diffs_t));
        ctx.insert_bel_attr_bitvec(tcid, bslot, ODELAY::ODELAY_VALUE_CUR, xlat_bitvec(diffs_f));
        ctx.collect_bel_attr_bi(tcid, bslot, ODELAY::FINEDELAY);
    }
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx) {
    collect_fuzzers_routing(ctx);
    collect_fuzzers_ilogic(ctx);
    collect_fuzzers_ologic(ctx);
    collect_fuzzers_iodelay(ctx);
}
