use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;
use std::marker::PhantomData;

use prjcombine_entity::{
    EntityBundleIndices, EntityBundleMap, EntityId, EntityKey, EntityMap, EntityPartMap,
    EntityPartVec, EntityRange, EntityVec, entity_id_u32,
};
use prjcombine_interconnect::db::{
    Bel, BelAttribute, BelAttributeEnum, BelAttributeType, BelBidirId, BelClass, BelClassAttribute,
    BelClassBidir, BelClassId, BelClassInput, BelClassOutput, BelClassPad, BelInfo, BelInput,
    BelInputId, BelKind, BelOutputId, BelPadId, BelPinIndexing, BelSlot, BelSlotId, Bidi,
    BitRectInfo, CellSlotId, ConnectorClass, ConnectorClassId, ConnectorSlot, ConnectorSlotId,
    ConnectorWire, DeviceDataSlot, EnumClass, IntDb, Mux, PermaBuf, PolTileWireCoord, ProgBuf,
    ProgInv, RegionSlotId, SwitchBox, SwitchBoxItem, Table, TableField, TableId, TableRow,
    TileClass, TileClassId, TileSlotId, TileWireCoord, WireKind, WireSlot, WireSlotId,
};
use prjcombine_types::bitrect::{PolTileBit, RectBitId, RectFrameId, TileBit};
use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use crate::ast::{ArrayIdRef, TemplateId};
use crate::db::{
    AnnotatedDbSet, BundleIdentData, BundleIdentRegistry, Const, IdentRegistry, VariantId,
};
use crate::{
    ast,
    db::AnnotatedDb,
    parse::{error_at, parse_str},
};

type Result<T> = core::result::Result<T, TokenStream>;

entity_id_u32!(PreId, PreTag tag, "PRE");

struct BundleIdentPreRegistry<I: EntityId, V: Clone> {
    ids: EntityMap<PreId, String, BundleIdentData<V>>,
    num_variants: usize,
    i: PhantomData<I>,
}

impl<I: EntityId, V: Clone> BundleIdentPreRegistry<I, V> {
    fn new(num_variants: usize) -> Self {
        Self {
            ids: Default::default(),
            num_variants,
            i: PhantomData,
        }
    }

    fn insert(
        &mut self,
        vid: VariantId,
        ident: &Ident,
        array_size: Option<usize>,
        val: V,
    ) -> core::result::Result<(), &'static str> {
        let name = ident.to_string();
        if let Some((_, pre)) = self.ids.get_mut(&name) {
            match (&mut pre.array_size, array_size) {
                (None, Some(_)) => return Err("non-array redefined as an array"),
                (Some(_), None) => return Err("array redefined as a non-array"),
                (None, None) => (),
                (Some(cur), Some(new)) => {
                    *cur = Ord::max(*cur, new);
                }
            }
            if pre.variants.contains_id(vid) {
                return Err("item redefined");
            }
            pre.variants.insert(vid, (array_size, val));
        } else {
            self.ids.insert(
                name.clone(),
                BundleIdentData {
                    array_size,
                    idents: vec![ident.clone()],
                    variants: EntityPartVec::from_iter([(vid, (array_size, val))]),
                },
            );
        }
        Ok(())
    }

    fn finish<NV: EntityKey>(
        &self,
        f: impl Fn(String, &V) -> NV,
    ) -> (
        BundleIdentRegistry<I, V>,
        EntityVec<VariantId, EntityPartMap<I, NV>>,
    ) {
        let mut reg = BundleIdentRegistry {
            ids: Default::default(),
        };
        let mut vars: EntityVec<_, _> = (0..self.num_variants)
            .map(|_| EntityPartMap::new())
            .collect();
        let mut allocator: EntityVec<I, ()> = EntityVec::new();
        for (_, name, pre) in &self.ids {
            match pre.array_size {
                None => {
                    let id = allocator.push(());
                    reg.ids
                        .insert(EntityBundleIndices::Single(id), name.clone(), pre.clone())
                        .unwrap();
                    for (vid, (_, data)) in &pre.variants {
                        vars[vid].insert(id, f(name.clone(), data)).unwrap();
                    }
                }
                Some(n) => {
                    let start = allocator.next_id().to_idx();
                    let ids = EntityRange::new(start, start + n);
                    for _ in 0..n {
                        allocator.push(());
                    }
                    reg.ids
                        .insert(EntityBundleIndices::Array(ids), name.clone(), pre.clone())
                        .unwrap();
                    for (vid, &(asize, ref data)) in &pre.variants {
                        let asize = asize.unwrap();
                        assert!(asize <= n);
                        for i in 0..asize {
                            vars[vid]
                                .insert(ids.index(i), f(format!("{name}[{i}]"), data))
                                .unwrap();
                        }
                    }
                }
            }
        }
        (reg, vars)
    }

    fn finish_bundle(
        &self,
    ) -> (
        BundleIdentRegistry<I, V>,
        EntityVec<VariantId, EntityBundleMap<I, V>>,
    ) {
        let mut reg = BundleIdentRegistry {
            ids: Default::default(),
        };
        let mut vars: EntityVec<_, _> = (0..self.num_variants)
            .map(|_| EntityBundleMap::new())
            .collect();
        let mut allocator: EntityVec<I, ()> = EntityVec::new();
        for (_, name, pre) in &self.ids {
            match pre.array_size {
                None => {
                    let id = allocator.push(());
                    reg.ids
                        .insert(EntityBundleIndices::Single(id), name.clone(), pre.clone())
                        .unwrap();
                    for (vid, (_, data)) in &pre.variants {
                        vars[vid].insert(id, name.clone(), data.clone()).unwrap();
                    }
                }
                Some(n) => {
                    let start = allocator.next_id().to_idx();
                    let ids = EntityRange::new(start, start + n);
                    for _ in 0..n {
                        allocator.push(());
                    }
                    reg.ids
                        .insert(EntityBundleIndices::Array(ids), name.clone(), pre.clone())
                        .unwrap();
                    for (vid, &(asize, ref data)) in &pre.variants {
                        let asize = asize.unwrap();
                        assert!(asize <= n);
                        let vids = EntityRange::new(start, start + asize);
                        vars[vid].insert(vids, name.clone(), data.clone()).unwrap();
                    }
                }
            }
        }
        (reg, vars)
    }
}

struct BelClassPre {
    pub input_pre: BundleIdentPreRegistry<BelInputId, BelClassInput>,
    pub output_pre: BundleIdentPreRegistry<BelOutputId, BelClassOutput>,
    pub bidir_pre: BundleIdentPreRegistry<BelBidirId, BelClassBidir>,
    pub pad_pre: BundleIdentPreRegistry<BelPadId, BelClassPad>,
}

impl BelClassPre {
    fn new(num_variants: usize) -> Self {
        Self {
            input_pre: BundleIdentPreRegistry::new(num_variants),
            output_pre: BundleIdentPreRegistry::new(num_variants),
            bidir_pre: BundleIdentPreRegistry::new(num_variants),
            pad_pre: BundleIdentPreRegistry::new(num_variants),
        }
    }
}

struct Context {
    dbs: AnnotatedDbSet,
    bslot_pre: BundleIdentPreRegistry<BelSlotId, TileSlotId>,
    rslot_pre: BundleIdentPreRegistry<RegionSlotId, ()>,
    wire_pre: BundleIdentPreRegistry<WireSlotId, ()>,
    bcls_pre: EntityPartVec<BelClassId, BelClassPre>,
    for_vars: HashMap<String, usize>,
    if_mask: EntityVec<VariantId, bool>,
}

#[derive(Copy, Clone)]
enum IfContext {
    Top,
    TileClass(TileClassId),
    ConnectorClass(ConnectorClassId),
    Bel(TileClassId, BelSlotId),
}

#[derive(Clone)]
enum WildCard {
    Ident(Ident),
    Prefix(Ident),
}

impl Context {
    fn active_variants(&self) -> Vec<VariantId> {
        self.if_mask
            .iter()
            .filter(|&(_, &c)| c)
            .map(|(id, _)| id)
            .collect()
    }

