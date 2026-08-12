use std::collections::BTreeSet;

use prjcombine_entity::EntityId;
use prjcombine_interconnect::db::WireSlotIdExt;
use prjcombine_re_collector::diff::{
    Diff, OcdMode, extract_bitvec_val_part, xlat_bit, xlat_bit_bi, xlat_bit_wide_bi, xlat_bitvec,
    xlat_enum_attr, xlat_enum_attr_ocd,
};
use prjcombine_re_hammer::Session;
use prjcombine_types::{bitrect::TileBit, bits};
use prjcombine_virtex4::defs::{
    bcls::{ILOGIC, IODELAY, OLOGIC},
    bslots, devdata, enums,
    virtex6::{tcls, wires},
};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::{
        fbuild::{FuzzBuilderBase, FuzzCtx},
        int::{BaseIntPip, FuzzIntPip},
        props::mutex::{WireMutexExclusive, WireMutexShared},
    },
    virtex4::specials,
    virtex5::io::HclkIoi,
};

fn add_fuzzers_routing<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::IO);
    for c in 0..2 {
        for w in [
            wires::IMUX_IOI_ICLK,
            wires::IMUX_IOI_OCLK,
            wires::IMUX_IOI_OCLKDIV,
        ] {
            let dst_a = w[0].cell(c);
            let dst_b = w[1].cell(c);
            let mux = &backend.edev.db_index.tile_classes[tcls::IO].muxes[&dst_a];
            for &src in mux.src.keys() {
                ctx.build()
                    .prop(WireMutexExclusive::new(dst_a))
                    .prop(WireMutexExclusive::new(dst_b))
                    .prop(WireMutexShared::new(src.tw))
                    .prop(BaseIntPip::new(dst_b, src.tw))
                    .test_routing(dst_a, src)
                    .prop(FuzzIntPip::new(dst_a, src.tw))
                    .commit();
                ctx.build()
                    .prop(WireMutexExclusive::new(dst_b))
                    .prop(WireMutexShared::new(src.tw))
                    .test_routing(dst_b, src)
                    .prop(FuzzIntPip::new(dst_b, src.tw))
                    .commit();
            }
        }
    }
}

