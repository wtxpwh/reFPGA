use std::collections::{BTreeMap, BTreeSet};

use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bslots, cslots, rslots, tcls, tslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{
        BelInfo, ConnectorWire, Mux, PermaBuf, ProgBuf, ProgDelay, SwitchBox, SwitchBoxItem,
        TileClassId, WireKind, WireSlotId, WireSlotIdExt,
    },
    grid::CellCoord,
};
use prjcombine_re_lattice_naming::WireName;
use prjcombine_types::bitrect::PolTileBit;

use crate::ChipContext;

impl ChipContext<'_> {
    fn collect_term_pips(&mut self) -> BTreeSet<(WireName, WireName)> {
        let mut term_pips = BTreeSet::new();
        // collect expected term pips.
        let mut out_w = vec![];
        let mut out_e = vec![];
        if self.chip.kind.has_out_f_branch() {
            out_w.extend([
                (wires::OUT_F_W[0], wires::OUT_F[0]),
                (wires::OUT_F_W[1], wires::OUT_F[1]),
                (wires::OUT_F_W[2], wires::OUT_F[2]),
            ]);
            out_e.extend([
                (wires::OUT_F_E[4], wires::OUT_F[4]),
                (wires::OUT_F_E[5], wires::OUT_F[5]),
                (wires::OUT_F_E[6], wires::OUT_F[6]),
                (wires::OUT_F_E[7], wires::OUT_F[7]),
            ]);
        }
        if self.chip.kind.has_out_ofx_branch() {
            if matches!(self.chip.kind, ChipKind::Ecp5 | ChipKind::Crosslink) {
                out_w.push((wires::OUT_F_W[3], wires::OUT_F[3]));
            } else {
                out_w.push((wires::OUT_OFX_W[3], wires::OUT_OFX[3]));
            }
        }
        for (cell, cell_data) in self.edev.cells() {
            for (slot, conn) in &cell_data.conns {
                if let Some(target) = conn.target {
                    let cell_sio = self.chip.kind == ChipKind::MachXo
                        && (cell.col == self.chip.col_w()
                            || cell.col == self.chip.col_e()
                            || cell.row == self.chip.row_s()
                            || cell.row == self.chip.row_n());
                    let target_sio = self.chip.kind == ChipKind::MachXo
                        && (target.col == self.chip.col_w()
                            || target.col == self.chip.col_e()
                            || target.row == self.chip.row_s()
                            || target.row == self.chip.row_n());
                    if slot == cslots::W && !target_sio && !cell_sio {
                        for &(wt, wf) in &out_w {
                            let wt = target.wire(wt);
                            let wf = cell.wire(wf);
                            let Some(&wtn) = self.naming.interconnect.get(&wt) else {
                                println!(
                                    "{name}: {wt} missing",
                                    name = self.name,
                                    wt = wt.to_string(self.intdb)
                                );
                                continue;
                            };
                            let Some(&wfn) = self.naming.interconnect.get(&wf) else {
                                println!(
                                    "{name}: {wf} missing",
                                    name = self.name,
                                    wf = wf.to_string(self.intdb)
                                );
                                continue;
                            };
                            term_pips.insert((wtn, wfn));
                        }
                    }
                    if slot == cslots::E && !target_sio && !cell_sio {
                        for &(wt, wf) in &out_e {
                            let wt = target.wire(wt);
                            let wf = cell.wire(wf);
                            let Some(&wtn) = self.naming.interconnect.get(&wt) else {
                                println!(
                                    "{name}: {wt} missing",
                                    name = self.name,
                                    wt = wt.to_string(self.intdb)
                                );
                                continue;
                            };
                            let Some(&wfn) = self.naming.interconnect.get(&wf) else {
                                println!(
                                    "{name}: {wf} missing",
                                    name = self.name,
                                    wf = wf.to_string(self.intdb)
                                );
                                continue;
                            };
                            term_pips.insert((wtn, wfn));
                        }
                    }
                } else {
                    let ccls = &self.intdb[conn.class];
                    for (wt, &wf) in &ccls.wires {
                        let ConnectorWire::Reflect(wf) = wf else {
                            unreachable!()
                        };
                        if wires::DSCLK_H.contains(wt) {
                            continue;
                        }
                        let wt = cell.wire(wt);
                        let wf = cell.wire(wf);
                        let Some(&wtn) = self.naming.interconnect.get(&wt) else {
                            continue;
                        };
                        let Some(&wfn) = self.naming.interconnect.get(&wf) else {
                            continue;
                        };
                        term_pips.insert((wtn, wfn));
                    }
                }
            }
        }
        term_pips
    }

    #[allow(clippy::type_complexity)]
    fn collect_int_pips(
        &mut self,
        mut term_pips: BTreeSet<(WireName, WireName)>,
    ) -> (
        BTreeMap<CellCoord, BTreeSet<(WireSlotId, WireSlotId)>>,
        BTreeMap<TileClassId, BTreeSet<(WireSlotId, WireSlotId)>>,
    ) {
        let mut cell_pips: BTreeMap<CellCoord, BTreeSet<(WireSlotId, WireSlotId)>> =
            BTreeMap::new();
        let mut sb_pips: BTreeMap<TileClassId, BTreeSet<(WireSlotId, WireSlotId)>> =
            BTreeMap::new();

        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if term_pips.remove(&(wtn, wfn)) {
                continue;
            }
            if term_pips.remove(&(wfn, wtn)) {
                continue;
            }
            if self.int_wires.contains_key(&wtn) && self.keep_nodes.contains(&wfn) {
                continue;
            }
            if self.discard_nodes.contains(&wtn) && self.keep_nodes.contains(&wfn) {
                continue;
            }
            if self.discard_nodes.contains(&wtn) && self.int_wires.contains_key(&wfn) {
                continue;
            }
            if let Some(wires_f) = self.int_wires.get(&wfn)
                && let Some(wires_t) = self.int_wires.get(&wtn)
                && let mut wt = wires_t[0]
                && !wires::RSCLK.contains(wt.slot)
                && !(wires::RSCLK_H.contains(wt.slot) && self.intdb[wires_f[0].slot].kind.is_tie())
                && !wires::IMUX_CLK_DELAY.contains(wt.slot)
            {
                if matches!(self.intdb[wt.slot].kind, WireKind::Branch(_))
                    && !wires::RSCLK_H.contains(wt.slot)
                {
                    println!(
                        "{name}: extra term pip {wtn} / {wt} <- {wfn} / {wf}",
                        name = self.name,
                        wtn = wtn.to_string(&self.naming),
                        wt = wt.to_string(self.intdb),
                        wfn = wfn.to_string(&self.naming),
                        wf = wires_f[0].to_string(self.intdb),
                    );
                    continue;
                }
                let wf = if wires::DSCLK_V.contains(wt.slot) {
                    if wires::DSCLK_H.contains(wires_f[0].slot) {
                        let cand_cells_t =
                            BTreeSet::from_iter(wires_t.iter().copied().map(|w| w.cell));
                        let cand_cells = BTreeSet::from_iter(
                            wires_f
                                .iter()
                                .copied()
                                .map(|w| w.cell)
                                .filter(|&cell| cand_cells_t.contains(&cell)),
                        );
                        assert_eq!(cand_cells.len(), 1);
                        let wires_f = Vec::from_iter(wires_f.iter().copied().filter(|w| {
                            w.slot == wires::DSCLK_H[0]
                                || w.slot == wires::DSCLK_H[3]
                                || w.slot == wires::DSCLK_H[6]
                        }));
                        let cell = cand_cells.into_iter().next().unwrap();
                        wt = wires_t.iter().find(|w| w.cell == cell).copied().unwrap();
                        wires_f.iter().find(|w| w.cell == cell).copied().unwrap()
                    } else {
                        let cand_cells_t =
                            BTreeSet::from_iter(wires_t.iter().copied().map(|w| w.cell));
                        let cand_cells = BTreeSet::from_iter(
                            wires_f
                                .iter()
                                .copied()
                                .map(|w| w.cell)
                                .filter(|&cell| cand_cells_t.contains(&cell)),
                        );
                        assert_eq!(cand_cells.len(), 1);
                        let cell = cand_cells.into_iter().next().unwrap();
                        wt = wires_t.iter().find(|w| w.cell == cell).copied().unwrap();
                        wires_f.iter().find(|w| w.cell == cell).copied().unwrap()
                    }
                } else if wires::DSCLK_H.contains(wt.slot) && self.chip.kind == ChipKind::Scm {
                    if wires::DSCLK_H.contains(wires_f[0].slot) {
                        let wires_t = Vec::from_iter(
                            wires_t
                                .iter()
                                .copied()
                                .filter(|w| w.slot == wires::DSCLK_H[6]),
                        );
                        let wires_f = Vec::from_iter(
                            wires_f
                                .iter()
                                .copied()
                                .filter(|w| w.slot == wires::DSCLK_H[0]),
                        );
                        let cand_cells_t =
                            BTreeSet::from_iter(wires_t.iter().copied().map(|w| w.cell));
                        let cand_cells = BTreeSet::from_iter(
                            wires_f
                                .iter()
                                .copied()
                                .map(|w| w.cell)
                                .filter(|&cell| cand_cells_t.contains(&cell)),
                        );
                        assert_eq!(cand_cells.len(), 1);
                        let cell = cand_cells.into_iter().next().unwrap();
                        wt = wires_t.iter().find(|w| w.cell == cell).copied().unwrap();
                        wires_f.iter().find(|w| w.cell == cell).copied().unwrap()
                    } else {
                        assert_eq!(wires_f.len(), 1);
                        wt = wires_t
                            .iter()
                            .find(|w| {
                                w.cell == wires_f[0].cell
                                    && (w.slot == wires::DSCLK_H[0] || w.slot == wires::DSCLK_H[6])
                            })
                            .copied()
                            .unwrap();
                        wires_f[0]
                    }
                } else if wires::RSCLK_H.contains(wt.slot) {
                    assert_eq!(wires_f.len(), 1);
                    wt = wires_t
                        .iter()
                        .find(|w| w.col == wires_f[0].col)
                        .copied()
                        .unwrap();
                    wires_f[0]
                } else if let Some(idx) = wires::RSCLK_V.index_of(wt.slot) {
                    let mut wf = wires_f.iter().find(|w| w.col == wt.col).copied().unwrap();
                    let wt_root = self.edev[wt.cell].region_root[rslots::RSCLK_V];
                    let wf_root = self.edev[wf.cell].region_root[rslots::RSCLK_V];
                    if wt_root == wf_root {
                        wt.cell = wf.cell;
                    } else {
                        assert_eq!(
                            self.edev[wf.cell.delta(0, -1)].region_root[rslots::RSCLK_V],
                            wt_root
                        );
                        wt.cell = wf.cell;
                        wt.slot = wires::RSCLK_V_N[idx];
                    }
                    if matches!(self.chip.kind, ChipKind::Ecp3 | ChipKind::Ecp3A)
                        && self.intdb[wf.slot].kind.is_tie()
                        && wf.cell.row <= self.chip.row_s() + 9
                    {
                        wf.cell.row = self.chip.row_s();
                        if !self.edev.has_bel(wf.cell.bel(bslots::INT)) {
                            wf.cell.row += 9;
                        }
                        wt.cell = wf.cell;
                    }
                    wf
                } else {
                    let wf = if let WireKind::Regional(region) = self.intdb[wires_f[0].slot].kind {
                        let wf_root = self.edev[wires_f[0].cell].region_root[region];
                        let wt_root = self.edev[wt.cell].region_root[region];
                        if wf_root == wt_root {
                            Some(wt.cell.wire(wires_f[0].slot))
                        } else if let Some(idx) = wires::RSCLK_V.index_of(wires_f[0].slot)
                            && self.edev[wt.cell.delta(0, -1)].region_root[rslots::RSCLK_V]
                                == wf_root
                        {
                            Some(wt.cell.wire(wires::RSCLK_V_N[idx]))
                        } else {
                            None
                        }
                    } else if self.intdb[wires_f[0].slot].kind.is_tie() {
                        Some(wt.cell.wire(wires_f[0].slot))
                    } else {
                        wires_f.iter().find(|w| w.cell == wt.cell).copied()
                    };
                    let Some(wf) = wf else {
                        println!(
                            "{name}: weird int pip {wtn} / {wt} <- {wfn} / {wf}",
                            name = self.name,
                            wtn = wtn.to_string(&self.naming),
                            wt = wt.to_string(self.intdb),
                            wfn = wfn.to_string(&self.naming),
                            wf = wires_f[0].to_string(self.intdb),
                        );
                        continue;
                    };
                    wf
                };
                cell_pips
                    .entry(wt.cell)
                    .or_default()
                    .insert((wt.slot, wf.slot));
                let tcid = self.edev[wt.cell.tile(tslots::INT)].class;
                sb_pips.entry(tcid).or_default().insert((wt.slot, wf.slot));
            } else {
                self.unclaimed_pips.insert((wtn, wfn));
                self.pips_bwd.entry(wtn).or_default().insert(wfn);
                self.pips_fwd.entry(wfn).or_default().insert(wtn);
            }
        }
        for (wtn, wfn) in term_pips {
            println!(
                "MISSING TERM PIP {wtn} <- {wfn}",
                wtn = wtn.to_string(&self.naming),
                wfn = wfn.to_string(&self.naming)
            );
        }

        (cell_pips, sb_pips)
    }

    fn create_switchboxes(
        &mut self,
        mut cell_pips: BTreeMap<CellCoord, BTreeSet<(WireSlotId, WireSlotId)>>,
        sb_pips: BTreeMap<TileClassId, BTreeSet<(WireSlotId, WireSlotId)>>,
    ) {
        for (tcid, pips) in sb_pips {
            for &tcrd in &self.edev.tile_index[tcid] {
                let tile_pips = cell_pips.remove(&tcrd.cell).unwrap();
                for &(wt, wf) in &pips {
                    if tile_pips.contains(&(wt, wf)) {
                        // OK
                        continue;
                    }
                    let wt = tcrd.wire(wt);
                    let wf = tcrd.wire(wf);
                    let wtsn = self.intdb.wires[wt.slot].name.as_str();
                    let wfsn = self.intdb.wires[wf.slot].name.as_str();
                    if !self.naming.interconnect.contains_key(&wt) && wtsn.starts_with("IMUX") {
                        continue;
                    }
                    if !self.naming.interconnect.contains_key(&wf)
                        && (wfsn.starts_with("OUT")
                            || (self.chip.kind == ChipKind::Scm && wfsn.starts_with("DSCLK")))
                    {
                        continue;
                    }
                    if self.chip.kind == ChipKind::Scm {
                        if wt.slot == wires::DSCLK_V[0]
                            && (wf.slot == wires::DSCLK_V[6] || wires::DSCLK_H.contains(wf.slot))
                            && self.edev[tcrd.cell].conns[cslots::N].target.is_none()
                        {
                            continue;
                        }
                        if (wt.slot == wires::DSCLK_V[0] || wt.slot == wires::DSCLK_V[6])
                            && (wf.slot == wires::DSCLK_H[0] || wf.slot == wires::DSCLK_H[6])
                            && self.edev.resolve_wire(wf)
                                == self.edev.resolve_wire(wf.wire(wires::DSCLK_H[3]))
                        {
                            continue;
                        }
                    } else {
                        if (wt.slot == wires::RSCLK_V_N[0]
                            || wt.slot == wires::RSCLK_V_N[1]
                            || wf.slot == wires::RSCLK_V_N[0]
                            || wf.slot == wires::RSCLK_V_N[1])
                            && tcrd.row != self.chip.row_s()
                            && !self.chip.rows[tcrd.row].sclk_break
                        {
                            continue;
                        }
                        if (wt.slot == wires::RSCLK_V[2]
                            || wt.slot == wires::RSCLK_V[3]
                            || wt.slot == wires::RSCLK_V_N[2]
                            || wt.slot == wires::RSCLK_V_N[3])
                            && tcrd.col != self.chip.col_e()
                        {
                            continue;
                        }
                        if (wt.slot == wires::RSCLK_V[4]
                            || wt.slot == wires::RSCLK_V[5]
                            || wt.slot == wires::RSCLK_V[6]
                            || wt.slot == wires::RSCLK_V[7]
                            || wt.slot == wires::RSCLK_V_N[4]
                            || wt.slot == wires::RSCLK_V_N[5]
                            || wt.slot == wires::RSCLK_V_N[6]
                            || wt.slot == wires::RSCLK_V_N[7])
                            && tcrd.col != self.chip.col_w()
                        {
                            continue;
                        }
                    }
                    if wf.slot == wires::RSCLK[3] || wf.slot == wires::RSCLK[7] {
                        // ???? for some inscrutable reason these two devices are missing SCLK3
                        // in these locations in particular
                        if self.chip.kind == ChipKind::Ecp2
                            && self.chip.rows.len() == 46
                            && tcrd.col == self.chip.col_e()
                            && tcrd.row.to_idx() == 17
                        {
                            continue;
                        }
                        if self.chip.kind == ChipKind::Ecp2M
                            && self.chip.rows.len() == 75
                            && tcrd.col.to_idx() == 72
                            && tcrd.row == self.chip.row_s()
                        {
                            continue;
                        }
                    }
                    println!(
                        "{name}: MISSING PIP {wt} <- {wf}",
                        name = self.name,
                        wt = wt.to_string(self.intdb),
                        wf = wf.to_string(self.intdb)
                    );
                }
            }
            let mut muxes = BTreeMap::new();
            for (wt, wf) in pips {
                let wt = wt.cell(0);
                let wf = wf.cell(0);
                muxes
                    .entry(wt)
                    .or_insert_with(|| Mux {
                        dst: wt,
                        bits: vec![],
                        src: BTreeMap::new(),
                        bits_off: None,
                    })
                    .src
                    .insert(wf.pos(), Default::default());
                if (self.intdb.wires[wt.wire]
                    .name
                    .as_str()
                    .starts_with("RSCLK_V")
                    && self.intdb.wires[wf.wire]
                        .name
                        .as_str()
                        .starts_with("RSCLK_V"))
                    || (self.intdb.wires[wt.wire]
                        .name
                        .as_str()
                        .starts_with("RSCLK_H")
                        && self.intdb.wires[wf.wire]
                            .name
                            .as_str()
                            .starts_with("RSCLK_H"))
                {
                    muxes
                        .entry(wf)
                        .or_insert_with(|| Mux {
                            dst: wf,
                            bits: vec![],
                            src: BTreeMap::new(),
                            bits_off: None,
                        })
                        .src
                        .insert(wt.pos(), Default::default());
                }
            }
            let mut sb = SwitchBox::default();
            for mux in muxes.into_values() {
                if mux.src.len() == 1 {
                    let src = mux.src.into_keys().next().unwrap();
                    let wtn = self.intdb.wires[mux.dst.wire].name.as_str();
                    if wtn.starts_with("X1") || wtn.starts_with("IMUX_M") {
                        let buf = PermaBuf { dst: mux.dst, src };
                        sb.items.push(SwitchBoxItem::PermaBuf(buf));
                    } else {
                        let buf = ProgBuf {
                            dst: mux.dst,
                            src,
                            bit: PolTileBit::DUMMY,
                        };
                        sb.items.push(SwitchBoxItem::ProgBuf(buf));
                    }
                } else {
                    sb.items.push(SwitchBoxItem::Mux(mux));
                }
            }
            if matches!(self.chip.kind, ChipKind::MachXo2(_)) && tcid == tcls::INT_EBR {
                for i in 0..8 {
                    let vsdclk = wires::RSCLK_V[i].cell(0);
                    let vsdclk_n = wires::RSCLK_V_N[i].cell(0);
                    sb.items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                        dst: vsdclk,
                        src: vsdclk_n.pos(),
                        bit: PolTileBit::DUMMY,
                    }));
                    sb.items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                        dst: vsdclk_n,
                        src: vsdclk.pos(),
                        bit: PolTileBit::DUMMY,
                    }));
                }
            }
            if self.chip.kind == ChipKind::Ecp4 && tcid != tcls::INT_PLC {
                for i in 0..2 {
                    let clk = wires::IMUX_CLK[i].cell(0);
                    let clk_delay = wires::IMUX_CLK_DELAY[i].cell(0);
                    sb.items.push(SwitchBoxItem::ProgDelay(ProgDelay {
                        dst: clk_delay,
                        src: clk.pos(),
                        bits: vec![],
                        steps: vec![Default::default(); 4],
                    }));
                }
            }
            if self.chip.kind == ChipKind::MachXo && tcid != tcls::INT_PLC {
                for i in 0..6 {
                    sb.items.push(SwitchBoxItem::Mux(Mux {
                        dst: wires::PLC_IO[i].cell(0),
                        bits: Default::default(),
                        src: BTreeMap::from_iter([
                            (wires::OUT_F_PLC_IO[i].cell(0).pos(), Default::default()),
                            (wires::OUT_Q_PLC_IO[i].cell(0).pos(), Default::default()),
                        ]),
                        bits_off: None,
                    }));
                }
            }
            self.bels
                .insert((tcid, bslots::INT), BelInfo::SwitchBox(sb));
        }
    }

    pub(super) fn process_int_pips(&mut self) {
        let term_pips = self.collect_term_pips();
        let (cell_pips, sb_pips) = self.collect_int_pips(term_pips);
        self.create_switchboxes(cell_pips, sb_pips);
    }
}
