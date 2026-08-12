use prjcombine_entity::EntityId;
use prjcombine_re_collector::diff::{Diff, xlat_bit, xlat_bit_wide, xlat_enum_attr};
use prjcombine_re_hammer::Session;
use prjcombine_virtex4::defs::{bcls, bslots, enums, virtex4::tcls};

use crate::{
    backend::{IseBackend, MultiValue},
    collector::CollectorCtx,
    generic::fbuild::{FuzzBuilderBase, FuzzCtx},
    virtex4::specials,
};

pub fn add_fuzzers<'a>(session: &mut Session<'a, IseBackend<'a>>, backend: &'a IseBackend<'a>) {
    let mut ctx = FuzzCtx::new(session, backend, tcls::BRAM);
    {
        let mut bctx = ctx.bel(bslots::BRAM);
        let mode = "RAMB16";
        bctx.build()
            .global_mutex("BRAM", "NOPE")
            .sub_unused(1)
            .test_bel_special(specials::PRESENT)
            .mode(mode)
            .commit();
        for pin in [
            bcls::BRAM::CLKA,
            bcls::BRAM::CLKB,
            bcls::BRAM::ENA,
            bcls::BRAM::ENB,
            bcls::BRAM::SSRA,
            bcls::BRAM::SSRB,
            bcls::BRAM::REGCEA,
            bcls::BRAM::REGCEB,
            bcls::BRAM::WEA[0],
            bcls::BRAM::WEA[1],
            bcls::BRAM::WEA[2],
            bcls::BRAM::WEA[3],
            bcls::BRAM::WEB[0],
            bcls::BRAM::WEB[1],
            bcls::BRAM::WEB[2],
            bcls::BRAM::WEB[3],
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .test_bel_input_inv_auto(pin);
        }
        for attr in [
            bcls::BRAM::INVERT_CLK_DOA_REG,
            bcls::BRAM::INVERT_CLK_DOB_REG,
            bcls::BRAM::EN_ECC_READ,
            bcls::BRAM::EN_ECC_WRITE,
            bcls::BRAM::SAVEDATA,
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .test_bel_attr_bool_auto(attr, "FALSE", "TRUE");
        }
        for attr in [bcls::BRAM::DOA_REG, bcls::BRAM::DOB_REG] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .test_bel_attr_bool_auto(attr, "0", "1");
        }
        for attr in [
            bcls::BRAM::READ_WIDTH_A,
            bcls::BRAM::READ_WIDTH_B,
            bcls::BRAM::WRITE_WIDTH_A,
            bcls::BRAM::WRITE_WIDTH_B,
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .attr("INIT_A", "0")
                .attr("INIT_B", "0")
                .attr("SRVAL_A", "0")
                .attr("SRVAL_B", "0")
                .test_bel_attr_auto(attr);
            bctx.mode(mode)
                .null_bits()
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .attr("INIT_A", "0")
                .attr("INIT_B", "0")
                .attr("SRVAL_A", "0")
                .attr("SRVAL_B", "0")
                .test_bel_attr_special(attr, specials::BRAM_DATA_WIDTH_0)
                .attr(
                    backend.edev.db[bcls::BRAM].attributes[attr].name.clone(),
                    "0",
                )
                .commit();
        }
        for (attr, aname) in [
            (bcls::BRAM::RAM_EXTENSION_A_LOWER, "RAM_EXTENSION_A"),
            (bcls::BRAM::RAM_EXTENSION_B_LOWER, "RAM_EXTENSION_B"),
        ] {
            for (val, vname) in [(false, "NONE"), (false, "UPPER"), (true, "LOWER")] {
                bctx.mode(mode)
                    .global_mutex("BRAM", "NOPE")
                    .sub_unused(1)
                    .test_bel_attr_bits_bi(attr, val)
                    .attr(aname, vname)
                    .commit();
            }
        }
        for attr in [bcls::BRAM::WRITE_MODE_A, bcls::BRAM::WRITE_MODE_B] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .sub_unused(1)
                .test_bel_attr_auto(attr);
        }
        for attr in [
            bcls::BRAM::INIT_A,
            bcls::BRAM::INIT_B,
            bcls::BRAM::SRVAL_A,
            bcls::BRAM::SRVAL_B,
        ] {
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .attr("READ_WIDTH_A", "36")
                .attr("READ_WIDTH_B", "36")
                .test_bel_attr_multi(attr, MultiValue::Hex(0));
        }
        for i in 0..0x40 {
            let attr = format!("INIT_{i:02X}");
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .test_bel_attr_bits_base(bcls::BRAM::DATA, i * 0x100)
                .multi_attr(attr, MultiValue::Hex(0), 0x100);
        }
        for i in 0..0x8 {
            let attr = format!("INITP_{i:02X}");
            bctx.mode(mode)
                .global_mutex("BRAM", "NOPE")
                .test_bel_attr_bits_base(bcls::BRAM::DATAP, i * 0x100)
                .multi_attr(attr, MultiValue::Hex(0), 0x100);
        }
        for (val, vname) in [
            (enums::BRAM_WW_VALUE::_0, "0"),
            (enums::BRAM_WW_VALUE::_1, "1"),
        ] {
            bctx.mode(mode)
                .global_mutex_here("BRAM")
                .test_bel_attr_val(bcls::BRAM::WW_VALUE_A, val)
                .global("Ibram_ww_value", vname)
                .commit();
        }
    }
    {
        let mut bctx = ctx.bel(bslots::BRAM).sub(1);
        let mode = "FIFO16";
        bctx.build()
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .test_bel_special(specials::BRAM_FIFO)
            .mode(mode)
            .commit();
        for (pin, pname) in [
            (bcls::BRAM::CLKA, "RDCLK"),
            (bcls::BRAM::CLKB, "WRCLK"),
            (bcls::BRAM::ENA, "RDEN"),
            (bcls::BRAM::ENB, "WREN"),
            (bcls::BRAM::SSRA, "RST"),
        ] {
            for val in [false, true] {
                bctx.mode(mode)
                    .global_mutex("BRAM", "NOPE")
                    .sub_unused(0)
                    .attr("DATA_WIDTH", "36")
                    .pin(pname)
                    .test_bel_input_inv(pin, val)
                    .attr(
                        format!("{pname}INV"),
                        if val {
                            format!("{pname}_B")
                        } else {
                            pname.to_string()
                        },
                    )
                    .commit();
            }
        }
        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .test_bel_attr_rename("DATA_WIDTH", bcls::BRAM::FIFO_WIDTH);
        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .attr("DATA_WIDTH", "36")
            .test_bel_attr_bool_auto(bcls::BRAM::FIRST_WORD_FALL_THROUGH, "FALSE", "TRUE");
        for attr in [bcls::BRAM::EN_ECC_READ, bcls::BRAM::EN_ECC_WRITE] {
            for (val, spec) in [
                (false, specials::FIFO_EN_ECC_FALSE),
                (true, specials::FIFO_EN_ECC_TRUE),
            ] {
                bctx.mode(mode)
                    .null_bits()
                    .global_mutex("BRAM", "NOPE")
                    .sub_unused(0)
                    .attr("DATA_WIDTH", "36")
                    .test_bel_attr_special(attr, spec)
                    .attr(
                        backend.edev.db[bcls::BRAM].attributes[attr].name.clone(),
                        if val { "TRUE" } else { "FALSE" },
                    )
                    .commit();
            }
        }
        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .attr("FIRST_WORD_FALL_THROUGH", "FALSE")
            .test_bel_attr_bits(bcls::BRAM::ALMOST_FULL_OFFSET)
            .multi_attr("ALMOST_FULL_OFFSET", MultiValue::Hex(0), 12);
        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .attr("FIRST_WORD_FALL_THROUGH", "FALSE")
            .test_bel_attr_bits(bcls::BRAM::ALMOST_EMPTY_OFFSET)
            .multi_attr("ALMOST_EMPTY_OFFSET", MultiValue::Hex(1), 12);

        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .attr("FIRST_WORD_FALL_THROUGH", "TRUE")
            .test_bel_attr_bits(bcls::BRAM::ALMOST_FULL_OFFSET)
            .multi_attr("ALMOST_FULL_OFFSET", MultiValue::Hex(0), 12);
        bctx.mode(mode)
            .global_mutex("BRAM", "NOPE")
            .sub_unused(0)
            .attr("FIRST_WORD_FALL_THROUGH", "TRUE")
            .test_bel_attr_bits(bcls::BRAM::ALMOST_EMPTY_OFFSET)
            .multi_attr("ALMOST_EMPTY_OFFSET", MultiValue::Hex(2), 12);
    }
}

