use prjcombine_entity::EntityId;
use prjcombine_interconnect::coord::BelCoord;
use prjcombine_re_xilinx_naming_versal::ExpandedNamedDevice;
use prjcombine_re_xilinx_rdverify::Verifier;
use prjcombine_versal::{
    defs::{bslots, tcls},
    expanded::ExpandedDevice,
};

pub fn verify_aie_core(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_out_far_claim("M_CASC_OUT")
        .extra_out_far_claim("M_CORE_CH0")
        .extra_out_far_claim("M_CORE_CH1")
        .extra_out_far_claim("M_TRACE")
        .extra_in_far("S_CASC_IN")
        .extra_in_far_claim("S_CORE_CH0")
        .extra_in_far_claim("S_CORE_CH1");
    let dx = if !bcrd.row.to_idx().is_multiple_of(2) {
        -1
    } else {
        1
    };
    let casc_in = edev
        .bel_delta(bcrd.cell, dx, 0, bcrd.slot)
        .or(edev.bel_delta(bcrd.cell, 0, -1, bcrd.slot));
    if let Some(obel) = casc_in {
        bel.verify_net(&[
            bel.wire_far("S_CASC_IN"),
            bel.bel_wire_far(obel, "M_CASC_OUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire_far("S_CASC_IN")]);
    }
    bel.commit();
}

pub fn verify_aie_ml_core(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_ML => "AIE_ML_CORE",
        tcls::AIE2PS => "AIE2PS_CORE",
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_CASC_EST_OUT")
        .extra_out_far_claim("M_CASC_STH_OUT")
        .extra_out_far_claim("M_CORE_CH0")
        .extra_out_far_claim("M_TRACE")
        .extra_in_far("S_CASC_WST_IN")
        .extra_in_far("S_CASC_NTH_IN")
        .extra_in_far_claim("S_CORE_CH0");
    if let Some(obel) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
        bel.verify_net(&[
            bel.wire_far("S_CASC_WST_IN"),
            bel.bel_wire_far(obel, "M_CASC_EST_OUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire_far("S_CASC_WST_IN")]);
    }
    if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 1, bcrd.slot) {
        bel.verify_net(&[
            bel.wire_far("S_CASC_NTH_IN"),
            bel.bel_wire_far(obel, "M_CASC_STH_OUT"),
        ]);
    } else {
        bel.claim_net(&[bel.wire_far("S_CASC_NTH_IN")]);
    }
    bel.commit();
}

pub fn verify_aie_memgrp(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE => "AIE_MEMGRP",
        tcls::AIE_ML => "AIE_ML_MEMGRP",
        tcls::AIE2PS => "AIE2PS_MEMGRP",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_DMA_CH0")
        .extra_out_far_claim("M_DMA_CH1")
        .extra_out_far_claim("M_TRACE")
        .extra_in_far_claim("S_DMA_CH0")
        .extra_in_far_claim("S_DMA_CH1")
        .commit();
}

pub fn verify_aie_ml_mem_memgrp(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_ML_MEM => "AIE_ML_MEM_MEMGRP",
        tcls::AIE2PS_MEM => "AIE2PS_MEM_MEMGRP",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_DMA_CH0")
        .extra_out_far_claim("M_DMA_CH1")
        .extra_out_far_claim("M_DMA_CH2")
        .extra_out_far_claim("M_DMA_CH3")
        .extra_out_far_claim("M_DMA_CH4")
        .extra_out_far_claim("M_DMA_CH5")
        .extra_out_far_claim("M_TRACE")
        .extra_in_far_claim("S_DMA_CH0")
        .extra_in_far_claim("S_DMA_CH1")
        .extra_in_far_claim("S_DMA_CH2")
        .extra_in_far_claim("S_DMA_CH3")
        .extra_in_far_claim("S_DMA_CH4")
        .extra_in_far_claim("S_DMA_CH5")
        .commit();
}

pub fn verify_aie2ps_shim_cntrl(_edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    vrf.verify_bel(bcrd)
        .extra_out_far_claim("M_SHIM_CNTRL_CH0")
        .extra_in_far_claim("S_SHIM_CNTRL_CH0")
        .extra_out_far_claim("TO_NOC")
        .commit();
}

pub fn verify_aie2ps_nmu_arbit(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf
        .verify_bel(bcrd)
        .extra_in_far_from_far(
            "FROM_SHIM_CNTRL",
            bcrd.bel(bslots::AIE2PS_SHIM_CNTRL),
            "TO_NOC",
        )
        .extra_in_far_from_far("FROM_AIE_0", bcrd.bel(bslots::AIE_NOC), "TO_NOC")
        .extra_in_far_claim("FROM_AIE_1")
        .extra_out_far_claim("TO_NEXT");
    if let Some(prev) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
        bel = bel.extra_in_far_from_far("FROM_PREV", prev, "TO_NEXT");
    } else {
        bel = bel.extra_in_far_claim("FROM_PREV");
    }
    if bcrd.col.to_idx() == 0 {
        bel = bel
            .extra_out_far_claim("TO_NOC_0")
            .extra_out_far_claim("TO_NOC_1");
    } else if bcrd.col.to_idx() == 1 {
        let nmu = [0, 1].map(|i| bcrd.delta(-1, -1).bel(bslots::HNOC_ME_NMU[i]));
        bel = bel
            .extra_out_far_to_far("TO_NOC_0", nmu[0], "AXI_IN")
            .extra_out_far_to_far("TO_NOC_1", nmu[1], "AXI_IN");
    } else {
        let idx = (bcrd.col.to_idx() - 2) % 4;
        let nmu = bcrd.delta(-(idx as i32), -1).bel(bslots::HNOC_ME_NMU[idx]);
        bel = bel
            .extra_out_far_to_far("TO_NOC_0", nmu, "AXI_IN")
            .extra_out_far_claim("TO_NOC_1");
    }
    bel.commit();
}

pub fn verify_aie_fifo(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE | tcls::AIE_SHIM | tcls::AIE_SHIM_PL | tcls::AIE_SHIM_NOC_PL => "AIE_FIFO",
        tcls::AIE_ML
        | tcls::AIE_ML_MEM
        | tcls::AIE_ML_SHIM
        | tcls::AIE_ML_SHIM_PL
        | tcls::AIE_ML_SHIM_NOC_PL => "AIE_ML_FIFO",
        tcls::AIE2PS | tcls::AIE2PS_MEM | tcls::AIE2PS_SHIM_NOC_PL => "AIE2P_FIFO",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_FIFO")
        .extra_in_far_claim("S_FIFO")
        .commit();
}

pub fn verify_aie_tilectrl(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE | tcls::AIE_SHIM | tcls::AIE_SHIM_PL | tcls::AIE_SHIM_NOC_PL => "AIE_TILECTRL",
        tcls::AIE_ML
        | tcls::AIE_ML_MEM
        | tcls::AIE_ML_SHIM
        | tcls::AIE_ML_SHIM_PL
        | tcls::AIE_ML_SHIM_NOC_PL => "AIE_ML_TILECTRL",
        tcls::AIE2PS | tcls::AIE2PS_MEM | tcls::AIE2PS_SHIM_NOC_PL => "AIE2P_TILECTRL",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_CONTROL")
        .extra_in_far_claim("S_CONTROL")
        .commit();
}

pub fn verify_aie_shimtrace(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_SHIM | tcls::AIE_SHIM_PL | tcls::AIE_SHIM_NOC_PL => "AIE_SHIMTRACE",
        tcls::AIE_ML_SHIM | tcls::AIE_ML_SHIM_PL | tcls::AIE_ML_SHIM_NOC_PL => "AIE_ML_SHIMTRACE",
        tcls::AIE2PS_SHIM_NOC_PL => "AIE2PS_SHIMTRACE",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("M_TRACE")
        .commit();
}

pub fn verify_aie_arrayswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_FIFO_0", bslots::AIE_FIFO[0], "S_FIFO"),
        ("M_FIFO_1", bslots::AIE_FIFO[1], "S_FIFO"),
        ("M_CORE_IN_CH0", bslots::AIE_CORE, "S_CORE_CH0"),
        ("M_CORE_IN_CH1", bslots::AIE_CORE, "S_CORE_CH1"),
        ("M_S2MM_CH0", bslots::AIE_MEMGRP, "S_DMA_CH0"),
        ("M_S2MM_CH1", bslots::AIE_MEMGRP, "S_DMA_CH1"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_FIFO_0", bslots::AIE_FIFO[0], "M_FIFO"),
        ("S_FIFO_1", bslots::AIE_FIFO[1], "M_FIFO"),
        ("S_CORE_OUT_CH0", bslots::AIE_CORE, "M_CORE_CH0"),
        ("S_CORE_OUT_CH1", bslots::AIE_CORE, "M_CORE_CH1"),
        ("S_CORETRACE", bslots::AIE_CORE, "M_TRACE"),
        ("S_MM2S_CH0", bslots::AIE_MEMGRP, "M_DMA_CH0"),
        ("S_MM2S_CH1", bslots::AIE_MEMGRP, "M_DMA_CH1"),
        ("S_MEMTRACE", bslots::AIE_MEMGRP, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_s).extra_in_far(pin_n);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 1, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_n)]);
        }
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        let pin_s = &format!("S_SOUTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_s);
        let obel = edev
            .bel_delta(bcrd.cell, 0, -1, bcrd.slot)
            .or(edev.bel_delta(bcrd.cell, 0, -1, bslots::AIE_SHIMSWITCH))
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_s), bel.bel_wire_far(obel, pin_n)]);
    }
    for i in 0..4 {
        let pin_n = &format!("M_NEXT_CH{i}");
        let pin_p = &format!("S_PREV_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        let dx = if !bcrd.row.to_idx().is_multiple_of(2) {
            -1
        } else {
            1
        };
        if let Some(obel) = edev.bel_delta(bcrd.cell, dx, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..4 {
        let pin_n = &format!("M_PREV_CH{i}");
        let pin_p = &format!("S_NEXT_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        let dx = if !bcrd.row.to_idx().is_multiple_of(2) {
            1
        } else {
            -1
        };
        if let Some(obel) = edev.bel_delta(bcrd.cell, dx, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    bel.commit();
}

pub fn verify_aie_shimswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_FIFO_0", bslots::AIE_FIFO[0], "S_FIFO"),
        ("M_FIFO_1", bslots::AIE_FIFO[1], "S_FIFO"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_FIFO_0", bslots::AIE_FIFO[0], "M_FIFO"),
        ("S_FIFO_1", bslots::AIE_FIFO[1], "M_FIFO"),
        ("S_TRACE", bslots::AIE_SHIMTRACE, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_in_far(pin_n);
        let obel = edev
            .bel_delta(bcrd.cell, 0, 1, bslots::AIE_ARRAYSWITCH)
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n);
    }
    for i in 0..4 {
        let pin_n = &format!("M_NEXT_CH{i}");
        let pin_p = &format!("S_PREV_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..4 {
        let pin_n = &format!("M_PREV_CH{i}");
        let pin_p = &format!("S_NEXT_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..8 {
        let pin = &format!("S_SOUTH_CH{i}");
        if matches!(i, 2 | 3 | 6 | 7)
            && let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_NOC)
        {
            bel = bel.extra_in_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("TO_AIE_{i}")),
            ]);
        } else if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_PL) {
            bel = bel.extra_in_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("TO_AIE_{i}")),
            ]);
        } else {
            bel = bel.extra_in_far_claim(pin);
        }
    }
    for i in 0..6 {
        let pin = &format!("M_SOUTH_CH{i}");
        if matches!(i, 2..6)
            && let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_NOC)
        {
            bel = bel.extra_out_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("FROM_AIE_{i}")),
            ]);
        } else if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_PL) {
            bel = bel.extra_out_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("FROM_AIE_{i}")),
            ]);
        } else {
            bel = bel.extra_out_far_claim(pin);
        }
    }
    bel.commit();
}