fn add_fuzzers_ilogic<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::IO);
    for i in 0..2 {
        let mut bctx = ctx.bel(bslots::ILOGIC[i]);

        bctx.build()
            .null_bits()
            .test_bel_special(specials::ILOGIC)
            .mode("ILOGICE1")
            .commit();
        bctx.build()
            .test_bel_special(specials::ISERDES)
            .mode("ISERDESE1")
            .commit();

        bctx.mode("ISERDESE1").pin("D").test_bel_attr_bool_rename(
            "DINV",
            ILOGIC::D_INV,
            "D",
            "D_B",
        );
        bctx.mode("ISERDESE1").pin("CLK").test_bel_attr_bool_rename(
            "CLKINV",
            ILOGIC::CLK_INV,
            "CLK",
            "CLK_B",
        );
        bctx.mode("ISERDESE1")
            .attr("DYN_CLKDIV_INV_EN", "FALSE")
            .test_bel_input_inv_auto(ILOGIC::CLKDIV);
        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::DYN_CLK_INV_EN, "FALSE", "TRUE");
        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::DYN_OCLK_INV_EN, "FALSE", "TRUE");
        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::DYN_CLKDIV_INV_EN, "FALSE", "TRUE");
        for (dr, val, spec) in [
            ("SDR", "OCLK", specials::ISERDES_OCLK_SDR),
            ("SDR", "OCLK_B", specials::ISERDES_OCLK_B_SDR),
            ("DDR", "OCLK", specials::ISERDES_OCLK_DDR),
            ("DDR", "OCLK_B", specials::ISERDES_OCLK_B_DDR),
        ] {
            bctx.mode("ISERDESE1")
                .attr("DATA_RATE", dr)
                .attr("OVERSAMPLE", "FALSE")
                .attr("DYN_OCLK_INV_EN", "FALSE")
                .attr("INTERFACE_TYPE", "")
                .pin("OCLK")
                .test_bel_special(spec)
                .attr("OCLKINV", val)
                .commit();
        }

        bctx.mode("ILOGICE1")
            .attr("IFFTYPE", "#FF")
            .pin("SR")
            .test_bel_attr_bits(ILOGIC::FFI_SR_ENABLE)
            .attr("SRUSED", "0")
            .commit();
        bctx.mode("ILOGICE1")
            .attr("IFFTYPE", "#FF")
            .pin("REV")
            .test_bel_attr_bits(ILOGIC::FFI_REV_ENABLE)
            .attr("REVUSED", "0")
            .commit();
        bctx.mode("ISERDESE1")
            .attr("DATA_WIDTH", "2")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_bool_auto(ILOGIC::SERDES, "FALSE", "TRUE");
        bctx.mode("ISERDESE1")
            .test_bel_attr_auto(ILOGIC::SERDES_MODE);
        bctx.mode("ISERDESE1")
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
                ],
            );
        bctx.mode("ISERDESE1").test_bel_attr_auto(ILOGIC::NUM_CE);

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
            bctx.mode("ISERDESE1")
                .test_bel_attr_bool_rename(aname, attr, "0", "1");
        }

        bctx.mode("ILOGICE1")
            .attr("IFFTYPE", "#FF")
            .test_bel_attr_bool_rename("SRTYPE", ILOGIC::FFI_SR_SYNC, "ASYNC", "SYNC");
        bctx.mode("ISERDESE1").test_bel_attr_bool_special_rename(
            "SRTYPE",
            ILOGIC::FFI_SR_SYNC,
            specials::ISERDES,
            "ASYNC",
            "SYNC",
        );

        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_CE, MultiValue::Bin);
        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_BITSLIPCNT, MultiValue::Bin);
        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_BITSLIP, MultiValue::Bin);
        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_RANK1_PARTIAL, MultiValue::Bin);
        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_RANK2, MultiValue::Bin);
        bctx.mode("ISERDESE1")
            .attr("DATA_RATE", "SDR")
            .test_bel_attr_multi(ILOGIC::INIT_RANK3, MultiValue::Bin);

        bctx.mode("ISERDESE1")
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
            bctx.mode("ISERDESE1")
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
            bctx.mode("ISERDESE1")
                .test_bel_special(spec)
                .attr("IOBDELAY", val)
                .commit();
        }

        bctx.mode("ILOGICE1")
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
        bctx.mode("ILOGICE1")
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
        bctx.mode("ILOGICE1")
            .attr("IDELMUX", "1")
            .attr("DINV", "")
            .pin("D")
            .pin("DDLY")
            .pin("O")
            .pin("TFB")
            .pin("OFB")
            .test_bel_attr_bool_rename("IMUX", ILOGIC::I_TSBYPASS_ENABLE, "1", "0");
        bctx.mode("ILOGICE1")
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
        bctx.mode("ILOGICE1")
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
        bctx.mode("ILOGICE1")
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

        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::D_EMU, "FALSE", "TRUE");
        bctx.mode("ISERDESE1")
            .test_bel_attr_auto(ILOGIC::D_EMU_OPTION);
        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::RANK12_DLY, "FALSE", "TRUE");
        bctx.mode("ISERDESE1")
            .test_bel_attr_bool_auto(ILOGIC::RANK23_DLY, "FALSE", "TRUE");

        for (val, vname) in [
            (enums::ILOGIC_INTERFACE_TYPE::NETWORKING, "NETWORKING"),
            (enums::ILOGIC_INTERFACE_TYPE::MEMORY, "MEMORY"),
            (enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6, "MEMORY_DDR3"),
            (enums::ILOGIC_INTERFACE_TYPE::OVERSAMPLE, "OVERSAMPLE"),
        ] {
            bctx.mode("ISERDESE1")
                .attr("OVERSAMPLE", "FALSE")
                .test_bel_attr_val(ILOGIC::INTERFACE_TYPE, val)
                .attr("INTERFACE_TYPE", vname)
                .commit();
        }
        bctx.mode("ISERDESE1")
            .attr("OVERSAMPLE", "FALSE")
            .test_bel_attr_special(ILOGIC::INTERFACE_TYPE, specials::ILOGIC_MEMORY_QDR)
            .attr("INTERFACE_TYPE", "MEMORY_QDR")
            .commit();
        bctx.mode("ISERDESE1")
            .attr("INIT_BITSLIPCNT", "1111")
            .attr("INIT_RANK1_PARTIAL", "11111")
            .attr("INIT_RANK2", "111111")
            .attr("INIT_RANK3", "111111")
            .attr("INIT_CE", "11")
            .test_bel_attr_auto(ILOGIC::DATA_RATE);
        bctx.mode("ISERDESE1")
            .test_bel_attr_auto(ILOGIC::DDR_CLK_EDGE);
        bctx.mode("ILOGICE1")
            .attr("IFFTYPE", "DDR")
            .test_bel_attr_auto(ILOGIC::DDR_CLK_EDGE);
        for (spec, val) in [
            (specials::ILOGIC_IFFTYPE_LATCH, "#LATCH"),
            (specials::ILOGIC_IFFTYPE_FF, "#FF"),
            (specials::ILOGIC_IFFTYPE_DDR, "DDR"),
        ] {
            bctx.mode("ILOGICE1")
                .test_bel_special(spec)
                .attr("IFFTYPE", val)
                .commit();
        }
    }
}

