use std::str::FromStr;

use prjcombine_entity::{EntityBundleIndices, EntityBundleMap, EntityId, EntityMap};
use prjcombine_interconnect::db::{BelPinIndexing, IntDb};
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::db::{AnnotatedDb, AnnotatedDbSet, BundleIdentRegistry, IdentRegistry, VariantId};

fn to_ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

fn keyword(kw: &str) -> TokenTree {
    TokenTree::Ident(Ident::new(kw, Span::call_site()))
}

fn joint(ch: char) -> TokenTree {
    TokenTree::Punct(Punct::new(ch, Spacing::Joint))
}

fn punct(ch: char) -> TokenTree {
    TokenTree::Punct(Punct::new(ch, Spacing::Alone))
}

fn parens(stream: TokenStream) -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::Parenthesis, stream))
}

fn num(n: usize) -> TokenTree {
    TokenTree::Literal(Literal::usize_unsuffixed(n))
}

fn emit_ids<I: EntityId>(cls: &str, idents: &EntityMap<I, String, Ident>) -> TokenStream {
    let mut res = TokenStream::new();
    emit_use(&mut res, &["prjcombine_interconnect", "db", cls]);

    for (id, _, name) in idents {
        res.extend([
            keyword("pub"),
            keyword("const"),
            TokenTree::Ident(name.clone()),
            punct(':'),
            keyword(cls),
            punct('='),
            keyword(cls),
            joint(':'),
            punct(':'),
            keyword("from_idx_const"),
            parens(TokenStream::from_iter([num(id.to_idx())])),
            punct(';'),
        ]);
    }

    res
}

fn emit_ids_new<I: EntityId>(cls: &str, reg: &IdentRegistry<I>) -> TokenStream {
    let mut res = TokenStream::new();
    emit_use(&mut res, &["prjcombine_interconnect", "db", cls]);
    let mut ctr = 0;

    for (id, _, idents) in &reg.ids {
        res.extend([
            keyword("pub"),
            keyword("const"),
            TokenTree::Ident(idents[0].clone()),
            punct(':'),
            keyword(cls),
            punct('='),
            keyword(cls),
            joint(':'),
            punct(':'),
            keyword("from_idx_const"),
            parens(TokenStream::from_iter([num(id.to_idx())])),
            punct(';'),
        ]);
        for id in &idents[1..] {
            res.extend([
                keyword("const"),
                TokenTree::Ident(Ident::new(&format!("_{cls}_{ctr}"), Span::call_site())),
                punct(':'),
                keyword(cls),
                punct('='),
                TokenTree::Ident(id.clone()),
                punct(';'),
            ]);
            ctr += 1;
        }
    }

    res
}

fn emit_array_ids<I: EntityId>(
    cls: &str,
    idents: &EntityBundleMap<I, Ident>,
    do_use: bool,
) -> TokenStream {
    let mut res = TokenStream::new();
    if do_use {
        emit_use(&mut res, &["prjcombine_entity", "id", "EntityStaticRange"]);
    }
    emit_use(&mut res, &["prjcombine_interconnect", "db", cls]);

    for (index, _, ident) in idents.bundles() {
        match index {
            EntityBundleIndices::Single(id) => {
                res.extend([
                    keyword("pub"),
                    keyword("const"),
                    TokenTree::Ident(ident.clone()),
                    punct(':'),
                    keyword(cls),
                    punct('='),
                    keyword(cls),
                    joint(':'),
                    punct(':'),
                    keyword("from_idx_const"),
                    parens(TokenStream::from_iter([num(id.to_idx())])),
                    punct(';'),
                ]);
            }
            EntityBundleIndices::Array(range) => {
                res.extend([
                    keyword("pub"),
                    keyword("const"),
                    TokenTree::Ident(ident.clone()),
                    punct(':'),
                    keyword("EntityStaticRange"),
                    punct('<'),
                    keyword(cls),
                    punct(','),
                    num(range.len()),
                    punct('>'),
                    punct('='),
                    keyword("EntityStaticRange"),
                    joint(':'),
                    punct(':'),
                    punct('<'),
                    keyword(cls),
                    punct(','),
                    keyword("_"),
                    punct('>'),
                    joint(':'),
                    punct(':'),
                    keyword("new_const"),
                    parens(TokenStream::from_iter([num(range
                        .first()
                        .unwrap()
                        .to_idx())])),
                    punct(';'),
                ]);
            }
        }
    }

    res
}

