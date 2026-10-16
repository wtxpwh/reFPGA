use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use prjcombine_entity::{EntityId, EntityPartVec};
use prjcombine_interconnect::db::{
    BelInfo, BelSlotId, CellSlotId, IntDb, PinDir, PolTileWireCoord, TestMux, TestMuxWire,
    TileClassId, TileWireCoord, WireKind, WireSlotId,
};
use prjcombine_re_xilinx_naming::db::{
    BelNaming, BelPinNaming, IntfWireInNaming, PipNaming, RawTileId, TileClassNaming,
    TileClassNamingId, WireNaming,
};
use prjcombine_re_xilinx_rawdump::{self as rawdump, Coord, NodeOrWire, Part, TkPipInversion};

use crate::{
    IntBuilder, PipMode, Pips,
    bel::{AnyBel, TempBel},
};

#[derive(Default, Debug)]
pub struct XTileResult {
    pub bels: Vec<(TempBel, BelNaming)>,
}

#[derive(Clone, Debug)]
pub(crate) struct ExtrBelInfoSub {
    pub(crate) slot: Option<rawdump::TkSiteSlot>,
    pub(crate) pin_renames: HashMap<String, String>,
    pub(crate) pins: HashMap<String, BelPinInfo>,
    pub(crate) raw_tile: usize,
}

#[derive(Clone, Debug)]
pub struct ExtrBelInfo<'rd> {
    pub rd: &'rd Part,
    pub bel: BelSlotId,
    pub manual: bool,
    pub naming_only: bool,
    pub(crate) subs: Vec<ExtrBelInfoSub>,
}

#[derive(Clone, Debug)]
pub enum BelPinInfo {
    Int,
    NameOnly(usize),
    IntNudge(TileWireCoord),
    ForceInt(TileWireCoord, String),
    ExtraInt(PinDir, Vec<String>),
    ExtraIntForce(PinDir, TileWireCoord, String),
    ExtraWire(Vec<String>),
    ExtraWireForce(String, Vec<PipNaming>, Option<(String, PinDir)>),
    Dummy,
}

#[derive(Debug)]
pub struct XTileRawTile {
    pub xy: Coord,
    pub tile_map: Option<EntityPartVec<CellSlotId, CellSlotId>>,
    pub extract_muxes: bool,
}

#[derive(Debug)]
pub struct XTileRef {
    pub xy: Coord,
    pub naming: Option<TileClassNamingId>,
    pub tile_map: EntityPartVec<CellSlotId, CellSlotId>,
}

pub struct XTileInfo<'rd, 'b> {
    pub builder: &'b mut IntBuilder<'rd>,
    pub tcid: TileClassId,
    pub naming: String,
    pub raw_tiles: Vec<XTileRawTile>,
    pub refs: Vec<XTileRef>,
    pub tmux_bel: Option<BelSlotId>,
    pub intf_sb: Option<BelSlotId>,
    pub extract_delay: bool,
    pub has_intf_out_bufs: bool,
    pub skip_muxes: BTreeSet<WireSlotId>,
    pub optin_muxes: BTreeSet<WireSlotId>,
    pub optin_muxes_tile: BTreeSet<TileWireCoord>,
    pub bels: Vec<ExtrBelInfo<'rd>>,
    pub force_names: HashMap<(usize, rawdump::WireId), (IntConnKind, TileWireCoord)>,
    pub force_skip_pips: HashSet<(TileWireCoord, TileWireCoord)>,
    pub force_pips: HashSet<(TileWireCoord, PolTileWireCoord)>,
    pub switchbox: Option<BelSlotId>,
    pub force_test_mux_in: bool,
    pub force_ext_pips: bool,
    pub skip_edges: HashSet<(rawdump::WireId, rawdump::WireId)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum IntConnKind {
    Raw,
    IntfIn,
}

impl ExtrBelInfo<'_> {
    pub fn manual(mut self) -> Self {
        self.manual = true;
        self
    }

    pub fn naming_only(mut self) -> Self {
        self.naming_only = true;
        self
    }

    pub fn pin_rename(mut self, name_from: impl Into<String>, name_to: impl Into<String>) -> Self {
        self.subs
            .last_mut()
            .unwrap()
            .pin_renames
            .insert(name_from.into(), name_to.into());
        self
    }

    pub fn pins_name_only(mut self, names: &[impl AsRef<str>]) -> Self {
        for name in names {
            self.subs
                .last_mut()
                .unwrap()
                .pins
                .insert(name.as_ref().to_string(), BelPinInfo::NameOnly(0));
        }
        self
    }

    pub fn pin_name_only(mut self, name: &str, buf_cnt: usize) -> Self {
        self.subs
            .last_mut()
            .unwrap()
            .pins
            .insert(name.to_string(), BelPinInfo::NameOnly(buf_cnt));
        self
    }

    pub fn pin_dummy(mut self, name: impl Into<String>) -> Self {
        self.subs
            .last_mut()
            .unwrap()
            .pins
            .insert(name.into(), BelPinInfo::Dummy);
        self
    }

    pub fn pin_force_int(
        mut self,
        name: &str,
        wire: TileWireCoord,
        wname: impl Into<String>,
    ) -> Self {
        self.subs
            .last_mut()
            .unwrap()
            .pins
            .insert(name.to_string(), BelPinInfo::ForceInt(wire, wname.into()));
        self
    }

    pub fn pin_int_nudge(mut self, name: &str, wire: TileWireCoord) -> Self {
        self.subs
            .last_mut()
            .unwrap()
            .pins
            .insert(name.to_string(), BelPinInfo::IntNudge(wire));
        self
    }