fn add_fuzzers_ologic<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::IO);
    for i in 0..2 {
        let mut bctx = ctx.bel(bslots::OLOGIC[i]);

        bctx.build()
            .test_bel_special(specials::OLOGIC)
            .mode("OLOGICE1")
            .commit();
        bctx.build()
            .test_bel_special(specials::OSERDES)
            .mode("OSERDESE1")
            .commit();

        for pin in [
            OLOGIC::D1,
            OLOGIC::D2,
            OLOGIC::D3,
            OLOGIC::D4,
            OLOGIC::D5,
            OLOGIC::D6,
            OLOGIC::T2,
            OLOGIC::T3,
            OLOGIC::T4,
            OLOGIC::CLKDIV,
            OLOGIC::CLKPERF,
        ] {
            bctx.mode("OSERDESE1").test_bel_input_inv_auto(pin);
        }
        bctx.mode("OLOGICE1")
            .attr("TMUX", "T1")
            .attr("T1USED", "0")
            .pin("TQ")
            .test_bel_input_inv_auto(OLOGIC::T1);
        bctx.mode("OSERDESE1")
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
        bctx.mode("OSERDESE1")
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

        bctx.mode("OLOGICE1")
            .attr("OUTFFTYPE", "#FF")
            .test_bel_attr_bool_rename("SRTYPE_OQ", OLOGIC::FFO_SR_SYNC, "ASYNC", "SYNC");
        bctx.mode("OLOGICE1")
            .attr("TFFTYPE", "#FF")
            .test_bel_attr_bool_rename("SRTYPE_TQ", OLOGIC::FFT_SR_SYNC, "ASYNC", "SYNC");
        bctx.mode("OSERDESE1").test_bel_attr_bool_special_rename(
            "SRTYPE",
            OLOGIC::FFO_SR_SYNC,
            specials::OSERDES,
            "ASYNC",
            "SYNC",
        );

        bctx.mode("OLOGICE1")
            .test_bel_attr_bool_rename("INIT_OQ", OLOGIC::FFO_INIT, "0", "1");
        bctx.mode("OLOGICE1")
            .test_bel_attr_bool_rename("INIT_TQ", OLOGIC::FFT_INIT, "0", "1");
        bctx.mode("OSERDESE1").test_bel_attr_bool_special_rename(
            "INIT_OQ",
            OLOGIC::FFO_INIT,
            specials::OSERDES,
            "0",
            "1",
        );
        bctx.mode("OSERDESE1").test_bel_attr_bool_special_rename(
            "INIT_TQ",
            OLOGIC::FFT_INIT,
            specials::OSERDES,
            "0",
            "1",
        );
        bctx.mode("OLOGICE1")
            .test_bel_attr_bool_rename("SRVAL_OQ", OLOGIC::FFO_SRVAL, "0", "1");
        bctx.mode("OLOGICE1")
            .test_bel_attr_bool_rename("SRVAL_TQ", OLOGIC::FFT_SRVAL, "0", "1");
        bctx.mode("OSERDESE1").test_bel_attr_bool_special_rename(
            "SRVAL_OQ",
            OLOGIC::FFO_SRVAL,
            specials::OSERDES,
            "0",
            "1",
        );
        bctx.mode("OSERDESE1").test_bel_attr_bool_special_rename(
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
            bctx.mode("OLOGICE1")
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
        for attr in ["OREVUSED", "TREVUSED", "OCEUSED", "TCEUSED"] {
            bctx.mode("OLOGICE1")
                .null_bits()
                .attr("OUTFFTYPE", "#FF")
                .attr("TFFTYPE", "#FF")
                .pin("OCE")
                .pin("TCE")
                .pin("REV")
                .pin("SR")
                .test_bel_special(specials::OLOGIC_REVCEUSED)
                .attr(attr, "0")
                .commit();
        }

        for (val, vname) in [
            (enums::OLOGIC_MUX_O::FF, "#FF"),
            (enums::OLOGIC_MUX_O::LATCH, "#LATCH"),
            (enums::OLOGIC_MUX_O::DDR, "DDR"),
        ] {
            bctx.mode("OLOGICE1")
                .attr("TFFTYPE", "")
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
            bctx.mode("OLOGICE1")
                .attr("OUTFFTYPE", "")
                .pin("TQ")
                .test_bel_attr_val(OLOGIC::MUX_T, val)
                .attr("TFFTYPE", vname)
                .commit();
        }

        for (val, vname) in [
            (enums::OLOGIC_MUX_O::SERDES_SDR, "SDR"),
            (enums::OLOGIC_MUX_O::SERDES_DDR, "DDR"),
        ] {
            bctx.mode("OSERDESE1")
                .test_bel_attr_val(OLOGIC::MUX_O, val)
                .attr("DATA_RATE_OQ", vname)
                .commit();
        }
        for (val, vname) in [
            (enums::OLOGIC_MUX_T::T1, "BUF"),
            (enums::OLOGIC_MUX_T::SERDES_SDR, "SDR"),
            (enums::OLOGIC_MUX_T::SERDES_DDR, "DDR"),
        ] {
            bctx.mode("OSERDESE1")
                .attr("T1INV", "T1")
                .pin("T1")
                .test_bel_attr_val(OLOGIC::MUX_T, val)
                .attr("DATA_RATE_TQ", vname)
                .commit();
        }

        bctx.mode("OLOGICE1")
            .global("ENABLEMISR", "Y")
            .test_bel_attr_bool_auto(OLOGIC::MISR_ENABLE, "FALSE", "TRUE");
        bctx.mode("OLOGICE1")
            .global("ENABLEMISR", "Y")
            .test_bel_attr_bool_auto(OLOGIC::MISR_ENABLE_FDBK, "FALSE", "TRUE");
        bctx.mode("OLOGICE1")
            .global("ENABLEMISR", "Y")
            .test_bel_attr_auto_default(
                OLOGIC::MISR_CLK_SELECT,
                enums::OLOGIC_MISR_CLK_SELECT::NONE,
            );

        bctx.mode("OSERDESE1")
            .test_bel_attr_bool_auto(OLOGIC::SERDES, "FALSE", "TRUE");
        bctx.mode("OSERDESE1")
            .test_bel_attr_auto(OLOGIC::SERDES_MODE);
        bctx.mode("OSERDESE1")
            .test_bel_attr_bool_auto(OLOGIC::SELFHEAL, "FALSE", "TRUE");
        bctx.mode("OSERDESE1")
            .attr("DATA_RATE_OQ", "SDR")
            .test_bel_attr_auto(OLOGIC::INTERFACE_TYPE);
        bctx.mode("OSERDESE1").test_bel_attr_subset_auto(
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
            bctx.mode("OSERDESE1")
                .attr("DATA_RATE_OQ", "SDR")
                .attr("INTERFACE_TYPE", "DEFAULT")
                .test_bel_attr_special_val(OLOGIC::DATA_WIDTH, specials::OSERDES_SDR, val)
                .attr("DATA_WIDTH", vname)
                .commit();
        }
        for (val, vname) in [
            (enums::IO_DATA_WIDTH::_4, "4"),
            (enums::IO_DATA_WIDTH::_6, "6"),
            (enums::IO_DATA_WIDTH::_8, "8"),
            (enums::IO_DATA_WIDTH::_10, "10"),
        ] {
            bctx.mode("OSERDESE1")
                .attr("DATA_RATE_OQ", "DDR")
                .attr("INTERFACE_TYPE", "DEFAULT")
                .test_bel_attr_special_val(OLOGIC::DATA_WIDTH, specials::OSERDES_DDR, val)
                .attr("DATA_WIDTH", vname)
                .commit();
        }
        bctx.mode("OSERDESE1")
            .test_bel_attr_bool_auto(OLOGIC::WC_DELAY, "0", "1");
        bctx.mode("OSERDESE1")
            .test_bel_attr_bool_auto(OLOGIC::DDR3_DATA, "0", "1");
        bctx.mode("OSERDESE1")
            .test_bel_attr_bool_auto(OLOGIC::ODELAY_USED, "0", "1");
        bctx.mode("OSERDESE1")
            .test_bel_attr_bits(OLOGIC::FFO_RANK1_INIT)
            .multi_attr("INIT_ORANK1", MultiValue::BinRev, 6);
        bctx.mode("OSERDESE1")
            .test_bel_attr_bits(OLOGIC::FFO_RANK2_INIT)
            .multi_attr("INIT_ORANK2_PARTIAL", MultiValue::BinRev, 4);
        bctx.mode("OSERDESE1")
            .test_bel_attr_bits(OLOGIC::FFT_RANK1_INIT)
            .multi_attr("INIT_TRANK1", MultiValue::BinRev, 4);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_LOADCNT, MultiValue::Bin);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_FIFO_ADDR, MultiValue::Bin);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_FIFO_RESET, MultiValue::Bin);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_DLY_CNT, MultiValue::Bin);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_PIPE_DATA0, MultiValue::Bin);
        bctx.mode("OSERDESE1")
            .test_bel_attr_multi(OLOGIC::INIT_PIPE_DATA1, MultiValue::Bin);
    }
    {
        let mut ctx = FuzzCtx::new_null(session, backend);
        ctx.build()
            .extra_tiles_by_bel_attr_bits(bslots::OLOGIC[0], OLOGIC::MISR_RESET)
            .global("ENABLEMISR", "Y")
            .test_global_special(specials::MISR_RESET)
            .global_diff("MISRRESET", "N", "Y")
            .commit();
    }
}

