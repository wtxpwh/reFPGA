use prjcombine_entity::EntityId;
use prjcombine_interconnect::db::{BelSlotId, CellSlotId, IntDb, TileWireCoord, WireSlotId};
use prjcombine_interconnect::dir::{Dir, DirH};
use prjcombine_re_xilinx_naming::db::NamingDb;
use prjcombine_re_xilinx_naming_versal::DeviceNaming;
use prjcombine_re_xilinx_rawdump::{Coord, NodeOrWire, Part};
use prjcombine_types::enumerable::EnumMap;
use prjcombine_versal::defs::{self, ccls};
use std::collections::BTreeMap;

use prjcombine_re_xilinx_rd2db_interconnect::{
    IntBuilder,
    tile::{ExtrBelInfo, XTileInfo, XTileRef},
};

use crate::bli::{BliConns, get_bli};

mod aie;
mod clk;
mod fabric;
mod gt;
mod hsr;
mod intf;
mod ps;
mod wire_names;
mod x5io;
mod xpio;

const SWZ12: [usize; 12] = [0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2];

trait IntBuilderExt {
    fn wire_names_pair(&mut self, wire: WireSlotId, raw_names: &[impl AsRef<str>; 2]);
}

impl IntBuilderExt for IntBuilder<'_> {
    fn wire_names_pair(&mut self, wire: WireSlotId, raw_names: &[impl AsRef<str>; 2]) {
        for (sub, name) in raw_names.iter().enumerate() {
            self.extra_name_cell(name.as_ref(), sub, wire);
        }
    }
}

trait XTileInfoExt {
    fn ref_int_side(self, xy: Coord, side: DirH, slot: usize) -> Self;
}

impl XTileInfoExt for XTileInfo<'_, '_> {
    fn ref_int_side(mut self, xy: Coord, side: DirH, slot: usize) -> Self {
        self.refs.push(XTileRef {
            xy,
            naming: None,
            tile_map: [(
                CellSlotId::from_idx(match side {
                    DirH::W => 0,
                    DirH::E => 1,
                }),
                CellSlotId::from_idx(slot),
            )]
            .into_iter()
            .collect(),
        });
        self
    }
}

struct IntMaker<'rd> {
    rd: &'rd Part,
    builder: IntBuilder<'rd>,
    // how many mental illnesses do you think I could be diagnosed with just from this repo?
    sng_fixup_map: BTreeMap<TileWireCoord, TileWireCoord>,
    iri_wires: Vec<WireSlotId>,
    dev_naming: &'rd DeviceNaming,
    bli: BliConns,
}

impl<'rd> IntMaker<'rd> {
    fn bel_pins(
        &self,
        xy: Coord,
        slot: &str,
        sx: usize,
        sy: usize,
    ) -> impl Iterator<Item = &'rd str> {
        let tile = &self.rd.tiles[&xy];
        let tk = &self.rd.tile_kinds[tile.kind];
        let slot = self.rd.slot_xy(slot, sx, sy).unwrap();
        let tksite = tk.sites.get(&slot).unwrap().1;
        tksite.pins.keys().map(|x| x.as_str())
    }

    fn bel_name_only(
        &self,
        xy: Coord,
        bslot: BelSlotId,
        slot: &str,
        sx: usize,
        sy: usize,
    ) -> ExtrBelInfo<'rd> {
        let mut bel = self.builder.bel_xy(bslot, slot, sx, sy);
        for pin in self.bel_pins(xy, slot, sx, sy) {
            bel = bel.pin_name_only(pin, 1);
        }
        bel
    }

    fn follow_wire(&self, xy: Coord, wn: &str) -> Option<Coord> {
        let ni = self.rd.lookup_wire(xy, wn)?;
        let NodeOrWire::Node(ni) = ni else {
            return None;
        };
        Some(self.rd.iter_node(ni).find(|&(cxy, _)| cxy != xy)?.0)
    }

    fn follow_wire_filter(&self, xy: Coord, wn: &str, p: impl Fn(&str) -> bool) -> Option<Coord> {
        let ni = self.rd.lookup_wire(xy, wn)?;
        let NodeOrWire::Node(ni) = ni else {
            return None;
        };
        Some(
            self.rd
                .iter_node(ni)
                .find(|&(cxy, _)| cxy != xy && p(&self.rd.tiles[&cxy].name))?
                .0,
        )
    }

    fn xlat_bli_full(&self, xy: Coord, wire: &str) -> Option<(Coord, &'rd str)> {
        let wi = self.rd.wires.get(wire)?;
        let &(bli_crd, w) = self.bli.get(&(xy, wi))?;
        Some((bli_crd, &self.rd.wires[w]))
    }

    fn xlat_bli(&self, xy: Coord, wire: &str) -> Option<Coord> {
        Some(self.xlat_bli_full(xy, wire)?.0)
    }
}

pub fn make_int_db(rd: &Part, dev_naming: &DeviceNaming) -> (IntDb, NamingDb) {
    let bli = get_bli(rd);
    let mut maker = IntMaker {
        rd,
        builder: IntBuilder::new(
            rd,
            bincode::decode_from_slice(defs::INIT, bincode::config::standard())
                .unwrap()
                .0,
        ),
        sng_fixup_map: BTreeMap::new(),
        iri_wires: vec![],
        dev_naming,
        bli,
    };

    maker
        .builder
        .inject_main_passes(EnumMap::from_fn(|dir| match dir {
            Dir::W => ccls::PASS_W,
            Dir::E => ccls::PASS_E,
            Dir::S => ccls::PASS_S,
            Dir::N => ccls::PASS_N,
        }));

    let crd = rd.tiles_by_kind_name("INT").first().unwrap();
    let tile = &rd.tiles[crd];
    if tile.name.contains("_S") {
        maker.builder.set_mirror_square();
    }
    maker.fill_wires();
    maker.fill_tiles_int();
    maker.fill_tiles_cle_bc();
    maker.fill_tiles_intf();
    maker.fill_tiles_bli_cle();
    maker.fill_tiles_bli();
    maker.fill_wires_bli();
    maker.fill_tiles_bli_mux();
    maker.fill_tiles_clk();
    maker.fill_tiles_cle();
    maker.fill_tiles_bram();
    maker.fill_tiles_uram();
    maker.fill_tiles_dsp();
    maker.fill_tiles_hard();
    maker.fill_tiles_hdio();
    maker.fill_tiles_rclk_hdio();
    maker.fill_tiles_vnoc();
    maker.fill_tiles_gt_misc();
    maker.fill_tiles_gt();
    maker.fill_tiles_vdu();
    maker.fill_tiles_bfr_b();
    maker.fill_tiles_rfadc_rfdac();
    maker.fill_tiles_vcu2();
    maker.fill_tiles_isp2();
    maker.fill_tiles_hnicx();
    maker.fill_tiles_ps();
    maker.fill_tiles_hsr();
    maker.fill_tiles_ai();

    maker.builder.build()
}
