//! `--` wire bindings (CopperTrace topology style) resolve against the
//! port's modport verb at parse time: `out` fields become `TwoWay`,
//! `in` fields become `Read`. Downstream stages never see `Binding::Wire`.

use mobius_lang::ast::*;
use mobius_lang::parse;

const SRC: &str = r#"
app Wires {
    interface Chrome {
        theme : f32 = 0.5
        clock : string = ""
        level : [f32; 4] = {0.5}
        modport ribbon (out theme, in clock, out level)
    }
    citizen Ribbon (c : Chrome.ribbon) {
        row {
            slider "Theme" 0.0..1.0 -- c.theme;
            clock -- c.clock;
            for ch in 0..4 {
                slider "Ch {ch}" 0.0..1.0 -- c.level[ch];
            }
        }
    }
}
"#;

fn ribbon_children(app: &App) -> &[WidgetNode] {
    let citizen = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Citizen(citizen) => Some(citizen),
            _ => None,
        })
        .expect("Ribbon citizen");
    let WidgetNode::Container { children, .. } = &citizen.body[0] else {
        panic!("expected row container")
    };
    children
}

#[test]
fn wire_on_out_port_resolves_to_twoway() {
    let (file, _) = parse(SRC).expect("wire app must parse");
    let children = ribbon_children(&file.apps[0]);
    let WidgetNode::Primitive(slider) = &children[0] else {
        panic!()
    };
    assert!(
        matches!(&slider.binding, Binding::TwoWay { path, index: None }
        if path.segments == ["c", "theme"]),
        "out port must resolve to TwoWay, got {:?}",
        slider.binding
    );
}

#[test]
fn wire_on_in_port_resolves_to_read() {
    let (file, _) = parse(SRC).expect("wire app must parse");
    let children = ribbon_children(&file.apps[0]);
    let WidgetNode::Primitive(clock) = &children[1] else {
        panic!()
    };
    assert!(
        matches!(&clock.binding, Binding::Read { path, index: None }
        if path.segments == ["c", "clock"]),
        "in port must resolve to Read, got {:?}",
        clock.binding
    );
}

#[test]
fn wire_with_loop_index_keeps_the_index() {
    let (file, _) = parse(SRC).expect("wire app must parse");
    let children = ribbon_children(&file.apps[0]);
    let WidgetNode::For { body, .. } = &children[2] else {
        panic!("expected for loop")
    };
    let WidgetNode::Primitive(slider) = &body[0] else {
        panic!()
    };
    let Binding::TwoWay { path, index: Some(Expr::Path(index)) } = &slider.binding else {
        panic!("indexed out port must resolve to TwoWay with index, got {:?}", slider.binding)
    };
    assert_eq!(path.segments, ["c", "level"]);
    assert_eq!(index.segments, ["ch"]);
}

#[test]
fn explicit_arrows_still_parse() {
    let explicit = SRC
        .replace("-- c.theme", "<-> c.theme")
        .replace("-- c.clock", "<- c.clock")
        .replace("-- c.level[ch]", "<-> c.level[ch]");
    let (file, _) = parse(&explicit).expect("explicit arrows must keep parsing");
    let children = ribbon_children(&file.apps[0]);
    let WidgetNode::Primitive(slider) = &children[0] else {
        panic!()
    };
    assert!(matches!(&slider.binding, Binding::TwoWay { .. }));
}