fn add_fuzzers_iodelay<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::IO);
    for i in 0..2 {
        let mut bctx = ctx.bel(bslots::IODELAY[i]);
        let bel_other = bslots::IODELAY[i ^ 1];

        bctx.build()
            .null_bits()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .test_bel_special(specials::PRESENT)
            .mode("IODELAYE1")
            .commit();
        for pin in [IODELAY::C, IODELAY::DATAIN] {
            bctx.mode("IODELAYE1")
                .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
                .test_bel_input_inv_auto(pin);
        }
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .pin("IDATAIN")
            .test_bel_attr_bool_rename("IDATAININV", IODELAY::IDATAIN_INV, "IDATAIN", "IDATAIN_B");
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .test_bel_attr_bool_auto(IODELAY::CINVCTRL_SEL, "FALSE", "TRUE");
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .test_bel_attr_bool_auto(IODELAY::HIGH_PERFORMANCE_MODE, "FALSE", "TRUE");
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "FIXED")
            .test_bel_attr_subset_auto(
                IODELAY::DELAY_SRC,
                &[
                    enums::IODELAY_DELAY_SRC::I,
                    enums::IODELAY_DELAY_SRC::O,
                    enums::IODELAY_DELAY_SRC::IO,
                    enums::IODELAY_DELAY_SRC::DATAIN,
                    enums::IODELAY_DELAY_SRC::CLKIN,
                ],
            );
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "FIXED")
            .test_bel_attr_val(IODELAY::DELAY_SRC, enums::IODELAY_DELAY_SRC::DELAYCHAIN_OSC)
            .attr("DELAY_SRC", "I")
            .attr("DELAYCHAIN_OSC", "TRUE")
            .commit();
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .attr("DELAY_SRC", "IO")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "FIXED")
            .test_bel_attr_bits(IODELAY::IDELAY_VALUE_INIT)
            .multi_attr("IDELAY_VALUE", MultiValue::Dec(0), 5);
        bctx.mode("IODELAYE1")
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .attr("DELAY_SRC", "IO")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "FIXED")
            .test_bel_attr_bits(IODELAY::ALT_DELAY_VALUE)
            .multi_attr("ODELAY_VALUE", MultiValue::Dec(0), 5);
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "DEFAULT")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_DEFAULT)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "DEFAULT")
            .attr("DELAY_SRC", "I")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_FIXED)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("DELAY_SRC", "I")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "VARIABLE")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_VARIABLE)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "VARIABLE")
            .attr("DELAY_SRC", "I")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "VAR_LOADABLE")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_VAR_LOADABLE)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "VAR_LOADABLE")
            .attr("DELAY_SRC", "I")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_O_FIXED)
            .mode("IODELAYE1")
            .attr("ODELAY_TYPE", "FIXED")
            .attr("DELAY_SRC", "O")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_O_VARIABLE)
            .mode("IODELAYE1")
            .attr("ODELAY_TYPE", "VARIABLE")
            .attr("DELAY_SRC", "O")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_O_VAR_LOADABLE)
            .mode("IODELAYE1")
            .attr("ODELAY_TYPE", "VAR_LOADABLE")
            .attr("DELAY_SRC", "O")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_IO_FIXED)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "FIXED")
            .attr("DELAY_SRC", "IO")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_VARIABLE_O_FIXED)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "VARIABLE")
            .attr("ODELAY_TYPE", "FIXED")
            .attr("DELAY_SRC", "IO")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_I_FIXED_O_VARIABLE)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "FIXED")
            .attr("ODELAY_TYPE", "VARIABLE")
            .attr("DELAY_SRC", "IO")
            .commit();
        bctx.build()
            .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
            .bel_mode(bel_other, "IODELAYE1")
            .bel_attr(bel_other, "IDELAY_TYPE", "FIXED")
            .bel_attr(bel_other, "DELAY_SRC", "I")
            .test_bel_special(specials::IODELAY_IO_VAR_LOADABLE)
            .mode("IODELAYE1")
            .attr("IDELAY_TYPE", "VAR_LOADABLE")
            .attr("ODELAY_TYPE", "VAR_LOADABLE")
            .attr("DELAY_SRC", "IO")
            .commit();
    }
}

