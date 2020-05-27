use macro_compose::{Collector, Context, Expand, Lint, Nothing};
use syn::{parse_quote, Error, ItemConst};

#[test]
fn basic() {
    let data: ItemConst = parse_quote!(
        const FOO: bool = true;
    );

    let mut collector = Collector::new();
    let mut ctx = Context::new(&mut collector, data);
    ctx.lint(&NoOpLint);
    ctx.expand(&NoOpExpand);
    assert_eq!(collector.has_errors(), false);
}

struct NoOpLint;

impl Lint<ItemConst> for NoOpLint {
    fn lint(&self, _: &ItemConst, _: &mut Collector) {}
}

struct NoOpExpand;

impl Expand<ItemConst> for NoOpExpand {
    type Output = Nothing;

    fn expand(&self, _: &ItemConst, _: &mut Collector) -> Option<Self::Output> {
        None
    }
}

#[test]
fn test_expand_disable() {
    let data: ItemConst = parse_quote!(
        const FOO: bool = true;
    );

    let mut collector = Collector::new();
    let mut ctx = Context::new(&mut collector, data);
    ctx.lint(&AlwaysErrorLint);
    ctx.expand(&PanickingExpand);
    assert_eq!(collector.has_errors(), true);
}

#[test]
#[should_panic]
fn test_panicking() {
    let data: ItemConst = parse_quote!(
        const FOO: bool = true;
    );

    let mut collector = Collector::new();
    let mut ctx = Context::new(&mut collector, data);
    ctx.expand(&PanickingExpand);
}

struct AlwaysErrorLint;

impl Lint<ItemConst> for AlwaysErrorLint {
    fn lint(&self, i: &ItemConst, c: &mut Collector) {
        c.error(Error::new_spanned(i, "some error message"));
    }
}

struct PanickingExpand;

impl Expand<ItemConst> for PanickingExpand {
    type Output = Nothing;

    fn expand(&self, _: &ItemConst, _: &mut Collector) -> Option<Self::Output> {
        unreachable!()
    }
}
