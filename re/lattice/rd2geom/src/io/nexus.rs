use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, NexusKind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, nexus::tcls::CLK_S_JE5D15, tcls, tslots, wires},
};
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirV},
    grid::{BelCoord, CellCoord, DieIdExt},
};
use prjcombine_re_lattice_naming::WireName;

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    fn get_nexus_pio_suffix(&self, bcrd: BelCoord) -> &'static str {
        let edge = self.chip.get_io_edge(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let is_hp = edge == Dir::S;
        if is_hp {
            if matches!(self.chip.kind, ChipKind::Nexus(NexusKind::LargePcs(_))) {
                ["DIFFIO18A_CORE_IOA", "SEIO18A_CORE_IOB"][idx]
            } else {
                ["DIFFIO18_CORE_IOA", "SEIO18_CORE_IOB"][idx]
            }
        } else {
            ["SEIO33_CORE_IOA", "SEIO33_CORE_IOB"][idx]
        }
    }

    fn get_nexus_iol_suffix(&self, bcrd: BelCoord) -> &'static str {
        let edge = self.chip.get_io_edge(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let is_hp = edge == Dir::S;
        if is_hp {
            [
                "IOLOGIC_CORE_I_GEARING_PIC_TOP_A",
                "IOLOGIC_CORE_I_GEARING_PIC_TOP_B",
            ][idx]
        } else {
            ["SIOLOGIC_CORE_IBASE_PIC_A", "SIOLOGIC_CORE_IBASE_PIC_B"][idx]
        }
    }

    fn get_nexus_dqs_suffix(&self) -> &'static str {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        match kind {
            NexusKind::SmallMipi | NexusKind::SmallMipiPcie => "DQSBUF_CORE_I_DQS_TOP",
            NexusKind::LargePcs(_) => "DQSBUFA_CORE_I_DQS_TOP",
            NexusKind::SmallHalfIo => unreachable!(),
            NexusKind::SmallDenseIo => "DQSBUFB_CORE_I_DQS_TOP",
        }
    }

    fn get_nexus_dqs_cell(&self, bcrd: BelCoord) -> CellCoord {
        let tcrd = self.edev.bel_tile(bcrd);
        match self.edev[tcrd].class {
            tcls::DQS_S | tcls::DQS_L => tcrd.delta(-6, 0),
            tcls::DQS_JD5D | tcls::DQS_JD5D_HOLE => tcrd.delta(-4, 0),
            tcls::DQS_JD5F => tcrd.delta(1, 0),
            tcls::DQS_JD5F_HOLE => tcrd.delta(-3, 0),
            _ => unreachable!(),
        }
    }

    pub fn get_inck_wire_nexus(&self, bcrd: BelCoord) -> WireName {
        let iol = self.get_nexus_iol_suffix(bcrd);
        self.rc_io_wire(bcrd.cell, &format!("JINCK_{iol}"))
    }

    pub fn get_paddi_wire_nexus(&self, bcrd: BelCoord) -> WireName {
        let pio = self.get_nexus_pio_suffix(bcrd);
        self.rc_io_wire(bcrd.cell, &format!("JPADDI_{pio}"))
    }

    pub fn get_paddo_wire_nexus(&self, bcrd: BelCoord) -> WireName {
        let pio = self.get_nexus_pio_suffix(bcrd);
        self.rc_io_wire(bcrd.cell, &format!("JPADDO_{pio}"))
    }

    pub fn get_paddt_wire_nexus(&self, bcrd: BelCoord) -> WireName {
        let pio = self.get_nexus_pio_suffix(bcrd);
        self.rc_io_wire(bcrd.cell, &format!("JPADDT_{pio}"))
    }

    fn get_nexus_ddrdll_rc_mode(&self, bcrd: BelCoord) -> RcMode {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        if bcrd.col == self.chip.col_w()
            && matches!(kind, NexusKind::LargePcs(_) | NexusKind::SmallDenseIo)
        {
            RcMode::Io
        } else {
            RcMode::Base
        }
    }

    pub(super) fn prep_io_nexus(&mut self) {
        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::IO[0]) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                for i in 0..2 {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    let wname = self.get_inck_wire_nexus(bcrd);
                    self.bind_wire(wname, bcrd.wire(wires::OUT_IO_INCK[i]));
                    self.claim_node(wname);
                }
            }
        }

        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::DLLDEL[0]) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                let wname = self.rc_wire(tcrd.cell, "JCLKOUT_DLLDEL_CORE_I0");
                self.bind_wire(wname, tcrd.wire(wires::OUT_DLLDEL_CLKO[0]));
            }
        }
    }

    pub(super) fn process_eclk_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let (bank_base, num_banks) = match kind {
            NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs(_) => (3, 3),
            NexusKind::SmallHalfIo => (2, 3),
            NexusKind::SmallDenseIo => (5, 2),
        };
        let tcrd = self.chip.clk_emid(Dir::S).tile(tslots::CLK);
        for i in 0..num_banks {
            let cell = tcrd.delta(-(i as i32), 0);
            let bank = bank_base + i;
            {
                let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
                ex.cell(cell);
                for j in 0..4 {
                    ex.output_bind_rc(
                        format!("JMUXIN{j}_ECLKBANK_CORE_ECLKBANK{bank}"),
                        wires::IMUX_ECLKSYNC_ECLKI[i * 4 + j].cell(6),
                    )
                    .input_xlat_rc(format!("JECLKCIB{j}_ECLKBANK_CORE_ECLKBANK{bank}"));
                }
                for j in [0, 2] {
                    let cslot = CLK_S_JE5D15::DLLDEL[i * 2 + j / 2];
                    let wn = format!("JPCLKT{j}_ECLKBANK_CORE_ECLKBANK{bank}");
                    let wire = cslot.wire(wires::OUT_DLLDEL_CLKO[0]);
                    if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                        ex.input_xlat_rc_nudge(wn, cslot);
                    } else {
                        let wn = ex.ctx.rc_wire(cell, &wn);
                        ex.input_raw(wn, wire);
                    }
                }
                for j in [1, 3] {
                    let cslot = CLK_S_JE5D15::IO[i * 2 + j / 2];
                    let wn = format!("JPCLKT{j}_ECLKBANK_CORE_ECLKBANK{bank}");
                    let wire = cslot.wire(wires::OUT_IO_INCK[0]);
                    if ex.ctx.edev[tcrd].cells.contains_id(cslot) {
                        ex.input_io_rc(wn, wire);
                    } else {
                        let wn = ex.ctx.rc_wire(cell, &wn);
                        ex.input_raw(wn, wire);
                    }
                }
                for (hv, corner, idx) in [(DirHV::SW, "LL", 0), (DirHV::SE, "LR", 1)] {
                    for (pin, wire) in [
                        ("CLKOP", wires::OUT_PLL_CLKOP[0]),
                        ("CLKOS", wires::OUT_PLL_CLKOS[0]),
                        ("CLKOS2", wires::OUT_PLL_CLKOS2[0]),
                        ("CLKOS3", wires::OUT_PLL_CLKOS3[0]),
                        ("CLKOS4", wires::OUT_PLL_CLKOS4[0]),
                        ("CLKOS5", wires::OUT_PLL_CLKOS5[0]),
                    ] {
                        let wn = format!("J{corner}{pin}_ECLKBANK_CORE_ECLKBANK{bank}");
                        if ex.ctx.chip.try_bel_pll(hv, 0).is_some() {
                            ex.input_xlat_rc(wn);
                        } else {
                            let cslot = CLK_S_JE5D15::PLL[idx];
                            let wn = ex.ctx.rc_wire(cell, &wn);
                            ex.input_raw(wn, cslot.wire(wire));
                        }
                    }
                }
            }
            {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::ECLKBANK_TEST[i]), cell);
                bel.name_rc("ECLKBANK_CORE");
                for j in 0..4 {
                    bel.input_xlat_rc(
                        bcls::ECLKBANK_TEST::TESTINP[j],
                        format!("JTESTINP{j}_ECLKBANK_CORE_ECLKBANK{bank}"),
                    );
                }
            }
            for j in 0..4 {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::ECLKSYNC[i * 4 + j]), cell);
                bel.name_rc_suffix("ECLKSYNC_CORE", ["A", "B", "C", "D"][j])
                    .input_xlat_rc(
                        bcls::ECLKSYNC::ECLKI,
                        format!("JECLKIN_ECLKSYNC_CORE_ECLKSYNC{bank}_{j}"),
                    )
                    .input_xlat_rc(
                        bcls::ECLKSYNC::STOP,
                        format!("JSTOP_ECLKSYNC_CORE_ECLKSYNC{bank}_{j}"),
                    )
                    .output_bind_rc(
                        bcls::ECLKSYNC::ECLKO,
                        format!("JECLKOUT_ECLKSYNC_CORE_ECLKSYNC{bank}_{j}"),
                        wires::OUT_ECLKSYNC_ECLKO[i * 4 + j].cell(6),
                    )
                    .claim_pip("ECLKO", "ECLKI");
            }
        }
        for i in 0..num_banks {
            let cell = tcrd.delta(-(i as i32), 0);
            let bank = bank_base + i;
            {
                let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
                ex.cell(cell);
                let cslot = CLK_S_JE5D15::IO[i * 2];
                for j in 0..4 {
                    ex.output_bind_rc(
                        format!("JECLKOUT{j}_ECLKCASMUX_CORE_ECLKCASMUX{bank}"),
                        cslot.wire(wires::ECLK[j]),
                    )
                    .input_xlat_rc(format!("JECLKSYNCM{j}_ECLKCASMUX_CORE_ECLKCASMUX{bank}"));
                    let wn = ex.ctx.rc_wire(
                        cell,
                        &format!("JECLKCAS0M{j}_ECLKCASMUX_CORE_ECLKCASMUX{bank}"),
                    );
                    if i == 0 && num_banks == 2 {
                        ex.input_bind(wn, wires::OUT_ECLKSYNC_ECLKO[8 + j].cell(6));
                    } else {
                        ex.input_xlat(wn);
                    }
                    let wn = ex.ctx.rc_wire(
                        cell,
                        &format!("JECLKCAS1M{j}_ECLKCASMUX_CORE_ECLKCASMUX{bank}"),
                    );
                    if i == 1 && num_banks == 2 {
                        ex.input_bind(wn, wires::OUT_ECLKSYNC_ECLKO[8 + j].cell(6));
                    } else {
                        ex.input_xlat(wn);
                    }
                }
            }
            {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::ECLKCASMUX_TEST[i]), cell);
                bel.name_rc("ECLKCASMUX_CORE");
                for j in 0..2 {
                    bel.input_xlat_rc(
                        bcls::ECLKBANK_TEST::TESTINP[j],
                        format!("JTESTINP{j}_ECLKCASMUX_CORE_ECLKCASMUX{bank}"),
                    );
                }
            }
            for j in 0..4 {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::ECLKDIV[i * 4 + j]), cell);
                bel.name_rc_suffix("ECLKDIV_CORE", ["A", "B", "C", "D"][j])
                    .input_xlat_rc_nudge(
                        bcls::ECLKDIV::ECLKIN,
                        format!("JECLKIN_ECLKDIV_CORE_ECLKDIVB{bank}_{j}"),
                        CLK_S_JE5D15::IO[i * 2],
                    )
                    .input_xlat_rc(
                        bcls::ECLKDIV::SLIP,
                        format!("JSLIP_ECLKDIV_CORE_ECLKDIVB{bank}_{j}"),
                    )
                    .input_xlat_rc(
                        bcls::ECLKDIV::DIVRST,
                        format!("JDIVRST_ECLKDIV_CORE_ECLKDIVB{bank}_{j}"),
                    )
                    .output_bind_rc(
                        bcls::ECLKDIV::DIVOUT,
                        format!("JDIVOUT_ECLKDIV_CORE_ECLKDIVB{bank}_{j}"),
                        wires::OUT_ECLKDIV_DIVOUT[i * 4 + j].cell(6),
                    );
                for k in 0..4 {
                    bel.input_xlat_rc(
                        bcls::ECLKDIV::TESTINP[k],
                        format!("JTESTINP{k}_ECLKDIV_CORE_ECLKDIVB{bank}_{j}"),
                    );
                }
            }
        }
        for (i, cell, lr) in [(0, tcrd.delta(-2, 0), 'L'), (1, tcrd.cell, 'R')] {
            let cslot = CLK_S_JE5D15::DDRDLL[i];
            let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
            ex.cell(cell);
            let (bidx0, bidx1) = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs(_) => {
                    ([2, 0][i], 1)
                }
                NexusKind::SmallHalfIo => [(1, 2), (0, 1)][i],
                NexusKind::SmallDenseIo => (0, 1),
            };
            let cslot_b0 = CLK_S_JE5D15::IO[2 * bidx0];
            let cslot_b1 = CLK_S_JE5D15::IO[2 * bidx1];
            for j in 0..4 {
                ex.output_bind_rc(
                    format!("JCLKOUT_ECLKDDR{lr}_{j}"),
                    cslot.wire(wires::ECLK_TO_DDRDLL[j]),
                )
                .input_xlat_rc_nudge(format!("JECLKTREE0_ECLKDDR{lr}_{j}"), cslot_b0)
                .input_xlat_rc_nudge(format!("JECLKTREE1_ECLKDDR{lr}_{j}"), cslot_b1);
            }
        }
        for (i, h, cell, lr) in [
            (0, DirH::W, tcrd.delta(-3, 0), 'L'),
            (1, DirH::E, tcrd.delta(1, 0), 'R'),
        ] {
            if self.chip.try_bel_pll(h & DirV::S, 0).is_none() {
                continue;
            }
            {
                let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
                ex.cell(cell).output_bind_rc(
                    format!("JECLKOUT_ECLKLOGICMUXPLLFB_CORE_ECLKPLLFB{lr}"),
                    CLK_S_JE5D15::PLL[i].wire(wires::ECLKFB),
                );
                for (hv, corner, idx) in [(DirHV::SW, "LL", 0), (DirHV::SE, "LR", 1)] {
                    for (pin, wire) in [
                        ("CLKOP", wires::OUT_PLL_CLKOP[0]),
                        ("CLKOS", wires::OUT_PLL_CLKOS[0]),
                        ("CLKOS2", wires::OUT_PLL_CLKOS2[0]),
                        ("CLKOS3", wires::OUT_PLL_CLKOS3[0]),
                        ("CLKOS4", wires::OUT_PLL_CLKOS4[0]),
                        ("CLKOS5", wires::OUT_PLL_CLKOS5[0]),
                    ] {
                        let wn = format!("J{corner}{pin}_ECLKLOGICMUXPLLFB_CORE_ECLKPLLFB{lr}");
                        if ex.ctx.chip.try_bel_pll(hv, 0).is_some() {
                            ex.input_xlat_rc(wn);
                        } else {
                            let cslot = CLK_S_JE5D15::PLL[idx];
                            let wn = ex.ctx.rc_wire(cell, &wn);
                            ex.input_raw(wn, cslot.wire(wire));
                        }
                    }
                }
            }
            {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::ECLKPLLFB_TEST[i]), cell);
                bel.name_rc("ECLKLOGICMUXPLLFB_CORE");
                for j in 0..4 {
                    bel.input_xlat_rc(
                        bcls::ECLKBANK_TEST::TESTINP[j],
                        format!("JTESTINP{j}_ECLKLOGICMUXPLLFB_CORE_ECLKPLLFB{lr}"),
                    );
                }
            }
        }
        self.extract_bel_cell(tcrd.bel(bslots::ECLK_TEST), tcrd.delta(-1, 0))
            .name_rc("ECLKCLKTEST_CORE")
            .add_wire_rc("NC", "JNC_ECLKCLKTEST_CORE_ACLKTEST");
    }

    pub(super) fn process_ddrdll_nexus(&mut self) {
        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::DDRDLL) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::DDRDLL);
                let mode = self.get_nexus_ddrdll_rc_mode(bcrd);
                self.extract_switchbox(bcrd.bel(bslots::BEL_INT))
                    .output_bind_rc_mode("JCLKOUT_I0", mode, wires::IMUX_DDRDLL_CLK.cell(0))
                    .input_xlat_rc_mode("JPCLK_I0", mode)
                    .input_xlat_rc_mode("JECLK0_I0", mode)
                    .input_xlat_rc_mode("JECLK1_I0", mode)
                    .input_xlat_rc_mode("JECLK2_I0", mode)
                    .input_xlat_rc_mode("JECLK3_I0", mode);
                let mut bel = self.extract_bel(bcrd);
                bel.name_rc("DDRDLL_CORE").suffix("DDRDLL_CORE_I1");
                for i in 0..9 {
                    bel.add_wire_rc_mode(
                        format!("CODE{i}"),
                        format!("JCODE{i}_DDRDLL_CORE_I1"),
                        mode,
                    );
                }
            }
        }
    }

    pub(super) fn process_dlldel_nexus(&mut self) {
        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::DLLDEL[0]) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::DLLDEL[0]);
                let mut bel = self.extract_bel(bcrd);
                bel.name_rc("DLLDEL_CORE")
                    .input_io_rc(
                        bcls::DLLDEL::CLKIN,
                        "JCLKIN_DLLDEL_CORE_I0",
                        wires::IO_DLLDEL.cell(0),
                    )
                    .suffix("DLLDEL_CORE_I0")
                    .claim_pip("CLKOUT", "CLKIN");
                if matches!(bel.ctx.chip.kind, ChipKind::Nexus(NexusKind::SmallHalfIo))
                    && tcrd.cell == bel.ctx.chip.special_loc[&SpecialLocKey::DllDel(Dir::N, 2)]
                {
                    // ????? on this particular chip, this particular DLLDEL has an edge from both clock 1 and 2?
                    // is this real or a bug?
                    let bcrd = bel.ctx.chip.special_io[&SpecialIoKey::Clock(Dir::N, 1)];
                    let wire_inck = bel.ctx.get_inck_wire_nexus(bcrd);
                    let wire_paddi = bel.ctx.get_paddi_wire_nexus(bcrd);
                    bel.claim_pip_from("CLKIN", wire_inck)
                        .claim_pip_from("CLKIN", wire_paddi);
                }
                for i in 0..9 {
                    bel.add_wire_rc(format!("CODE{i}"), format!("JCODE{i}_DLLDEL_CORE_I0"))
                        .add_wire_rc(format!("CODE{i}_MUX"), format!("JZ_I4_{i}"))
                        .add_wire_rc(format!("CODE{i}_I0"), format!("JD0_I4_{i}"))
                        .add_wire_rc(format!("CODE{i}_I1"), format!("JD1_I4_{i}"))
                        .claim_pip(format!("CODE{i}"), format!("CODE{i}_MUX"))
                        .claim_pip(format!("CODE{i}_MUX"), format!("CODE{i}_I0"))
                        .claim_pip(format!("CODE{i}_MUX"), format!("CODE{i}_I1"));
                }
                for (i, h) in DirH::DIRS.into_iter().enumerate() {
                    let bcrd = bel.ctx.chip.special_loc[&SpecialLocKey::DdrDll(h & DirV::S)]
                        .bel(bslots::DDRDLL);
                    let mode = bel.ctx.get_nexus_ddrdll_rc_mode(bcrd);
                    for j in 0..9 {
                        let wire = bel.ctx.rc_wire_mode(
                            bcrd.cell,
                            &format!("JCODE{j}_DDRDLL_CORE_I1"),
                            mode,
                        );
                        bel.claim_pip_from(format!("CODE{j}_I{i}"), wire);
                    }
                }
            }
        }
    }

    fn process_dqs_nexus(&mut self, bcrd: BelCoord) {
        let bcrd_io = bcrd.bel(bslots::IO[0]);
        let dqs = self.get_nexus_dqs_suffix();
        let cell = self.get_nexus_dqs_cell(bcrd);
        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            ex.cell(cell)
                .output_bind_rc(
                    "JECLKOUT_I_DQS_TOP_ECLK_ROUTING_MUX",
                    wires::IMUX_DQS_ECLK.cell(0),
                )
                .input_xlat_rc("JECLK0_I_DQS_TOP_ECLK_ROUTING_MUX")
                .input_xlat_rc("JECLK1_I_DQS_TOP_ECLK_ROUTING_MUX")
                .input_xlat_rc("JECLK2_I_DQS_TOP_ECLK_ROUTING_MUX")
                .input_xlat_rc("JECLK3_I_DQS_TOP_ECLK_ROUTING_MUX");
        }
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let dqs_kind = match kind {
            NexusKind::SmallMipi | NexusKind::SmallMipiPcie => "DQSBUF_CORE",
            NexusKind::LargePcs(_) => "DQSBUFA_CORE",
            NexusKind::SmallHalfIo => unreachable!(),
            NexusKind::SmallDenseIo => "DQSBUFB_CORE",
        };
        let wire_inck = self.get_inck_wire_nexus(bcrd_io);
        let wire_paddi = self.get_paddi_wire_nexus(bcrd_io);
        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.name_rc(dqs_kind)
            .suffix(dqs)
            .add_wire_rc("DQSI", format!("JDQSI_{dqs}"))
            .claim_pip_from("DQSI", wire_inck)
            .claim_pip_from("DQSI", wire_paddi);
        for i in 0..9 {
            bel.add_wire_rc(format!("DLLCODE{i}"), format!("JDLLCODE{i}_{dqs}"))
                .add_wire_rc(
                    format!("DLLCODE{i}_MUX"),
                    format!("JSELDLLCD{i}_I_DQS_TOP_DLL_CODE_ROUTING_MUX"),
                )
                .add_wire_rc(
                    format!("DLLCODE{i}_I0"),
                    format!("JCODEI0{i}_I_DQS_TOP_DLL_CODE_ROUTING_MUX"),
                )
                .add_wire_rc(
                    format!("DLLCODE{i}_I1"),
                    format!("JCODEI1{i}_I_DQS_TOP_DLL_CODE_ROUTING_MUX"),
                )
                .claim_pip(format!("DLLCODE{i}"), format!("DLLCODE{i}_MUX"))
                .claim_pip(format!("DLLCODE{i}_MUX"), format!("DLLCODE{i}_I0"))
                .claim_pip(format!("DLLCODE{i}_MUX"), format!("DLLCODE{i}_I1"));
        }
        for (i, h) in DirH::DIRS.into_iter().enumerate() {
            let bcrd =
                bel.ctx.chip.special_loc[&SpecialLocKey::DdrDll(h & DirV::S)].bel(bslots::DDRDLL);
            let mode = bel.ctx.get_nexus_ddrdll_rc_mode(bcrd);
            for j in 0..9 {
                let wire =
                    bel.ctx
                        .rc_wire_mode(bcrd.cell, &format!("JCODE{j}_DDRDLL_CORE_I1"), mode);
                bel.claim_pip_from(format!("DLLCODE{j}_I{i}"), wire);
            }
        }
        for pin in [
            "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1", "WRPNTR2", "DQSW", "DQSW270",
            "DQSR90",
        ] {
            bel.add_wire_rc(pin, format!("J{pin}_{dqs}"));
        }
    }

    pub(super) fn process_ivref_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        if !matches!(kind, NexusKind::LargePcs(_)) {
            return;
        }
        let mut banks = vec![];
        let mut dqs = BTreeMap::new();
        for (col, cd) in &self.chip.columns {
            if cd.io[DirV::S] == IoGroupKind::DoubleDqs {
                let bank = cd.bank[DirV::S];
                let cell = Chip::DIE.cell(col, self.chip.row_s());
                match dqs.entry(bank) {
                    btree_map::Entry::Vacant(e) => {
                        e.insert(vec![cell]);
                        banks.push(bank);
                    }
                    btree_map::Entry::Occupied(mut e) => {
                        e.get_mut().push(cell);
                    }
                }
            }
        }
        let mut idx = 1;
        for bank in banks {
            let cells_dqs = &dqs[&bank];
            let bcrd = self.chip.special_loc[&SpecialLocKey::IVref(bank)].bel(bslots::IVREF);
            let mut names = vec![];
            for _ in 0..cells_dqs.len() {
                names.push(format!("IVREF_{idx}"));
                idx += 1;
            }
            let mut bel = self.extract_bel(bcrd);
            bel.names(names);
            for (j, &cell_dqs) in cells_dqs.iter().enumerate() {
                let cell = bcrd.delta(
                    match (bank, j) {
                        (5, _) => j as i32,
                        (4, 0) => 0,
                        (4, 1) => -1,
                        (4, 2) => 3,
                        (4, 3) => 4,
                        (3, 0) => 2,
                        (3, 1) => -1,
                        (3, 2) => -2,
                        (3, 3) => -3,
                        _ => unreachable!(),
                    },
                    0,
                );
                let suffix = if bank == 4 && j == 0 {
                    "IVREF_CORE_BNKVRGEN180"
                } else {
                    "IVREF_CORE"
                };
                bel.cell = cell;
                for k in 0..7 {
                    bel.input_xlat_rename_rc(
                        bcls::IVREF::IVREFI[k],
                        format!("DQS{j}_IVREFI{k}"),
                        format!("JIVREFI{k}_{suffix}"),
                    );
                }
                let wname =
                    bel.add_wire_rc_get(format!("DQS{j}_IVREFO"), format!("JIVREFO_{suffix}"));
                bel.ctx.ivref_map.insert(cell_dqs, wname);
            }
        }
    }

    fn process_single_io_nexus(&mut self, bcrd: BelCoord) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let is_jtag_en = bcrd == self.chip.special_io[&SpecialIoKey::JtagEn];
        let edge = self.chip.get_io_edge(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let ab = ["A", "B"][idx];
        let (r, c) = self.rc(bcrd.cell);
        let names = match edge {
            Dir::W => [format!("PL{r}{ab}"), format!("IOL_L{r}{ab}")],
            Dir::E => [format!("PR{r}{ab}"), format!("IOL_R{r}{ab}")],
            Dir::S => [format!("PB{c}{ab}"), format!("IOL_B{c}{ab}")],
            Dir::N => [format!("PT{c}{ab}"), format!("IOL_T{c}{ab}")],
        };
        let tcrd = self.edev.bel_tile(bcrd);
        let is_lpddr4 = self.edev[tcrd].class == tcls::IO_HP_S_LPDDR4;
        let is_jd5u = self.edev[tcrd].class == tcls::IO_HP_S_JD5U;
        let skip_hpio_pip = matches!(kind, NexusKind::SmallDenseIo | NexusKind::SmallHalfIo);
        let is_hp = edge == Dir::S;
        let iol = if is_hp {
            format!("IOLOGIC_CORE_I_GEARING_PIC_TOP_{ab}")
        } else {
            format!("SIOLOGIC_CORE_IBASE_PIC_{ab}")
        };
        let pio = self.get_nexus_pio_suffix(bcrd);
        let mut bel = self.extract_bel(bcrd);
        if is_jtag_en {
            bel.name(&names[1]);
        } else {
            bel.names(names);
        };
        bel.input_xlat_rename_rc_io(bcls::IO::SCLK, "SCLKIN", format!("JSCLKIN_{iol}"))
            .input_xlat_rename_rc_io(bcls::IO::SCLK, "SCLKOUT", format!("JSCLKOUT_{iol}"))
            .input_xlat_rename_rc_io(bcls::IO::LSR, "LSRIN", format!("JLSRIN_{iol}"))
            .input_xlat_rename_rc_io(bcls::IO::LSR, "LSROUT", format!("JLSROUT_{iol}"))
            .input_xlat_rename_rc_io(bcls::IO::CE, "CEIN", format!("JCEIN_{iol}"))
            .input_xlat_rename_rc_io(bcls::IO::CE, "CEOUT", format!("JCEOUT_{iol}"))
            .output_xlat_rc_io(bcls::IO::INDD, format!("JINDD_{iol}"))
            .output_xlat_rc_io(bcls::IO::INFF, format!("JINFF_{iol}"))
            .output_xlat_rc_io(bcls::IO::INCK, format!("JINCK_{iol}"))
            .add_wire_rc_io("DI", format!("JDI_{iol}"))
            .add_wire_rc_io("TOUT", format!("JTOUT_{iol}"))
            .add_wire_rc_io("DOUT", format!("JDOUT_{iol}"));
        if !is_jtag_en {
            bel.input_xlat_rename_rc_io(bcls::IO::TXDATA[0], "PADDO", format!("JPADDO_{pio}"))
                .input_xlat_rename_rc_io(bcls::IO::TSDATA[0], "PADDT", format!("JPADDT_{pio}"))
                .output_xlat_rename_rc_io(bcls::IO::INDD, "PADDI", format!("JPADDI_{pio}"))
                .add_wire_rc_io("IOPAD", format!("JIOPAD_{pio}"))
                .claim_pip("DI", "PADDI")
                .claim_pip("PADDT", "TOUT")
                .claim_pip("PADDO", "DOUT");
        }
        let (num_d, num_t) = if is_hp { (10, 4) } else { (2, 1) };
        for i in 0..num_d {
            bel.input_xlat_rc_io(bcls::IO::TXDATA[i], format!("JTXDATA{i}_{iol}"))
                .output_xlat_rc_io(bcls::IO::RXDATA[i], format!("JRXDATA{i}_{iol}"));
        }
        for i in 0..num_t {
            bel.input_xlat_rc_io(bcls::IO::TSDATA[i], format!("JTSDATA{i}_{iol}"));
        }
        if !(is_hp && skip_hpio_pip) {
            bel.claim_pip("INDD", "DI")
                .claim_pip("DOUT", "TXDATA0")
                .claim_pip("TOUT", "TSDATA0");
        }
        if is_hp {
            bel.input_xlat_rc_io(bcls::IO::ECLK, format!("JECLK_{iol}"))
                .input_xlat_rc_io(bcls::IO::MOVE, format!("JMOVE_{iol}"))
                .input_xlat_rc_io(bcls::IO::LOAD_N, format!("JLOAD_N_{iol}"))
                .input_xlat_rc_io(bcls::IO::DIR, format!("JDIR_{iol}"))
                .output_xlat_rc_io(bcls::IO::COUT, format!("JCOUT_{iol}"))
                .output_xlat_rc_io(bcls::IO::EDETERR, format!("JEDETERR_{iol}"))
                .input_xlat_rc_io(bcls::IO::WORDALIGN, format!("JWORDALIGN_{iol}"))
                .input_xlat_rc_io(bcls::IO::RANKENABLE, format!("JRANKENABLE_{iol}"))
                .input_xlat_rc_io(bcls::IO::RANK0UPDATE, format!("JRANK0UPDATE_{iol}"))
                .input_xlat_rc_io(bcls::IO::RANK1UPDATE, format!("JRANK1UPDATE_{iol}"))
                .input_xlat_rc_io(bcls::IO::RANKSELECT, format!("JRANKSELECT_{iol}"))
                .input_xlat_rc_io(bcls::IO::CIBCRS[0], format!("JCIBCRS0_{iol}"))
                .input_xlat_rc_io(bcls::IO::CIBCRS[1], format!("JCIBCRS1_{iol}"))
                .input_xlat_rc_io(bcls::IO::TDOLP, format!("JTDOLP_{iol}"))
                .input_xlat_rename_rc_io(bcls::IO::TDOLP, "DOLP_PIO", format!("JDOLP_{pio}"))
                .add_wire_rc_io("DOLP", format!("JDOLP_{iol}"))
                .claim_pip("DOLP_PIO", "DOLP")
                .output_xlat_rename_rc_io(bcls::IO::INDD, "INLP", format!("JINLP_{pio}"))
                .add_wire_rc_io("INLP_IOL", format!("JINLP_{iol}"))
                .claim_pip("INLP_IOL", "INLP")
                .add_wire_rc_io("INADC", format!("JINADC_{pio}"));
            if !is_jd5u {
                bel.input_xlat_rc_io(bcls::IO::CIBTSTSG, format!("JCIBTSTSG_{iol}"));
            } else {
                bel.add_wire_rc_io("CIBTSTSG", format!("JCIBTSTSG_{iol}"));
            }
            if is_lpddr4 {
                bel.output_xlat_rc_io(bcls::IO::COUT_LPDDR4_WT, format!("JCOUT_LPDDR4_WT_{iol}"))
                    .input_xlat_rc_io(bcls::IO::MOVE_LPDDR4_WT, format!("JMOVE_LPDDR4_WT_{iol}"))
                    .input_xlat_rc_io(
                        bcls::IO::LOAD_N_LPDDR4_WT,
                        format!("JLOAD_N_LPDDR4_WT_{iol}"),
                    )
                    .input_xlat_rc_io(
                        bcls::IO::DIRECTION_LPDDR4_WT,
                        format!("JDIRECTION_LPDDR4_WT_{iol}"),
                    );
            }
            if idx == 0 {
                bel.input_xlat_rc_io(bcls::IO::HSTXEN, format!("JHSTXEN_{pio}"))
                    .input_xlat_rc_io(bcls::IO::HSRXEN, format!("JHSRXEN_{pio}"));
            }
            for pin in [
                "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1", "WRPNTR2", "DQSW",
                "DQSW270", "DQSR90",
            ] {
                bel.add_wire_rc_io(pin, format!("J{pin}_{iol}"));
                if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd) {
                    let cell_dqs = bel.ctx.get_nexus_dqs_cell(bcrd_dqs);
                    let dqs = bel.ctx.get_nexus_dqs_suffix();
                    let wire_dqs = bel.ctx.rc_wire(cell_dqs, &format!("J{pin}_{dqs}"));
                    bel.claim_pip_from(pin, wire_dqs);
                }
            }
            if matches!(kind, NexusKind::LargePcs(_)) {
                bel.add_wire_rc_io("IVREF_IN", format!("JIVREF_IN_{pio}"));
                if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd)
                    && let Some(&wire) = bel.ctx.ivref_map.get(&bcrd_dqs.cell)
                {
                    bel.claim_pip_from("IVREF_IN", wire);
                }
            }
        } else {
            if !is_jtag_en {
                bel.input_xlat_rc_io(bcls::IO::I3CWKPU, format!("JI3CWKPU_{pio}"))
                    .input_xlat_rc_io(bcls::IO::I3CRESEN, format!("JI3CRESEN_{pio}"));
            } else if bel.ctx.chip.kind == ChipKind::Nexus(NexusKind::SmallMipi) {
                bel.input(bcls::IO::I3CWKPU, wires::IMUX_MUXA[7].cell(0))
                    .input(bcls::IO::I3CRESEN, wires::IMUX_MUXB[1].cell(0));
            } else {
                bel.input(bcls::IO::I3CWKPU, wires::IMUX_MUXB[2].cell(0))
                    .input(bcls::IO::I3CRESEN, wires::IMUX_MUXA[2].cell(0));
            }
        }
    }

    pub(super) fn process_io_nexus(&mut self) {
        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::DQS[0]) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                self.process_dqs_nexus(tcrd.bel(bslots::DQS[0]));
            }
        }
        for (tcid, tcls) in &self.intdb.tile_classes {
            if !tcls.bels.contains_id(bslots::IO[0]) {
                continue;
            }
            for &tcrd in &self.edev.tile_index[tcid] {
                if tcls.bels.contains_id(bslots::IO_INT) {
                    let mut ex = self.extract_switchbox(tcrd.bel(bslots::IO_INT));
                    ex.output_bind_rc_io("JECLKOUT_I217", wires::IMUX_IO_ECLK[0].cell(0))
                        .output_bind_rc_io("JECLKOUT_I218", wires::IMUX_IO_ECLK[1].cell(0))
                        .input_xlat_rc_io("JECLKIN0_I217")
                        .input_xlat_rc_io("JECLKIN1_I217")
                        .input_xlat_rc_io("JECLKIN2_I217")
                        .input_xlat_rc_io("JECLKIN3_I217")
                        .input_xlat_rc_io("JECLKIN0_I218")
                        .input_xlat_rc_io("JECLKIN1_I218")
                        .input_xlat_rc_io("JECLKIN2_I218")
                        .input_xlat_rc_io("JECLKIN3_I218");
                }
                for i in 0..2 {
                    self.process_single_io_nexus(tcrd.bel(bslots::IO[i]));
                }
            }
        }
    }

    pub(super) fn process_bnkref_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let bcrd = cell.bel(bslots::BNKREF);
            let (is_hp, suffix) = match (kind, bank) {
                (
                    NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs(_),
                    0..=2 | 6..=7,
                ) => (false, "BNKREF33_CORE"),
                (NexusKind::SmallMipi, 3..=5) => (true, "BNKREF18_CORE_BNKREF180"),
                (NexusKind::SmallMipiPcie, 3..=5) => (true, "BNKREF18_CORE"),
                (NexusKind::LargePcs(_), 4) => (true, "BNKREF18_CORE_BNKREF180"),
                (NexusKind::LargePcs(_), 3 | 5) => (true, "BNKREF18_CORE"),
                (NexusKind::SmallHalfIo, 0..=1 | 5) => (false, "BNKREF33_CORE"),
                (NexusKind::SmallHalfIo, 2..=4) => (true, "BNKREF18_CORE"),
                (NexusKind::SmallDenseIo, 0..=4 | 7..=8) => (false, "BNKREF33_CORE"),
                (NexusKind::SmallDenseIo, 9) => (false, "BNKREF33_CORE_BNKREF330"),
                (NexusKind::SmallDenseIo, 5 | 6) => (true, "BNKREF18_CORE"),
                _ => unreachable!(),
            };
            let cell = match (kind, bank) {
                (NexusKind::SmallMipi, 0) => bcrd.step(Dir::E, 1),
                (NexusKind::SmallMipi, 1) => bcrd.step(Dir::S, 1),
                (NexusKind::SmallMipi, 3 | 5) => bcrd.step(Dir::W, 16),
                (NexusKind::SmallMipi, 4) => bcrd.step(Dir::W, 10),
                _ => bcrd.cell,
            };
            self.extract_bel_cell(bcrd, cell)
                .name_rc(if is_hp {
                    "BNKREF18_CORE"
                } else {
                    "BNKREF33_CORE"
                })
                .suffix(suffix);
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::PvtCal33];
            let (_, c) = self.rc(cell);
            let mut bel = self.extract_bel(cell.bel(bslots::PVTCAL));
            if matches!(kind, NexusKind::SmallMipi | NexusKind::SmallMipiPcie) {
                bel.name_rc("PVTCAL33_CORE");
            } else {
                bel.name(format!("TPVTCAL33_CORE{c}"));
            }
            bel.suffix("PVTCAL33_CORE");
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::PvtCal18];
            let (r, _) = self.rc(cell);
            let mut bel = self.extract_bel(cell.bel(bslots::PVTCAL));
            if matches!(kind, NexusKind::LargePcs(_) | NexusKind::SmallDenseIo) {
                bel.name(format!("LPVTCAL18_CORE{r}"));
            } else {
                bel.name_rc("PVTCAL18_CORE");
            }
            bel.suffix(if kind == NexusKind::SmallMipi {
                "PVTCAL18_CORE"
            } else {
                "PVTCAL18_CORE_PVTCAL0"
            });
        }
    }

    pub(super) fn process_cdr_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        if let Some(&cell_tile) = self.chip.special_loc.get(&SpecialLocKey::Cdr) {
            let cdrs = match kind {
                NexusKind::SmallMipi => [cell_tile.delta(1, 0), cell_tile],
                NexusKind::SmallMipiPcie | NexusKind::LargePcs(_) => {
                    [cell_tile.delta(0, 2), cell_tile.delta(0, 1)]
                }
                NexusKind::SmallHalfIo => unreachable!(),
                NexusKind::SmallDenseIo => [cell_tile.delta(0, 3), cell_tile.delta(0, 1)],
            };
            for (i, cell) in cdrs.into_iter().enumerate() {
                let bcrd = cell_tile.bel(bslots::CDR[i]);
                let (r, c) = self.rc(cell);
                let (name, mode) = if kind == NexusKind::SmallMipi {
                    (format!("SGMIICDR_CORE_R{r}C{c}"), RcMode::Base)
                } else {
                    (format!("LSGMIICDR_CORE{r}"), RcMode::Io)
                };
                let wire_io =
                    self.get_paddi_wire_nexus(self.chip.special_io[&SpecialIoKey::CdrRx(i as u8)]);
                self.extract_bel_cell(bcrd, cell)
                    .name(name)
                    .suffix("SGMIICDR_CORE")
                    .output_bind_rc_mode(
                        bcls::CDR::SRCLK,
                        "JSRCLK_SGMIICDR_CORE",
                        mode,
                        wires::OUT_CDR_SRCLK[i].cell(0),
                    )
                    .add_wire_rc_mode("SGMIIIN", "JSGMIIIN_SGMIICDR_CORE", mode)
                    .claim_pip_from("SGMIIIN", wire_io);
            }
        }
    }

    pub(super) fn process_adc_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let (inadca, inadcb) = if matches!(kind, NexusKind::LargePcs(_)) {
            ("JINADC_DIFFIO18A_CORE_IOA", "JINADC_SEIO18A_CORE_IOB")
        } else {
            ("JINADC_DIFFIO18_CORE_IOA", "JINADC_SEIO18_CORE_IOB")
        };
        if let Some(&cell_tile) = self.chip.special_loc.get(&SpecialLocKey::Adc) {
            let cell = cell_tile;
            let bcrd = cell_tile.bel(bslots::ADC);
            let (r, c) = self.rc(cell);
            let mut names = vec![
                format!("ADC_CORE_R{r}C{c}"),
                "ADC_DP0".to_string(),
                "ADC_DN0".to_string(),
                "ADC_DP1".to_string(),
                "ADC_DN1".to_string(),
            ];
            if kind == NexusKind::SmallMipi {
                // what.
                names.push(format!("APIO_R{r}C{c}E"));
            }
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.names(names).suffix("ADC_CORE_ADC0");
            for i in 0..16 {
                bel.add_wire_rc(format!("GPIOP{i}"), format!("JGPIOP{i}_ADC_CORE_ADC0"))
                    .add_wire_rc(format!("GPION{i}"), format!("JGPION{i}_ADC_CORE_ADC0"));
                if let Some(&io) = bel.ctx.chip.special_io.get(&SpecialIoKey::AdcCh(i)) {
                    let wire_a = bel.ctx.rc_io_wire(io.cell, inadca);
                    let wire_b = bel.ctx.rc_io_wire(io.cell, inadcb);
                    bel.claim_pip_from(format!("GPIOP{i}"), wire_a)
                        .claim_pip_from(format!("GPION{i}"), wire_b);
                } else if i == 4 {
                    // ?????? JE5D15 only
                    bel.add_wire_rc("GPION4_APIO", "JPAD_APIO_GPION4")
                        .claim_pip("GPION4", "GPION4_APIO")
                        .claim_pip("GPION4_APIO", "GPION4");
                }
            }
            for i in 1..=3 {
                let io = bel.ctx.chip.special_io[&SpecialIoKey::AdcCompI(i)];
                let wire_a = bel.ctx.rc_io_wire(io.cell, inadca);
                let wire_b = bel.ctx.rc_io_wire(io.cell, inadcb);
                bel.add_wire_rc(format!("COMP{i}IP"), format!("JCOMP{i}IP_ADC_CORE_ADC0"))
                    .add_wire_rc(format!("COMP{i}IN"), format!("JCOMP{i}IN_ADC_CORE_ADC0"))
                    .claim_pip_from(format!("COMP{i}IP"), wire_a)
                    .claim_pip_from(format!("COMP{i}IN"), wire_b);
            }
            for pin in ["DP0", "DN0", "DP1", "DN1"] {
                let pin_apio = &format!("{pin}_APIO");
                bel.add_wire_rc(pin, format!("J{pin}_ADC_CORE_ADC0"))
                    .add_wire_rc(pin_apio, format!("JPAD_APIO_{pin}"))
                    .claim_pip(pin, pin_apio)
                    .claim_pip(pin_apio, pin);
            }
        }
    }
}