pub fn add_fuzzers<'a>(
    session: &mut Session<'a, IseBackend<'a>>,
    backend: &'a IseBackend<'a>,
    devdata_only: bool,
) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::IO);
    if devdata_only {
        for i in 0..2 {
            let mut bctx = ctx.bel(bslots::IODELAY[i]);
            let bel_other = bslots::IODELAY[i ^ 1];
            bctx.build()
                .related_tile_mutex(HclkIoi, "IDELAYCTRL", "USE")
                .bel_mode(bel_other, "IODELAYE1")
                .bel_attr(bel_other, "IDELAY_TYPE", "DEFAULT")
                .bel_attr(bel_other, "DELAY_SRC", "I")
                .test_bel_special(specials::IODELAY_I_DEFAULT)
                .mode("IODELAYE1")
                .attr("IDELAY_TYPE", "DEFAULT")
                .attr("DELAY_SRC", "I")
                .commit();
        }
        return;
    }

    add_fuzzers_routing(session, backend);
    add_fuzzers_ilogic(session, backend);
    add_fuzzers_ologic(session, backend);
    add_fuzzers_iodelay(session, backend);
}

fn collect_fuzzers_routing(ctx: &mut CollectorCtx) {
    let tcid = tcls::IO;
    for c in 0..2 {
        for w in [
            wires::IMUX_IOI_ICLK,
            wires::IMUX_IOI_OCLK,
            wires::IMUX_IOI_OCLKDIV,
        ] {
            ctx.collect_mux(tcid, w[0].cell(c));
            ctx.collect_mux(tcid, w[1].cell(c));
        }
    }
}

fn collect_fuzzers_ilogic(ctx: &mut CollectorCtx) {
    let tcid = tcls::IO;

    for i in 0..2 {
        let bslot = bslots::ILOGIC[i];

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::D_INV);
        ctx.collect_bel_input_inv_bi(tcid, bslot, ILOGIC::CLKDIV);
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::CLK_INV, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::CLK_INV, true),
        );
        ctx.insert_bel_attr_bitvec(tcid, bslot, ILOGIC::CLK_INV, bits);

        let diff1 = ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES_OCLK_DDR);
        let diff2 = ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES_OCLK_B_DDR);
        ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES_OCLK_B_SDR)
            .assert_empty();
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES_OCLK_SDR);
        diff = diff.combine(&!&diff1);
        diff = diff.combine(&!&diff2);
        diff.assert_empty();
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::OCLK1_INV, xlat_bit(!diff1));
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::OCLK2_INV, xlat_bit(!diff2));

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::DYN_CLK_INV_EN);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::DYN_CLKDIV_INV_EN);
        let bits = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::DYN_OCLK_INV_EN, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, ILOGIC::DYN_OCLK_INV_EN, true),
        );
        ctx.insert_bel_attr_bitvec(tcid, bslot, ILOGIC::DYN_OCLK_INV_EN, bits);

        ctx.collect_bel_attr(tcid, bslot, ILOGIC::FFI_SR_ENABLE);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::FFI_REV_ENABLE);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::SERDES);
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
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_RANK1_PARTIAL);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_RANK2);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_RANK3);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_BITSLIP);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_BITSLIPCNT);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::INIT_CE);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI_SR_SYNC);
        ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            ILOGIC::FFI_SR_SYNC,
            specials::ISERDES,
            0,
            false,
        )
        .assert_empty();
        let mut diff = ctx.get_diff_attr_special_bit_bi(
            tcid,
            bslot,
            ILOGIC::FFI_SR_SYNC,
            specials::ISERDES,
            0,
            true,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_SR_SYNC),
            true,
            false,
        );
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::BITSLIP_SR_SYNC, xlat_bit(diff));

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI1_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI2_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI3_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI4_INIT);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI1_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI2_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI3_SRVAL);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::FFI4_SRVAL);

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
            enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6,
        );
        let diff_os = ctx.get_diff_attr_val(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            enums::ILOGIC_INTERFACE_TYPE::OVERSAMPLE,
        );
        let bitslip_en = diff_net.combine(&!&diff_qdr);
        let diff_ddr3 = diff_ddr3.combine(&!&bitslip_en);
        let diff_os = diff_os.combine(&!&bitslip_en);
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::BITSLIP_ENABLE, xlat_bit(bitslip_en));
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            ILOGIC::INTERFACE_TYPE,
            xlat_enum_attr(vec![
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY, diff_mem),
                (enums::ILOGIC_INTERFACE_TYPE::NETWORKING, diff_qdr),
                (enums::ILOGIC_INTERFACE_TYPE::MEMORY_DDR3_V6, diff_ddr3),
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
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::FFI_LATCH, xlat_bit(!diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::ILOGIC_IFFTYPE_DDR);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::INTERFACE_TYPE),
            enums::ILOGIC_INTERFACE_TYPE::NETWORKING,
            enums::ILOGIC_INTERFACE_TYPE::MEMORY,
        );
        ctx.insert_bel_attr_bool(tcid, bslot, ILOGIC::FFI_LATCH, xlat_bit(!diff));

        let mut diffs = vec![];
        for val in [enums::IO_DATA_RATE::SDR, enums::IO_DATA_RATE::DDR] {
            let mut diff = ctx.get_diff_attr_val(tcid, bslot, ILOGIC::DATA_RATE, val);
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_SR_ENABLE),
                true,
                false,
            );
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, ILOGIC::FFI_LATCH),
                false,
                true,
            );
            diffs.push((val, diff));
        }
        ctx.insert_bel_attr_enum(tcid, bslot, ILOGIC::DATA_RATE, xlat_enum_attr(diffs));

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

        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::D_EMU);
        ctx.collect_bel_attr(tcid, bslot, ILOGIC::D_EMU_OPTION);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::RANK12_DLY);
        ctx.collect_bel_attr_bi(tcid, bslot, ILOGIC::RANK23_DLY);

        let mut present_iserdes = ctx.get_diff_bel_special(tcid, bslot, specials::ISERDES);
        present_iserdes.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, ILOGIC::MUX_TSBYPASS),
            enums::ILOGIC_MUX_TSBYPASS::GND,
            enums::ILOGIC_MUX_TSBYPASS::T,
        );
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

        ctx.insert_bel_attr_bool(
            tcid,
            bslot,
            ILOGIC::READBACK_I,
            [TileBit::new(0, 26, 61), TileBit::new(1, 27, 2)][i].pos(),
        );
    }
}