pub fn verify_aie_ml_arrayswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_ML => "AIE_ML_ARRAYSWITCH",
        tcls::AIE2PS => "AIE2PS_ARRAYSWITCH",
        _ => unreachable!(),
    };
    let mut bel = vrf.verify_bel(bcrd).kind(kind);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_FIFO_0", bslots::AIE_FIFO[0], "S_FIFO"),
        ("M_CORE_IN_CH0", bslots::AIE_ML_CORE, "S_CORE_CH0"),
        ("M_S2MM_CH0", bslots::AIE_MEMGRP, "S_DMA_CH0"),
        ("M_S2MM_CH1", bslots::AIE_MEMGRP, "S_DMA_CH1"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_FIFO_0", bslots::AIE_FIFO[0], "M_FIFO"),
        ("S_CORE_OUT_CH0", bslots::AIE_ML_CORE, "M_CORE_CH0"),
        ("S_CORETRACE", bslots::AIE_ML_CORE, "M_TRACE"),
        ("S_MM2S_CH0", bslots::AIE_MEMGRP, "M_DMA_CH0"),
        ("S_MM2S_CH1", bslots::AIE_MEMGRP, "M_DMA_CH1"),
        ("S_MEMTRACE", bslots::AIE_MEMGRP, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_s).extra_in_far(pin_n);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 1, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_n)]);
        }
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        let pin_s = &format!("S_SOUTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_s);
        let obel = edev
            .bel_delta(bcrd.cell, 0, -1, bcrd.slot)
            .or(edev.bel_delta(bcrd.cell, 0, -1, bslots::AIE_ML_SHIMSWITCH))
            .or(edev.bel_delta(bcrd.cell, 0, -1, bslots::AIE_ML_MEM_ARRAYSWITCH))
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_s), bel.bel_wire_far(obel, pin_n)]);
    }
    for i in 0..4 {
        let pin_n = &format!("M_EAST_CH{i}");
        let pin_p = &format!("S_WEST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..4 {
        let pin_n = &format!("M_WEST_CH{i}");
        let pin_p = &format!("S_EAST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    bel.commit();
}

pub fn verify_aie_ml_mem_arrayswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_ML_MEM => "AIE_ML_MEM_ARRAYSWITCH",
        tcls::AIE2PS_MEM => "AIE2PS_MEM_ARRAYSWITCH",
        _ => unreachable!(),
    };
    let mut bel = vrf.verify_bel(bcrd).kind(kind);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_S2MM_CH0", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH0"),
        ("M_S2MM_CH1", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH1"),
        ("M_S2MM_CH2", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH2"),
        ("M_S2MM_CH3", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH3"),
        ("M_S2MM_CH4", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH4"),
        ("M_S2MM_CH5", bslots::AIE_ML_MEM_MEMGRP, "S_DMA_CH5"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_MM2S_CH0", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH0"),
        ("S_MM2S_CH1", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH1"),
        ("S_MM2S_CH2", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH2"),
        ("S_MM2S_CH3", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH3"),
        ("S_MM2S_CH4", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH4"),
        ("S_MM2S_CH5", bslots::AIE_ML_MEM_MEMGRP, "M_DMA_CH5"),
        ("S_MEMTRACE", bslots::AIE_ML_MEM_MEMGRP, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_s).extra_in_far(pin_n);
        if let Some(obel) = edev
            .bel_delta(bcrd.cell, 0, 1, bcrd.slot)
            .or(edev.bel_delta(bcrd.cell, 0, 1, bslots::AIE_ML_ARRAYSWITCH))
        {
            bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_n)]);
        }
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        let pin_s = &format!("S_SOUTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_s);
        let obel = edev
            .bel_delta(bcrd.cell, 0, -1, bcrd.slot)
            .or(edev.bel_delta(bcrd.cell, 0, -1, bslots::AIE_ML_SHIMSWITCH))
            .or(edev.bel_delta(bcrd.cell, 0, -1, bslots::AIE2PS_SHIMSWITCH))
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_s), bel.bel_wire_far(obel, pin_n)]);
    }
    bel.commit();
}

pub fn verify_aie_ml_shimswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_FIFO_0", bslots::AIE_FIFO[0], "S_FIFO"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_FIFO_0", bslots::AIE_FIFO[0], "M_FIFO"),
        ("S_TRACE", bslots::AIE_SHIMTRACE, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_in_far(pin_n);
        let obel = edev
            .bel_delta(bcrd.cell, 0, 1, bslots::AIE_ML_ARRAYSWITCH)
            .or(edev.bel_delta(bcrd.cell, 0, 1, bslots::AIE_ML_MEM_ARRAYSWITCH))
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n);
    }
    for i in 0..4 {
        let pin_n = &format!("M_EAST_CH{i}");
        let pin_p = &format!("S_WEST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..4 {
        let pin_n = &format!("M_WEST_CH{i}");
        let pin_p = &format!("S_EAST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..8 {
        let pin = &format!("S_SOUTH_CH{i}");
        if matches!(i, 2 | 3 | 6 | 7)
            && let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_NOC)
        {
            bel = bel.extra_in_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("TO_AIE_{i}")),
            ]);
        } else if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_PL) {
            bel = bel.extra_in_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("TO_AIE_{i}")),
            ]);
        } else {
            bel = bel.extra_in_far_claim(pin);
        }
    }
    for i in 0..6 {
        let pin = &format!("M_SOUTH_CH{i}");
        if matches!(i, 2..6)
            && let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_NOC)
        {
            bel = bel.extra_out_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("FROM_AIE_{i}")),
            ]);
        } else if let Some(obel) = edev.bel_delta(bcrd.cell, 0, 0, bslots::AIE_PL) {
            bel = bel.extra_out_far(pin);
            bel.verify_net(&[
                bel.wire_far(pin),
                bel.bel_wire_far(obel, &format!("FROM_AIE_{i}")),
            ]);
        } else {
            bel = bel.extra_out_far_claim(pin);
        }
    }
    bel.commit();
}