    fn eval_templ_subexpr(&self, span: Span, expr: TokenStream) -> Result<usize> {
        #[derive(Copy, Clone, Eq, PartialEq)]
        enum SumOp {
            Add,
            Sub,
        }
        #[derive(Copy, Clone, Eq, PartialEq)]
        enum ProdOp {
            Mul,
            Div,
            Mod,
        }
        let mut cur_term = None;
        let mut sum = 0;
        let mut sum_op = SumOp::Add;
        let mut prod = 1;
        let mut prod_op = ProdOp::Mul;
        for token in expr {
            if let Some(term) = cur_term {
                match prod_op {
                    ProdOp::Mul => prod *= term,
                    ProdOp::Div => prod /= term,
                    ProdOp::Mod => prod %= term,
                }
                cur_term = None;
                let TokenTree::Punct(punct) = token else {
                    error_at(span, &format!("unexpected token {token}"))?
                };
                match punct.to_string().as_str() {
                    "*" => prod_op = ProdOp::Mul,
                    "/" => prod_op = ProdOp::Div,
                    "%" => prod_op = ProdOp::Mod,
                    "+" => {
                        match sum_op {
                            SumOp::Add => sum += prod,
                            SumOp::Sub => sum -= prod,
                        }
                        prod = 1;
                        prod_op = ProdOp::Mul;
                        sum_op = SumOp::Add;
                    }
                    "-" => {
                        match sum_op {
                            SumOp::Add => sum += prod,
                            SumOp::Sub => sum -= prod,
                        }
                        prod = 1;
                        prod_op = ProdOp::Mul;
                        sum_op = SumOp::Sub;
                    }
                    _ => error_at(span, &format!("unexpected token {punct}"))?,
                }
            } else {
                match token {
                    TokenTree::Group(group) => {
                        if group.delimiter() != Delimiter::Parenthesis {
                            error_at(span, &format!("group {group} not supported"))?;
                        }
                        cur_term = Some(self.eval_templ_subexpr(span, group.stream())?);
                    }
                    TokenTree::Ident(ident) => {
                        let Some(&val) = self.for_vars.get(&ident.to_string()) else {
                            error_at(span, &format!("unknown variable {ident}"))?
                        };
                        cur_term = Some(val);
                    }
                    TokenTree::Punct(punct) => error_at(span, &format!("unexpected {punct}"))?,
                    TokenTree::Literal(literal) => {
                        let Ok(val) = literal.to_string().parse() else {
                            error_at(span, &format!("unhandled literal {literal}"))?
                        };
                        let val: usize = val;
                        cur_term = Some(val);
                    }
                }
            }
        }
        let Some(cur_term) = cur_term else {
            error_at(span, "unexpected end of expression")?
        };
        match prod_op {
            ProdOp::Mul => prod *= cur_term,
            ProdOp::Div => prod /= cur_term,
            ProdOp::Mod => prod %= cur_term,
        }
        match sum_op {
            SumOp::Add => sum += prod,
            SumOp::Sub => sum -= prod,
        }
        Ok(sum)
    }

    fn eval_templ_expr(&self, span: Span, expr: &str) -> Result<usize> {
        let Ok(expr) = expr.parse::<TokenStream>() else {
            error_at(span, "malformed expression")?
        };
        self.eval_templ_subexpr(span, expr)
    }

    fn eval_templ_wildcard(&self, id: &ast::TemplateId) -> Result<WildCard> {
        Ok(match id {
            ast::TemplateId::Raw(ident) => WildCard::Ident(ident.clone()),
            ast::TemplateId::String(lit) => {
                let mut res = String::new();
                let s = parse_str(lit)?;
                let mut suffix = &s[..];
                while let Some((prefix, rest)) = suffix.split_once('{') {
                    res.push_str(prefix);
                    let Some((expr, rest)) = rest.split_once('}') else {
                        error_at(lit.span(), "unterminated interpolation")?
                    };
                    let (expr, fmt) = expr.split_once(':').unwrap_or((expr, ""));

                    let val = self.eval_templ_expr(lit.span(), expr)?;
                    if fmt.is_empty() {
                        write!(res, "{val}").unwrap();
                    } else {
                        let width: usize = fmt.parse().expect("cannot parse width");
                        write!(res, "{val:0>width$}").unwrap();
                    }
                    suffix = rest;
                }
                res.push_str(suffix);
                if let Some(prefix) = res.strip_suffix('*') {
                    WildCard::Prefix(Ident::new(prefix, lit.span()))
                } else {
                    WildCard::Ident(Ident::new(&res, lit.span()))
                }
            }
        })
    }

    fn eval_templ_id(&self, id: &ast::TemplateId) -> Result<Ident> {
        match self.eval_templ_wildcard(id)? {
            WildCard::Ident(ident) => Ok(ident),
            WildCard::Prefix(_) => error_at(id.span(), "wildcard not accepted here")?,
        }
    }

    fn eval_expr(&self, vid: VariantId, expr: &ast::Expr) -> Result<usize> {
        match expr {
            ast::Expr::Ident(ident) => {
                let name = ident.to_string();
                if let Some(&n) = self.for_vars.get(&name) {
                    Ok(n)
                } else if let Some((_, c)) = self.dbs.variants[vid].consts.get(&name) {
                    Ok(c.value)
                } else {
                    error_at(ident.span(), "undefined variable")?
                }
            }
            ast::Expr::Literal(n) => Ok(*n),
            ast::Expr::Add(a, b) => Ok(self.eval_expr(vid, a)? + self.eval_expr(vid, b)?),
            ast::Expr::Sub(a, b) => Ok(self.eval_expr(vid, a)? - self.eval_expr(vid, b)?),
            ast::Expr::Mul(a, b) => Ok(self.eval_expr(vid, a)? * self.eval_expr(vid, b)?),
            ast::Expr::Div(a, b) => Ok(self.eval_expr(vid, a)? / self.eval_expr(vid, b)?),
            ast::Expr::Mod(a, b) => Ok(self.eval_expr(vid, a)? % self.eval_expr(vid, b)?),
        }
    }

    fn eval_ref<I: EntityId, V: EntityKey>(
        &mut self,
        vid: VariantId,
        ident: &Ident,
        reg: impl Fn(&mut AnnotatedDbSet) -> &mut IdentRegistry<I>,
        map: impl Fn(&AnnotatedDb) -> &EntityPartMap<I, V>,
        kind: &str,
    ) -> Result<I> {
        let Some(id) = reg(&mut self.dbs).get_ref(ident) else {
            error_at(ident.span(), &format!("undefined {kind}: {ident}"))?
        };
        if !map(&self.dbs.variants[vid]).contains_id(id) {
            let var = self.dbs.variants[vid].name.as_ref().unwrap();
            error_at(
                ident.span(),
                &format!("undefined {kind} for variant {var}: {ident}"),
            )?
        }
        Ok(id)
    }

    fn eval_templ_ref<I: EntityId, V: EntityKey>(
        &mut self,
        vid: VariantId,
        ident: &TemplateId,
        reg: impl Fn(&mut AnnotatedDbSet) -> &mut IdentRegistry<I>,
        map: impl Fn(&AnnotatedDb) -> &EntityPartMap<I, V>,
        kind: &str,
    ) -> Result<I> {
        let ident = self.eval_templ_id(ident)?;
        self.eval_ref(vid, &ident, reg, map, kind)
    }

    fn eval_array_ref_wide<I: EntityId, V>(
        &mut self,
        vid: VariantId,
        id: &ast::ArrayIdRef,
        reg: impl Fn(&mut AnnotatedDbSet) -> &mut BundleIdentRegistry<I, V>,
        kind: &'static str,
    ) -> Result<EntityBundleIndices<I>> {
        match id {
            ast::ArrayIdRef::Plain(id) => {
                let ident = self.eval_templ_id(id)?;
                let Some((item, _)) = reg(&mut self.dbs).get_ref(vid, &ident) else {
                    error_at(ident.span(), &format!("undefined {kind}: {ident}"))?
                };
                Ok(item)
            }
            ast::ArrayIdRef::Indexed(id, index) => {
                let ident = self.eval_templ_id(id)?;
                let index = self.eval_expr(vid, index)?;
                let Some((item, _)) = reg(&mut self.dbs).get_ref(vid, &ident) else {
                    error_at(ident.span(), &format!("undefined {kind}: {ident}"))?
                };
                match item {
                    EntityBundleIndices::Single(_) => {
                        error_at(ident.span(), "object is not an array")?
                    }
                    EntityBundleIndices::Array(range) => {
                        if index >= range.len() {
                            error_at(ident.span(), "index out of bounds")?
                        }
                        let id = range.index(index);
                        Ok(EntityBundleIndices::Single(id))
                    }
                }
            }
        }
    }

    fn eval_array_ref<I: EntityId, V>(
        &mut self,
        vid: VariantId,
        id: &ast::ArrayIdRef,
        reg: impl Fn(&mut AnnotatedDbSet) -> &mut BundleIdentRegistry<I, V>,
        kind: &'static str,
    ) -> Result<I> {
        match self.eval_array_ref_wide(vid, id, reg, kind)? {
            EntityBundleIndices::Single(id) => Ok(id),
            EntityBundleIndices::Array(_) => error_at(id.span(), "object is an array")?,
        }
    }

    fn eval_array_def(
        &self,
        vid: VariantId,
        name: &ast::ArrayIdDef,
    ) -> Result<(Ident, Option<usize>)> {
        let (name, num) = match name {
            ast::ArrayIdDef::Plain(id) => (id, None),
            ast::ArrayIdDef::Array(id, n) => (id, Some(self.eval_expr(vid, n)?)),
        };
        let name = self.eval_templ_id(name)?;
        Ok((name, num))
    }

    fn eval_cell_ref(
        &self,
        vid: VariantId,
        tcls: TileClassId,
        id: &ast::ArrayIdRef,
    ) -> Result<CellSlotId> {
        match id {
            ast::ArrayIdRef::Plain(id) => {
                let ident = self.eval_templ_id(id)?;
                let Some((item, _)) =
                    self.dbs.variants[vid].tcls_cell_id[tcls].get(&ident.to_string())
                else {
                    error_at(ident.span(), &format!("undefined cell: {ident}"))?
                };
                match item {
                    EntityBundleIndices::Single(id) => Ok(id),
                    EntityBundleIndices::Array(_) => error_at(id.span(), "object is an array")?,
                }
            }
            ast::ArrayIdRef::Indexed(id, index) => {
                let ident = self.eval_templ_id(id)?;
                let index = self.eval_expr(vid, index)?;
                let Some((item, _)) =
                    self.dbs.variants[vid].tcls_cell_id[tcls].get(&ident.to_string())
                else {
                    error_at(ident.span(), &format!("undefined cell: {ident}"))?
                };
                match item {
                    EntityBundleIndices::Single(_) => {
                        error_at(ident.span(), "object is not an array")?
                    }
                    EntityBundleIndices::Array(range) => {
                        if index > range.len() {
                            error_at(ident.span(), "index out of bounds")?
                        }
                        let id = range.index(index);
                        Ok(id)
                    }
                }
            }
        }
    }