fn emit_array_ids_new<I: EntityId>(
    cls: &str,
    reg: &BundleIdentRegistry<I>,
    vid: Option<VariantId>,
    do_use: bool,
) -> TokenStream {
    let mut res = TokenStream::new();
    let mut ctr = 0;
    if do_use {
        emit_use(&mut res, &["prjcombine_entity", "id", "EntityStaticRange"]);
    }
    emit_use(&mut res, &["prjcombine_interconnect", "db", cls]);

    for (index, _, data) in reg.ids.bundles() {
        if let Some(vid) = vid
            && !data.variants.contains_id(vid)
        {
            continue;
        }
        match index {
            EntityBundleIndices::Single(id) => {
                res.extend([
                    keyword("pub"),
                    keyword("const"),
                    TokenTree::Ident(data.idents[0].clone()),
                    punct(':'),
                    keyword(cls),
                    punct('='),
                    keyword(cls),
                    joint(':'),
                    punct(':'),
                    keyword("from_idx_const"),
                    parens(TokenStream::from_iter([num(id.to_idx())])),
                    punct(';'),
                ]);

                for id in &data.idents[1..] {
                    res.extend([
                        keyword("const"),
                        TokenTree::Ident(Ident::new(&format!("_{cls}_{ctr}"), Span::call_site())),
                        punct(':'),
                        keyword(cls),
                        punct('='),
                        TokenTree::Ident(id.clone()),
                        punct(';'),
                    ]);
                    ctr += 1;
                }
            }
            EntityBundleIndices::Array(range) => {
                let n = if let Some(vid) = vid {
                    data.variants[vid].unwrap()
                } else {
                    range.len()
                };
                res.extend([
                    keyword("pub"),
                    keyword("const"),
                    TokenTree::Ident(data.idents[0].clone()),
                    punct(':'),
                    keyword("EntityStaticRange"),
                    punct('<'),
                    keyword(cls),
                    punct(','),
                    num(n),
                    punct('>'),
                    punct('='),
                    keyword("EntityStaticRange"),
                    joint(':'),
                    punct(':'),
                    punct('<'),
                    keyword(cls),
                    punct(','),
                    keyword("_"),
                    punct('>'),
                    joint(':'),
                    punct(':'),
                    keyword("new_const"),
                    parens(TokenStream::from_iter([num(range
                        .first()
                        .unwrap()
                        .to_idx())])),
                    punct(';'),
                ]);

                for id in &data.idents[1..] {
                    res.extend([
                        keyword("const"),
                        TokenTree::Ident(Ident::new(&format!("_{cls}_{ctr}"), Span::call_site())),
                        punct(':'),
                        keyword("EntityStaticRange"),
                        punct('<'),
                        keyword(cls),
                        punct(','),
                        num(n),
                        punct('>'),
                        punct('='),
                        TokenTree::Ident(id.clone()),
                        punct(';'),
                    ]);
                    ctr += 1;
                }
            }
        }
    }

    res
}

fn emit_pin_array_ids<I: EntityId>(
    cls: &str,
    idents: &EntityBundleMap<I, (Ident, BelPinIndexing)>,
    do_use: bool,
) -> TokenStream {
    let mut new_idents: EntityBundleMap<I, _> = EntityBundleMap::new();
    for (idx, name, (ident, _)) in idents.bundles() {
        match idx {
            EntityBundleIndices::Single(_) => {
                new_idents.insert(name.into(), ident.clone());
            }
            EntityBundleIndices::Array(range) => {
                new_idents.insert_array(name.into(), range.len(), ident.clone());
            }
        }
    }
    emit_array_ids(cls, &new_idents, do_use)
}

fn emit_tile_classes(stream: &mut TokenStream, dbs: &AnnotatedDbSet, adb: &AnnotatedDb) {
    let mut mod_stream = emit_ids_new("TileClassId", &dbs.tcls_id);
    for (tcid, cell_ids) in &adb.tcls_cell_id {
        let cell_stream = emit_array_ids("CellSlotId", cell_ids, true);
        mod_stream.extend(TokenStream::from_str("#[allow(non_snake_case)]").unwrap());
        emit_mod(
            &mut mod_stream,
            dbs.tcls_id.ids[tcid][0].clone(),
            cell_stream,
        );
    }
    emit_mod(stream, to_ident("tcls"), mod_stream);
}