pub fn verify_aie2ps_shimswitch(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let mut bel = vrf.verify_bel(bcrd);
    for (pin, bslot, opin) in [
        ("M_CONTROL", bslots::AIE_TILECTRL, "S_CONTROL"),
        ("M_UCNTRL", bslots::AIE2PS_SHIM_CNTRL, "S_SHIM_CNTRL_CH0"),
        ("M_FIFO_0", bslots::AIE_FIFO[0], "S_FIFO"),
    ] {
        bel = bel.extra_out_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for (pin, bslot, opin) in [
        ("S_CONTROL", bslots::AIE_TILECTRL, "M_CONTROL"),
        ("S_UCNTRL", bslots::AIE2PS_SHIM_CNTRL, "M_SHIM_CNTRL_CH0"),
        ("S_FIFO_0", bslots::AIE_FIFO[0], "M_FIFO"),
        ("S_TRACE", bslots::AIE_SHIMTRACE, "M_TRACE"),
    ] {
        bel = bel.extra_in_far(pin);
        bel.verify_net(&[bel.wire_far(pin), bel.bel_wire_far(bcrd.bel(bslot), opin)]);
    }
    for i in 0..4 {
        let pin_s = &format!("M_SOUTH_CH{i}");
        let pin_n = &format!("S_NORTH_CH{i}");
        bel = bel.extra_in_far(pin_n);
        let obel = edev
            .bel_delta(bcrd.cell, 0, 1, bslots::AIE_ML_ARRAYSWITCH)
            .or(edev.bel_delta(bcrd.cell, 0, 1, bslots::AIE_ML_MEM_ARRAYSWITCH))
            .unwrap();
        bel.verify_net(&[bel.wire_far(pin_n), bel.bel_wire_far(obel, pin_s)]);
    }
    for i in 0..6 {
        let pin_n = &format!("M_NORTH_CH{i}");
        bel = bel.extra_out_far_claim(pin_n);
    }
    for i in 0..4 {
        let pin_n = &format!("M_EAST_CH{i}");
        let pin_p = &format!("S_WEST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, -1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    for i in 0..4 {
        let pin_n = &format!("M_WEST_CH{i}");
        let pin_p = &format!("S_EAST_CH{i}");
        bel = bel.extra_out_far_claim(pin_n).extra_in_far(pin_p);
        if let Some(obel) = edev.bel_delta(bcrd.cell, 1, 0, bcrd.slot) {
            bel.verify_net(&[bel.wire_far(pin_p), bel.bel_wire_far(obel, pin_n)]);
        } else {
            bel.claim_net(&[bel.wire_far(pin_p)]);
        }
    }
    let aie_noc = bcrd.bel(bslots::AIE_NOC);
    let aie_pl = bcrd.bel(bslots::AIE_PL);
    for i in 0..8 {
        let pin = &format!("S_SOUTH_CH{i}");
        if matches!(i, 3 | 7) {
            bel = bel.extra_in_far_from_far(pin, aie_noc, format!("TO_AIE_{i}"));
        } else {
            bel = bel.extra_in_far_from_far(pin, aie_pl, format!("TO_AIE_{i}"));
        }
    }
    for i in 0..6 {
        let pin = &format!("M_SOUTH_CH{i}");
        if matches!(i, 1 | 3) {
            bel = bel.extra_out_far_to_far(pin, aie_noc, format!("FROM_AIE_{ii}", ii = i + 1));
        } else {
            bel = bel.extra_out_far_to_far(pin, aie_pl, format!("FROM_AIE_{i}"));
        }
    }
    bel.commit();
}

pub fn verify_aie_noc(endev: &ExpandedNamedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let edev = endev.edev;
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_SHIM_NOC_PL => "AIE_NOC",
        tcls::AIE_ML_SHIM_NOC_PL => "AIE_ML_NOC",
        tcls::AIE2PS_SHIM_NOC_PL => "AIE2PS_NOC",
        _ => unreachable!(),
    };
    let mut bel = vrf
        .verify_bel(bcrd)
        .kind(kind)
        .extra_out_far_claim("TO_AIE_2")
        .extra_out_far_claim("TO_AIE_3")
        .extra_out_far_claim("TO_AIE_6")
        .extra_out_far_claim("TO_AIE_7")
        .extra_in_far_claim("FROM_AIE_2")
        .extra_in_far_claim("FROM_AIE_3")
        .extra_in_far_claim("FROM_AIE_4")
        .extra_in_far_claim("FROM_AIE_5");
    let aie_pl = bcrd.bel(bslots::AIE_PL);
    if edev[tcrd].class == tcls::AIE2PS_SHIM_NOC_PL {
        bel = bel
            .extra_out_far_claim("M_AXI_ACLK")
            .extra_out_far_claim("S_AXI_ACLK")
            .extra_in_far_claim("FROM_PL_2")
            .extra_in_far_claim("FROM_PL_6")
            .extra_in_far_from_far("FROM_PL_3", aie_pl, "TO_AIE_3")
            .extra_in_far_from_far("FROM_PL_7", aie_pl, "TO_AIE_7")
            .extra_out_far_claim("TO_PL_3")
            .extra_out_far_claim("TO_PL_5")
            .extra_out_far_to_far("TO_PL_2", aie_pl, "FROM_AIE_1")
            .extra_out_far_to_far("TO_PL_4", aie_pl, "FROM_AIE_3")
            .extra_out_far_claim("TO_NOC");
        let idx = (bcrd.col.to_idx() - 2) % 4;
        if idx == 1 {
            let nsu = bcrd.delta(-(idx as i32), -1).bel(bslots::HNOC_ME_NSU[0]);
            bel = bel.extra_in_far_from_far("FROM_NOC", nsu, "AXI_OUT");
        } else {
            bel = bel.extra_in_far_claim("FROM_NOC");
        }
    } else {
        for i in 2..6 {
            let to_pl = &format!("TO_PL_{i}");
            let from_aie = &format!("FROM_AIE_{i}");
            bel = bel.extra_out_far_to_far(to_pl, aie_pl, from_aie);
        }
        for i in [2, 3, 6, 7] {
            let from_pl = &format!("FROM_PL_{i}");
            let to_aie = &format!("TO_AIE_{i}");
            bel = bel.extra_in_far_from_far(from_pl, aie_pl, to_aie);
        }
        bel = bel.extra_in_far("FROM_NOC").extra_out_far("TO_NOC");
        let idx = (bcrd.col.to_idx() - 1) % 4;
        if let Some(obel) =
            edev.bel_delta(bcrd.cell, -(idx as i32), -1, bslots::HNOC_ME_NMU[idx - 1])
        {
            bel.verify_net(&[bel.wire_far("TO_NOC"), bel.bel_wire_far(obel, "AXI_IN")]);
        } else {
            bel.claim_net(&[bel.wire_far("TO_NOC")]);
        }
        if let Some(obel) =
            edev.bel_delta(bcrd.cell, -(idx as i32), -1, bslots::HNOC_ME_NSU[idx - 1])
        {
            bel.verify_net(&[bel.wire_far("FROM_NOC"), bel.bel_wire_far(obel, "AXI_OUT")]);
        } else {
            bel.claim_net(&[bel.wire_far("FROM_NOC")]);
        }
    }
    if !endev.dev_naming.skip_bels.contains(&bcrd) {
        bel.commit();
    }
}

pub fn verify_aie_pl(endev: &ExpandedNamedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let edev = endev.edev;
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_SHIM_PL | tcls::AIE_SHIM_NOC_PL => "AIE_PL",
        tcls::AIE_ML_SHIM_PL | tcls::AIE_ML_SHIM_NOC_PL => "AIE_ML_PL",
        tcls::AIE2PS_SHIM_NOC_PL => "AIE2P_PL",
        _ => unreachable!(),
    };
    let mut bel = vrf.verify_bel(bcrd).kind(kind);
    for i in 0..8 {
        bel = bel.extra_out_far_claim(format!("TO_AIE_{i}"));
    }
    for i in 0..6 {
        bel = bel.extra_in_far_claim(format!("FROM_AIE_{i}"));
    }
    if !endev.dev_naming.skip_bels.contains(&bcrd) {
        bel.commit();
    }
}