    fn eval_conn_slot_ref(&mut self, vid: VariantId, ident: &Ident) -> Result<ConnectorSlotId> {
        self.eval_ref(
            vid,
            ident,
            |dbs| &mut dbs.cslot_id,
            |db| &db.db.conn_slots,
            "connector slot",
        )
    }

    fn eval_bel_slot_ref(&mut self, vid: VariantId, ident: &ArrayIdRef) -> Result<BelSlotId> {
        self.eval_array_ref(vid, ident, |dbs| &mut dbs.bslot_id, "bel slot")
    }

    fn eval_region_slot_ref(&mut self, vid: VariantId, ident: &ArrayIdRef) -> Result<RegionSlotId> {
        self.eval_array_ref(vid, ident, |dbs| &mut dbs.rslot_id, "region slot")
    }

    fn eval_wire_ref_wide(
        &mut self,
        vid: VariantId,
        ident: &ArrayIdRef,
    ) -> Result<EntityBundleIndices<WireSlotId>> {
        self.eval_array_ref_wide(vid, ident, |dbs| &mut dbs.wire_id, "wire")
    }

    fn eval_wire_ref(&mut self, vid: VariantId, ident: &ArrayIdRef) -> Result<WireSlotId> {
        self.eval_array_ref(vid, ident, |dbs| &mut dbs.wire_id, "wire")
    }

    fn eval_tile_wire_ref(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        wref: &ast::WireRef,
    ) -> Result<TileWireCoord> {
        Ok(match wref {
            ast::WireRef::Simple(wref) => {
                if self.dbs.variants[vid].db.tile_classes[tcls].cells.len() != 1 {
                    error_at(
                        wref.span(),
                        "non-qualified wire references are only valid in single-cell tiles",
                    )?
                }
                TileWireCoord {
                    cell: CellSlotId::from_idx(0),
                    wire: self.eval_wire_ref(vid, wref)?,
                }
            }
            ast::WireRef::Qualified(cref, wref) => TileWireCoord {
                cell: self.eval_cell_ref(vid, tcls, cref)?,
                wire: self.eval_wire_ref(vid, wref)?,
            },
        })
    }

    fn eval_pol_tile_wire_ref(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        wref: &ast::PolWireRef,
    ) -> Result<PolTileWireCoord> {
        Ok(match wref {
            ast::PolWireRef::Pos(wire_ref) => self.eval_tile_wire_ref(vid, tcls, wire_ref)?.pos(),
            ast::PolWireRef::Neg(wire_ref) => self.eval_tile_wire_ref(vid, tcls, wire_ref)?.neg(),
        })
    }

    fn eval_pol_tile_bit(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        tbit: &ast::TileBit,
    ) -> Result<PolTileBit> {
        let name = self.eval_templ_id(&tbit.name)?;
        let mut index = vec![];
        for idx in &tbit.index {
            index.push(self.eval_expr(vid, idx)?);
        }
        let Some((rect, _)) = self.dbs.variants[vid].tcls_bitrect_id[tcls].get(&name.to_string())
        else {
            error_at(name.span(), "undefined bitrect")?
        };
        let rect = match rect {
            EntityBundleIndices::Single(r) => r,
            EntityBundleIndices::Array(range) => {
                if index.is_empty() {
                    error_at(name.span(), "missing bitrect index")?
                }
                let idx = index.remove(0);
                if idx >= range.len() {
                    error_at(name.span(), "bitrect out of bounds")?
                }
                range.index(idx)
            }
        };
        let geom = self.dbs.variants[vid].db.tile_classes[tcls].bitrects[rect].geometry;
        let (frame, bit) = match *index.as_slice() {
            [] => error_at(name.span(), "missing bit coordinates")?,
            [bit] => {
                if geom.frames != 1 {
                    error_at(
                        name.span(),
                        "single index can only be used for single-frame bitrects",
                    )?;
                }
                if bit >= geom.bits {
                    error_at(name.span(), "bit out of range")?;
                }
                (RectFrameId::from_idx(0), RectBitId::from_idx(bit))
            }
            [frame, bit] => {
                if frame >= geom.frames {
                    error_at(name.span(), "frame out of range")?;
                }
                if bit >= geom.bits {
                    error_at(name.span(), "bit out of range")?;
                }
                (RectFrameId::from_idx(frame), RectBitId::from_idx(bit))
            }
            _ => error_at(name.span(), "too many coordinates")?,
        };
        Ok(PolTileBit {
            bit: TileBit { rect, frame, bit },
            inv: tbit.inv,
        })
    }

    fn eval_tile_bit(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        tbit: &ast::TileBit,
    ) -> Result<TileBit> {
        let bit = self.eval_pol_tile_bit(vid, tcls, tbit)?;
        if bit.inv {
            error_at(tbit.name.span(), "inversion not accepted here")?;
        }
        Ok(bit.bit)
    }

    fn eval_for<T>(
        &mut self,
        loop_: &ast::ForLoop<T>,
        mut eval_item: impl FnMut(&mut Context, &T) -> Result<()>,
    ) -> Result<()> {
        let var = &loop_.var;
        let var_name = var.to_string();
        if self.for_vars.contains_key(&var_name) {
            error_at(var.span(), "loop variable {var} redefined")?
        }
        let orig_mask = self.if_mask.clone();
        let mut ranges = EntityVec::new();
        for (vid, &en) in &orig_mask {
            let range = if en {
                Some(match loop_.iterator {
                    ast::ForIterator::Range(ref start, ref end) => {
                        (self.eval_expr(vid, start)?, self.eval_expr(vid, end)?)
                    }
                    ast::ForIterator::RangeInclusive(ref start, ref end) => {
                        (self.eval_expr(vid, start)?, self.eval_expr(vid, end)? + 1)
                    }
                })
            } else {
                None
            };
            ranges.push(range);
        }
        let start = ranges
            .values()
            .filter_map(|&range| range.map(|r| r.0))
            .min();
        let end = ranges
            .values()
            .filter_map(|&range| range.map(|r| r.1))
            .max();
        if let Some(start) = start
            && let Some(end) = end
        {
            for i in start..end {
                self.if_mask = ranges.map_values(|&r| match r {
                    Some((s, e)) => (s..e).contains(&i),
                    None => false,
                });
                if self.if_mask.values().any(|&x| x) {
                    self.for_vars.insert(var_name.clone(), i);
                    for item in &loop_.items {
                        eval_item(self, item)?;
                    }
                }
            }
        }
        self.if_mask = orig_mask;
        self.for_vars.remove(&var_name);
        Ok(())
    }