    pub fn extra_int_out(
        mut self,
        name: impl Into<String>,
        wire_names: &[impl AsRef<str>],
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraInt(
                PinDir::Output,
                wire_names.iter().map(|x| x.as_ref().to_string()).collect(),
            ),
        );
        self
    }

    pub fn extra_int_in(mut self, name: impl Into<String>, wire_names: &[impl AsRef<str>]) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraInt(
                PinDir::Input,
                wire_names.iter().map(|x| x.as_ref().to_string()).collect(),
            ),
        );
        self
    }

    pub fn extra_int_inout(
        mut self,
        name: impl Into<String>,
        wire_names: &[impl AsRef<str>],
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraInt(
                PinDir::Inout,
                wire_names.iter().map(|x| x.as_ref().to_string()).collect(),
            ),
        );
        self
    }
    pub fn extra_int_out_force(
        mut self,
        name: impl Into<String>,
        wire: TileWireCoord,
        wire_name: impl Into<String>,
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraIntForce(PinDir::Output, wire, wire_name.into()),
        );
        self
    }

    pub fn extra_int_in_force(
        mut self,
        name: impl Into<String>,
        wire: TileWireCoord,
        wire_name: impl Into<String>,
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraIntForce(PinDir::Input, wire, wire_name.into()),
        );
        self
    }

    pub fn extra_wire(mut self, name: impl Into<String>, wire_names: &[impl AsRef<str>]) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraWire(wire_names.iter().map(|x| x.as_ref().to_string()).collect()),
        );
        self
    }

    pub fn extra_wire_force(
        mut self,
        name: impl Into<String>,
        wire_name: impl Into<String>,
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraWireForce(wire_name.into(), vec![], None),
        );
        self
    }

    pub fn extra_wire_force_out(
        mut self,
        name: impl Into<String>,
        wire_name: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.subs.last_mut().unwrap().pins.insert(
            name.clone(),
            BelPinInfo::ExtraWireForce(wire_name.into(), vec![], Some((name, PinDir::Output))),
        );
        self
    }

    pub fn extra_wire_force_pip(
        mut self,
        name: impl Into<String>,
        wire_name: impl Into<String>,
        pip: PipNaming,
    ) -> Self {
        self.subs.last_mut().unwrap().pins.insert(
            name.into(),
            BelPinInfo::ExtraWireForce(wire_name.into(), vec![pip], None),
        );
        self
    }

    pub fn sub_indexed(mut self, slot: &str, idx: usize) -> Self {
        self.subs.push(ExtrBelInfoSub {
            slot: Some(rawdump::TkSiteSlot::Indexed(
                self.rd.slot_kinds.get(slot).unwrap(),
                idx as u8,
            )),
            pins: Default::default(),
            pin_renames: Default::default(),
            raw_tile: 0,
        });
        self
    }

    pub fn sub_xy(mut self, slot: &str, x: usize, y: usize) -> Self {
        self.subs.push(ExtrBelInfoSub {
            slot: Some(self.rd.slot_xy(slot, x, y).expect("missing slot kind")),
            pins: Default::default(),
            pin_renames: Default::default(),
            raw_tile: 0,
        });
        self
    }

    pub fn sub_virtual(mut self) -> Self {
        self.subs.push(ExtrBelInfoSub {
            slot: None,
            pins: Default::default(),
            pin_renames: Default::default(),
            raw_tile: 0,
        });
        self
    }

    pub fn raw_tile(mut self, idx: usize) -> Self {
        self.subs.last_mut().unwrap().raw_tile = idx;
        self
    }
}