pub fn verify_aie_pll(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let kind = match edev[tcrd].class {
        tcls::AIE_PLL => "AIE_PLL",
        tcls::AIE_ML_PLL => "AIE_ML_PLL",
        tcls::AIE2PS_PLL => "AIE2PS_PLL",
        _ => unreachable!(),
    };
    vrf.verify_bel(bcrd).kind(kind).commit();
}

pub fn verify_aie_intf_test(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::AIE_INTF_TEST_NC;
    let mut bel = vrf.verify_bel(bcrd).kind(if is_nc {
        "AIE_INTF_C_TEST"
    } else {
        "AIE_INTF_B_TEST"
    });
    if is_nc {
        bel = bel
            .extra_in_far_claim("PL_AIE_SCAN_PLL_RESET")
            .extra_in_far_claim("PL_AIE_SCAN_CLKDIV_RSTN")
            .extra_in_far_claim("PL_AIE_SCAN_CLK_N")
            .extra_in_far_claim("PL_AIE_SCAN_MODE_RST_N")
            .extra_in_far_claim("PL_AIE_SCAN_EN_N")
            .extra_in_far_claim("PL_AIE_SCAN_EDT_UPDT_N")
            .extra_in_far_claim_bus("PL_AIE_SCAN_CHNL_IN", 5)
            .extra_out_far_claim_bus("AIE_PL_SCAN_CHNL_OUT", 33)
            .extra_in_far_claim_bus("PL_AIE_SCAN_SPARE_IN", 2)
            .extra_out_far_claim_bus("AIE_PL_SCAN_SPARE_OUT", 2);
    }
    bel.commit();
}