fn emit_bel_classes(stream: &mut TokenStream, dbs: &AnnotatedDbSet, adb: &AnnotatedDb) {
    let mut mod_stream = emit_ids_new("BelClassId", &dbs.bcls_id);
    for (bcid, bcls) in &adb.bcls {
        let mut inner = TokenStream::new();
        inner.extend(emit_pin_array_ids("BelInputId", &bcls.input_id, true));
        inner.extend(emit_pin_array_ids("BelOutputId", &bcls.output_id, false));
        inner.extend(emit_pin_array_ids("BelBidirId", &bcls.bidir_id, false));
        inner.extend(emit_array_ids("BelPadId", &bcls.pad_id, false));
        inner.extend(emit_ids("BelAttributeId", &bcls.attr_id));
        mod_stream.extend(TokenStream::from_str("#[allow(non_snake_case)]").unwrap());
        emit_mod(&mut mod_stream, dbs.bcls_id.ids[bcid][0].clone(), inner);
    }
    emit_mod(stream, to_ident("bcls"), mod_stream);
}

fn emit_enums(stream: &mut TokenStream, dbs: &AnnotatedDbSet, adb: &AnnotatedDb) {
    let mut mod_stream = emit_ids_new("EnumClassId", &dbs.enum_id);
    for (ecid, vals) in &adb.eval_id {
        let val_stream = emit_ids("EnumValueId", vals);
        mod_stream.extend(TokenStream::from_str("#[allow(non_snake_case)]").unwrap());
        emit_mod(
            &mut mod_stream,
            dbs.enum_id.ids[ecid][0].clone(),
            val_stream,
        );
    }
    emit_mod(stream, to_ident("enums"), mod_stream);
}

fn emit_tables(stream: &mut TokenStream, dbs: &AnnotatedDbSet, adb: &AnnotatedDb) {
    let mut mod_stream = emit_ids_new("TableId", &dbs.table_id);
    for (tid, tdata) in &adb.table {
        let mut inner = TokenStream::new();
        inner.extend(emit_ids("TableFieldId", &tdata.field_id));
        inner.extend(emit_ids("TableRowId", &tdata.row_id));
        mod_stream.extend(TokenStream::from_str("#[allow(non_snake_case)]").unwrap());
        emit_mod(&mut mod_stream, dbs.table_id.ids[tid][0].clone(), inner);
    }
    emit_mod(stream, to_ident("tables"), mod_stream);
}

fn emit_use(stream: &mut TokenStream, name: &[&str]) {
    stream.extend([keyword("use")]);
    for part in name {
        stream.extend([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            keyword(part),
        ]);
    }
    stream.extend([Punct::new(';', Spacing::Alone)]);
}

fn emit_mod(stream: &mut TokenStream, name: Ident, block: TokenStream) {
    let mut group = TokenTree::Group(Group::new(Delimiter::Brace, block));
    group.set_span(name.span());
    stream.extend([
        keyword("pub"),
        keyword("mod"),
        TokenTree::Ident(name),
        group,
    ]);
}

fn emit_init(stream: &mut TokenStream, db: &IntDb) {
    let data = bincode::encode_to_vec(db, bincode::config::standard()).unwrap();
    stream.extend(TokenStream::from_str("pub const INIT: &[u8] = ").unwrap());
    stream.extend([TokenTree::Literal(Literal::byte_string(&data)), punct(';')]);
}