fn collect_fuzzers_ologic(ctx: &mut CollectorCtx) {
    let tcid = tcls::IO;

    for i in 0..2 {
        let bslot = bslots::OLOGIC[i];

        for pin in [
            OLOGIC::D1,
            OLOGIC::D2,
            OLOGIC::D3,
            OLOGIC::D4,
            OLOGIC::D5,
            OLOGIC::D6,
            OLOGIC::T2,
            OLOGIC::T3,
            OLOGIC::T4,
            OLOGIC::CLKPERF,
            OLOGIC::CLKDIV,
        ] {
            ctx.collect_bel_input_inv_bi(tcid, bslot, pin);
        }

        let diff0 = ctx.get_diff_bel_input_inv(tcid, bslot, OLOGIC::T1, false);
        let diff1 = ctx.get_diff_bel_input_inv(tcid, bslot, OLOGIC::T1, true);
        let (diff0, diff1, _) = Diff::split(diff0, diff1);
        ctx.insert_bel_input_inv(tcid, bslot, OLOGIC::T1, xlat_bit_bi(diff0, diff1));

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

        let ffo_sr_sync = TileBit::new(i, 32 + i, [43, 20][i]).pos();
        let ffo_rank1_sr_sync = TileBit::new(i, 33 - i, [0, 63][i]).pos();
        let ffo_rank2_sr_sync = TileBit::new(i, 36 + i, [38, 25][i]).pos();
        let ffo_loadgen_sr_sync = TileBit::new(i, 33 - i, [19, 44][i]).pos();
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFO_SR_SYNC, ffo_sr_sync);
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFO_RANK1_SR_SYNC, ffo_rank1_sr_sync);
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFO_RANK2_SR_SYNC, ffo_rank2_sr_sync);
        ctx.insert_bel_attr_bool(
            tcid,
            bslot,
            OLOGIC::FFO_LOADGEN_SR_SYNC,
            ffo_loadgen_sr_sync,
        );
        let fft_sr_sync = TileBit::new(i, 36 + i, [48, 15][i]).pos();
        let fft_rank1_sr_sync = TileBit::new(i, 32 + i, [32, 31][i]).pos();
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFT_SR_SYNC, fft_sr_sync);
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::FFT_RANK1_SR_SYNC, fft_rank1_sr_sync);

        let item_oq = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFO_SR_SYNC, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFO_SR_SYNC, true),
        );
        let item_tq = xlat_bit_wide_bi(
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFT_SR_SYNC, false),
            ctx.get_diff_attr_bool_bi(tcid, bslot, OLOGIC::FFT_SR_SYNC, true),
        );
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
        diff.apply_bitvec_diff(&item_oq, &bits![1; 4], &bits![0; 4]);
        diff.apply_bitvec_diff(&item_tq, &bits![1; 2], &bits![0; 2]);
        diff.assert_empty();
        assert_eq!(
            BTreeSet::from_iter(item_oq),
            BTreeSet::from([
                ffo_sr_sync,
                ffo_rank1_sr_sync,
                ffo_rank2_sr_sync,
                ffo_loadgen_sr_sync
            ])
        );
        assert_eq!(
            BTreeSet::from_iter(item_tq),
            BTreeSet::from([fft_sr_sync, fft_rank1_sr_sync,])
        );

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
            TileBit::new(i, 32 + i, [40, 23][i]).neg(),
            TileBit::new(i, 33 - i, [36, 27][i]).neg(),
            TileBit::new(i, 32 + i, [38, 25][i]).neg(),
        ];
        ctx.insert_bel_attr_bitvec(tcid, bslot, OLOGIC::FFO_SRVAL, ffo_srval);
        let fft_srval = [
            TileBit::new(i, 37 - i, [44, 19][i]).neg(),
            TileBit::new(i, 37 - i, [43, 20][i]).neg(),
            TileBit::new(i, 36 + i, [46, 17][i]).neg(),
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
                false,
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
        ] {
            diffs.push((
                val,
                ratio,
                true,
                ctx.get_diff_attr_special_val(
                    tcid,
                    bslot,
                    OLOGIC::DATA_WIDTH,
                    specials::OSERDES_DDR,
                    val,
                ),
            ));
        }
        for (_, _, _, diff) in &mut diffs {
            diff.apply_bit_diff(
                ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_SR_ENABLE),
                true,
                false,
            );
        }
        let mut ddr3_byp = diffs[0].3.clone();
        for (_, _, _, diff) in &diffs {
            ddr3_byp.bits.retain(|k, _| diff.bits.contains_key(k));
        }
        let ddr3_byp = xlat_bit(ddr3_byp);
        for (_, _, _, diff) in &mut diffs {
            diff.apply_bit_diff(ddr3_byp, true, false);
        }
        ctx.insert_bel_attr_bool(tcid, bslot, OLOGIC::DDR3_BYPASS, ddr3_byp);
        let mut diff_sdr = diffs[0].3.clone();
        for (_width, _ratio, is_ddr, diff) in &diffs {
            if !is_ddr {
                diff_sdr.bits.retain(|k, _| diff.bits.contains_key(k));
            }
        }
        for (_width, _ratio, is_ddr, diff) in &mut diffs {
            if !*is_ddr {
                *diff = diff.combine(&!&diff_sdr);
            }
        }
        let mut diffs_width = vec![(enums::IO_DATA_WIDTH::NONE, Diff::default())];
        let mut diffs_ratio = vec![(enums::OLOGIC_CLOCK_RATIO::NONE, Diff::default())];
        for &(width, ratio, _, ref diff) in &diffs {
            let mut diff_ratio = Diff::default();
            let mut diff_width = Diff::default();
            for (&bit, &val) in &diff.bits {
                if diffs.iter().any(|&(owidth, _, _, ref odiff)| {
                    width != owidth && odiff.bits.contains_key(&bit)
                }) {
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

        let diff_buf =
            !ctx.get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::SERDES_SDR);
        let diff_ddr = ctx
            .get_diff_attr_val(tcid, bslot, OLOGIC::MUX_O, enums::OLOGIC_MUX_O::SERDES_DDR)
            .combine(&diff_buf);
        let item = xlat_enum_attr(vec![
            (enums::OLOGIC_MUX_O::NONE, Diff::default()),
            (enums::OLOGIC_MUX_O::D1, diff_buf),
            (enums::OLOGIC_MUX_O::SERDES_SDR, diff_sdr),
            (enums::OLOGIC_MUX_O::SERDES_DDR, diff_ddr),
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

        let mut diff_sdr = ctx.get_diff_attr_val(
            tcid,
            bslot,
            OLOGIC::MUX_T,
            enums::OLOGIC_MUX_T::SERDES_SDR,
        );
        let mut diff_ddr = ctx.get_diff_attr_val(
            tcid,
            bslot,
            OLOGIC::MUX_T,
            enums::OLOGIC_MUX_T::SERDES_DDR,
        );
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
            (enums::OLOGIC_MUX_T::SERDES_DDR, diff_ddr),
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

        ctx.get_diff_attr_val(
            tcid,
            bslot,
            OLOGIC::INTERFACE_TYPE,
            enums::OLOGIC_INTERFACE_TYPE::DEFAULT,
        )
        .assert_empty();
        let mut diff = ctx.get_diff_attr_val(
            tcid,
            bslot,
            OLOGIC::INTERFACE_TYPE,
            enums::OLOGIC_INTERFACE_TYPE::MEMORY_DDR3,
        );

        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, OLOGIC::MUX_O),
            enums::OLOGIC_MUX_O::SERDES_DDR,
            enums::OLOGIC_MUX_O::NONE,
        );
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, OLOGIC::DATA_WIDTH),
            enums::IO_DATA_WIDTH::_4,
            enums::IO_DATA_WIDTH::NONE,
        );
        diff.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::FFO_SR_ENABLE),
            true,
            false,
        );
        assert_eq!(diff.bits.len(), 1);
        ctx.insert_bel_attr_enum(
            tcid,
            bslot,
            OLOGIC::INTERFACE_TYPE,
            xlat_enum_attr(vec![
                (enums::OLOGIC_INTERFACE_TYPE::DEFAULT, Diff::default()),
                (enums::OLOGIC_INTERFACE_TYPE::MEMORY_DDR3, diff),
            ]),
        );

        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::SERDES);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::SERDES_MODE);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::SELFHEAL);
        ctx.collect_bel_attr_subset(
            tcid,
            bslot,
            OLOGIC::TRISTATE_WIDTH,
            &[
                enums::OLOGIC_TRISTATE_WIDTH::_1,
                enums::OLOGIC_TRISTATE_WIDTH::_4,
            ],
        );
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::WC_DELAY);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::DDR3_DATA);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::ODELAY_USED);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_LOADCNT);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_FIFO_ADDR);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_FIFO_RESET);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_DLY_CNT);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_PIPE_DATA0);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::INIT_PIPE_DATA1);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::FFO_RANK1_INIT);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::FFO_RANK2_INIT);
        ctx.collect_bel_attr(tcid, bslot, OLOGIC::FFT_RANK1_INIT);

        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::MISR_ENABLE);
        ctx.collect_bel_attr_bi(tcid, bslot, OLOGIC::MISR_ENABLE_FDBK);
        ctx.collect_bel_attr_default(
            tcid,
            bslot,
            OLOGIC::MISR_CLK_SELECT,
            enums::OLOGIC_MISR_CLK_SELECT::NONE,
        );

        let mut present_ologic = ctx.get_diff_bel_special(tcid, bslot, specials::OLOGIC);
        present_ologic.apply_bit_diff(
            ctx.bel_attr_bit(tcid, bslot, OLOGIC::DDR3_BYPASS),
            true,
            false,
        );
        present_ologic.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, OLOGIC::FFT_SRVAL),
            0,
            7,
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
        present_oserdes.apply_bit_diff(
            ctx.bel_input_inv(tcid, bslot, OLOGIC::CLKPERF),
            false,
            true,
        );
        present_oserdes.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, OLOGIC::MUX_O),
            enums::OLOGIC_MUX_O::D1,
            enums::OLOGIC_MUX_O::NONE,
        );
        present_oserdes.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, OLOGIC::MUX_T),
            enums::OLOGIC_MUX_T::T1,
            enums::OLOGIC_MUX_T::NONE,
        );
        present_oserdes.assert_empty();
    }

    let mut diff = ctx.get_diff_attr_bool(tcid, bslots::OLOGIC[0], OLOGIC::MISR_RESET);
    let diff1 = diff.split_bits_by(|bit| bit.rect.to_idx() > 0);
    ctx.insert_bel_attr_bool(tcid, bslots::OLOGIC[0], OLOGIC::MISR_RESET, xlat_bit(diff));
    ctx.insert_bel_attr_bool(tcid, bslots::OLOGIC[1], OLOGIC::MISR_RESET, xlat_bit(diff1));
}