pub fn verify_aie_ml_intf_test(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::AIE_ML_INTF_TEST_NC;
    let mut bel = vrf.verify_bel(bcrd).kind(if is_nc {
        "AIE_ML_INTF_C_TEST"
    } else {
        "AIE_ML_INTF_B_TEST"
    });
    if is_nc {
        bel = bel
            .extra_in_far_claim("PL_AIE_SCAN_CLK_N")
            .extra_in_far_claim("PL_AIE_SCAN_MODE_RST_N")
            .extra_in_far_claim("PL_AIE_SCAN_EN_N")
            .extra_in_far_claim("PL_AIE_SCAN_EDT_UPDT_N")
            .extra_in_far_claim_bus("PL_AIE_SCAN_CHNL_IN", 9)
            .extra_out_far_claim_bus("AIE_PL_SCAN_CHNL_OUT", 35);
    }
    bel.commit();
}

pub fn verify_aie2ps_intf_test(edev: &ExpandedDevice, vrf: &mut Verifier, bcrd: BelCoord) {
    let tcrd = edev.bel_tile(bcrd);
    let is_nc = edev[tcrd].class == tcls::AIE2PS_INTF_TEST_NC;
    let mut bel = vrf.verify_bel(bcrd).kind(if bcrd.col.to_idx() == 0 {
        "AIE2PS_INTF_A_TEST"
    } else {
        "AIE2PS_INTF_C_TEST"
    });
    if is_nc {
        bel = bel
            .extra_in_far_claim("PL_AIE_SCAN_CLK_N")
            .extra_in_far_claim("PL_AIE_SCAN_MODE_RST_N")
            .extra_in_far_claim("PL_AIE_SCAN_EN_N")
            .extra_in_far_claim("PL_AIE_SCAN_EDT_UPDT_N")
            .extra_in_far_claim_bus("PL_AIE_SCAN_CHNL_IN", 32)
            .extra_out_far_claim_bus("AIE_PL_SCAN_CHNL_OUT", 12);
    }
    bel.commit();
}