impl<'rd> XTileInfo<'rd, '_> {
    pub fn raw_tile(mut self, xy: Coord) -> Self {
        self.raw_tiles.push(XTileRawTile {
            xy,
            tile_map: None,
            extract_muxes: false,
        });
        self
    }

    pub fn raw_tile_single(mut self, xy: Coord, slot: usize) -> Self {
        self.raw_tiles.push(XTileRawTile {
            xy,
            tile_map: Some(
                [(CellSlotId::from_idx(0), CellSlotId::from_idx(slot))]
                    .into_iter()
                    .collect(),
            ),
            extract_muxes: false,
        });
        self
    }

    pub fn raw_tile_xlat(mut self, xy: Coord, slots: &[Option<usize>]) -> Self {
        self.raw_tiles.push(XTileRawTile {
            xy,
            tile_map: Some(
                slots
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        x.map(|x| (CellSlotId::from_idx(i), CellSlotId::from_idx(x)))
                    })
                    .collect(),
            ),
            extract_muxes: false,
        });
        self
    }

    pub fn ref_int(mut self, xy: Coord, slot: usize) -> Self {
        self.refs.push(XTileRef {
            xy,
            naming: None,
            tile_map: [(CellSlotId::from_idx(0), CellSlotId::from_idx(slot))]
                .into_iter()
                .collect(),
        });
        self
    }

    pub fn ref_single(mut self, xy: Coord, slot: usize, naming: TileClassNamingId) -> Self {
        self.refs.push(XTileRef {
            xy,
            naming: Some(naming),
            tile_map: [(CellSlotId::from_idx(0), CellSlotId::from_idx(slot))]
                .into_iter()
                .collect(),
        });
        self
    }

    pub fn ref_xlat(
        mut self,
        xy: Coord,
        slots: &[Option<usize>],
        naming: TileClassNamingId,
    ) -> Self {
        self.refs.push(XTileRef {
            xy,
            naming: Some(naming),
            tile_map: slots
                .iter()
                .enumerate()
                .filter_map(|(i, x)| x.map(|x| (CellSlotId::from_idx(i), CellSlotId::from_idx(x))))
                .collect(),
        });
        self
    }

    pub fn switchbox(mut self, sb: BelSlotId) -> Self {
        self.switchbox = Some(sb);
        self
    }

    pub fn extract_muxes(mut self, sb: BelSlotId) -> Self {
        self.switchbox = Some(sb);
        self.raw_tiles[0].extract_muxes = true;
        self
    }

    pub fn extract_muxes_rt(mut self, sb: BelSlotId, rt: usize) -> Self {
        self.switchbox = Some(sb);
        self.raw_tiles[rt].extract_muxes = true;
        self
    }

    pub fn extract_delay(mut self) -> Self {
        self.extract_delay = true;
        self
    }

    pub fn extract_intfs(
        mut self,
        tmux_bel: BelSlotId,
        sb_bel: Option<BelSlotId>,
        has_out_bufs: bool,
    ) -> Self {
        self.tmux_bel = Some(tmux_bel);
        self.intf_sb = sb_bel;
        self.has_intf_out_bufs = has_out_bufs;
        self
    }

    pub fn bel(mut self, bel: ExtrBelInfo<'rd>) -> Self {
        self.bels.push(bel);
        self
    }

    pub fn bels(mut self, bels: impl IntoIterator<Item = ExtrBelInfo<'rd>>) -> Self {
        for bel in bels {
            self.bels.push(bel);
        }
        self
    }

    pub fn skip_muxes<'a>(mut self, wires: impl IntoIterator<Item = &'a WireSlotId>) -> Self {
        self.skip_muxes.extend(wires.into_iter().copied());
        self
    }

    pub fn optin_muxes<'a>(mut self, wires: impl IntoIterator<Item = &'a WireSlotId>) -> Self {
        self.optin_muxes.extend(wires.into_iter().copied());
        self
    }

    pub fn optin_muxes_tile<'a>(
        mut self,
        wires: impl IntoIterator<Item = &'a TileWireCoord>,
    ) -> Self {
        self.optin_muxes_tile.extend(wires.into_iter().copied());
        self
    }

    pub fn force_name(mut self, rti: usize, name: &str, wire: TileWireCoord) -> Self {
        if let Some(w) = self.builder.rd.wires.get(name) {
            self.force_names.insert((rti, w), (IntConnKind::Raw, wire));
        }
        self
    }

    pub fn force_skip_pip(mut self, wt: TileWireCoord, wf: TileWireCoord) -> Self {
        self.force_skip_pips.insert((wt, wf));
        self
    }

    pub fn force_pip(mut self, wt: TileWireCoord, wf: PolTileWireCoord) -> Self {
        self.force_pips.insert((wt, wf));
        self
    }

    pub fn force_test_mux_in(mut self) -> Self {
        self.force_test_mux_in = true;
        self
    }

    pub fn force_ext_pips(mut self) -> Self {
        self.force_ext_pips = true;
        self
    }

    pub fn skip_edge(mut self, wt: &str, wf: &str) -> Self {
        if let Some(wti) = self.builder.rd.wires.get(wt)
            && let Some(wfi) = self.builder.rd.wires.get(wf)
        {
            self.skip_edges.insert((wti, wfi));
        }
        self
    }

    pub fn extract(self) -> XTileResult {
        let rd = self.builder.rd;

        let mut names: HashMap<NodeOrWire, (IntConnKind, TileWireCoord)> = HashMap::new();

        let mut edges_in: HashMap<_, Vec<_>> = HashMap::new();
        let mut edges_out: HashMap<_, Vec<_>> = HashMap::new();

        for (i, rt) in self.raw_tiles.iter().enumerate() {
            let tile = &rd.tiles[&rt.xy];
            let tk = &rd.tile_kinds[tile.kind];
            for &wi in tk.wires.keys() {
                let nw = rd.lookup_wire_raw_force(rt.xy, wi);
                if let Some(w) = self.builder.get_wire_by_name(tile.kind, &rd.wires[wi]) {
                    let mut w = w;
                    if let Some(ref tile_map) = rt.tile_map {
                        w.cell = tile_map[w.cell];
                    } else if self.builder.db[self.tcid].cells.len() == 1 {
                        w.cell = CellSlotId::from_idx(0);
                    }
                    names.entry(nw).or_insert((IntConnKind::Raw, w));
                }
            }
            for &(wfi, wti) in tk.pips.keys() {
                if self.skip_edges.contains(&(wti, wfi)) {
                    continue;
                }
                let nwf = rd.lookup_wire_raw_force(rt.xy, wfi);
                let nwt = rd.lookup_wire_raw_force(rt.xy, wti);
                edges_in.entry(nwt).or_default().push((nwf, i, wti, wfi));
                edges_out.entry(nwf).or_default().push((nwt, i, wti, wfi));
            }
        }

        for round in [0, 1] {
            for r in &self.refs {
                let tile = &rd.tiles[&r.xy];
                let tk = &rd.tile_kinds[tile.kind];

                let naming = if let Some(n) = r.naming {
                    n
                } else if let Some(n) = self.builder.get_int_naming(r.xy) {
                    n
                } else {
                    continue;
                };
                let naming = &self.builder.ndb.tile_class_namings[naming];
                for (&k, wn) in &naming.wires {
                    if round == 0
                        && matches!(
                            self.builder.db[k.wire].kind,
                            WireKind::Branch(_) | WireKind::MultiBranch(_)
                        )
                    {
                        continue;
                    }
                    if let Some(&wf) = self.builder.test_mux_ins.get(&k.wire) {
                        let wf = TileWireCoord {
                            cell: k.cell,
                            wire: wf,
                        };
                        if let Some(wfn) = naming.wires.get(&wf)
                            && wfn.name == wn.name
                        {
                            continue;
                        }
                    }
                    if let Some(nw) = rd.lookup_wire(r.xy, &wn.name)
                        && let Some(&ti) = r.tile_map.get(k.cell)
                    {
                        names.entry(nw).or_insert((
                            IntConnKind::Raw,
                            TileWireCoord {
                                cell: ti,
                                wire: k.wire,
                            },
                        ));
                    }
                }
                for (&k, v) in &naming.intf_wires_in {
                    match v {
                        IntfWireInNaming::Simple { name: n }
                        | IntfWireInNaming::TestBuf { name_in: n, .. } => {
                            if let Some(nw) = rd.lookup_wire(r.xy, n) {
                                names.entry(nw).or_insert((
                                    IntConnKind::Raw,
                                    TileWireCoord {
                                        cell: r.tile_map[k.cell],
                                        wire: k.wire,
                                    },
                                ));
                            }
                        }
                        IntfWireInNaming::Buf { name_out: n, .. } => {
                            if let Some(nw) = rd.lookup_wire(r.xy, n) {
                                names.entry(nw).or_insert((
                                    IntConnKind::IntfIn,
                                    TileWireCoord {
                                        cell: r.tile_map[k.cell],
                                        wire: k.wire,
                                    },
                                ));
                            }
                        }
                        IntfWireInNaming::Anonymous => (),
                    }
                }

                for &wi in tk.wires.keys() {
                    if let Some(nw) = rd.lookup_wire_raw(r.xy, wi)
                        && let Some(w) = self.builder.get_wire_by_name(tile.kind, &rd.wires[wi])
                    {
                        if round == 0
                            && matches!(
                                self.builder.db[w.wire].kind,
                                WireKind::Branch(_) | WireKind::MultiBranch(_)
                            )
                        {
                            continue;
                        }
                        if let Some(&t) = r.tile_map.get(w.cell) {
                            names.entry(nw).or_insert((
                                IntConnKind::Raw,
                                TileWireCoord {
                                    cell: t,
                                    wire: w.wire,
                                },
                            ));
                            continue;
                        }
                    }
                }
            }
        }

        let buf_out: HashMap<_, _> = edges_out
            .iter()
            .filter_map(|(&wt, wfs)| {
                if wfs.len() == 1 {
                    Some((wt, wfs[0]))
                } else {
                    None
                }
            })
            .collect();

        let int_out: HashMap<_, _> = edges_out
            .iter()
            .filter_map(|(&wt, wfs)| {
                let filtered: Vec<_> = wfs
                    .iter()
                    .copied()
                    .filter_map(|(x, t, wt, wf)| {
                        if let Some(&(ick, w)) = names.get(&x) {
                            Some((ick, w, t, wt, wf))
                        } else {
                            None
                        }
                    })
                    .collect();
                if !filtered.is_empty() {
                    Some((wt, filtered))
                } else {
                    None
                }
            })
            .collect();

        let buf_in: HashMap<_, _> = edges_in
            .iter()
            .filter_map(|(&wt, wfs)| {
                if wfs.len() == 1 {
                    Some((wt, wfs[0]))
                } else {
                    None
                }
            })
            .collect();

        let int_in: HashMap<_, _> = edges_in
            .iter()
            .filter_map(|(&wt, wfs)| {
                let filtered: Vec<_> = wfs
                    .iter()
                    .copied()
                    .filter_map(|(x, t, wt, wf)| {
                        if let Some(&(ick, w)) = names.get(&x) {
                            Some((ick, w, t, wt, wf))
                        } else {
                            None
                        }
                    })
                    .collect();
                if filtered.len() == 1 {
                    Some((wt, filtered[0]))
                } else {
                    None
                }
            })
            .collect();

        let mut extractor = XTileExtractor {
            rd: self.builder.rd,
            db: &self.builder.db,
            xtile: &self,
            names,
            buf_out,
            buf_in,
            int_out,
            int_in,
            bels: Default::default(),
            tcls_naming: TileClassNaming::default(),
            alt_names: Default::default(),
            alt_pips_to: Default::default(),
            alt_pips_from: Default::default(),
            pips: vec![],
            result: XTileResult::default(),
        };

        if self.raw_tiles.iter().any(|x| x.extract_muxes)
            || !self.optin_muxes.is_empty()
            || !self.optin_muxes_tile.is_empty()
        {
            extractor.extract_muxes();
        }

        extractor.extract_delay();
        if self.tmux_bel.is_some() {
            extractor.extract_tmux();
        }

        for bel in &self.bels {
            extractor.extract_bel(bel);
        }

        let mut tcls_naming = extractor.tcls_naming;

        for (wire, wn) in &mut tcls_naming.wires {
            wn.alt_name = extractor.alt_names.remove(wire);
            wn.alt_pips_to = extractor.alt_pips_to.remove(wire).unwrap_or_default();
            wn.alt_pips_from = extractor.alt_pips_from.remove(wire).unwrap_or_default();
        }

        let pips = extractor.pips;
        let result = extractor.result;
        for (bslot, bel) in extractor.bels {
            self.builder.insert_tcls_bel(self.tcid, bslot, bel);
        }
        for (bslot, pips) in pips {
            self.builder.insert_tcls_pips(self.tcid, bslot, pips);
        }
        self.builder.insert_tcls_naming(&self.naming, tcls_naming);

        result
    }

    fn name(&self) -> &str {
        &self.builder.db.tile_classes[self.tcid].name
    }
}