fn collect_fuzzers_iodelay(ctx: &mut CollectorCtx) {
    let tcid = tcls::IO;

    for i in 0..2 {
        let bslot = bslots::IODELAY[i];
        ctx.collect_bel_input_inv_bi(tcid, bslot, IODELAY::C);
        ctx.collect_bel_input_inv_bi(tcid, bslot, IODELAY::DATAIN);
        ctx.collect_bel_attr_bi(tcid, bslot, IODELAY::IDATAIN_INV);
        ctx.collect_bel_attr_bi(tcid, bslot, IODELAY::HIGH_PERFORMANCE_MODE);
        ctx.collect_bel_attr_bi(tcid, bslot, IODELAY::CINVCTRL_SEL);
        let mut diffs_t = vec![];
        let mut diffs_f = vec![];
        for diff in ctx.get_diffs_attr_bits(tcid, bslot, IODELAY::IDELAY_VALUE_INIT, 5) {
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
        ctx.insert_bel_attr_bitvec(
            tcid,
            bslot,
            IODELAY::IDELAY_VALUE_INIT,
            xlat_bitvec(diffs_t),
        );
        ctx.insert_bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR, xlat_bitvec(diffs_f));
        ctx.collect_bel_attr(tcid, bslot, IODELAY::ALT_DELAY_VALUE);
        let (_, _, mut diff) = Diff::split(
            ctx.peek_diff_attr_val(tcid, bslot, IODELAY::DELAY_SRC, enums::IODELAY_DELAY_SRC::I)
                .clone(),
            ctx.peek_diff_attr_val(tcid, bslot, IODELAY::DELAY_SRC, enums::IODELAY_DELAY_SRC::O)
                .clone(),
        );
        diff.discard_polbits(ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR));
        ctx.insert_bel_attr_bool(tcid, bslot, IODELAY::ENABLE, xlat_bit(diff));
        let mut diffs = vec![(enums::IODELAY_DELAY_SRC::NONE, Diff::default())];
        for val in [
            enums::IODELAY_DELAY_SRC::I,
            enums::IODELAY_DELAY_SRC::IO,
            enums::IODELAY_DELAY_SRC::O,
            enums::IODELAY_DELAY_SRC::DATAIN,
            enums::IODELAY_DELAY_SRC::CLKIN,
            enums::IODELAY_DELAY_SRC::DELAYCHAIN_OSC,
        ] {
            let mut diff = ctx.get_diff_attr_val(tcid, bslot, IODELAY::DELAY_SRC, val);
            diff.apply_bitvec_diff_int(
                ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
                0,
                0x1f,
            );
            diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
            diffs.push((val, diff));
        }
        ctx.insert_bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC, xlat_enum_attr(diffs));

        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_DEFAULT);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::I,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        let val = extract_bitvec_val_part(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            &bits![1; 5],
            &mut diff,
        );
        ctx.insert_devdata_bitvec(devdata::IODELAY_IDELAY_DEFAULT, val);
        let val = extract_bitvec_val_part(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_INIT),
            &bits![0; 5],
            &mut diff,
        );
        ctx.insert_devdata_bitvec(devdata::IODELAY_IDELAY_DEFAULT, val);
        ctx.insert_bel_attr_bool(tcid, bslot, IODELAY::EXTRA_DELAY, xlat_bit(diff));

        let mut diffs = vec![];
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_FIXED);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::I,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::FIXED, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_VARIABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::I,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VARIABLE, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_VAR_LOADABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::I,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VAR_LOADABLE, diff));

        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_O_FIXED);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::O,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::FIXED, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_O_VARIABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::O,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VARIABLE, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_O_VAR_LOADABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::O,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VAR_LOADABLE, diff));

        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_IO_FIXED);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::IO,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::FIXED, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_FIXED_O_VARIABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::IO,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VARIABLE_SWAPPED, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_VARIABLE_O_FIXED);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::IO,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::VARIABLE, diff));
        let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_IO_VAR_LOADABLE);
        diff.apply_bit_diff(ctx.bel_attr_bit(tcid, bslot, IODELAY::ENABLE), true, false);
        diff.apply_enum_diff(
            ctx.bel_attr_enum(tcid, bslot, IODELAY::DELAY_SRC),
            enums::IODELAY_DELAY_SRC::IO,
            enums::IODELAY_DELAY_SRC::NONE,
        );
        diff.apply_bitvec_diff_int(
            ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
            0,
            0x1f,
        );
        diffs.push((enums::IODELAY_DELAY_TYPE::IO_VAR_LOADABLE, diff));
        ctx.insert_bel_attr_enum(tcid, bslot, IODELAY::DELAY_TYPE, xlat_enum_attr(diffs));
    }
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx, devdata_only: bool) {
    let tcid = tcls::IO;
    if devdata_only {
        for i in 0..2 {
            let bslot = bslots::IODELAY[i];
            let mut diff = ctx.get_diff_bel_special(tcid, bslot, specials::IODELAY_I_DEFAULT);
            let val = extract_bitvec_val_part(
                ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_CUR),
                &bits![1; 5],
                &mut diff,
            );
            ctx.insert_devdata_bitvec(devdata::IODELAY_IDELAY_DEFAULT, val);
            let val = extract_bitvec_val_part(
                ctx.bel_attr_bitvec(tcid, bslot, IODELAY::IDELAY_VALUE_INIT),
                &bits![0; 5],
                &mut diff,
            );
            ctx.insert_devdata_bitvec(devdata::IODELAY_IDELAY_DEFAULT, val);
        }
        return;
    }

    collect_fuzzers_routing(ctx);
    collect_fuzzers_ilogic(ctx);
    collect_fuzzers_ologic(ctx);
    collect_fuzzers_iodelay(ctx);
}