pub fn collect_fuzzers(ctx: &mut CollectorCtx) {
    let tcid = tcls::BRAM;
    let bslot = bslots::BRAM;
    for pin in [
        bcls::BRAM::CLKA,
        bcls::BRAM::CLKB,
        bcls::BRAM::ENA,
        bcls::BRAM::ENB,
        bcls::BRAM::SSRA,
        bcls::BRAM::SSRB,
        bcls::BRAM::REGCEA,
        bcls::BRAM::REGCEB,
    ] {
        ctx.collect_bel_input_inv_int_bi(&[tcls::INT; 4], tcid, bslot, pin);
    }
    for pin in [
        bcls::BRAM::WEA[0],
        bcls::BRAM::WEA[1],
        bcls::BRAM::WEA[2],
        bcls::BRAM::WEA[3],
        bcls::BRAM::WEB[0],
        bcls::BRAM::WEB[1],
        bcls::BRAM::WEB[2],
        bcls::BRAM::WEB[3],
    ] {
        ctx.collect_bel_input_inv_bi(tcid, bslot, pin);
    }
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::INVERT_CLK_DOA_REG);
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::INVERT_CLK_DOB_REG);
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::DOA_REG);
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::DOB_REG);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::READ_WIDTH_A);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::READ_WIDTH_B);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::WRITE_WIDTH_A);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::WRITE_WIDTH_B);
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::RAM_EXTENSION_A_LOWER);
    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::RAM_EXTENSION_B_LOWER);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::WRITE_MODE_A);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::WRITE_MODE_B);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::INIT_A);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::INIT_B);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::SRVAL_A);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::SRVAL_B);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::DATA);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::DATAP);

    for attr in [
        bcls::BRAM::EN_ECC_READ,
        bcls::BRAM::EN_ECC_WRITE,
        bcls::BRAM::SAVEDATA,
    ] {
        ctx.get_diff_attr_bool_bi(tcid, bslot, attr, false)
            .assert_empty();
        let diff = ctx.get_diff_attr_bool_bi(tcid, bslot, attr, true);
        ctx.insert_bel_attr_bitvec(tcid, bslot, attr, xlat_bit_wide(diff));
    }

    ctx.collect_bel_attr_bi(tcid, bslot, bcls::BRAM::FIRST_WORD_FALL_THROUGH);
    let mut diffs = vec![];
    let item_ra = ctx
        .bel_attr_enum(tcid, bslot, bcls::BRAM::READ_WIDTH_A)
        .clone();
    let item_wb = ctx
        .bel_attr_enum(tcid, bslot, bcls::BRAM::WRITE_WIDTH_B)
        .clone();
    for (val_fifo, val_port) in [
        (enums::BRAM_FIFO_WIDTH::_4, enums::BRAM_DATA_WIDTH::_4),
        (enums::BRAM_FIFO_WIDTH::_9, enums::BRAM_DATA_WIDTH::_9),
        (enums::BRAM_FIFO_WIDTH::_18, enums::BRAM_DATA_WIDTH::_18),
        (enums::BRAM_FIFO_WIDTH::_36, enums::BRAM_DATA_WIDTH::_36),
    ] {
        let mut diff = ctx.get_diff_attr_val(tcid, bslot, bcls::BRAM::FIFO_WIDTH, val_fifo);
        diff.apply_enum_diff(&item_ra, val_port, enums::BRAM_DATA_WIDTH::_1);
        diff.apply_enum_diff(&item_wb, val_port, enums::BRAM_DATA_WIDTH::_1);
        diffs.push((val_fifo, diff));
    }
    ctx.insert_bel_attr_enum(tcid, bslot, bcls::BRAM::FIFO_WIDTH, xlat_enum_attr(diffs));

    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::ALMOST_EMPTY_OFFSET);
    ctx.collect_bel_attr(tcid, bslot, bcls::BRAM::ALMOST_FULL_OFFSET);

    let mut present_bram = ctx.get_diff_bel_special(tcid, bslot, specials::PRESENT);
    let mut present_fifo = ctx.get_diff_bel_special(tcid, bslot, specials::BRAM_FIFO);
    for attr in [
        bcls::BRAM::INIT_A,
        bcls::BRAM::INIT_B,
        bcls::BRAM::SRVAL_A,
        bcls::BRAM::SRVAL_B,
    ] {
        present_bram.discard_polbits(ctx.bel_attr_bitvec(tcid, bslot, attr));
        present_fifo.discard_polbits(ctx.bel_attr_bitvec(tcid, bslot, attr));
    }
    present_bram.apply_enum_diff(
        ctx.bel_attr_enum(tcid, bslot, bcls::BRAM::WRITE_MODE_A),
        enums::BRAM_WRITE_MODE::WRITE_FIRST,
        enums::BRAM_WRITE_MODE::READ_FIRST,
    );
    present_bram.apply_enum_diff(
        ctx.bel_attr_enum(tcid, bslot, bcls::BRAM::WRITE_MODE_B),
        enums::BRAM_WRITE_MODE::WRITE_FIRST,
        enums::BRAM_WRITE_MODE::READ_FIRST,
    );
    present_fifo.apply_enum_diff(
        ctx.bel_attr_enum(tcid, bslot, bcls::BRAM::WRITE_MODE_A),
        enums::BRAM_WRITE_MODE::WRITE_FIRST,
        enums::BRAM_WRITE_MODE::READ_FIRST,
    );
    present_fifo.apply_enum_diff(
        ctx.bel_attr_enum(tcid, bslot, bcls::BRAM::WRITE_MODE_B),
        enums::BRAM_WRITE_MODE::WRITE_FIRST,
        enums::BRAM_WRITE_MODE::READ_FIRST,
    );
    present_fifo.apply_bit_diff(
        ctx.bel_attr_bit(tcid, bslot, bcls::BRAM::DOA_REG),
        true,
        false,
    );
    present_fifo.apply_bit_diff(
        ctx.bel_attr_bit(tcid, bslot, bcls::BRAM::DOB_REG),
        true,
        false,
    );
    present_bram.assert_empty();
    ctx.insert_bel_attr_bool(tcid, bslot, bcls::BRAM::FIFO_ENABLE, xlat_bit(present_fifo));

    let mut diffs_a = vec![(enums::BRAM_WW_VALUE::NONE, Diff::default())];
    let mut diffs_b = vec![(enums::BRAM_WW_VALUE::NONE, Diff::default())];
    for val in [enums::BRAM_WW_VALUE::_0, enums::BRAM_WW_VALUE::_1] {
        let mut diff = ctx.get_diff_attr_val(tcid, bslot, bcls::BRAM::WW_VALUE_A, val);
        let diff_b = diff.split_bits_by(|bit| matches!(bit.bit.to_idx(), 29 | 30));
        diffs_a.push((val, diff));
        diffs_b.push((val, diff_b));
    }

    ctx.insert_bel_attr_enum(tcid, bslot, bcls::BRAM::WW_VALUE_A, xlat_enum_attr(diffs_a));
    ctx.insert_bel_attr_enum(tcid, bslot, bcls::BRAM::WW_VALUE_B, xlat_enum_attr(diffs_b));
}