#[allow(clippy::type_complexity)]
struct XTileExtractor<'a, 'b, 'c> {
    rd: &'c Part,
    db: &'c IntDb,
    xtile: &'a XTileInfo<'b, 'c>,
    names: HashMap<NodeOrWire, (IntConnKind, TileWireCoord)>,
    buf_out: HashMap<NodeOrWire, (NodeOrWire, usize, rawdump::WireId, rawdump::WireId)>,
    buf_in: HashMap<NodeOrWire, (NodeOrWire, usize, rawdump::WireId, rawdump::WireId)>,
    int_out: HashMap<
        NodeOrWire,
        Vec<(
            IntConnKind,
            TileWireCoord,
            usize,
            rawdump::WireId,
            rawdump::WireId,
        )>,
    >,
    int_in: HashMap<
        NodeOrWire,
        (
            IntConnKind,
            TileWireCoord,
            usize,
            rawdump::WireId,
            rawdump::WireId,
        ),
    >,
    bels: BTreeMap<BelSlotId, AnyBel>,
    tcls_naming: TileClassNaming,
    alt_names: BTreeMap<TileWireCoord, String>,
    alt_pips_to: BTreeMap<TileWireCoord, BTreeSet<TileWireCoord>>,
    alt_pips_from: BTreeMap<TileWireCoord, BTreeSet<TileWireCoord>>,
    pips: Vec<(BelSlotId, Pips)>,
    result: XTileResult,
}