    fn eval_cond(
        &mut self,
        vid: VariantId,
        ictx: IfContext,
        span: Span,
        cond: &ast::IfCond,
    ) -> Result<bool> {
        match cond {
            ast::IfCond::Variant(idents) => {
                let Some(ref our_name) = self.dbs.variants[vid].name else {
                    error_at(span, "no variants defined")?
                };
                for id in idents {
                    if id.to_string() == our_name.to_string() {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            ast::IfCond::TileClass(ids) => {
                let tcls = match ictx {
                    IfContext::TileClass(tcls) => tcls,
                    IfContext::Bel(tcls, _) => tcls,
                    _ => error_at(span, "not in tile class")?,
                };
                let name = &self.dbs.variants[vid].db.tile_classes[tcls].name;
                for id in ids {
                    let ok = match self.eval_templ_wildcard(id)? {
                        WildCard::Ident(ident) => name == &ident.to_string(),
                        WildCard::Prefix(ident) => name.starts_with(&ident.to_string()),
                    };
                    if ok {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            ast::IfCond::ConnectorClass(ids) => {
                let ccls = match ictx {
                    IfContext::ConnectorClass(ccls) => ccls,
                    _ => error_at(span, "not in connector class")?,
                };
                let name = &self.dbs.variants[vid].db.conn_classes[ccls].name;
                for id in ids {
                    let ok = match self.eval_templ_wildcard(id)? {
                        WildCard::Ident(ident) => name == &ident.to_string(),
                        WildCard::Prefix(ident) => name.starts_with(&ident.to_string()),
                    };
                    if ok {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            ast::IfCond::BelSlot(ids) => {
                let our_bslot = match ictx {
                    IfContext::Bel(_, bslot) => bslot,
                    _ => error_at(span, "not in bel")?,
                };
                for id in ids {
                    let bslot = self.eval_bel_slot_ref(vid, id)?;
                    if bslot == our_bslot {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    fn eval_if<T>(
        &mut self,
        ictx: IfContext,
        if_: &ast::If<T>,
        mut eval_item: impl FnMut(&mut Context, &T) -> Result<()>,
    ) -> Result<()> {
        let orig_mask = self.if_mask.clone();
        let mut else_mask = orig_mask.clone();
        for branch in &if_.branches {
            self.if_mask = orig_mask.map_values(|_| false);
            for vid in orig_mask.ids() {
                if else_mask[vid] && self.eval_cond(vid, ictx, if_.span, &branch.0)? {
                    self.if_mask[vid] = true;
                    else_mask[vid] = false;
                }
            }
            if self.if_mask.values().any(|&v| v) {
                for item in &branch.1 {
                    eval_item(self, item)?;
                }
            }
        }
        if else_mask.values().any(|&v| v) {
            self.if_mask = else_mask;
            for item in &if_.else_items {
                eval_item(self, item)?;
            }
        }
        self.if_mask = orig_mask;
        Ok(())
    }

    fn eval_pin_indexing(
        &self,
        vid: VariantId,
        name: &ast::PinArrayIdDef,
    ) -> Result<(Ident, Option<usize>, BelPinIndexing)> {
        Ok(match *name {
            ast::PinArrayIdDef::Plain(ref name) => {
                let name = self.eval_templ_id(name)?;
                (name, None, Default::default())
            }
            ast::PinArrayIdDef::Array(ref name, ref width) => {
                let width = self.eval_expr(vid, width)?;
                let name = self.eval_templ_id(name)?;
                (name, Some(width), Default::default())
            }
            ast::PinArrayIdDef::ArrayRange(ref name, ref msb, ref lsb) => {
                let msb = self.eval_expr(vid, msb)?;
                let lsb = self.eval_expr(vid, lsb)?;
                let name = self.eval_templ_id(name)?;
                if msb < lsb {
                    (
                        name,
                        Some(lsb - msb + 1),
                        BelPinIndexing {
                            lsb_index: lsb,
                            wrong_endian: true,
                        },
                    )
                } else {
                    (
                        name,
                        Some(lsb - msb + 1),
                        BelPinIndexing {
                            lsb_index: lsb,
                            wrong_endian: false,
                        },
                    )
                }
            }
        })
    }

    fn eval_attribute_type(
        &mut self,
        vid: VariantId,
        typ: &ast::AttributeType,
    ) -> Result<BelAttributeType> {
        Ok(match *typ {
            ast::AttributeType::Bool => BelAttributeType::Bool,
            ast::AttributeType::BitVec(ref width) => {
                let width = self.eval_expr(vid, width)?;
                BelAttributeType::BitVec(width)
            }
            ast::AttributeType::BitVecArray(ref width, ref depth) => {
                let width = self.eval_expr(vid, width)?;
                let depth = self.eval_expr(vid, depth)?;
                BelAttributeType::BitVecArray(width, depth)
            }
            ast::AttributeType::Enum(ref ident) => {
                let eid = self.eval_templ_ref(
                    vid,
                    ident,
                    |dbs| &mut dbs.enum_id,
                    |db| &db.db.enum_classes,
                    "enum",
                )?;
                BelAttributeType::Enum(eid)
            }
            ast::AttributeType::U32 => BelAttributeType::U32,
        })
    }

    fn eval_bel_class_phase1(&mut self, bcid: BelClassId, item: &ast::BelClassItem) -> Result<()> {
        match item {
            ast::BelClassItem::Input(pin) => {
                for vid in self.active_variants() {
                    for name in &pin.names {
                        let (name, width, indexing) = self.eval_pin_indexing(vid, name)?;
                        let inp = BelClassInput {
                            nonroutable: pin.nonroutable,
                            indexing,
                        };
                        if let Err(msg) =
                            self.bcls_pre[bcid].input_pre.insert(vid, &name, width, inp)
                        {
                            error_at(name.span(), msg)?
                        }
                    }
                }
            }
            ast::BelClassItem::Output(pin) => {
                for vid in self.active_variants() {
                    for name in &pin.names {
                        let (name, width, indexing) = self.eval_pin_indexing(vid, name)?;
                        let outp = BelClassOutput {
                            nonroutable: pin.nonroutable,
                            indexing,
                        };
                        if let Err(msg) = self.bcls_pre[bcid]
                            .output_pre
                            .insert(vid, &name, width, outp)
                        {
                            error_at(name.span(), msg)?
                        }
                    }
                }
            }
            ast::BelClassItem::Bidir(pin) => {
                for vid in self.active_variants() {
                    for name in &pin.names {
                        let (name, width, indexing) = self.eval_pin_indexing(vid, name)?;
                        let bidi = BelClassBidir {
                            nonroutable: pin.nonroutable,
                            indexing,
                        };
                        if let Err(msg) = self.bcls_pre[bcid]
                            .bidir_pre
                            .insert(vid, &name, width, bidi)
                        {
                            error_at(name.span(), msg)?
                        }
                    }
                }
            }
            ast::BelClassItem::Pad(pad) => {
                for vid in self.active_variants() {
                    for name in &pad.names {
                        let (name, num) = self.eval_array_def(vid, name)?;
                        if let Err(msg) = self.bcls_pre[bcid].pad_pre.insert(
                            vid,
                            &name,
                            num,
                            BelClassPad { kind: pad.kind },
                        ) {
                            error_at(name.span(), msg)?
                        }
                    }
                }
            }
            ast::BelClassItem::Attribute(attr) => {
                for name in &attr.names {
                    let ident = self.eval_templ_id(name)?;
                    self.dbs.bcls[bcid].attr_id.insert(&ident);
                }
            }
            ast::BelClassItem::ForLoop(for_loop) => self.eval_for(for_loop, |ctx, subitem| {
                ctx.eval_bel_class_phase1(bcid, subitem)
            })?,
            ast::BelClassItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_bel_class_phase1(bcid, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_bel_class(&mut self, bcid: BelClassId, item: &ast::BelClassItem) -> Result<()> {
        match item {
            ast::BelClassItem::Input(_pin) => {}
            ast::BelClassItem::Output(_pin) => {}
            ast::BelClassItem::Bidir(_pin) => {}
            ast::BelClassItem::Pad(_pad) => {}
            ast::BelClassItem::Attribute(attr) => {
                for vid in self.active_variants() {
                    let typ = self.eval_attribute_type(vid, &attr.typ)?;
                    for name in &attr.names {
                        let ident = self.eval_templ_id(name)?;
                        let aid = self.dbs.bcls[bcid].attr_id.get(&ident);
                        let attribute = BelClassAttribute {
                            name: ident.to_string(),
                            typ,
                        };
                        if self.dbs.variants[vid].db.bel_classes[bcid]
                            .attributes
                            .insert(aid, attribute)
                            .is_err()
                        {
                            error_at(name.span(), "bel attribute redefined")?;
                        }
                    }
                }
            }
            ast::BelClassItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_bel_class(bcid, subitem))?
            }
            ast::BelClassItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_bel_class(bcid, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_tile_slot_phase1(&mut self, tslot: TileSlotId, item: &ast::TileSlotItem) -> Result<()> {
        match item {
            ast::TileSlotItem::BelSlot(bslot) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, &bslot.name)?;
                    if let Err(msg) = self.bslot_pre.insert(vid, &name, num, tslot) {
                        error_at(name.span(), msg)?
                    }
                }
            }
            ast::TileSlotItem::TileClass(tcls) => {
                for raw_name in &tcls.names {
                    let name = self.eval_templ_id(raw_name)?;
                    let (tcid, name) = self.dbs.tcls_id.insert(&name);
                    for vid in self.active_variants() {
                        if self.dbs.variants[vid]
                            .db
                            .tile_classes
                            .insert(
                                tcid,
                                TileClass {
                                    name: name.clone(),
                                    slot: tslot,
                                    cells: Default::default(),
                                    bitrects: Default::default(),
                                    bels: Default::default(),
                                },
                            )
                            .is_err()
                        {
                            error_at(raw_name.span(), "tile class redefined")?;
                        }
                    }
                }
            }
            ast::TileSlotItem::ForLoop(for_loop) => self.eval_for(for_loop, |ctx, subitem| {
                ctx.eval_tile_slot_phase1(tslot, subitem)
            })?,
            ast::TileSlotItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_tile_slot_phase1(tslot, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_tile_slot(&mut self, tslot: TileSlotId, item: &ast::TileSlotItem) -> Result<()> {
        let _ = tslot;
        match item {
            ast::TileSlotItem::BelSlot(bslot) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, &bslot.name)?;
                    let kind = match &bslot.kind {
                        ast::BelKind::Routing => BelKind::Routing,
                        ast::BelKind::Class(id) => {
                            let bcls = self.eval_ref(
                                vid,
                                id,
                                |dbs| &mut dbs.bcls_id,
                                |db| &db.db.bel_classes,
                                "bel class",
                            )?;
                            BelKind::Class(bcls)
                        }
                        ast::BelKind::Legacy => BelKind::Legacy,
                    };
                    if let Some(num) = num {
                        for i in 0..num {
                            let id = self.dbs.bslot_id.get(&name, Some(i));
                            let bslot = &mut self.dbs.variants[vid].db.bel_slots[id];
                            bslot.kind = kind;
                        }
                    } else {
                        let id = self.dbs.bslot_id.get(&name, None);
                        let bslot = &mut self.dbs.variants[vid].db.bel_slots[id];
                        bslot.kind = kind;
                    }
                }
            }
            ast::TileSlotItem::TileClass(tcls) => {
                for name in &tcls.names {
                    let name = self.eval_templ_id(name)?;
                    let tcid = self.dbs.tcls_id.get(&name);

                    for vid in self.active_variants() {
                        self.dbs.variants[vid]
                            .tcls_cell_id
                            .insert(tcid, Default::default());
                        self.dbs.variants[vid]
                            .tcls_bitrect_id
                            .insert(tcid, Default::default());
                    }
                    for item in &tcls.items {
                        self.eval_tile_class(tcid, item)?;
                    }
                }
            }
            ast::TileSlotItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_tile_slot(tslot, subitem))?
            }
            ast::TileSlotItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_tile_slot(tslot, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_tile_class(&mut self, tcls: TileClassId, item: &ast::TileClassItem) -> Result<()> {
        match item {
            ast::TileClassItem::Cell(names) => {
                for name in names {
                    for vid in self.active_variants() {
                        let (name, num) = self.eval_array_def(vid, name)?;
                        if let Some(num) = num {
                            let start = self.dbs.variants[vid].db.tile_classes[tcls]
                                .cells
                                .next_id()
                                .to_idx();
                            let ids = EntityRange::new(start, start + num);
                            for i in 0..num {
                                self.dbs.variants[vid].db.tile_classes[tcls]
                                    .cells
                                    .push(format!("{name}[{i}]"));
                            }

                            if self.dbs.variants[vid].tcls_cell_id[tcls]
                                .insert(ids, name.to_string(), name.clone())
                                .is_err()
                            {
                                error_at(name.span(), "cell slot redefined")?
                            };
                        } else {
                            let id = self.dbs.variants[vid].db.tile_classes[tcls]
                                .cells
                                .push(name.to_string());
                            if self.dbs.variants[vid].tcls_cell_id[tcls]
                                .insert(id, name.to_string(), name.clone())
                                .is_err()
                            {
                                error_at(name.span(), "cell slot redefined")?
                            };
                        }
                    }
                }
            }
            ast::TileClassItem::BitRect(name, class) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, name)?;
                    let Some(&geometry) =
                        self.dbs.variants[vid].bitrect_geoms.get(&class.to_string())
                    else {
                        error_at(class.span(), "undefined bitrect class")?
                    };
                    if let Some(num) = num {
                        let start = self.dbs.variants[vid].db.tile_classes[tcls]
                            .bitrects
                            .next_id()
                            .to_idx();
                        let ids = EntityRange::new(start, start + num);
                        for i in 0..num {
                            self.dbs.variants[vid].db.tile_classes[tcls].bitrects.push(
                                BitRectInfo {
                                    name: format!("{name}[{i}]"),
                                    geometry,
                                },
                            );
                        }

                        if self.dbs.variants[vid].tcls_bitrect_id[tcls]
                            .insert(ids, name.to_string(), name.clone())
                            .is_err()
                        {
                            error_at(name.span(), "bitrect redefined")?
                        };
                    } else {
                        let id = self.dbs.variants[vid].db.tile_classes[tcls].bitrects.push(
                            BitRectInfo {
                                name: name.to_string(),
                                geometry,
                            },
                        );
                        if self.dbs.variants[vid].tcls_bitrect_id[tcls]
                            .insert(id, name.to_string(), name.clone())
                            .is_err()
                        {
                            error_at(name.span(), "bitrect redefined")?
                        };
                    }
                }
            }
            ast::TileClassItem::SwitchBox(sbox) => {
                for vid in self.active_variants() {
                    let slot = self.eval_bel_slot_ref(vid, &sbox.slot)?;
                    if self.dbs.variants[vid].db.bel_slots[slot].kind != BelKind::Routing {
                        error_at(sbox.slot.span(), "switchbox must be on routing bel")?;
                    }
                    let mut switchbox = SwitchBox::default();
                    for subitem in &sbox.items {
                        self.eval_switchbox(vid, tcls, slot, &mut switchbox, subitem)?;
                    }
                    self.dbs.variants[vid].db.tile_classes[tcls]
                        .bels
                        .insert(slot, BelInfo::SwitchBox(switchbox));
                }
            }
            ast::TileClassItem::Bel(sbel) => {
                for vid in self.active_variants() {
                    let slot = self.eval_bel_slot_ref(vid, &sbel.slot)?;
                    match self.dbs.variants[vid].db.bel_slots[slot].kind {
                        BelKind::Routing => {
                            error_at(sbel.slot.span(), "bel kind must not be switchbox")?
                        }
                        BelKind::Class(bcls) => {
                            let mut bel = Bel::default();
                            for subitem in &sbel.items {
                                self.eval_bel(vid, tcls, slot, bcls, &mut bel, subitem)?;
                            }
                            self.dbs.variants[vid].db.tile_classes[tcls]
                                .bels
                                .insert(slot, BelInfo::Bel(bel));
                        }
                        BelKind::Legacy => {
                            if !sbel.items.is_empty() {
                                error_at(sbel.slot.span(), "legacy bel must not have items")?;
                            }
                            self.dbs.variants[vid].db.tile_classes[tcls]
                                .bels
                                .insert(slot, BelInfo::Legacy(Default::default()));
                        }
                    }
                }
            }
            ast::TileClassItem::Skip => {
                for vid in self.active_variants() {
                    self.dbs.variants[vid].skip_tcls.insert(tcls);
                }
            }
            ast::TileClassItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_tile_class(tcls, subitem))?
            }
            ast::TileClassItem::If(if_) => {
                self.eval_if(IfContext::TileClass(tcls), if_, |ctx, subitem| {
                    ctx.eval_tile_class(tcls, subitem)
                })?
            }
        }
        Ok(())
    }

    fn eval_switchbox(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        bslot: BelSlotId,
        switchbox: &mut SwitchBox,
        item: &ast::SwitchBoxItem,
    ) -> Result<()> {
        match item {
            ast::SwitchBoxItem::PermaBuf(dst, src) => {
                let dst = self.eval_tile_wire_ref(vid, tcls, dst)?;
                let src = self.eval_pol_tile_wire_ref(vid, tcls, src)?;
                switchbox
                    .items
                    .push(SwitchBoxItem::PermaBuf(PermaBuf { dst, src }));
            }
            ast::SwitchBoxItem::ProgBuf(dst, src) => {
                let dst = self.eval_tile_wire_ref(vid, tcls, dst)?;
                let src = self.eval_pol_tile_wire_ref(vid, tcls, src)?;
                switchbox.items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                    dst,
                    src,
                    bit: PolTileBit::DUMMY,
                }));
            }
            ast::SwitchBoxItem::ProgInv(dst, src) => {
                let dst = self.eval_tile_wire_ref(vid, tcls, dst)?;
                let src = self.eval_tile_wire_ref(vid, tcls, src)?;
                switchbox.items.push(SwitchBoxItem::ProgInv(ProgInv {
                    dst,
                    src,
                    bit: PolTileBit::DUMMY,
                }));
            }
            ast::SwitchBoxItem::Mux(dst, srcs) => {
                let mut mux = Mux {
                    dst: self.eval_tile_wire_ref(vid, tcls, dst)?,
                    bits: Default::default(),
                    src: Default::default(),
                    bits_off: None,
                };
                for src in srcs {
                    let src = self.eval_pol_tile_wire_ref(vid, tcls, src)?;
                    mux.src.insert(src, Default::default());
                }
                switchbox.items.push(SwitchBoxItem::Mux(mux));
            }
            ast::SwitchBoxItem::Bidi(conn, wire) => {
                let ident = self.eval_templ_id(conn)?;
                let csid = self.eval_conn_slot_ref(vid, &ident)?;
                let bidi = Bidi {
                    conn: csid,
                    wire: self.eval_tile_wire_ref(vid, tcls, wire)?,
                    bit_upstream: PolTileBit::DUMMY,
                };
                switchbox.items.push(SwitchBoxItem::Bidi(bidi));
            }
            ast::SwitchBoxItem::ForLoop(for_loop) => self.eval_for(for_loop, |ctx, subitem| {
                ctx.eval_switchbox(vid, tcls, bslot, switchbox, subitem)
            })?,
            ast::SwitchBoxItem::If(if_) => {
                self.eval_if(IfContext::Bel(tcls, bslot), if_, |ctx, subitem| {
                    if ctx.if_mask[vid] {
                        ctx.eval_switchbox(vid, tcls, bslot, switchbox, subitem)
                    } else {
                        Ok(())
                    }
                })?
            }
        }
        Ok(())
    }

    fn eval_pin_array_ref<I: EntityId, T>(
        &mut self,
        vid: VariantId,
        id: &ast::ArrayIdRef,
        reg: impl Fn(&mut AnnotatedDbSet) -> &mut BundleIdentRegistry<I, T>,
        get_indexing: impl Fn(&T) -> BelPinIndexing,
        kind: &'static str,
    ) -> Result<I> {
        match id {
            ast::ArrayIdRef::Plain(id) => {
                let ident = self.eval_templ_id(id)?;
                let Some((item, _)) = reg(&mut self.dbs).get_ref(vid, &ident) else {
                    error_at(ident.span(), &format!("undefined {kind}: {ident}"))?
                };
                match item {
                    EntityBundleIndices::Single(id) => Ok(id),
                    EntityBundleIndices::Array(_) => error_at(ident.span(), "object is an array")?,
                }
            }
            ast::ArrayIdRef::Indexed(id, index) => {
                let ident = self.eval_templ_id(id)?;
                let Some((item, data)) = reg(&mut self.dbs).get_ref(vid, &ident) else {
                    error_at(ident.span(), &format!("undefined {kind}: {ident}"))?
                };

                let indexing = get_indexing(data);
                let index = self.eval_expr(vid, index)?;
                match item {
                    EntityBundleIndices::Single(_) => {
                        error_at(ident.span(), "object is not an array")?
                    }
                    EntityBundleIndices::Array(range) => {
                        let Some(index) = indexing.try_virt_to_phys(index) else {
                            error_at(ident.span(), "index out of bounds")?
                        };
                        if index > range.len() {
                            error_at(ident.span(), "index out of bounds")?
                        }
                        Ok(range.index(index))
                    }
                }
            }
        }
    }

    fn eval_bel(
        &mut self,
        vid: VariantId,
        tcls: TileClassId,
        bslot: BelSlotId,
        bcid: BelClassId,
        bel: &mut Bel,
        item: &ast::BelItem,
    ) -> Result<()> {
        match item {
            ast::BelItem::Input(pin, wire_ref) => {
                let pin = self.eval_pin_array_ref(
                    vid,
                    pin,
                    |dbs| &mut dbs.bcls[bcid].input_id,
                    |inp| inp.indexing,
                    "bel input",
                )?;
                let wire = self.eval_pol_tile_wire_ref(vid, tcls, wire_ref)?;
                bel.inputs.insert(pin, BelInput::Fixed(wire));
            }
            ast::BelItem::Output(pin, wire_refs) => {
                let pin = self.eval_pin_array_ref(
                    vid,
                    pin,
                    |dbs| &mut dbs.bcls[bcid].output_id,
                    |outp| outp.indexing,
                    "bel output",
                )?;
                let mut wires = BTreeSet::new();
                for wref in wire_refs {
                    let wire = self.eval_tile_wire_ref(vid, tcls, wref)?;
                    wires.insert(wire);
                }
                bel.outputs.insert(pin, wires);
            }
            ast::BelItem::Bidir(pin, wire_ref) => {
                let pin = self.eval_pin_array_ref(
                    vid,
                    pin,
                    |dbs| &mut dbs.bcls[bcid].bidir_id,
                    |bidi| bidi.indexing,
                    "bel bidir",
                )?;
                let wire = self.eval_tile_wire_ref(vid, tcls, wire_ref)?;
                bel.bidirs.insert(pin, wire);
            }
            ast::BelItem::Attribute(attr) => {
                let name = self.eval_templ_id(&attr.name)?;
                let Some((aid, cattr)) = self.dbs.variants[vid].db.bel_classes[bcid]
                    .attributes
                    .get(&name.to_string())
                else {
                    error_at(name.span(), "unknown attribute")?
                };
                let typ = cattr.typ;
                if let Some(ref avalues) = attr.values {
                    let mut bits = vec![];
                    for bit in &attr.bits {
                        bits.push(self.eval_tile_bit(vid, tcls, bit)?);
                    }
                    let BelAttributeType::Enum(ecid) = typ else {
                        unreachable!()
                    };
                    let mut values = EntityPartVec::new();
                    for (vname, val) in avalues {
                        let vname = self.eval_templ_id(vname)?;
                        let Some((vid, _)) = self.dbs.variants[vid].db.enum_classes[ecid]
                            .values
                            .get(&vname.to_string())
                        else {
                            error_at(name.span(), "unknown enum value")?
                        };
                        values.insert(vid, val.clone());
                    }
                    bel.attributes
                        .insert(aid, BelAttribute::Enum(BelAttributeEnum { bits, values }));
                } else {
                    let mut bits = vec![];
                    for bit in &attr.bits {
                        bits.push(self.eval_pol_tile_bit(vid, tcls, bit)?);
                    }
                    match typ {
                        BelAttributeType::U32 => {
                            error_at(name.span(), "u32 not allowed in attributes")?
                        }
                        BelAttributeType::Enum(_) => error_at(name.span(), "missing enum values")?,
                        BelAttributeType::Bool => {
                            if bits.len() != 1 {
                                error_at(name.span(), "expected single bit")?;
                            }
                        }
                        BelAttributeType::BitVec(width) => {
                            if bits.len() != width {
                                error_at(
                                    name.span(),
                                    &format!("expected {width} bits, got {n}", n = bits.len()),
                                )?;
                            }
                        }
                        BelAttributeType::BitVecArray(_, _) => todo!("bitvec array"),
                    }
                    bel.attributes.insert(aid, BelAttribute::BitVec(bits));
                }
            }
            ast::BelItem::ForLoop(for_loop) => self.eval_for(for_loop, |ctx, subitem| {
                ctx.eval_bel(vid, tcls, bslot, bcid, bel, subitem)
            })?,
            ast::BelItem::If(if_) => {
                self.eval_if(IfContext::Bel(tcls, bslot), if_, |ctx, subitem| {
                    if ctx.if_mask[vid] {
                        ctx.eval_bel(vid, tcls, bslot, bcid, bel, subitem)
                    } else {
                        Ok(())
                    }
                })?
            }
        }
        Ok(())
    }

    fn eval_connector_slot_phase1(
        &mut self,
        cslot: ConnectorSlotId,
        item: &ast::ConnectorSlotItem,
    ) -> Result<()> {
        match item {
            ast::ConnectorSlotItem::Opposite(_) => (),
            ast::ConnectorSlotItem::ConnectorClass(ccls) => {
                for raw_name in &ccls.names {
                    let name = self.eval_templ_id(raw_name)?;
                    let (ccid, name) = self.dbs.ccls_id.insert(&name);
                    for vid in self.active_variants() {
                        if self.dbs.variants[vid]
                            .db
                            .conn_classes
                            .insert(
                                ccid,
                                ConnectorClass {
                                    name: name.clone(),
                                    slot: cslot,
                                    wires: Default::default(),
                                },
                            )
                            .is_err()
                        {
                            error_at(raw_name.span(), "connector class redefined")?;
                        }
                    }
                }
            }
            ast::ConnectorSlotItem::ForLoop(for_loop) => self
                .eval_for(for_loop, |ctx, subitem| {
                    ctx.eval_connector_slot_phase1(cslot, subitem)
                })?,
            ast::ConnectorSlotItem::If(if_) => {
                self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                    ctx.eval_connector_slot_phase1(cslot, subitem)
                })?
            }
        }
        Ok(())
    }

    fn eval_connector_slot(
        &mut self,
        cslot: ConnectorSlotId,
        got_opposite: &mut EntityVec<VariantId, bool>,
        item: &ast::ConnectorSlotItem,
    ) -> Result<()> {
        match item {
            ast::ConnectorSlotItem::Opposite(ident) => {
                let ident = self.eval_templ_id(ident)?;
                for vid in self.active_variants() {
                    if got_opposite[vid] {
                        error_at(ident.span(), "opposite slot redefined")?;
                    }
                    got_opposite[vid] = true;
                    let opposite = self.eval_conn_slot_ref(vid, &ident)?;
                    self.dbs.variants[vid].db.conn_slots[cslot].opposite = opposite;
                }
            }
            ast::ConnectorSlotItem::ConnectorClass(ccls) => {
                for name in &ccls.names {
                    let name = self.eval_templ_id(name)?;
                    let ccid = self.dbs.ccls_id.get(&name);
                    for item in &ccls.items {
                        self.eval_connector_class(ccid, item)?;
                    }
                }
            }
            ast::ConnectorSlotItem::ForLoop(for_loop) => self
                .eval_for(for_loop, |ctx, subitem| {
                    ctx.eval_connector_slot(cslot, got_opposite, subitem)
                })?,
            ast::ConnectorSlotItem::If(if_) => {
                self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                    ctx.eval_connector_slot(cslot, got_opposite, subitem)
                })?
            }
        }
        Ok(())
    }

    fn eval_connector_class(
        &mut self,
        ccls: ConnectorClassId,
        item: &ast::ConnectorClassItem,
    ) -> Result<()> {
        match item {
            ast::ConnectorClassItem::Pass(dst, src) => {
                for vid in self.active_variants() {
                    let span = dst.span();
                    let dst = self.eval_wire_ref_wide(vid, dst)?;
                    let src = self.eval_wire_ref_wide(vid, src)?;
                    let ccls = &mut self.dbs.variants[vid].db.conn_classes[ccls];
                    match (dst, src) {
                        (EntityBundleIndices::Single(dst), EntityBundleIndices::Single(src)) => {
                            ccls.wires.insert(dst, ConnectorWire::Pass(src));
                        }
                        (EntityBundleIndices::Array(dsts), EntityBundleIndices::Array(srcs))
                            if dsts.len() == srcs.len() =>
                        {
                            for (dst, src) in dsts.into_iter().zip(srcs) {
                                ccls.wires.insert(dst, ConnectorWire::Pass(src));
                            }
                        }
                        _ => error_at(span, "assignment width mismatch")?,
                    }
                }
            }
            ast::ConnectorClassItem::Reflect(dst, src) => {
                for vid in self.active_variants() {
                    let span = dst.span();
                    let dst = self.eval_wire_ref_wide(vid, dst)?;
                    let src = self.eval_wire_ref_wide(vid, src)?;
                    let ccls = &mut self.dbs.variants[vid].db.conn_classes[ccls];
                    match (dst, src) {
                        (EntityBundleIndices::Single(dst), EntityBundleIndices::Single(src)) => {
                            ccls.wires.insert(dst, ConnectorWire::Reflect(src));
                        }
                        (EntityBundleIndices::Array(dsts), EntityBundleIndices::Array(srcs))
                            if dsts.len() == srcs.len() =>
                        {
                            for (dst, src) in dsts.into_iter().zip(srcs) {
                                ccls.wires.insert(dst, ConnectorWire::Reflect(src));
                            }
                        }
                        _ => error_at(span, "assignment width mismatch")?,
                    }
                }
            }
            ast::ConnectorClassItem::Blackhole(dst) => {
                for vid in self.active_variants() {
                    let dst = self.eval_wire_ref_wide(vid, dst)?;
                    let ccls = &mut self.dbs.variants[vid].db.conn_classes[ccls];
                    match dst {
                        EntityBundleIndices::Single(dst) => {
                            ccls.wires.insert(dst, ConnectorWire::BlackHole);
                        }
                        EntityBundleIndices::Array(dsts) => {
                            for dst in dsts {
                                ccls.wires.insert(dst, ConnectorWire::BlackHole);
                            }
                        }
                    }
                }
            }
            ast::ConnectorClassItem::Skip => {
                for vid in self.active_variants() {
                    self.dbs.variants[vid].skip_ccls.insert(ccls);
                }
            }
            ast::ConnectorClassItem::ForLoop(for_loop) => self
                .eval_for(for_loop, |ctx, subitem| {
                    ctx.eval_connector_class(ccls, subitem)
                })?,
            ast::ConnectorClassItem::If(if_) => {
                self.eval_if(IfContext::ConnectorClass(ccls), if_, |ctx, subitem| {
                    ctx.eval_connector_class(ccls, subitem)
                })?
            }
        }
        Ok(())
    }

    fn eval_table_phase1(&mut self, tid: TableId, item: &ast::TableItem) -> Result<()> {
        match item {
            ast::TableItem::Field(field) => {
                for name in &field.names {
                    let ident = self.eval_templ_id(name)?;
                    self.dbs.table[tid].field_id.insert(&ident);
                }
            }
            ast::TableItem::Row(names) => {
                for name in names {
                    let ident = self.eval_templ_id(name)?;
                    self.dbs.table[tid].row_id.insert(&ident);
                }
            }
            ast::TableItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_table_phase1(tid, subitem))?
            }
            ast::TableItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_table_phase1(tid, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_table(&mut self, tid: TableId, item: &ast::TableItem) -> Result<()> {
        match item {
            ast::TableItem::Field(field) => {
                for vid in self.active_variants() {
                    let typ = self.eval_attribute_type(vid, &field.typ)?;
                    for name in &field.names {
                        let ident = self.eval_templ_id(name)?;
                        let fid = self.dbs.table[tid].field_id.get(&ident);
                        if self.dbs.variants[vid].db.tables[tid]
                            .fields
                            .insert(
                                fid,
                                TableField {
                                    name: ident.to_string(),
                                    typ,
                                },
                            )
                            .is_err()
                        {
                            error_at(name.span(), "table field redefined")?;
                        }
                    }
                }
            }
            ast::TableItem::Row(names) => {
                for vid in self.active_variants() {
                    for name in names {
                        let ident = self.eval_templ_id(name)?;
                        let rid = self.dbs.table[tid].row_id.get(&ident);
                        if self.dbs.variants[vid].db.tables[tid]
                            .rows
                            .insert(
                                rid,
                                TableRow {
                                    name: ident.to_string(),
                                    fields: Default::default(),
                                },
                            )
                            .is_err()
                        {
                            error_at(name.span(), "table row redefined")?;
                        }
                    }
                }
            }
            ast::TableItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_table(tid, subitem))?
            }
            ast::TableItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_table(tid, subitem)
            })?,
        }
        Ok(())
    }

    fn eval_wire_kind(&mut self, vid: VariantId, kind: &ast::WireKind) -> Result<WireKind> {
        match kind {
            ast::WireKind::Tie0 => Ok(WireKind::Tie0),
            ast::WireKind::Tie1 => Ok(WireKind::Tie1),
            ast::WireKind::TiePullup => Ok(WireKind::TiePullup),
            ast::WireKind::Regional(ident) => {
                let rslot = self.eval_region_slot_ref(vid, ident)?;
                Ok(WireKind::Regional(rslot))
            }
            ast::WireKind::Mux => Ok(WireKind::MuxOut),
            ast::WireKind::Bel => Ok(WireKind::BelOut),
            ast::WireKind::Test => Ok(WireKind::TestOut),
            ast::WireKind::MultiRoot => Ok(WireKind::MultiRoot),
            ast::WireKind::MultiBranch(ident) => {
                let csid = self.eval_conn_slot_ref(vid, ident)?;
                Ok(WireKind::MultiBranch(csid))
            }
            ast::WireKind::Branch(ident) => {
                let csid = self.eval_conn_slot_ref(vid, ident)?;
                Ok(WireKind::Branch(csid))
            }
            ast::WireKind::Special => Ok(WireKind::Special),
        }
    }

    fn eval_wire_kinds(
        &mut self,
        vid: VariantId,
        span: Span,
        kind: &ast::WireKinds,
        idx: Option<usize>,
    ) -> Result<WireKind> {
        match kind {
            ast::WireKinds::Single(kind) => self.eval_wire_kind(vid, kind),
            ast::WireKinds::Multi(kinds) => {
                let idx = idx.unwrap();
                for (start, end, kind) in kinds {
                    let start = self.eval_expr(vid, start)?;
                    let end = self.eval_expr(vid, end)?;
                    if (start..end).contains(&idx) {
                        return self.eval_wire_kind(vid, kind);
                    }
                }
                error_at(span, "index not in match")?
            }
        }
    }

    fn eval_top_phase1(&mut self, item: &ast::TopItem) -> Result<()> {
        match item {
            ast::TopItem::Variant(ident) => {
                error_at(ident.span(), "variants must be at the top level")?;
            }
            ast::TopItem::Const(c) => {
                for vid in self.active_variants() {
                    let value = self.eval_expr(vid, &c.value)?;
                    if self.dbs.variants[vid]
                        .consts
                        .insert(
                            c.name.to_string(),
                            Const {
                                ident: c.name.clone(),
                                value,
                            },
                        )
                        .1
                        .is_some()
                    {
                        error_at(c.name.span(), "const redefined")?;
                    };
                }
            }
            ast::TopItem::BitRectClass(brc) => {
                for vid in self.active_variants() {
                    if self.dbs.variants[vid]
                        .bitrect_geoms
                        .insert(brc.name.to_string(), brc.geometry)
                        .is_some()
                    {
                        error_at(brc.name.span(), "bitrect class redefined")?;
                    }
                }
            }
            ast::TopItem::EnumClass(a_ecls) => {
                let (ecid, name) = self.dbs.enum_id.insert(&a_ecls.name);
                if !self.dbs.ecls.contains_id(ecid) {
                    self.dbs.ecls.insert(ecid, Default::default());
                }

                let mut values = EntityPartMap::new();
                for val in &a_ecls.values {
                    let (vid, vname) = self.dbs.ecls[ecid].insert(val);
                    if values.insert(vid, vname).is_err() {
                        error_at(val.span(), "enum value redefined")?;
                    }
                }
                let ecls = EnumClass { name, values };
                for vid in self.active_variants() {
                    let variant = &mut self.dbs.variants[vid];
                    if variant.db.enum_classes.insert(ecid, ecls.clone()).is_err() {
                        error_at(a_ecls.name.span(), "enum class redefined")?;
                    }
                }
            }
            ast::TopItem::BelClass(bcls) => {
                let (bcid, name) = self.dbs.bcls_id.insert(&bcls.name);
                if !self.dbs.bcls.contains_id(bcid) {
                    self.dbs.bcls.insert(bcid, Default::default());
                    self.bcls_pre
                        .insert(bcid, BelClassPre::new(self.dbs.variants.len()));
                }

                for vid in self.active_variants() {
                    let variant = &mut self.dbs.variants[vid];
                    if variant
                        .db
                        .bel_classes
                        .insert(
                            bcid,
                            BelClass {
                                name: name.clone(),
                                inputs: Default::default(),
                                outputs: Default::default(),
                                bidirs: Default::default(),
                                pads: Default::default(),
                                attributes: Default::default(),
                            },
                        )
                        .is_err()
                    {
                        error_at(bcls.name.span(), "bel class redefined")?;
                    }
                }
                for item in &bcls.items {
                    self.eval_bel_class_phase1(bcid, item)?;
                }
            }
            ast::TopItem::TileSlot(tslot) => {
                let ident = self.eval_templ_id(&tslot.name)?;
                let (tsid, name) = self.dbs.tslot_id.insert(&ident);

                for vid in self.active_variants() {
                    let variant = &mut self.dbs.variants[vid];
                    if variant.db.tile_slots.insert(tsid, name.clone()).is_err() {
                        error_at(tslot.name.span(), "tile slot redefined")?;
                    }
                }
                for item in &tslot.items {
                    self.eval_tile_slot_phase1(tsid, item)?;
                }
            }
            ast::TopItem::RegionSlot(rslot) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, &rslot.name)?;
                    if let Err(msg) = self.rslot_pre.insert(vid, &name, num, ()) {
                        error_at(name.span(), msg)?
                    }
                }
            }
            ast::TopItem::ConnectorSlot(cslot) => {
                let ident = self.eval_templ_id(&cslot.name)?;
                let (csid, name) = self.dbs.cslot_id.insert(&ident);

                for vid in self.active_variants() {
                    let variant = &mut self.dbs.variants[vid];
                    if variant
                        .db
                        .conn_slots
                        .insert(
                            csid,
                            ConnectorSlot {
                                name: name.to_string(),
                                opposite: ConnectorSlotId::from_idx(0),
                            },
                        )
                        .is_err()
                    {
                        error_at(cslot.name.span(), "connector slot redefined")?;
                    }
                }
                for item in &cslot.items {
                    self.eval_connector_slot_phase1(csid, item)?;
                }
            }
            ast::TopItem::Wire(wire) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, &wire.name)?;
                    if let Err(msg) = self.wire_pre.insert(vid, &name, num, ()) {
                        error_at(name.span(), msg)?
                    }
                }
            }
            ast::TopItem::Table(table) => {
                let ident = self.eval_templ_id(&table.name)?;
                let (tid, name) = self.dbs.table_id.insert(&ident);
                if !self.dbs.table.contains_id(tid) {
                    self.dbs.table.insert(tid, Default::default());
                }
                for vid in self.active_variants() {
                    let variant = &mut self.dbs.variants[vid];
                    if variant
                        .db
                        .tables
                        .insert(
                            tid,
                            Table {
                                name: name.clone(),
                                fields: Default::default(),
                                rows: Default::default(),
                            },
                        )
                        .is_err()
                    {
                        error_at(table.name.span(), "table redefined")?;
                    }
                }
                for item in &table.items {
                    self.eval_table_phase1(tid, item)?;
                }
            }
            ast::TopItem::DeviceData(devdata) => {
                for raw_name in &devdata.names {
                    let ident = self.eval_templ_id(raw_name)?;
                    let (ddid, name) = self.dbs.devdata_id.insert(&ident);
                    for vid in self.active_variants() {
                        let variant = &mut self.dbs.variants[vid];
                        if variant
                            .db
                            .devdata
                            .insert(
                                ddid,
                                DeviceDataSlot {
                                    name: name.clone(),
                                    // overwritten later
                                    typ: BelAttributeType::Bool,
                                },
                            )
                            .is_err()
                        {
                            error_at(raw_name.span(), "device data redefined")?;
                        }
                    }
                }
            }
            ast::TopItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_top_phase1(subitem))?
            }
            ast::TopItem::If(if_) => self.eval_if(IfContext::Top, if_, |ctx, subitem| {
                ctx.eval_top_phase1(subitem)
            })?,
        }
        Ok(())
    }

    fn eval_top(&mut self, item: &ast::TopItem) -> Result<()> {
        match item {
            ast::TopItem::Variant(ident) => {
                error_at(ident.span(), "variants must be at the top level")?;
            }
            ast::TopItem::Const(_) => (),
            ast::TopItem::BitRectClass(_) => (),
            ast::TopItem::EnumClass(_) => (),
            ast::TopItem::BelClass(bcls) => {
                let bcid = self.dbs.bcls_id.get(&bcls.name);
                for item in &bcls.items {
                    self.eval_bel_class(bcid, item)?;
                }
            }
            ast::TopItem::TileSlot(tslot) => {
                let name = self.eval_templ_id(&tslot.name)?;
                let tsid = self.dbs.tslot_id.get(&name);
                for item in &tslot.items {
                    self.eval_tile_slot(tsid, item)?;
                }
            }
            ast::TopItem::RegionSlot(_) => (),
            ast::TopItem::ConnectorSlot(cslot) => {
                let name = self.eval_templ_id(&cslot.name)?;
                let csid = self.dbs.cslot_id.get(&name);

                let mut got_opposite = self.dbs.variants.map_values(|_| false);
                for item in &cslot.items {
                    self.eval_connector_slot(csid, &mut got_opposite, item)?;
                }
                for vid in self.active_variants() {
                    if !got_opposite[vid] {
                        error_at(cslot.name.span(), "missing opposite connector slot")?;
                    }
                }
            }
            ast::TopItem::Wire(wire) => {
                for vid in self.active_variants() {
                    let (name, num) = self.eval_array_def(vid, &wire.name)?;
                    if let Some(num) = num {
                        for i in 0..num {
                            let id = self.dbs.wire_id.get(&name, Some(i));
                            let kind =
                                self.eval_wire_kinds(vid, name.span(), &wire.kinds, Some(i))?;
                            self.dbs.variants[vid].db.wires[id].kind = kind;
                        }
                    } else {
                        let id = self.dbs.wire_id.get(&name, None);
                        let kind = self.eval_wire_kinds(vid, name.span(), &wire.kinds, None)?;
                        self.dbs.variants[vid].db.wires[id].kind = kind;
                    }
                }
            }
            ast::TopItem::Table(table) => {
                let ident = self.eval_templ_id(&table.name)?;
                let tid = self.dbs.table_id.get(&ident);
                for item in &table.items {
                    self.eval_table(tid, item)?;
                }
            }
            ast::TopItem::DeviceData(devdata) => {
                for vid in self.active_variants() {
                    let typ = self.eval_attribute_type(vid, &devdata.typ)?;
                    for raw_name in &devdata.names {
                        let ident = self.eval_templ_id(raw_name)?;
                        let ddid = self.dbs.devdata_id.get(&ident);
                        self.dbs.variants[vid].db.devdata[ddid].typ = typ;
                    }
                }
            }
            ast::TopItem::ForLoop(for_loop) => {
                self.eval_for(for_loop, |ctx, subitem| ctx.eval_top(subitem))?
            }
            ast::TopItem::If(if_) => {
                self.eval_if(IfContext::Top, if_, |ctx, subitem| ctx.eval_top(subitem))?
            }
        }
        Ok(())
    }
}

