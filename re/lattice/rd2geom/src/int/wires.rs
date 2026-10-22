use std::{
    assert_matches,
    collections::{BTreeSet, btree_map},
};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind},
    defs::{bslots, cslots, wiredata, wires},
};
use prjcombine_entity::{EntityId, EntityIdArith};
use prjcombine_interconnect::{
    db::{BelInput, CellSlotId, TileWireCoord, WireKind},
    dir::{Dir, DirV},
    grid::{BelCoord, CellCoord, DieIdExt, WireCoord},
};
use prjcombine_re_lattice_naming::WireName;

use crate::{ChipContext, chip::ChipExt};

impl ChipContext<'_> {
    fn classify_int_wire(&self, wn: WireName) -> Vec<WireCoord> {
        let suffix = &self.naming.strings[wn.suffix];
        if suffix.starts_with("H00") || suffix.starts_with("V00") {
            assert_eq!(suffix.len(), 8);
            let dir = match (&suffix[0..1], &suffix[3..4]) {
                ("H", "L") => Dir::W,
                ("H", "R") => Dir::E,
                ("V", "B") => Dir::S,
                ("V", "T") => Dir::N,
                _ => unreachable!(),
            };
            let idx: usize = suffix[4..6].parse().unwrap();
            let seg: u8 = suffix[6..8].parse().unwrap();
            assert!(seg <= 1);
            let mut cell = self.chip.xlat_rc_wire(wn);
            let conn_bwd = wiredata::CONNS[!dir];
            if seg != 0 {
                if let Some(conn) = self.edev[cell].conns.get(conn_bwd)
                    && let Some(target) = conn.target
                {
                    cell = target;
                } else {
                    return vec![];
                }
            }
            let wire = match dir {
                Dir::W => wires::X0_W[idx],
                Dir::E => wires::X0_E[idx],
                Dir::S => wires::X0_S[idx],
                Dir::N => wires::X0_N[idx],
            };
            let wire = cell.wire(wire);
            vec![wire]
        } else if (suffix.starts_with("H0") || suffix.starts_with("V0")) && &suffix[3..4] != "M" {
            let mut result = vec![];
            assert_eq!(suffix.len(), 8);
            let len: usize = suffix[1..3].parse().unwrap();
            let mut idx: usize = suffix[4..6].parse().unwrap();
            let mut seg: usize = suffix[6..8].parse().unwrap();
            let dir = match (&suffix[0..1], &suffix[3..4]) {
                ("H", "W") => Dir::W,
                ("H", "E") => Dir::E,
                ("V", "S") => Dir::S,
                ("V", "N") => Dir::N,
                ("V", "B") if idx == 0 => {
                    idx = seg;
                    seg = 1;
                    Dir::N
                }
                ("V", "B") if idx == 1 => {
                    idx = seg;
                    seg = 0;
                    Dir::S
                }
                _ => unreachable!(),
            };
            assert!(seg <= len);
            let mut cell = self.chip.xlat_rc_wire(wn);
            if cell.col > self.chip.col_e() {
                // what the fuck. this is LIFCL-33 btw.
                assert_eq!(suffix, "H01E0100");
                assert_eq!(wn.c, 52);
                assert_eq!(wn.r, 37);
                cell = cell.delta(-2, 0);
            }
            let conn_fwd = wiredata::CONNS[dir];
            let conn_bwd = wiredata::CONNS[!dir];
            if self.has_v01b.contains(&cell) && dir == Dir::N && len == 6 && seg == 4 {
                // hwaet.
                seg = 3;
            }
            while seg != 0 {
                if let Some(conn) = self.edev[cell].conns.get(conn_bwd)
                    && let Some(target) = conn.target
                {
                    cell = target;
                    seg -= 1;
                } else {
                    break;
                }
            }
            if matches!(self.chip.kind, ChipKind::Nexus(_)) && len == 1 && seg == 0 {
                match dir {
                    Dir::H(_) => {
                        result.push(cell.wire(wires::X1_H[idx]));
                        if let Some(cell) = self.edev.cell_delta(cell, 1, 0) {
                            result.push(cell.wire(wires::X1_E1[idx]));
                        }
                        if let Some(cell) = self.edev.cell_delta(cell, -1, 0) {
                            result.push(cell.wire(wires::X1_W1[idx]));
                        }
                    }
                    Dir::V(_) => {
                        result.push(cell.wire(wires::X1_V[idx]));
                        if let Some(cell) = self.edev.cell_delta(cell, 0, 1) {
                            result.push(cell.wire(wires::X1_N1[idx]));
                        }
                        if let Some(cell) = self.edev.cell_delta(cell, 0, -1) {
                            result.push(cell.wire(wires::X1_S1[idx]));
                        }
                    }
                }
            } else {
                while seg <= len {
                    let wire = match (len, dir) {
                        (1, Dir::W) => wiredata::X1_W[seg][idx],
                        (1, Dir::E) => wiredata::X1_E[seg][idx],
                        (1, Dir::S) => wiredata::X1_S[seg][idx],
                        (1, Dir::N) => wiredata::X1_N[seg][idx],
                        (2, Dir::W) => wiredata::X2_W[seg][idx],
                        (2, Dir::E) => wiredata::X2_E[seg][idx],
                        (2, Dir::S) => wiredata::X2_S[seg][idx],
                        (2, Dir::N) => wiredata::X2_N[seg][idx],
                        (6, Dir::W) => wiredata::X6_W[seg][idx],
                        (6, Dir::E) => wiredata::X6_E[seg][idx],
                        (6, Dir::S) => wiredata::X6_S[seg][idx],
                        (6, Dir::N) => wiredata::X6_N[seg][idx],
                        _ => panic!("ummm X{len}_{dir}"),
                    };
                    let wire = cell.wire(wire);
                    result.push(wire);
                    if let Some(conn) = self.edev[cell].conns.get(conn_fwd)
                        && let Some(target) = conn.target
                    {
                        cell = target;
                        seg += 1;
                    } else {
                        break;
                    }
                }
            }
            result
        } else if suffix.starts_with("HPBX") || suffix.starts_with("HSBX") {
            assert_eq!(suffix.len(), 8);
            let mut idx: usize = suffix[4..6].parse().unwrap();
            if self.chip.kind == ChipKind::Crosslink {
                idx = match idx {
                    2..6 => idx - 2,
                    12..16 => idx - 12 + 4,
                    _ => unreachable!(),
                };
            }
            if matches!(self.chip.kind, ChipKind::Nexus(_)) && idx >= 8 {
                return vec![];
            }
            let is_sclk = suffix.starts_with("HSBX");
            if (is_sclk && self.chip.kind.has_distributed_sclk())
                || (!is_sclk && self.chip.kind == ChipKind::Scm)
            {
                if suffix.ends_with("00") {
                    return vec![];
                }
                assert!(suffix.ends_with("01"));
            } else {
                assert!(suffix.ends_with("00"));
            }
            let wire = if is_sclk {
                if self.chip.kind == ChipKind::Scm {
                    wires::DSCLK[idx]
                } else if self.chip.kind.has_distributed_sclk() {
                    wires::SCLK[idx]
                } else if matches!(
                    self.chip.kind,
                    ChipKind::Ecp | ChipKind::Xp | ChipKind::MachXo
                ) {
                    wires::PCLK[idx + 4]
                } else {
                    panic!("ummmm {suffix}")
                }
            } else {
                wires::PCLK[idx]
            };
            let cell = self.chip.xlat_rc_wire(wn);
            match self.chip.kind {
                ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 if is_sclk => {
                    let WireKind::Regional(region) = self.intdb[wire].kind else {
                        unreachable!()
                    };
                    let root = self.edev[cell].region_root[region];
                    let mut cell_start = cell;
                    let mut cell_end = cell;
                    while let Some(cell_new) = self.edev.cell_delta(cell_start, -1, 0)
                        && self.edev.has_bel(cell_new.bel(bslots::INT))
                        && self.edev[cell_new].region_root[region] == root
                    {
                        cell_start = cell_new;
                    }
                    while let Some(cell_new) = self.edev.cell_delta(cell_end, 1, 0)
                        && self.edev.has_bel(cell_new.bel(bslots::INT))
                        && self.edev[cell_new].region_root[region] == root
                    {
                        cell_end = cell_new;
                    }
                    if self.chip.kind == ChipKind::Xp2
                        && self.chip.rows[cell.row].kind == RowKind::Ebr
                        && matches!(suffix.as_str(), "HSBX0101" | "HSBX0501")
                        && cell.col.to_idx() < 6
                    {
                        // toolchain bug or actual hardware issue? you tell me.
                        if cell.col.to_idx() < 2 {
                            cell_end.col -= 4;
                        } else {
                            cell_start.col += 2;
                        }
                    }
                    cell_start
                        .col
                        .range(cell_end.col + 1)
                        .map(|col| cell.with_col(col).wire(wire))
                        .collect()
                }
                ChipKind::Ecp3(_) | ChipKind::MachXo2(_) if is_sclk => {
                    let WireKind::Regional(region) = self.intdb[wire].kind else {
                        unreachable!()
                    };
                    let root = self.edev[cell].region_root[region];
                    let mut cell_start = cell;
                    let mut cell_end = cell;
                    while let Some(cell_new) = self.edev.cell_delta(cell_start, -1, 0)
                        && self.edev[cell_new].region_root[region] == root
                    {
                        cell_start = cell_new;
                    }
                    while let Some(cell_new) = self.edev.cell_delta(cell_end, 1, 0)
                        && self.edev[cell_new].region_root[region] == root
                    {
                        cell_end = cell_new;
                    }
                    cell_start
                        .col
                        .range(cell_end.col + 1)
                        .map(|col| cell.with_col(col).wire(wire))
                        .collect()
                }
                ChipKind::Scm | ChipKind::Ecp4 => {
                    vec![cell.wire(wire)]
                }
                _ => {
                    let wire = cell.wire(wire);
                    let wire = self.edev.resolve_wire(wire).unwrap();
                    self.edev.wire_tree(wire)
                }
            }
        } else if self.chip.kind == ChipKind::Scm
            && (suffix.starts_with("HSTX") || suffix.starts_with("HSTE"))
        {
            let mut cell = self.chip.xlat_rc_wire(wn);
            if suffix.starts_with("HSTE") {
                cell.col -= 1;
            }
            let idx: usize = suffix[4..6].parse().unwrap();
            assert_eq!(suffix.len(), 8);
            if suffix.starts_with("HSTE") {
                assert!(suffix.ends_with("01"));
            } else {
                assert!(suffix.ends_with("00"));
            }
            let wire = wires::DSCLK[idx];
            let wire = cell.wire(wire);
            let mut res = vec![wire];
            if let Some(cell_w) = self.edev.cell_delta(cell, -1, 0)
                && self.edev.has_bel(cell_w.bel(bslots::INT))
            {
                let wire = wires::DSCLK_W[idx];
                let wire = cell_w.wire(wire);
                res.push(wire);
            }
            if let Some(cell_e) = self.edev.cell_delta(cell, 1, 0)
                && self.edev.has_bel(cell_e.bel(bslots::INT))
            {
                let wire = wires::DSCLK_E[idx];
                let wire = cell_e.wire(wire);
                res.push(wire);
            }
            res
        } else if (suffix.starts_with("VSBX") || suffix.starts_with("VSBB"))
            && self.chip.kind == ChipKind::Scm
        {
            assert_eq!(suffix.len(), 8);
            let idx: u8 = suffix[4..6].parse().unwrap();
            let mut seg: usize = suffix[6..8].parse().unwrap();
            assert_eq!(idx, 0);
            if suffix.starts_with("VSBB") {
                assert_eq!(seg, 0);
                seg = 6;
            } else {
                assert!(seg < 6);
            }
            let mut cell = self.chip.xlat_rc_wire(wn);
            while seg != 0 {
                if let Some(conn) = self.edev[cell].conns.get(cslots::S)
                    && let Some(target) = conn.target
                {
                    cell = target;
                    seg -= 1;
                } else {
                    break;
                }
            }
            let mut result = vec![];
            while seg <= 6 {
                let wire = cell.wire(wires::DSCLK_V[seg]);
                result.push(wire);
                if let Some(conn) = self.edev[cell].conns.get(cslots::N)
                    && let Some(target) = conn.target
                {
                    cell = target;
                    seg += 1;
                } else {
                    break;
                }
            }
            result
        } else if (suffix.starts_with("HSSX") || suffix.starts_with("HSCX"))
            && self.chip.kind == ChipKind::Scm
        {
            assert_eq!(suffix.len(), 8);
            let idx: u8 = suffix[4..6].parse().unwrap();
            let seg: usize = suffix[6..8].parse().unwrap();
            assert_eq!(idx, 0);
            if suffix.starts_with("HSCX") {
                assert!(matches!(seg, 1 | 2 | 4));
            } else {
                assert_eq!(seg, 3);
            }
            let cell = self.chip.xlat_rc_wire(wn);
            let wire = cell.wire(wires::DSCLK_H[seg]);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if suffix.starts_with("HSSX") && suffix.len() == 8 {
            assert!(suffix.ends_with("00"));
            let cell = self.hsdclk_locs[&wn];
            let idx: usize = suffix[4..6].parse().unwrap();
            let wire = wires::SCLK_H[idx];
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if suffix.starts_with("VSTX") && suffix.len() == 8 {
            assert!(suffix.ends_with("00"));
            let idx: usize = suffix[4..6].parse().unwrap();
            let cell = self.chip.xlat_rc_wire(wn);
            let idx = if self.chip.kind.has_distributed_sclk_ecp3() {
                let is_w = cell.col == self.chip.col_w();
                match idx {
                    2 => 0,
                    3 => 4,
                    6 if is_w => 3,
                    7 if is_w => 7,
                    6 => 1,
                    7 => 5,
                    10 => 2,
                    11 => 6,
                    _ => unreachable!(),
                }
            } else {
                match idx {
                    0 => 0,
                    1 => 4,
                    2 => 0,
                    3 => 4,
                    _ => unreachable!(),
                }
            };
            let wire = wires::RSCLK_V[idx];
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if matches!(self.chip.kind, ChipKind::Ecp3(_) | ChipKind::MachXo2(_))
            && suffix.starts_with("VPTX")
            && suffix.len() == 8
        {
            assert!(suffix.ends_with("00"));
            let idx: usize = suffix[4..6].parse().unwrap();
            let cell = self.chip.xlat_rc_wire(wn);
            let idx = self.chip.clk_to_rclk(cell.col, idx);
            let wire = wires::RPCLK_COL[idx];
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            let mut res = self.edev.wire_tree(wire);
            if matches!(self.chip.kind, ChipKind::Ecp3(_))
                && self.chip.rows_pclk.range(cell.row).start() == self.chip.row_s()
            {
                let row_serdes = self.chip.row_serdes(DirV::S);
                res.retain(|&w| (w.row < row_serdes) == (cell.row < row_serdes));
            }
            res
        } else if matches!(self.chip.kind, ChipKind::Ecp3(_) | ChipKind::MachXo2(_))
            && suffix.starts_with("HPSX")
            && suffix.len() == 8
        {
            assert!(suffix.ends_with("00"));
            let idx: usize = suffix[4..6].parse().unwrap();
            let mut cell = self.chip.xlat_rc_wire(wn);
            if self.chip.rows_pclk[cell.row] != cell.row {
                assert_matches!(self.chip.kind, ChipKind::MachXo2(_));
                assert_eq!(self.chip.rows_pclk[cell.row], self.chip.row_s());
                cell.row = self.chip.row_s();
            }
            let wire = wires::PCLK_ROW[idx];
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if ((suffix.starts_with("VPTX")
            && matches!(self.chip.kind, ChipKind::Ecp5 | ChipKind::Crosslink))
            || (suffix.starts_with("VPSX")
                && matches!(self.chip.kind, ChipKind::Scm | ChipKind::Nexus(_))))
            && suffix.len() == 8
        {
            assert!(suffix.ends_with("00"));
            let idx: usize = suffix[4..6].parse().unwrap();
            let mut cell = self.chip.xlat_rc_wire(wn);
            if matches!(self.chip.kind, ChipKind::Nexus(_)) && cell.col < self.chip.col_clk {
                // okay
            } else {
                cell.col += 1;
            }
            let wire = wires::PCLK_COL[idx];
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if ((suffix.starts_with("HPRX")
            && matches!(
                self.chip.kind,
                ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink | ChipKind::Nexus(_)
            ))
            || (suffix.starts_with("VPBX")
                && matches!(
                    self.chip.kind,
                    ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2
                ))
            || (suffix.starts_with("HPCX") && self.chip.kind == ChipKind::Scm))
            && suffix.len() == 8
        {
            assert!(suffix.ends_with("00"));
            let idx: usize = suffix[4..6].parse().unwrap();
            let mut cell = self.chip.xlat_rc_wire(wn);
            // ????
            if self.chip.kind == ChipKind::Xp2 && cell.col == self.chip.col_clk - 1 {
                cell.col = self.chip.col_clk;
            }
            let wire = match self.chip.kind {
                ChipKind::Ecp4 => wires::PCLK_ROW[idx],
                ChipKind::Crosslink => wires::PCLK_HALF[idx],
                _ => wires::PCLK_ROOT[idx],
            };
            let wire = cell.wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if self.chip.kind == ChipKind::Ecp4
            && let Some(idx) = suffix
                .strip_prefix("LLPCLK")
                .or(suffix.strip_prefix("ULPCLK"))
                .or(suffix.strip_prefix("LRPCLK"))
                .or(suffix.strip_prefix("URPCLK"))
            && let Ok(idx) = idx.parse::<usize>()
        {
            let col = if &suffix[1..2] == "L" {
                self.chip.col_w()
            } else {
                self.chip.col_e()
            };
            let row = if suffix.starts_with("L") {
                self.chip.row_s()
            } else {
                self.chip.row_n()
            };
            let wire = wires::PCLK_ROOT[idx];
            let wire = Chip::DIE.cell(col, row).wire(wire);
            let wire = self.edev.resolve_wire(wire).unwrap();
            self.edev.wire_tree(wire)
        } else if (suffix.starts_with("BPCX")
            || suffix.starts_with("TPCX")
            || suffix.starts_with("BSCX")
            || suffix.starts_with("TSCX"))
            && suffix.len() == 8
            && suffix.ends_with("00")
            && matches!(self.chip.kind, ChipKind::Ecp3(_))
        {
            let idx: usize = suffix[4..6].parse().unwrap();
            let wire = if &suffix[1..2] == "P" {
                wires::PCLK_ROOT[idx]
            } else {
                wires::SCLK_ROOT[idx]
            };
            let cell = self
                .chip
                .xlat_rc_wire(wn)
                .with_row(if suffix.starts_with("B") {
                    self.chip.row_s()
                } else {
                    self.chip.row_n()
                });
            let wire = self.edev.resolve_wire(cell.wire(wire)).unwrap();
            self.edev.wire_tree(wire)
        } else if (suffix.starts_with("VPRX") || suffix.starts_with("VSRX"))
            && suffix.len() == 8
            && suffix.ends_with("00")
            && matches!(self.chip.kind, ChipKind::MachXo2(_))
        {
            let idx: usize = suffix[4..6].parse().unwrap();
            let wire = if &suffix[1..2] == "P" {
                wires::PCLK_ROOT[idx]
            } else {
                wires::SCLK_ROOT[idx]
            };
            let cell = self.chip.xlat_rc_wire(wn);
            let wire = self.edev.resolve_wire(cell.wire(wire)).unwrap();
            self.edev.wire_tree(wire)
        } else if (suffix.starts_with("VECL")
            || suffix.starts_with("VECR")
            || suffix.starts_with("HECB")
            || suffix.starts_with("HECT"))
            && suffix.len() == 8
            && suffix.ends_with("00")
            && self.chip.kind == ChipKind::Scm
        {
            let idx: usize = suffix[4..6].parse().unwrap();
            let wire = wires::ECLK[idx];
            let cell = self.chip.xlat_rc_wire(wn);
            let wire = self.edev.resolve_wire(cell.wire(wire)).unwrap();
            self.edev.wire_tree(wire)
        } else if let Some(w) = match suffix.as_str() {
            "HL7W0000" if self.chip.kind.has_ecp_plc() && self.chip.kind.has_out_ofx_branch() => {
                Some(wires::OUT_OFX_W[3])
            }
            "HL7W0001" if !self.chip.kind.has_ecp_plc() && self.chip.kind.has_out_ofx_branch() => {
                if matches!(self.chip.kind, ChipKind::Ecp5 | ChipKind::Crosslink) {
                    Some(wires::OUT_F_W[3])
                } else {
                    Some(wires::OUT_OFX_W[3])
                }
            }
            "HF0W0000" if self.chip.kind.has_out_f_branch() => Some(wires::OUT_F_W[0]),
            "HF1W0000" if self.chip.kind.has_out_f_branch() => Some(wires::OUT_F_W[1]),
            "HF2W0000" if self.chip.kind.has_out_f_branch() => Some(wires::OUT_F_W[2]),
            _ => None,
        } {
            let cell = self.chip.xlat_rc_wire(wn);
            if cell.col == self.chip.col_w() {
                return vec![];
            }
            vec![cell.delta(-1, 0).wire(w)]
        } else if matches!(suffix.as_str(), "GND" | "VCC")
            && !matches!(self.chip.kind, ChipKind::Nexus(_))
        {
            let w = if suffix == "VCC" {
                wires::TIE1
            } else {
                wires::TIE0
            };
            let cell = self.chip.xlat_rc_wire(wn);
            if matches!(self.chip.kind, ChipKind::Ecp3(_)) && cell.row <= self.chip.row_s() + 9 {
                if cell.col < self.chip.col_clk {
                    Vec::from_iter(
                        self.chip
                            .row_s()
                            .range(self.chip.row_s() + 10)
                            .into_iter()
                            .flat_map(|row| self.edev.row(cell.die, row))
                            .filter(|cell| cell.col < self.chip.col_clk)
                            .map(|cell| cell.wire(w)),
                    )
                } else {
                    Vec::from_iter(
                        self.chip
                            .row_s()
                            .range(self.chip.row_s() + 10)
                            .into_iter()
                            .flat_map(|row| self.edev.row(cell.die, row))
                            .filter(|cell| cell.col >= self.chip.col_clk)
                            .map(|cell| cell.wire(w)),
                    )
                }
            } else {
                if cell.col < self.chip.col_clk {
                    Vec::from_iter(
                        self.edev
                            .row(cell.die, cell.row)
                            .filter(|cell| cell.col < self.chip.col_clk)
                            .map(|cell| cell.wire(w)),
                    )
                } else {
                    Vec::from_iter(
                        self.edev
                            .row(cell.die, cell.row)
                            .filter(|cell| cell.col >= self.chip.col_clk)
                            .map(|cell| cell.wire(w)),
                    )
                }
            }
        } else if matches!(suffix.as_str(), "VCCHPBX" | "VCCVSPINE") {
            let cell = self.chip.xlat_rc_wire(wn);
            let cols = self.chip.cols_pclk.range(cell.col);
            let ChipKind::Nexus(kind) = self.chip.kind else {
                unreachable!()
            };
            let rows = if kind.is_large() {
                if cell.row < self.chip.row_clk {
                    self.chip.rows.ids().start().range(self.chip.row_clk)
                } else {
                    self.chip.row_clk.range(self.chip.rows.ids().end())
                }
            } else {
                self.chip.rows.ids()
            };
            let mut res = vec![];
            for col in cols {
                for row in rows {
                    res.push(Chip::DIE.cell(col, row).wire(wires::TIE1));
                }
            }
            res
        } else {
            let w = match suffix.as_str() {
                "A0" | "JA0" => wires::IMUX_A[0],
                "A1" | "JA1" => wires::IMUX_A[1],
                "A2" | "JA2" => wires::IMUX_A[2],
                "A3" | "JA3" => wires::IMUX_A[3],
                "A4" | "JA4" => wires::IMUX_A[4],
                "A5" | "JA5" => wires::IMUX_A[5],
                "A6" | "JA6" => wires::IMUX_A[6],
                "A7" | "JA7" => wires::IMUX_A[7],
                "B0" | "JB0" => wires::IMUX_B[0],
                "B1" | "JB1" => wires::IMUX_B[1],
                "B2" | "JB2" => wires::IMUX_B[2],
                "B3" | "JB3" => wires::IMUX_B[3],
                "B4" | "JB4" => wires::IMUX_B[4],
                "B5" | "JB5" => wires::IMUX_B[5],
                "B6" | "JB6" => wires::IMUX_B[6],
                "B7" | "JB7" => wires::IMUX_B[7],
                "C0" | "JC0" => wires::IMUX_C[0],
                "C1" | "JC1" => wires::IMUX_C[1],
                "C2" | "JC2" => wires::IMUX_C[2],
                "C3" | "JC3" => wires::IMUX_C[3],
                "C4" | "JC4" => wires::IMUX_C[4],
                "C5" | "JC5" => wires::IMUX_C[5],
                "C6" | "JC6" => wires::IMUX_C[6],
                "C7" | "JC7" => wires::IMUX_C[7],
                "D0" | "JD0" => wires::IMUX_D[0],
                "D1" | "JD1" => wires::IMUX_D[1],
                "D2" | "JD2" => wires::IMUX_D[2],
                "D3" | "JD3" => wires::IMUX_D[3],
                "D4" | "JD4" => wires::IMUX_D[4],
                "D5" | "JD5" => wires::IMUX_D[5],
                "D6" | "JD6" => wires::IMUX_D[6],
                "D7" | "JD7" => wires::IMUX_D[7],
                "M0" | "JM0" => wires::IMUX_M[0],
                "M1" | "JM1" => wires::IMUX_M[1],
                "M2" | "JM2" => wires::IMUX_M[2],
                "M3" | "JM3" => wires::IMUX_M[3],
                "M4" | "JM4" => wires::IMUX_M[4],
                "M5" | "JM5" => wires::IMUX_M[5],
                "M6" | "JM6" => wires::IMUX_M[6],
                "M7" | "JM7" => wires::IMUX_M[7],
                "JCIBMUXOUTA0" => wires::IMUX_MUXA[0],
                "JCIBMUXOUTA1" => wires::IMUX_MUXA[1],
                "JCIBMUXOUTA2" => wires::IMUX_MUXA[2],
                "JCIBMUXOUTA3" => wires::IMUX_MUXA[3],
                "JCIBMUXOUTA4" => wires::IMUX_MUXA[4],
                "JCIBMUXOUTA5" => wires::IMUX_MUXA[5],
                "JCIBMUXOUTA6" => wires::IMUX_MUXA[6],
                "JCIBMUXOUTA7" => wires::IMUX_MUXA[7],
                "JCIBMUXOUTB0" => wires::IMUX_MUXB[0],
                "JCIBMUXOUTB1" => wires::IMUX_MUXB[1],
                "JCIBMUXOUTB2" => wires::IMUX_MUXB[2],
                "JCIBMUXOUTB3" => wires::IMUX_MUXB[3],
                "JCIBMUXOUTB4" => wires::IMUX_MUXB[4],
                "JCIBMUXOUTB5" => wires::IMUX_MUXB[5],
                "JCIBMUXOUTB6" => wires::IMUX_MUXB[6],
                "JCIBMUXOUTB7" => wires::IMUX_MUXB[7],
                "JCIBMUXOUTC0" => wires::IMUX_MUXC[0],
                "JCIBMUXOUTC1" => wires::IMUX_MUXC[1],
                "JCIBMUXOUTC2" => wires::IMUX_MUXC[2],
                "JCIBMUXOUTC3" => wires::IMUX_MUXC[3],
                "JCIBMUXOUTC4" => wires::IMUX_MUXC[4],
                "JCIBMUXOUTC5" => wires::IMUX_MUXC[5],
                "JCIBMUXOUTC6" => wires::IMUX_MUXC[6],
                "JCIBMUXOUTC7" => wires::IMUX_MUXC[7],
                "JCIBMUXOUTD0" => wires::IMUX_MUXD[0],
                "JCIBMUXOUTD1" => wires::IMUX_MUXD[1],
                "JCIBMUXOUTD2" => wires::IMUX_MUXD[2],
                "JCIBMUXOUTD3" => wires::IMUX_MUXD[3],
                "JCIBMUXOUTD4" => wires::IMUX_MUXD[4],
                "JCIBMUXOUTD5" => wires::IMUX_MUXD[5],
                "JCIBMUXOUTD6" => wires::IMUX_MUXD[6],
                "JCIBMUXOUTD7" => wires::IMUX_MUXD[7],
                "JCLK0" if self.chip.kind == ChipKind::Ecp4 => wires::IMUX_CLK_DELAY[0],
                "JCLK1" if self.chip.kind == ChipKind::Ecp4 => wires::IMUX_CLK_DELAY[1],
                "MUXCLK0" => wires::IMUX_MUXCLK[0],
                "MUXCLK1" => wires::IMUX_MUXCLK[1],
                "MUXCLK2" => wires::IMUX_MUXCLK[2],
                "MUXCLK3" => wires::IMUX_MUXCLK[3],
                "MUXLSR0" => wires::IMUX_MUXLSR[0],
                "MUXLSR1" => wires::IMUX_MUXLSR[1],
                "MUXLSR2" => wires::IMUX_MUXLSR[2],
                "MUXLSR3" => wires::IMUX_MUXLSR[3],
                "CLK0" | "JCLK0" => wires::IMUX_CLK[0],
                "CLK1" | "JCLK1" => wires::IMUX_CLK[1],
                "CLK2" | "JCLK2" => wires::IMUX_CLK[2],
                "CLK3" | "JCLK3" => wires::IMUX_CLK[3],
                "LSR0" | "JLSR0" => wires::IMUX_LSR[0],
                "LSR1" | "JLSR1" => wires::IMUX_LSR[1],
                "LSR2" | "JLSR2" => wires::IMUX_LSR[2],
                "LSR3" | "JLSR3" => wires::IMUX_LSR[3],
                "CE0" | "JCE0" => wires::IMUX_CE[0],
                "CE1" | "JCE1" => wires::IMUX_CE[1],
                "CE2" | "JCE2" => wires::IMUX_CE[2],
                "CE3" | "JCE3" => wires::IMUX_CE[3],
                "F0" | "JF0" => wires::OUT_F[0],
                "F1" | "JF1" => wires::OUT_F[1],
                "F2" | "JF2" => wires::OUT_F[2],
                "F3" | "JF3" => wires::OUT_F[3],
                "F4" | "JF4" => wires::OUT_F[4],
                "F5" | "JF5" => wires::OUT_F[5],
                "F6" | "JF6" => wires::OUT_F[6],
                "F7" | "JF7" => wires::OUT_F[7],
                "Q0" | "JQ0" => wires::OUT_Q[0],
                "Q1" | "JQ1" => wires::OUT_Q[1],
                "Q2" | "JQ2" => wires::OUT_Q[2],
                "Q3" | "JQ3" => wires::OUT_Q[3],
                "Q4" | "JQ4" => wires::OUT_Q[4],
                "Q5" | "JQ5" => wires::OUT_Q[5],
                "Q6" | "JQ6" => wires::OUT_Q[6],
                "Q7" | "JQ7" => wires::OUT_Q[7],
                "OFX0" | "JOFX0" => wires::OUT_OFX[0],
                "OFX1" | "JOFX1" => wires::OUT_OFX[1],
                "OFX2" | "JOFX2" => wires::OUT_OFX[2],
                "OFX3" | "JOFX3" => wires::OUT_OFX[3],
                "OFX4" | "JOFX4" => wires::OUT_OFX[4],
                "OFX5" | "JOFX5" => wires::OUT_OFX[5],
                "OFX6" | "JOFX6" => wires::OUT_OFX[6],
                "OFX7" | "JOFX7" => wires::OUT_OFX[7],
                "JTI0" => wires::OUT_TI[0],
                "JTI1" => wires::OUT_TI[1],
                "JTI2" => wires::OUT_TI[2],
                "JTI3" => wires::OUT_TI[3],
                "JTI4" => wires::OUT_TI[4],
                "JTI5" => wires::OUT_TI[5],
                "JTI6" => wires::OUT_TI[6],
                "JTI7" => wires::OUT_TI[7],
                "JTI8" => wires::OUT_TI[8],
                "JTI9" => wires::OUT_TI[9],
                "JTI10" => wires::OUT_TI[10],
                "JTI11" => wires::OUT_TI[11],
                "HF4E0001" if self.chip.kind.has_out_f_branch() => wires::OUT_F_E[4],
                "HF5E0001" if self.chip.kind.has_out_f_branch() => wires::OUT_F_E[5],
                "HF6E0001" if self.chip.kind.has_out_f_branch() => wires::OUT_F_E[6],
                "HF7E0001" if self.chip.kind.has_out_f_branch() => wires::OUT_F_E[7],
                "H01M0100" => wires::X1_H[1],
                "H01M0400" => wires::X1_H[4],
                "V01M0100" => wires::X1_V[1],
                "V01M0400" => wires::X1_V[4],
                _ => return vec![],
            };
            let cell = self.chip.xlat_rc_wire(wn);
            vec![cell.wire(w)]
        }
    }

    fn classify_int_alias(&self, wn: WireName) -> Option<WireCoord> {
        let suffix = &self.naming.strings[wn.suffix];
        let (wires, idx) = if let Some(idx) = suffix.strip_prefix("JCIBMUXINA") {
            (wires::IMUX_A, idx)
        } else if let Some(idx) = suffix.strip_prefix("JCIBMUXINB") {
            (wires::IMUX_B, idx)
        } else if let Some(idx) = suffix.strip_prefix("JCIBMUXINC") {
            (wires::IMUX_C, idx)
        } else if let Some(idx) = suffix.strip_prefix("JCIBMUXIND") {
            (wires::IMUX_D, idx)
        } else {
            return None;
        };
        let idx: usize = idx.parse().unwrap();
        let cell = self.chip.xlat_rc_wire(wn);
        Some(cell.wire(wires[idx]))
    }

    fn gather_v01b(&mut self) {
        let v01b = self.naming.strings.get("V01B0000").unwrap();
        for &wn in self.nodes.values() {
            if wn.suffix != v01b {
                continue;
            }
            let cell = self.chip.xlat_rc_wire(wn);
            self.has_v01b.insert(cell);
        }
    }

    fn classify_sdclk(&mut self) {
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if !self.naming.strings[wtn.suffix].starts_with("HSSX") {
                continue;
            }
            if !matches!(
                self.naming.strings[wfn.suffix].as_str(),
                "JF6" | "JCE1" | "JCLK2"
            ) {
                continue;
            }
            let cell = self.chip.xlat_rc_wire(wfn);
            self.hsdclk_locs.insert(wtn, cell);
        }
    }

    pub(super) fn process_int_wires(&mut self) {
        if self.chip.kind == ChipKind::Scm {
            self.gather_v01b();
        }
        if self.chip.kind.has_distributed_sclk() {
            self.classify_sdclk();
        }
        for &wn in self.nodes.values() {
            let wires = self.classify_int_wire(wn);
            if wires.is_empty() {
                let suffix = self.naming.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "WESTKEEP"
                        | "EASTKEEP"
                        | "SOUTHKEEP"
                        | "SOUTHKEEP1"
                        | "NORTHKEEP"
                        | "NORTHKEEP1"
                        | "WBOUNCE"
                        | "EBOUNCE"
                        | "SBOUNCE"
                        | "NBOUNCE"
                ) {
                    self.keep_nodes.insert(wn);
                } else if matches!(
                    suffix,
                    "HF0W0000"
                        | "HF1W0000"
                        | "HF2W0000"
                        | "HF0W0001"
                        | "HF1W0001"
                        | "HF2W0001"
                        | "HF4E0000"
                        | "HF5E0000"
                        | "HF6E0000"
                        | "HF7E0000"
                        | "HF4E0001"
                        | "HF5E0001"
                        | "HF6E0001"
                        | "HF7E0001"
                        | "HL7W0000"
                        | "H01M0101"
                        | "H01M0401"
                        | "V01M0101"
                        | "V01M0401"
                        | "H00L0001"
                        | "H00L0101"
                        | "H00L0201"
                        | "H00L0301"
                        | "H00R0001"
                        | "H00R0101"
                        | "H00R0201"
                        | "H00R0301"
                        | "V00T0001"
                        | "V00T0101"
                        | "V00T0201"
                        | "V00T0301"
                        | "V00B0001"
                        | "V00B0101"
                        | "V00B0201"
                        | "V00B0301"
                ) || (suffix == "HL7W0001" && self.chip.kind == ChipKind::Ecp4)
                    || (matches!(
                        suffix,
                        "HPBX0800"
                            | "HPBX0900"
                            | "HPBX1000"
                            | "HPBX1100"
                            | "HPBX1200"
                            | "HPBX1300"
                            | "HPBX1400"
                            | "HPBX1500"
                    ) && matches!(self.chip.kind, ChipKind::Nexus(_)))
                {
                    self.discard_nodes.insert(wn);
                } else if let Some(wire) = self.classify_int_alias(wn) {
                    self.int_aliases.insert(wn, wire);
                } else {
                    self.unclaimed_nodes.insert(wn);
                }
            } else {
                let rwire0 = self.edev.resolve_wire(wires[0]).unwrap();
                for &wire in &wires {
                    if let Some(prev) = self.naming.interconnect.insert(wire, wn) {
                        panic!(
                            "wire {w} has two names: {prev} {cur}",
                            w = wire.to_string(self.intdb),
                            prev = prev.to_string(&self.naming),
                            cur = wn.to_string(&self.naming),
                        );
                    }
                    if !self.intdb[wire.slot].kind.is_tie() {
                        let rwire = self.edev.resolve_wire(wire).unwrap();
                        assert_eq!(
                            rwire,
                            rwire0,
                            "wire resolve mismatch: {wn} {w0} is {rw0}; {w} is {rw}",
                            wn = wn.to_string(&self.naming),
                            w0 = wires[0].to_string(self.intdb),
                            rw0 = rwire0.to_string(self.intdb),
                            w = wire.to_string(self.intdb),
                            rw = rwire.to_string(self.intdb),
                        );
                    }
                }
                self.int_wires.insert(wn, wires);
            }
        }
    }

    pub fn wire_to_tw(
        &self,
        bcrd: BelCoord,
        wire: WireCoord,
        nudge: Option<CellSlotId>,
    ) -> Option<TileWireCoord> {
        let tcrd = self.edev.bel_tile(bcrd);
        for (cslot, &cell) in &self.edev[tcrd].cells {
            if nudge.is_some() && nudge != Some(cslot) {
                continue;
            }
            let found = if let WireKind::Regional(rslot) = self.edev.db[wire.slot].kind {
                self.edev[cell].region_root[rslot] == self.edev[wire.cell].region_root[rslot]
            } else {
                cell == wire.cell
            };
            if found {
                return Some(TileWireCoord {
                    cell: cslot,
                    wire: wire.slot,
                });
            }
        }
        None
    }

    pub fn try_xlat_input(
        &mut self,
        bcrd: BelCoord,
        wire: WireName,
        filter: bool,
        nudge: Option<CellSlotId>,
    ) -> Option<BelInput> {
        for &wfn in self.pips_bwd.get(&wire).into_iter().flatten() {
            if let Some(wires_f) = self
                .int_wires
                .get(&wfn)
                .or_else(|| self.io_int_wires.get(&wfn))
            {
                let wf = wires_f[0];
                let Some(wf) = self.wire_to_tw(bcrd, wf, nudge) else {
                    if !filter {
                        println!(
                            "{name}: fail to xlat int wire {wf} in {bel}",
                            name = self.name,
                            wf = wf.to_string(self.intdb),
                            bel = bcrd.to_string(self.intdb)
                        );
                    }
                    continue;
                };
                self.claim_pip(wire, wfn);
                return Some(BelInput::Fixed(wf.pos()));
            }
        }
        None
    }

    pub fn xlat_input(&mut self, bcrd: BelCoord, wire: WireName) -> BelInput {
        self.try_xlat_input(bcrd, wire, false, None).unwrap()
    }

    pub fn xlat_input_nudge(
        &mut self,
        bcrd: BelCoord,
        wire: WireName,
        cslot: CellSlotId,
    ) -> BelInput {
        self.try_xlat_input(bcrd, wire, false, Some(cslot)).unwrap()
    }

    pub fn try_xlat_output(
        &mut self,
        bcrd: BelCoord,
        wire: WireName,
        filter: bool,
        quiet: bool,
        nudge: Option<CellSlotId>,
    ) -> Option<BTreeSet<TileWireCoord>> {
        if let Some(&(bcrd_out, _, ref out)) = self.hybrid_out.get(&wire) {
            assert_eq!(bcrd, bcrd_out);
            return Some(out.clone());
        }
        if let Some(wires) = self.io_int_wires.get(&wire) {
            let w = self.wire_to_tw(bcrd, wires[0], nudge).unwrap();
            return Some(BTreeSet::from([w]));
        }
        let mut out_wires = BTreeSet::new();
        for wtn in self.pips_fwd.get(&wire).cloned().into_iter().flatten() {
            if let Some(&(bcrd_out, _, ref out)) = self.hybrid_out.get(&wtn) {
                assert_eq!(bcrd, bcrd_out);
                let out = out.clone();
                self.claim_pip(wtn, wire);
                return Some(out);
            }
            if let Some(wires_t) = self
                .int_wires
                .get(&wtn)
                .or_else(|| self.io_int_wires.get(&wtn))
            {
                let wt = wires_t[0];
                if filter && !self.intdb.wires[wt.slot].name.starts_with("OUT") {
                    continue;
                }
                let Some(wt) = self.wire_to_tw(bcrd, wt, nudge) else {
                    if !quiet {
                        println!(
                            "{name}: fail to xlat int wire {wt} in {bel}",
                            name = self.name,
                            wt = wt.to_string(self.intdb),
                            bel = bcrd.to_string(self.intdb)
                        );
                    }
                    continue;
                };
                self.claim_pip(wtn, wire);
                out_wires.insert(wt);
            }
        }
        if !out_wires.is_empty() {
            Some(out_wires)
        } else {
            None
        }
    }

    pub fn xlat_output(&mut self, bcrd: BelCoord, wire: WireName) -> BTreeSet<TileWireCoord> {
        self.try_xlat_output(bcrd, wire, false, false, None)
            .unwrap()
    }

    pub fn xlat_output_nudge(
        &mut self,
        bcrd: BelCoord,
        wire: WireName,
        cslot: CellSlotId,
    ) -> BTreeSet<TileWireCoord> {
        self.try_xlat_output(bcrd, wire, false, false, Some(cslot))
            .unwrap()
    }

    pub fn xlat_output_quiet(&mut self, bcrd: BelCoord, wire: WireName) -> BTreeSet<TileWireCoord> {
        self.try_xlat_output(bcrd, wire, false, true, None).unwrap()
    }

    pub fn xlat_sc_wire(&mut self, wname: WireName, bcrd: BelCoord) -> TileWireCoord {
        let wname_int = [self.pips_fwd.get(&wname), self.pips_bwd.get(&wname)]
            .into_iter()
            .flatten()
            .flatten()
            .copied()
            .find(|&w| self.io_int_wires.contains_key(&w))
            .unwrap();
        self.claim_pip_bi(wname, wname_int);
        let wire = self.io_int_wires[&wname_int][0];
        let tile = &self.edev[self.edev.bel_tile(bcrd)];
        let wire = self
            .edev
            .wire_tree(wire)
            .into_iter()
            .find(|&w| tile.cells.values().any(|&c| w.cell == c))
            .unwrap();
        self.wire_to_tw(bcrd, wire, None).unwrap()
    }

    pub fn bind_wire(&mut self, name: WireName, wire: WireCoord) {
        let wire = self.edev.resolve_wire(wire).unwrap();
        match self.io_int_wires.entry(name) {
            btree_map::Entry::Vacant(e) => {
                e.insert(vec![wire]);
            }
            btree_map::Entry::Occupied(e) => {
                assert_eq!(*e.get(), [wire]);
            }
        }
    }

    pub fn hybrid_out(&mut self, bcrd: BelCoord, name: WireName, wire: TileWireCoord) {
        let mut out = self.xlat_output(bcrd, name);
        out.insert(wire);
        let wire = self.edev.resolve_bel_wire(bcrd, wire).unwrap();
        assert!(self.hybrid_out.insert(name, (bcrd, wire, out)).is_none());
        self.bind_wire(name, wire);
    }

    pub fn hybrid_out_rc_cell(
        &mut self,
        bcrd: BelCoord,
        cell: CellCoord,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) {
        let name = self.rc_wire(cell, suffix.as_ref());
        self.hybrid_out(bcrd, name, wire);
    }

    pub fn hybrid_out_rc(&mut self, bcrd: BelCoord, suffix: impl AsRef<str>, wire: TileWireCoord) {
        self.hybrid_out_rc_cell(bcrd, bcrd.cell, suffix, wire);
    }

    pub fn hybrid_out_rc_io_cell(
        &mut self,
        bcrd: BelCoord,
        cell: CellCoord,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) {
        let name = self.rc_io_wire(cell, suffix.as_ref());
        self.hybrid_out(bcrd, name, wire);
    }

    pub fn hybrid_out_rc_io(
        &mut self,
        bcrd: BelCoord,
        suffix: impl AsRef<str>,
        wire: TileWireCoord,
    ) {
        self.hybrid_out_rc_io_cell(bcrd, bcrd.cell, suffix, wire);
    }
}