impl XTileExtractor<'_, '_, '_> {
    fn walk_to_int(
        &self,
        pin: &str,
        dir: PinDir,
        tile: usize,
        wire: rawdump::WireId,
    ) -> (
        IntConnKind,
        BTreeSet<TileWireCoord>,
        rawdump::WireId,
        Vec<PipNaming>,
        BTreeMap<TileWireCoord, PipNaming>,
    ) {
        let mut wn = wire;
        let mut nw = self
            .rd
            .lookup_wire_raw_force(self.xtile.raw_tiles[tile].xy, wire);
        let mut pips = Vec::new();
        loop {
            if let Some(&(ick, w)) = self.names.get(&nw) {
                return (ick, [w].into_iter().collect(), wn, pips, BTreeMap::new());
            }
            match dir {
                PinDir::Input => {
                    if let Some(&(ick, w, rt, wt, wf)) = self.int_in.get(&nw) {
                        pips.push(PipNaming {
                            tile: RawTileId::from_idx(rt),
                            wire_to: self.rd.wires[wt].clone(),
                            wire_from: self.rd.wires[wf].clone(),
                        });
                        if rt == tile {
                            wn = wf;
                        }
                        return (ick, [w].into_iter().collect(), wn, pips, BTreeMap::new());
                    }
                    if let Some(&(nnw, rt, wt, wf)) = self.buf_in.get(&nw) {
                        pips.push(PipNaming {
                            tile: RawTileId::from_idx(rt),
                            wire_to: self.rd.wires[wt].clone(),
                            wire_from: self.rd.wires[wf].clone(),
                        });
                        if rt == tile {
                            wn = wf;
                        }
                        nw = nnw;
                        continue;
                    }
                    panic!(
                        "CANNOT WALK INPUT WIRE {} {} {}",
                        self.rd.part,
                        self.xtile.name(),
                        pin
                    );
                }
                PinDir::Output => {
                    if let Some(nxt) = self.int_out.get(&nw) {
                        if nxt.len() == 1 {
                            let (ick, mut w, rt, wt, wf) = nxt[0];
                            if self.xtile.force_test_mux_in
                                && let Some(&nw) = self.xtile.builder.test_mux_ins.get(&w.wire)
                            {
                                w.wire = nw;
                            }
                            pips.push(PipNaming {
                                tile: RawTileId::from_idx(rt),
                                wire_to: self.rd.wires[wt].clone(),
                                wire_from: self.rd.wires[wf].clone(),
                            });
                            if rt == tile {
                                wn = wt;
                            }
                            return (ick, [w].into_iter().collect(), wn, pips, BTreeMap::new());
                        } else {
                            let mut wires = BTreeSet::new();
                            let mut int_pips = BTreeMap::new();
                            let mut ick = None;
                            for &(cick, mut w, rt, wt, wf) in nxt {
                                ick = Some(cick);
                                if self.xtile.force_test_mux_in
                                    && let Some(&nw) = self.xtile.builder.test_mux_ins.get(&w.wire)
                                {
                                    w.wire = nw;
                                }
                                wires.insert(w);
                                int_pips.insert(
                                    w,
                                    PipNaming {
                                        tile: RawTileId::from_idx(rt),
                                        wire_to: self.rd.wires[wt].clone(),
                                        wire_from: self.rd.wires[wf].clone(),
                                    },
                                );
                            }
                            return (ick.unwrap(), wires, wn, pips, int_pips);
                        }
                    }
                    if let Some(&(nnw, rt, wt, wf)) = self.buf_out.get(&nw) {
                        pips.push(PipNaming {
                            tile: RawTileId::from_idx(rt),
                            wire_to: self.rd.wires[wt].clone(),
                            wire_from: self.rd.wires[wf].clone(),
                        });
                        if rt == tile {
                            wn = wt;
                        }
                        nw = nnw;
                        continue;
                    }
                    panic!(
                        "CANNOT WALK OUTPUT WIRE {} {} {}",
                        self.rd.part,
                        self.xtile.name(),
                        pin
                    );
                }
                PinDir::Inout => {
                    panic!(
                        "CANNOT WALK INOUT WIRE {} {} {}",
                        self.rd.part,
                        self.xtile.name(),
                        pin
                    );
                }
            }
        }
    }

    fn walk_count(
        &self,
        pin: &str,
        dir: PinDir,
        cnt: usize,
        tile: usize,
        wire: rawdump::WireId,
    ) -> (rawdump::WireId, Vec<PipNaming>) {
        let mut wn = wire;
        let mut nw = self
            .rd
            .lookup_wire_raw_force(self.xtile.raw_tiles[tile].xy, wire);
        let mut pips = Vec::new();
        for _ in 0..cnt {
            match dir {
                PinDir::Input => {
                    if let Some(&(nnw, rt, wt, wf)) = self.buf_in.get(&nw) {
                        pips.push(PipNaming {
                            tile: RawTileId::from_idx(rt),
                            wire_to: self.rd.wires[wt].clone(),
                            wire_from: self.rd.wires[wf].clone(),
                        });
                        if rt == tile {
                            wn = wf;
                        }
                        nw = nnw;
                        continue;
                    }
                }
                PinDir::Output => {
                    if let Some(&(nnw, rt, wt, wf)) = self.buf_out.get(&nw) {
                        pips.push(PipNaming {
                            tile: RawTileId::from_idx(rt),
                            wire_to: self.rd.wires[wt].clone(),
                            wire_from: self.rd.wires[wf].clone(),
                        });
                        if rt == tile {
                            wn = wt;
                        }
                        nw = nnw;
                        continue;
                    }
                }
                PinDir::Inout => (),
            }
            panic!(
                "CANNOT WALK WIRE {} {} {}",
                self.rd.part,
                self.xtile.name(),
                pin
            );
        }
        (wn, pips)
    }

    fn extract_subbel(&mut self, bel: &mut TempBel, naming: &mut BelNaming, sub: &ExtrBelInfoSub) {
        let rt = RawTileId::from_idx(sub.raw_tile);
        naming.tiles.push(rt);
        let crd = self.xtile.raw_tiles[sub.raw_tile].xy;
        let tile = &self.rd.tiles[&crd];
        let tk = &self.rd.tile_kinds[tile.kind];
        if let Some(slot) = sub.slot {
            let tks = tk.sites.get(&slot).expect("missing site slot in tk").1;
            for (orig_pin, tksp) in &tks.pins {
                let name = sub.pin_renames.get(orig_pin).unwrap_or(orig_pin);
                let dir = match tksp.dir {
                    rawdump::TkSitePinDir::Input => PinDir::Input,
                    rawdump::TkSitePinDir::Output => PinDir::Output,
                    rawdump::TkSitePinDir::Bidir => PinDir::Inout,
                };
                match sub.pins.get(name).unwrap_or(&BelPinInfo::Int) {
                    &BelPinInfo::Int => {
                        if tksp.wire.is_none() {
                            panic!(
                                "missing site wire for pin {name} tile {tile}",
                                tile = self.xtile.name()
                            );
                        }
                        let (ick, wires, wnf, pips, int_pips) =
                            self.walk_to_int(name, dir, sub.raw_tile, tksp.wire.unwrap());
                        naming.pins.insert(
                            name.clone(),
                            BelPinNaming {
                                tile: rt,
                                orig_pin: Some((orig_pin.clone(), dir)),
                                name: self.rd.wires[tksp.wire.unwrap()].clone(),
                                name_far: self.rd.wires[wnf].clone(),
                                pips,
                                int_pips,
                                is_intf: ick != IntConnKind::Raw,
                            },
                        );
                        bel.insert_pin(name, dir, wires);
                    }
                    &BelPinInfo::IntNudge(wire) => {
                        if tksp.wire.is_none() {
                            panic!(
                                "missing site wire for pin {name} tile {tile}",
                                tile = self.xtile.name()
                            );
                        }
                        let (ick, _wires, wnf, pips, int_pips) =
                            self.walk_to_int(name, dir, sub.raw_tile, tksp.wire.unwrap());
                        naming.pins.insert(
                            name.clone(),
                            BelPinNaming {
                                tile: rt,
                                orig_pin: Some((orig_pin.clone(), dir)),
                                name: self.rd.wires[tksp.wire.unwrap()].clone(),
                                name_far: self.rd.wires[wnf].clone(),
                                pips,
                                int_pips,
                                is_intf: ick != IntConnKind::Raw,
                            },
                        );
                        bel.insert_pin(name, dir, [wire]);
                    }
                    &BelPinInfo::ForceInt(wire, ref wname) => {
                        naming.pins.insert(
                            name.clone(),
                            BelPinNaming {
                                tile: rt,
                                orig_pin: Some((orig_pin.clone(), dir)),
                                name: self.rd.wires[tksp.wire.unwrap()].clone(),
                                name_far: wname.clone(),
                                pips: Vec::new(),
                                int_pips: BTreeMap::new(),
                                is_intf: false,
                            },
                        );
                        bel.insert_pin(name, dir, [wire]);
                    }
                    &BelPinInfo::NameOnly(buf_cnt) => {
                        if tksp.wire.is_none() {
                            panic!(
                                "missing site wire for pin {name} tile {tile}",
                                tile = self.xtile.name()
                            );
                        }
                        if buf_cnt == 0 {
                            naming.pins.insert(
                                name.clone(),
                                BelPinNaming {
                                    tile: rt,
                                    orig_pin: Some((orig_pin.clone(), dir)),
                                    name: self.rd.wires[tksp.wire.unwrap()].clone(),
                                    name_far: self.rd.wires[tksp.wire.unwrap()].clone(),
                                    pips: Vec::new(),
                                    int_pips: BTreeMap::new(),
                                    is_intf: false,
                                },
                            );
                        } else {
                            let (wn, pips) = self.walk_count(
                                name,
                                dir,
                                buf_cnt,
                                sub.raw_tile,
                                tksp.wire.unwrap(),
                            );
                            naming.pins.insert(
                                name.clone(),
                                BelPinNaming {
                                    tile: rt,
                                    orig_pin: Some((orig_pin.clone(), dir)),
                                    name: self.rd.wires[tksp.wire.unwrap()].clone(),
                                    name_far: self.rd.wires[wn].clone(),
                                    pips,
                                    int_pips: BTreeMap::new(),
                                    is_intf: false,
                                },
                            );
                        }
                    }
                    BelPinInfo::Dummy => (),
                    BelPinInfo::ExtraWireForce(_, _, _) => (),
                    BelPinInfo::ExtraInt(_, _) => (),
                    BelPinInfo::ExtraWire(_) => (),
                    _ => unreachable!(),
                }
            }
        }
        for (name, pd) in &sub.pins {
            match *pd {
                BelPinInfo::ExtraInt(dir, ref names) => {
                    let mut wn = None;
                    for w in names {
                        if let Some(w) = self.rd.wires.get(w)
                            && tk.wires.contains_key(&w)
                        {
                            assert!(wn.is_none());
                            wn = Some(w);
                        }
                    }
                    if wn.is_none() {
                        println!("NOT FOUND: {name}");
                    }
                    let wn = wn.unwrap();
                    let (ick, wires, wnf, pips, int_pips) =
                        self.walk_to_int(name, dir, sub.raw_tile, wn);
                    naming.pins.insert(
                        name.clone(),
                        BelPinNaming {
                            tile: rt,
                            orig_pin: None,
                            name: self.rd.wires[wn].clone(),
                            name_far: self.rd.wires[wnf].clone(),
                            pips,
                            int_pips,
                            is_intf: ick != IntConnKind::Raw,
                        },
                    );
                    bel.insert_pin(name, dir, wires);
                }
                BelPinInfo::ExtraIntForce(dir, wire, ref wname) => {
                    naming.pins.insert(
                        name.clone(),
                        BelPinNaming {
                            tile: rt,
                            orig_pin: None,
                            name: wname.clone(),
                            name_far: wname.clone(),
                            pips: vec![],
                            int_pips: BTreeMap::new(),
                            is_intf: false,
                        },
                    );
                    bel.insert_pin(name, dir, [wire]);
                }
                BelPinInfo::ExtraWire(ref names) => {
                    let mut wn = None;
                    for w in names {
                        if let Some(w) = self.rd.wires.get(w)
                            && tk.wires.contains_key(&w)
                        {
                            if let Some(wn) = wn {
                                println!(
                                    "COLLISION {wn} {w}",
                                    wn = self.rd.wires[wn],
                                    w = self.rd.wires[w]
                                );
                            }
                            assert!(wn.is_none());
                            wn = Some(w);
                        }
                    }
                    if wn.is_none() {
                        println!("NOT FOUND: {name}");
                    }
                    let wn = wn.unwrap();
                    naming.pins.insert(
                        name.clone(),
                        BelPinNaming {
                            tile: rt,
                            orig_pin: None,
                            name: self.rd.wires[wn].clone(),
                            name_far: self.rd.wires[wn].clone(),
                            pips: Vec::new(),
                            int_pips: BTreeMap::new(),
                            is_intf: false,
                        },
                    );
                }
                BelPinInfo::ExtraWireForce(ref wname, ref pips, ref orig_pin) => {
                    naming.pins.insert(
                        name.clone(),
                        BelPinNaming {
                            tile: rt,
                            orig_pin: orig_pin.clone(),
                            name: wname.clone(),
                            name_far: wname.clone(),
                            pips: pips.clone(),
                            int_pips: BTreeMap::new(),
                            is_intf: false,
                        },
                    );
                }
                _ => (),
            }
        }
    }

    fn extract_bel(&mut self, info: &ExtrBelInfo) {
        let mut bel = TempBel::default();
        let mut naming = BelNaming {
            tiles: vec![],
            pins: Default::default(),
        };
        for sub in &info.subs {
            self.extract_subbel(&mut bel, &mut naming, sub);
        }
        if info.manual {
            self.result.bels.push((bel, naming));
        } else {
            if !info.naming_only {
                self.bels.insert(info.bel, AnyBel::Temp(bel));
            }
            self.tcls_naming.bels.insert(info.bel, naming);
        }
    }

    fn get_wire_by_name(&self, rti: usize, name: rawdump::WireId) -> Option<TileWireCoord> {
        let rt = &self.xtile.raw_tiles[rti];
        let tile = &self.rd.tiles[&rt.xy];
        if let Some(&(IntConnKind::Raw, res)) = self.xtile.force_names.get(&(rti, name)) {
            return Some(res);
        }
        if let Some(TileWireCoord { cell: t, wire: w }) = self
            .xtile
            .builder
            .get_wire_by_name(tile.kind, &self.rd.wires[name])
            && let Some(&xt) = rt.tile_map.as_ref().and_then(|x| x.get(t))
        {
            return Some(TileWireCoord { cell: xt, wire: w });
        }
        let nw = self.rd.lookup_wire_raw_force(rt.xy, name);
        if let Some(&(_, w)) = self.names.get(&nw) {
            return Some(w);
        }
        None
    }

    fn get_alt_by_name(&self, rti: usize, name: rawdump::WireId) -> Option<TileWireCoord> {
        let rt = &self.xtile.raw_tiles[rti];
        let tile = &self.rd.tiles[&rt.xy];
        if let Some(TileWireCoord { cell: t, wire: w }) = self
            .xtile
            .builder
            .get_wire_by_alt_name(tile.kind, &self.rd.wires[name])
        {
            if let Some(ref tm) = rt.tile_map {
                if let Some(&xt) = tm.get(t) {
                    return Some(TileWireCoord { cell: xt, wire: w });
                }
            } else {
                return Some(TileWireCoord { cell: t, wire: w });
            }
        }
        None
    }

    fn extract_muxes(&mut self) {
        let mut pips = Pips::default();
        for &(wt, wf) in &self.xtile.force_pips {
            let mode = self.xtile.builder.pip_mode(wt.wire);
            pips.pips.insert((wt, wf), mode);
        }
        for (i, rt) in self.xtile.raw_tiles.iter().enumerate() {
            let tile = &self.rd.tiles[&rt.xy];
            let tk = &self.rd.tile_kinds[tile.kind];

            for (_, &(wfi, wti), pip) in &tk.pips {
                let inv = pip.inversion == TkPipInversion::Always;
                if self.xtile.skip_edges.contains(&(wti, wfi)) {
                    continue;
                }
                if let Some(wt) = self.get_wire_by_name(i, wti) {
                    let mut pass = rt.extract_muxes
                        && !matches!(self.db[wt.wire].kind, WireKind::BelOut)
                        && !self.xtile.skip_muxes.contains(&wt.wire);
                    if self.xtile.optin_muxes.contains(&wt.wire) {
                        pass = true;
                    }
                    if self.xtile.optin_muxes_tile.contains(&wt) {
                        pass = true;
                    }
                    if !pass {
                        continue;
                    }
                    if let Some(wf) = self.get_wire_by_name(i, wfi) {
                        let wf = PolTileWireCoord { tw: wf, inv };
                        if self.xtile.force_skip_pips.contains(&(wt, wf.tw)) {
                            continue;
                        }
                        if i == 0 && !self.xtile.force_ext_pips {
                            self.tcls_naming.wires.insert(
                                wt,
                                WireNaming {
                                    name: self.rd.wires[wti].to_string(),
                                    alt_name: None,
                                    alt_pips_to: Default::default(),
                                    alt_pips_from: Default::default(),
                                },
                            );
                            self.tcls_naming.wires.insert(
                                wf.tw,
                                WireNaming {
                                    name: self.rd.wires[wfi].to_string(),
                                    alt_name: None,
                                    alt_pips_to: Default::default(),
                                    alt_pips_from: Default::default(),
                                },
                            );
                        } else {
                            self.tcls_naming.ext_pips.insert(
                                (wt, wf.tw),
                                PipNaming {
                                    tile: RawTileId::from_idx(i),
                                    wire_to: self.rd.wires[wti].to_string(),
                                    wire_from: self.rd.wires[wfi].to_string(),
                                },
                            );
                        }
                        let mode = self.xtile.builder.pip_mode(wt.wire);
                        pips.pips.insert((wt, wf), mode);
                    } else if i == 0
                        && let Some(wf) = self.get_alt_by_name(i, wfi)
                    {
                        let wf = PolTileWireCoord { tw: wf, inv };
                        self.tcls_naming.wires.insert(
                            wt,
                            WireNaming {
                                name: self.rd.wires[wti].to_string(),
                                alt_name: None,
                                alt_pips_to: Default::default(),
                                alt_pips_from: Default::default(),
                            },
                        );
                        self.alt_names.insert(wf.tw, self.rd.wires[wfi].to_string());
                        if wt == wf.tw {
                            continue;
                        }
                        if self.xtile.force_skip_pips.contains(&(wt, wf.tw)) {
                            continue;
                        }
                        self.alt_pips_from.entry(wf.tw).or_default().insert(wt);
                        let mode = self.xtile.builder.pip_mode(wt.wire);
                        pips.pips.insert((wt, wf), mode);
                    } else if self.xtile.builder.stub_outs.contains(&self.rd.wires[wfi]) {
                        // ignore
                    } else {
                        println!(
                            "UNEXPECTED XTILE MUX IN {} {} {} {}",
                            self.rd.tile_kinds.key(tile.kind),
                            tile.name,
                            self.rd.wires[wti],
                            self.rd.wires[wfi]
                        );
                    }
                } else if i == 0
                    && let Some(wt) = self.get_alt_by_name(i, wti)
                {
                    let mut pass = rt.extract_muxes
                        && !matches!(self.db[wt.wire].kind, WireKind::BelOut)
                        && !self.xtile.skip_muxes.contains(&wt.wire);
                    if self.xtile.optin_muxes.contains(&wt.wire) {
                        pass = true;
                    }
                    if self.xtile.optin_muxes_tile.contains(&wt) {
                        pass = true;
                    }
                    if !pass {
                        continue;
                    }
                    if let Some(wf) = self.get_wire_by_name(i, wfi) {
                        let wf = PolTileWireCoord { tw: wf, inv };
                        self.tcls_naming.wires.insert(
                            wf.tw,
                            WireNaming {
                                name: self.rd.wires[wfi].to_string(),
                                alt_name: None,
                                alt_pips_to: Default::default(),
                                alt_pips_from: Default::default(),
                            },
                        );
                        self.alt_names.insert(wt, self.rd.wires[wti].to_string());
                        if wt == wf.tw {
                            continue;
                        }
                        if self.xtile.force_skip_pips.contains(&(wt, wf.tw)) {
                            continue;
                        }
                        self.alt_pips_to.entry(wt).or_default().insert(wf.tw);
                        let mode = self.xtile.builder.pip_mode(wt.wire);
                        pips.pips.insert((wt, wf), mode);
                    }
                }
            }
        }
        self.pips.push((self.xtile.switchbox.unwrap(), pips));
    }

    fn extract_delay(&mut self) {
        let crd = self.xtile.raw_tiles[0].xy;
        let tile = &self.rd.tiles[&crd];
        let tk = &self.rd.tile_kinds[tile.kind];
        if let Some(sb) = self.xtile.intf_sb
            && self.xtile.extract_delay
        {
            let mut pips = Pips::default();
            for &(wfi, wdi) in tk.pips.keys() {
                let nwf = self.rd.lookup_wire_raw_force(crd, wfi);
                let nwd = self.rd.lookup_wire_raw_force(crd, wdi);
                if !self.buf_in.contains_key(&nwd) {
                    continue;
                }
                let Some(&(_, rt, wti, bwdi)) = self.buf_out.get(&nwd) else {
                    continue;
                };
                if rt != 0 {
                    continue;
                }
                if !tk.pips.contains_key(&(wfi, wti)) {
                    continue;
                }
                let nwt = self.rd.lookup_wire_raw_force(crd, wti);
                if let Some(&(_, wf)) = self.names.get(&nwf) {
                    if !matches!(self.db[wf.wire].kind, WireKind::MuxOut) {
                        continue;
                    }
                    let Some(&wtw) = self.xtile.builder.delay_wires.get(&wf.wire) else {
                        continue;
                    };
                    let wt = TileWireCoord {
                        cell: wf.cell,
                        wire: wtw,
                    };
                    assert_eq!(bwdi, wdi);
                    self.tcls_naming.wires.insert(
                        wf,
                        WireNaming {
                            name: self.rd.wires[wfi].clone(),
                            alt_name: None,
                            alt_pips_to: Default::default(),
                            alt_pips_from: Default::default(),
                        },
                    );
                    self.tcls_naming.wires.insert(
                        wt,
                        WireNaming {
                            name: self.rd.wires[wti].clone(),
                            alt_name: None,
                            alt_pips_to: Default::default(),
                            alt_pips_from: Default::default(),
                        },
                    );
                    self.tcls_naming
                        .delay_wires
                        .insert(wt, self.rd.wires[wdi].clone());
                    self.names.insert(nwt, (IntConnKind::Raw, wt));
                    pips.pips.insert((wt, wf.pos()), PipMode::Delay);
                }
            }
            self.pips.push((sb, pips));
        }
    }

    fn extract_tmux(&mut self) {
        let crd = self.xtile.raw_tiles[0].xy;
        let tile = &self.rd.tiles[&crd];
        let tk = &self.rd.tile_kinds[tile.kind];
        let mut out_muxes: BTreeMap<TileWireCoord, (Vec<TileWireCoord>, Option<TileWireCoord>)> =
            BTreeMap::new();
        for &(wfi, wti) in tk.pips.keys() {
            let nwt = self.rd.lookup_wire_raw_force(crd, wti);
            if let Some(&(_, wt)) = self.names.get(&nwt) {
                if !matches!(self.db[wt.wire].kind, WireKind::BelOut) {
                    continue;
                }
                self.tcls_naming
                    .wires
                    .entry(wt)
                    .or_insert_with(|| WireNaming {
                        name: self.rd.wires[wti].clone(),
                        alt_name: None,
                        alt_pips_to: Default::default(),
                        alt_pips_from: Default::default(),
                    });
                if !self.xtile.has_intf_out_bufs {
                    let wf = TileWireCoord {
                        cell: wt.cell,
                        wire: self.xtile.builder.test_mux_ins[&wt.wire],
                    };
                    self.tcls_naming.wires.insert(
                        wf,
                        WireNaming {
                            name: self.rd.wires[wti].clone(),
                            alt_name: None,
                            alt_pips_to: Default::default(),
                            alt_pips_from: Default::default(),
                        },
                    );
                }
                let nwf = self.rd.lookup_wire_raw_force(crd, wfi);
                if let Some(&(_, mut wf)) = self.names.get(&nwf) {
                    if let Some(&w) = self.xtile.builder.optinvs.get(&wf.wire) {
                        wf.wire = w;
                    }
                    self.tcls_naming.intf_wires_in.insert(
                        wf,
                        IntfWireInNaming::Simple {
                            name: self.rd.wires[wfi].clone(),
                        },
                    );
                    self.tcls_naming.wires.insert(
                        wf,
                        WireNaming {
                            name: self.rd.wires[wfi].clone(),
                            alt_name: None,
                            alt_pips_to: Default::default(),
                            alt_pips_from: Default::default(),
                        },
                    );
                    if self.db[wf.wire].kind == WireKind::BelOut
                        || self.xtile.builder.test_mux_pass.contains(&wf.wire)
                    {
                        assert!(out_muxes.entry(wt).or_default().1.replace(wf).is_none());
                    } else {
                        out_muxes.entry(wt).or_default().0.push(wf);
                    }
                } else if let Some(&(_, mut wf, rt, bwti, bwfi)) = self.int_in.get(&nwf) {
                    if let Some(&w) = self.xtile.builder.optinvs.get(&wf.wire) {
                        wf.wire = w;
                    }
                    if !self.buf_in.contains_key(&nwf) {
                        assert!(!self.xtile.has_intf_out_bufs);
                        continue;
                    }
                    assert_eq!(rt, 0);
                    assert_eq!(bwti, wfi);
                    self.tcls_naming.intf_wires_in.insert(
                        wf,
                        IntfWireInNaming::TestBuf {
                            name_out: self.rd.wires[wfi].clone(),
                            name_in: self.rd.wires[bwfi].clone(),
                        },
                    );
                    out_muxes.entry(wt).or_default().0.push(wf);
                } else if self.xtile.has_intf_out_bufs {
                    out_muxes.entry(wt).or_default();
                    let wf = TileWireCoord {
                        cell: wt.cell,
                        wire: self.xtile.builder.test_mux_ins[&wt.wire],
                    };
                    self.tcls_naming.wires.insert(
                        wf,
                        WireNaming {
                            name: self.rd.wires[wfi].clone(),
                            alt_name: None,
                            alt_pips_to: Default::default(),
                            alt_pips_from: Default::default(),
                        },
                    );
                }
            }
        }
        let mut pips = Pips::default();
        for (&wt, (wfs, _)) in out_muxes.iter_mut() {
            if let Some(&nwf) = self.xtile.builder.test_mux_ins_test.get(&wt.wire) {
                let nwf = TileWireCoord {
                    cell: wt.cell,
                    wire: nwf,
                };
                self.tcls_naming
                    .intf_wires_in
                    .insert(nwf, IntfWireInNaming::Anonymous);
                for &wf in wfs.iter() {
                    pips.pips.insert((nwf, wf.pos()), PipMode::Mux);
                }
                *wfs = vec![nwf];
            }
            wfs.sort();
        }
        if !pips.pips.is_empty() {
            self.pips.push((self.xtile.intf_sb.unwrap(), pips));
        }
        let num_groups = out_muxes.values().map(|(wfs, _)| wfs.len()).max().unwrap();
        let mut tm = TestMux {
            bits: vec![],
            groups: vec![Default::default(); num_groups],
            bits_primary: Default::default(),
            wires: Default::default(),
        };
        for (wt, (wfs, pwf)) in out_muxes {
            let mut wfs = Vec::from_iter(wfs.into_iter().map(|x| Some(x.pos())));
            while wfs.len() < num_groups {
                wfs.push(None);
            }
            let pwf = pwf.unwrap_or_else(|| TileWireCoord {
                cell: wt.cell,
                wire: self.xtile.builder.test_mux_ins[&wt.wire],
            });
            tm.wires.insert(
                wt,
                TestMuxWire {
                    primary_src: pwf.pos(),
                    test_src: wfs,
                },
            );
        }
        self.bels
            .insert(self.xtile.tmux_bel.unwrap(), BelInfo::TestMux(tm).into());
    }
}