pub fn eval(items: Vec<ast::TopItem>) -> Result<AnnotatedDbSet> {
    let mut variants = vec![];
    for item in &items {
        if let ast::TopItem::Variant(variant) = item {
            variants.push(Some(variant.clone()));
        }
    }
    if variants.is_empty() {
        variants.push(None);
    }
    let num_variants = variants.len();
    let if_mask = variants.iter().map(|_| true).collect();
    let mut ctx = Context {
        dbs: AnnotatedDbSet {
            variants: variants
                .into_iter()
                .map(|variant| AnnotatedDb {
                    name: variant,
                    db: IntDb::default(),
                    tcls_cell_id: Default::default(),
                    tcls_bitrect_id: Default::default(),
                    bitrect_geoms: Default::default(),
                    skip_tcls: Default::default(),
                    skip_ccls: Default::default(),
                    consts: Default::default(),
                })
                .collect(),
            enum_id: Default::default(),
            tslot_id: Default::default(),
            cslot_id: Default::default(),
            bslot_id: Default::default(),
            rslot_id: Default::default(),
            wire_id: Default::default(),
            tcls_id: Default::default(),
            ccls_id: Default::default(),
            bcls_id: Default::default(),
            table_id: Default::default(),
            devdata_id: Default::default(),
            ecls: Default::default(),
            bcls: Default::default(),
            table: Default::default(),
        },
        if_mask,
        for_vars: Default::default(),
        bcls_pre: Default::default(),
        bslot_pre: BundleIdentPreRegistry::new(num_variants),
        rslot_pre: BundleIdentPreRegistry::new(num_variants),
        wire_pre: BundleIdentPreRegistry::new(num_variants),
    };
    for item in &items {
        if let ast::TopItem::Variant(_) = item {
            continue;
        }
        ctx.eval_top_phase1(item)?;
    }

    let (bslot_id, vars) = ctx.bslot_pre.finish(|name, &tile_slot| BelSlot {
        name,
        tile_slot,
        // overwritten later
        kind: BelKind::Legacy,
    });
    ctx.dbs.bslot_id = bslot_id;
    for (vid, bel_slots) in vars {
        ctx.dbs.variants[vid].db.bel_slots = bel_slots;
    }

    let (rslot_id, vars) = ctx.rslot_pre.finish(|name, _| name);
    ctx.dbs.rslot_id = rslot_id;
    for (vid, region_slots) in vars {
        ctx.dbs.variants[vid].db.region_slots = region_slots;
    }

    let (wire_id, vars) = ctx.wire_pre.finish(|name, _| WireSlot {
        name,
        // overwritten later
        kind: WireKind::Special,
    });
    ctx.dbs.wire_id = wire_id;
    for (vid, wires) in vars {
        ctx.dbs.variants[vid].db.wires = wires;
    }

    for (bcid, pre) in &ctx.bcls_pre {
        let (input_id, vars) = pre.input_pre.finish_bundle();
        ctx.dbs.bcls[bcid].input_id = input_id;
        for (vid, inputs) in vars {
            if !inputs.is_empty() {
                ctx.dbs.variants[vid].db.bel_classes[bcid].inputs = inputs;
            }
        }
        let (output_id, vars) = pre.output_pre.finish_bundle();
        ctx.dbs.bcls[bcid].output_id = output_id;
        for (vid, outputs) in vars {
            if !outputs.is_empty() {
                ctx.dbs.variants[vid].db.bel_classes[bcid].outputs = outputs;
            }
        }
        let (bidir_id, vars) = pre.bidir_pre.finish_bundle();
        ctx.dbs.bcls[bcid].bidir_id = bidir_id;
        for (vid, bidirs) in vars {
            if !bidirs.is_empty() {
                ctx.dbs.variants[vid].db.bel_classes[bcid].bidirs = bidirs;
            }
        }
        let (pad_id, vars) = pre.pad_pre.finish_bundle();
        ctx.dbs.bcls[bcid].pad_id = pad_id;
        for (vid, pads) in vars {
            if !pads.is_empty() {
                ctx.dbs.variants[vid].db.bel_classes[bcid].pads = pads;
            }
        }
    }

    for item in &items {
        if let ast::TopItem::Variant(_) = item {
            continue;
        }
        ctx.eval_top(item)?;
    }

    for db in ctx.dbs.variants.values_mut() {
        for &tcid in &db.skip_tcls {
            db.db.tile_classes.remove(tcid);
        }
        for &ccid in &db.skip_ccls {
            db.db.conn_classes.remove(ccid);
        }
    }

    Ok(ctx.dbs)
}
