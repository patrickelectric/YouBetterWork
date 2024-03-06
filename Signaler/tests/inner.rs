use decorators::*;
use signal::*;

use test_log::test;

#[derive(Default, Signaler)]
struct SimpleTalker {
    #[property]
    value: u64,
    #[property]
    other_value: u64,
}

#[test]
fn test_inner_simple() {
    let mut simple = SimpleTalkerSignaler::default();
    simple.on_inner_value_changed(|s: &mut SimpleTalkerSignaler, value: u64| {
        s.set_other_value(value);
    });
    simple.set_value(3);
    assert_eq!(simple.other_value(), 3);
}