pub fn emit(dbs: AnnotatedDbSet) -> TokenStream {
    let mut enum_uniform = true;
    let mut bcls_uniform = true;
    let mut tcls_uniform = true;
    let mut table_uniform = true;
    let db0 = dbs.variants.first().unwrap();
    for db in dbs.variants.values() {
        if db.db.enum_classes != db0.db.enum_classes {
            enum_uniform = false;
        }
        if db.db.bel_classes != db0.db.bel_classes {
            bcls_uniform = false;
        }
        if db.db.tile_classes != db0.db.tile_classes {
            tcls_uniform = false;
        }
        if db.db.tables != db0.db.tables {
            table_uniform = false;
        }
    }
    let mut res = TokenStream::new();
    let mut variant_outs = dbs.variants.map_values(|_| TokenStream::new());

    if enum_uniform {
        emit_enums(&mut res, &dbs, db0);
        if dbs.variants.len() != 1 {
            for stream in variant_outs.values_mut() {
                stream.extend(TokenStream::from_str("pub use super::enums;").unwrap());
            }
        }
    } else {
        emit_mod(
            &mut res,
            to_ident("enums"),
            emit_ids_new("EnumClassId", &dbs.enum_id),
        );
        for (vid, adb) in &dbs.variants {
            emit_enums(&mut variant_outs[vid], &dbs, adb);
        }
    }

    emit_mod(
        &mut res,
        to_ident("tslots"),
        emit_ids_new("TileSlotId", &dbs.tslot_id),
    );

    emit_mod(
        &mut res,
        to_ident("rslots"),
        emit_array_ids_new("RegionSlotId", &dbs.rslot_id, None, true),
    );

    emit_mod(
        &mut res,
        to_ident("cslots"),
        emit_ids_new("ConnectorSlotId", &dbs.cslot_id),
    );

    if tcls_uniform {
        emit_tile_classes(&mut res, &dbs, db0);
        if dbs.variants.len() != 1 {
            for stream in variant_outs.values_mut() {
                stream.extend(TokenStream::from_str("pub use super::tcls;").unwrap());
            }
        }
    } else {
        emit_mod(
            &mut res,
            to_ident("tcls"),
            emit_ids_new("TileClassId", &dbs.tcls_id),
        );
        for (adb, stream) in dbs.variants.values().zip(variant_outs.values_mut()) {
            emit_tile_classes(stream, &dbs, adb);
        }
    }

    if bcls_uniform {
        emit_bel_classes(&mut res, &dbs, db0);
        if dbs.variants.len() != 1 {
            for stream in variant_outs.values_mut() {
                stream.extend(TokenStream::from_str("pub use super::bcls;").unwrap());
            }
        }
    } else {
        emit_mod(
            &mut res,
            to_ident("bcls"),
            emit_ids_new("BelClassId", &dbs.bcls_id),
        );
        for (adb, stream) in dbs.variants.values().zip(variant_outs.values_mut()) {
            emit_bel_classes(stream, &dbs, adb);
        }
    }

    emit_mod(
        &mut res,
        to_ident("ccls"),
        emit_ids_new("ConnectorClassId", &dbs.ccls_id),
    );

    emit_mod(
        &mut res,
        to_ident("bslots"),
        emit_array_ids_new("BelSlotId", &dbs.bslot_id, None, true),
    );

    emit_mod(
        &mut res,
        to_ident("wires"),
        emit_array_ids_new("WireSlotId", &dbs.wire_id, None, true),
    );
    if variant_outs.len() != 1 {
        for (vid, stream) in &mut variant_outs {
            emit_mod(
                stream,
                to_ident("wires"),
                emit_array_ids_new("WireSlotId", &dbs.wire_id, Some(vid), true),
            );
        }
    }

    if table_uniform {
        emit_tables(&mut res, &dbs, db0);
        if dbs.variants.len() != 1 {
            for stream in variant_outs.values_mut() {
                stream.extend(TokenStream::from_str("use super::tables;").unwrap());
            }
        }
    } else {
        for (adb, stream) in dbs.variants.values().zip(variant_outs.values_mut()) {
            emit_tables(stream, &dbs, adb);
        }
        emit_mod(
            &mut res,
            to_ident("tables"),
            emit_ids_new("TableId", &dbs.table_id),
        );
    }

    if dbs.variants.len() == 1 {
        emit_init(&mut res, &db0.db);
    } else {
        for (adb, stream) in dbs.variants.values().zip(variant_outs.values_mut()) {
            emit_init(stream, &adb.db);
        }
    }

    emit_mod(
        &mut res,
        to_ident("devdata"),
        emit_ids_new("DeviceDataId", &dbs.devdata_id),
    );

    for (adb, stream) in dbs.variants.values().zip(variant_outs.into_values()) {
        if !stream.is_empty() {
            emit_mod(&mut res, adb.name.clone().unwrap(), stream);
        }
    }
    res
}
